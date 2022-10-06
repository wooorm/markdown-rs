extern crate swc_common;
extern crate swc_ecma_ast;
use crate::{
    micromark::{id_cont_ as id_cont, id_start_ as id_start},
    test_utils::to_swc::Program,
};
use swc_ecma_visit::{noop_visit_mut_type, VisitMut, VisitMutWith};

/// Configuration.
#[derive(Debug, Default, Clone)]
pub struct Options {
    /// Place to import a provider from.
    ///
    /// See [MDX provider](https://mdxjs.com/docs/using-mdx/#mdx-provider)
    /// on the MDX website for more info.
    pub provider_import_source: Option<String>,
    /// Whether to add extra information to error messages in generated code.
    /// This is not yet supported.
    pub development: bool,
}

/// Rewrite JSX in an MDX file so that components can be passed in and provided.
#[allow(dead_code)]
pub fn jsx_rewrite(mut program: Program, options: &Options) -> Program {
    let mut state = State {
        scopes: vec![],
        provider: options.provider_import_source.is_some(),
        create_provider_import: false,
        create_error_helper: false,
    };
    state.enter(Some(Info::default()));
    program.module.visit_mut_with(&mut state);

    // If a provider is used (and can be used), import it.
    if let Some(source) = &options.provider_import_source {
        if state.create_provider_import {
            program
                .module
                .body
                .insert(0, create_import_provider(source))
        }
    }

    // If potentially missing components are used, add the helper used for
    // errors.
    if state.create_error_helper {
        program.module.body.push(create_error_helper());
    }

    program
}

/// Collection of different SWC functions.
#[derive(Debug)]
enum Func<'a> {
    /// Function declaration.
    Decl(&'a mut swc_ecma_ast::FnDecl),
    /// Function expression.
    Expr(&'a mut swc_ecma_ast::FnExpr),
    /// Arrow function.
    Arrow(&'a mut swc_ecma_ast::ArrowExpr),
}

/// Info for a function scope.
#[derive(Debug, Default, Clone)]
struct Info {
    /// Function name.
    name: Option<String>,
    /// Used objects (`a` in `<a.b />`).
    objects: Vec<String>,
    /// Used components (`<A />`).
    components: Vec<String>,
    /// Used literals (`<a />`).
    tags: Vec<String>,
    /// List of JSX identifiers of literal tags that are not valid JS
    /// identifiers in the shape of `Vec<(invalid, valid)>`.
    ///
    /// Example:
    ///
    /// ```
    /// vec![("a-b".into(), "_component0".into())]
    /// ```
    aliases: Vec<(String, String)>,
    /// Non-literal references in the shape of `Vec<(name, is_component)>`.
    ///
    /// Example:
    ///
    /// ```
    /// vec![("a".into(), false), ("a.b".into(), true)]
    /// ```
    // To do: add positional info later.
    references: Vec<(String, bool)>,
}

/// Scope (block or function/global).
#[derive(Debug, Clone)]
struct Scope {
    /// If this is a function (or global) scope, we track info.
    info: Option<Info>,
    /// Things that are defined in this scope.
    defined: Vec<String>,
}

/// Context.
#[derive(Debug, Default, Clone)]
struct State {
    /// List of current scopes.
    scopes: Vec<Scope>,
    /// Whether the user uses a provider.
    provider: bool,
    /// Whether a provider is referenced.
    create_provider_import: bool,
    /// Whether a missing component helper is referenced.
    ///
    /// When things are referenced that might not be defined, we reference a
    /// helper function to throw when they are missing.
    create_error_helper: bool,
}

impl State {
    /// Open a new scope.
    fn enter(&mut self, info: Option<Info>) {
        self.scopes.push(Scope {
            info,
            defined: vec![],
        });
    }

    /// Close the current scope.
    fn exit(&mut self) -> Scope {
        self.scopes.pop().expect("expected scope")
    }

    /// Close a function.
    fn exit_func(&mut self, func: Func) {
        let mut scope = self.exit();
        let mut defaults = vec![];
        let mut info = scope.info.take().unwrap();
        let mut index = 0;

        // Create defaults for tags.
        //
        // ```jsx
        // {h1: 'h1'}
        // ```
        while index < info.tags.len() {
            let name = &info.tags[index];

            defaults.push(swc_ecma_ast::PropOrSpread::Prop(Box::new(
                swc_ecma_ast::Prop::KeyValue(swc_ecma_ast::KeyValueProp {
                    key: if is_identifier_name(name) {
                        swc_ecma_ast::PropName::Ident(create_ident(name))
                    } else {
                        swc_ecma_ast::PropName::Str(swc_ecma_ast::Str {
                            value: name.clone().into(),
                            span: swc_common::DUMMY_SP,
                            raw: None,
                        })
                    },
                    value: Box::new(swc_ecma_ast::Expr::Lit(swc_ecma_ast::Lit::Str(
                        swc_ecma_ast::Str {
                            value: name.clone().into(),
                            span: swc_common::DUMMY_SP,
                            raw: None,
                        },
                    ))),
                }),
            )));

            index += 1;
        }

        let mut actual = info.components.split_off(0);
        let mut index = 0;

        // In some cases, a component is used directly (`<X>`) but it’s also
        // used as an object (`<X.Y>`).
        while index < info.objects.len() {
            if !actual.contains(&info.objects[index]) {
                actual.push(info.objects[index].clone());
            }
            index += 1;
        }

        let mut statements = vec![];

        if !defaults.is_empty() || !actual.is_empty() || !info.aliases.is_empty() {
            let mut parameters = vec![];

            // Use a provider, if configured.
            //
            // ```jsx
            // _provideComponents()
            // ```
            if self.provider {
                self.create_provider_import = true;
                parameters.push(swc_ecma_ast::Expr::Call(swc_ecma_ast::CallExpr {
                    callee: swc_ecma_ast::Callee::Expr(Box::new(create_ident_expression(
                        "_provideComponents",
                    ))),
                    args: vec![],
                    type_args: None,
                    span: swc_common::DUMMY_SP,
                }));
            }

            // Accept `components` as a prop if this is the `MDXContent` or
            // `_createMdxContent` function.
            //
            // ```jsx
            // props.components
            // ```
            if is_props_receiving_fn(&info.name) {
                parameters.push(swc_ecma_ast::Expr::Member(swc_ecma_ast::MemberExpr {
                    obj: Box::new(create_ident_expression("props")),
                    prop: swc_ecma_ast::MemberProp::Ident(create_ident("components")),
                    span: swc_common::DUMMY_SP,
                }));
            }

            // Inject an object at the start, when:
            // - there are defaults,
            // - there are two sources
            //
            // ```jsx
            // (_provideComponents(), props.components)
            // ()
            // ```
            //
            // To:
            //
            // ```jsx
            // ({}, _provideComponents(), props.components)
            // ({h1: 'h1'})
            // ```
            if !defaults.is_empty() || parameters.len() > 1 {
                parameters.insert(
                    0,
                    swc_ecma_ast::Expr::Object(swc_ecma_ast::ObjectLit {
                        props: defaults,
                        span: swc_common::DUMMY_SP,
                    }),
                );
            }

            // Merge things and prevent errors.
            //
            // ```jsx
            // {}, _provideComponents(), props.components
            // props.components
            // _provideComponents()
            // ```
            //
            // To:
            //
            // ```jsx
            // Object.assign({}, _provideComponents(), props.components)
            // props.components || {}
            // _provideComponents()
            // ```
            let mut components_init = if parameters.len() > 1 {
                let mut args = vec![];
                parameters.reverse();
                while let Some(param) = parameters.pop() {
                    args.push(swc_ecma_ast::ExprOrSpread {
                        spread: None,
                        expr: Box::new(param),
                    });
                }
                swc_ecma_ast::Expr::Call(swc_ecma_ast::CallExpr {
                    callee: swc_ecma_ast::Callee::Expr(Box::new(swc_ecma_ast::Expr::Member(
                        swc_ecma_ast::MemberExpr {
                            obj: Box::new(create_ident_expression("Object")),
                            prop: swc_ecma_ast::MemberProp::Ident(create_ident("assign")),
                            span: swc_common::DUMMY_SP,
                        },
                    ))),
                    args,
                    type_args: None,
                    span: swc_common::DUMMY_SP,
                })
            } else {
                // Always one.
                let param = parameters.pop().unwrap();

                if let swc_ecma_ast::Expr::Member(_) = param {
                    create_binary_expression(
                        vec![
                            param,
                            swc_ecma_ast::Expr::Object(swc_ecma_ast::ObjectLit {
                                props: vec![],
                                span: swc_common::DUMMY_SP,
                            }),
                        ],
                        swc_ecma_ast::BinaryOp::LogicalOr,
                    )
                } else {
                    param
                }
            };

            // Add components to scope.
            //
            // For `['MyComponent', 'MDXLayout']` this generates:
            //
            // ```js
            // const {MyComponent, wrapper: MDXLayout} = _components
            // ```
            //
            // Note that MDXLayout is special as it’s taken from
            // `_components.wrapper`.
            let components_pattern = if actual.is_empty() {
                None
            } else {
                let mut props = vec![];
                actual.reverse();
                while let Some(key) = actual.pop() {
                    // `wrapper: MDXLayout`
                    if key == "MDXLayout" {
                        props.push(swc_ecma_ast::ObjectPatProp::KeyValue(
                            swc_ecma_ast::KeyValuePatProp {
                                key: swc_ecma_ast::PropName::Ident(create_ident("wrapper")),
                                value: Box::new(swc_ecma_ast::Pat::Ident(
                                    swc_ecma_ast::BindingIdent {
                                        id: create_ident(&key),
                                        type_ann: None,
                                    },
                                )),
                            },
                        ))
                    }
                    // `MyComponent`
                    else {
                        props.push(swc_ecma_ast::ObjectPatProp::Assign(
                            swc_ecma_ast::AssignPatProp {
                                key: create_ident(&key),
                                value: None,
                                span: swc_common::DUMMY_SP,
                            },
                        ))
                    }
                }

                Some(swc_ecma_ast::ObjectPat {
                    props,
                    optional: false,
                    span: swc_common::DUMMY_SP,
                    type_ann: None,
                })
            };

            let mut declarators = vec![];

            // If there are tags, they take them from `_components`, so we need
            // to make it defined.
            if !info.tags.is_empty() {
                declarators.push(swc_ecma_ast::VarDeclarator {
                    span: swc_common::DUMMY_SP,
                    name: swc_ecma_ast::Pat::Ident(swc_ecma_ast::BindingIdent {
                        id: create_ident("_components"),
                        type_ann: None,
                    }),
                    init: Some(Box::new(components_init)),
                    definite: false,
                });
                components_init = create_ident_expression("_components");
            }

            // For JSX IDs that can’t be represented as JavaScript IDs (as in,
            // those with dashes, such as `custom-element`), we generated a
            // separate variable that is a valid JS ID (such as `_component0`),
            // and here we take it from components:
            // ```js
            // const _component0 = _components['custom-element']
            // ```
            if !info.aliases.is_empty() {
                info.aliases.reverse();

                while let Some((id, name)) = info.aliases.pop() {
                    declarators.push(swc_ecma_ast::VarDeclarator {
                        span: swc_common::DUMMY_SP,
                        name: swc_ecma_ast::Pat::Ident(swc_ecma_ast::BindingIdent {
                            id: create_ident(&name),
                            type_ann: None,
                        }),
                        init: Some(Box::new(swc_ecma_ast::Expr::Member(
                            swc_ecma_ast::MemberExpr {
                                obj: Box::new(create_ident_expression("_components")),
                                prop: swc_ecma_ast::MemberProp::Computed(
                                    swc_ecma_ast::ComputedPropName {
                                        expr: Box::new(swc_ecma_ast::Expr::Lit(
                                            swc_ecma_ast::Lit::Str(swc_ecma_ast::Str {
                                                value: id.into(),
                                                span: swc_common::DUMMY_SP,
                                                raw: None,
                                            }),
                                        )),
                                        span: swc_common::DUMMY_SP,
                                    },
                                ),
                                span: swc_common::DUMMY_SP,
                            },
                        ))),
                        definite: false,
                    });
                }
            }

            if let Some(pat) = components_pattern {
                declarators.push(swc_ecma_ast::VarDeclarator {
                    name: swc_ecma_ast::Pat::Object(pat),
                    init: Some(Box::new(components_init)),
                    span: swc_common::DUMMY_SP,
                    definite: false,
                });
            }

            // Add the variable declaration.
            statements.push(swc_ecma_ast::Stmt::Decl(swc_ecma_ast::Decl::Var(Box::new(
                swc_ecma_ast::VarDecl {
                    kind: swc_ecma_ast::VarDeclKind::Const,
                    decls: declarators,
                    span: swc_common::DUMMY_SP,
                    declare: false,
                },
            ))));
        }

        // Add checks at runtime to verify that object/components are passed.
        //
        // ```js
        // if (!a) _missingMdxReference("a", false);
        // if (!a.b) _missingMdxReference("a.b", true);
        // ```
        for (id, component) in info.references {
            self.create_error_helper = true;
            statements.push(swc_ecma_ast::Stmt::If(swc_ecma_ast::IfStmt {
                test: Box::new(swc_ecma_ast::Expr::Unary(swc_ecma_ast::UnaryExpr {
                    op: swc_ecma_ast::UnaryOp::Bang,
                    arg: Box::new(create_member_expression(&id)),
                    span: swc_common::DUMMY_SP,
                })),
                cons: Box::new(swc_ecma_ast::Stmt::Expr(swc_ecma_ast::ExprStmt {
                    span: swc_common::DUMMY_SP,
                    expr: Box::new(swc_ecma_ast::Expr::Call(swc_ecma_ast::CallExpr {
                        callee: swc_ecma_ast::Callee::Expr(Box::new(create_ident_expression(
                            "_missingMdxReference",
                        ))),
                        args: vec![
                            swc_ecma_ast::ExprOrSpread {
                                spread: None,
                                expr: Box::new(swc_ecma_ast::Expr::Lit(swc_ecma_ast::Lit::Str(
                                    swc_ecma_ast::Str {
                                        value: id.into(),
                                        span: swc_common::DUMMY_SP,
                                        raw: None,
                                    },
                                ))),
                            },
                            swc_ecma_ast::ExprOrSpread {
                                spread: None,
                                expr: Box::new(swc_ecma_ast::Expr::Lit(swc_ecma_ast::Lit::Bool(
                                    swc_ecma_ast::Bool {
                                        value: component,
                                        span: swc_common::DUMMY_SP,
                                    },
                                ))),
                            },
                        ],
                        type_args: None,
                        span: swc_common::DUMMY_SP,
                    })),
                })),
                alt: None,
                span: swc_common::DUMMY_SP,
            }));
        }

        // Add statements to functions.
        if !statements.is_empty() {
            let mut body: &mut swc_ecma_ast::BlockStmt = match func {
                Func::Expr(expr) => {
                    if expr.function.body.is_none() {
                        expr.function.body = Some(swc_ecma_ast::BlockStmt {
                            stmts: vec![],
                            span: swc_common::DUMMY_SP,
                        });
                    }
                    expr.function.body.as_mut().unwrap()
                }
                Func::Decl(decl) => {
                    if decl.function.body.is_none() {
                        decl.function.body = Some(swc_ecma_ast::BlockStmt {
                            stmts: vec![],
                            span: swc_common::DUMMY_SP,
                        });
                    }
                    decl.function.body.as_mut().unwrap()
                }
                Func::Arrow(arr) => {
                    if let swc_ecma_ast::BlockStmtOrExpr::Expr(expr) = &mut arr.body {
                        arr.body =
                            swc_ecma_ast::BlockStmtOrExpr::BlockStmt(swc_ecma_ast::BlockStmt {
                                stmts: vec![swc_ecma_ast::Stmt::Return(swc_ecma_ast::ReturnStmt {
                                    // To do: figure out non-clone.
                                    arg: Some(expr.clone()),
                                    span: swc_common::DUMMY_SP,
                                })],
                                span: swc_common::DUMMY_SP,
                            });
                    }
                    arr.body.as_mut_block_stmt().unwrap()
                }
            };

            statements.append(&mut body.stmts.split_off(0));
            body.stmts = statements;
        }
    }

    /// Get the current function scope.
    fn current_fn_scope_mut(&mut self) -> &mut Scope {
        let mut index = self.scopes.len();

        while index > 0 {
            index -= 1;
            if self.scopes[index].info.is_some() {
                return &mut self.scopes[index];
            }
        }

        unreachable!("expected scope")
    }

    /// Get the current scope.
    fn current_scope_mut(&mut self) -> &mut Scope {
        self.scopes.last_mut().expect("expected scope")
    }

    /// Get the top-level scope’s info.
    fn current_top_level_info(&self) -> Option<&Info> {
        if let Some(scope) = self.scopes.get(1) {
            scope.info.as_ref()
        } else {
            None
        }
    }
    /// Get the top-level scope’s info, mutably.
    fn current_top_level_info_mut(&mut self) -> Option<&mut Info> {
        if let Some(scope) = self.scopes.get_mut(1) {
            scope.info.as_mut()
        } else {
            None
        }
    }

    /// Check if `id` is in scope.
    fn in_scope(&self, id: &String) -> bool {
        let mut index = self.scopes.len();

        while index > 0 {
            index -= 1;
            if self.scopes[index].defined.contains(id) {
                return true;
            }
        }

        false
    }

    /// Add an identifier to a scope.
    fn add_id(&mut self, id: String, block: bool) {
        let scope = if block {
            self.current_scope_mut()
        } else {
            self.current_fn_scope_mut()
        };
        scope.defined.push(id);
    }

    // Add a pattern to a scope.
    fn add_pat(&mut self, pat: &swc_ecma_ast::Pat, block: bool) {
        match pat {
            // `x`
            swc_ecma_ast::Pat::Ident(d) => self.add_id(d.id.sym.to_string(), block),
            // `...x`
            swc_ecma_ast::Pat::Array(d) => {
                let mut index = 0;
                while index < d.elems.len() {
                    if let Some(d) = &d.elems[index] {
                        self.add_pat(d, block);
                    }
                    index += 1;
                }
            }
            // `...x`
            swc_ecma_ast::Pat::Rest(d) => self.add_pat(&d.arg, block),
            // `{x=y}`
            swc_ecma_ast::Pat::Assign(d) => self.add_pat(&d.left, block),
            swc_ecma_ast::Pat::Object(d) => {
                let mut index = 0;
                while index < d.props.len() {
                    match &d.props[index] {
                        // `{...x}`
                        swc_ecma_ast::ObjectPatProp::Rest(d) => {
                            self.add_pat(&d.arg, block);
                        }
                        // `{key: value}`
                        swc_ecma_ast::ObjectPatProp::KeyValue(d) => {
                            self.add_pat(&d.value, block);
                        }
                        // `{key}` or `{key = value}`
                        swc_ecma_ast::ObjectPatProp::Assign(d) => {
                            self.add_id(d.key.to_string(), block);
                        }
                    }
                    index += 1;
                }
            }
            // Ignore `Invalid` / `Expr`.
            _ => {}
        }
    }
}

impl VisitMut for State {
    noop_visit_mut_type!();

    /// Rewrite JSX identifiers.
    fn visit_mut_jsx_element(&mut self, node: &mut swc_ecma_ast::JSXElement) {
        // If there is a top-level, non-global, scope which is a function.
        if let Some(info) = self.current_top_level_info() {
            // Rewrite only if we can rewrite.
            if is_props_receiving_fn(&info.name) || self.provider {
                match &node.opening.name {
                    // `<x.y>`, `<Foo.Bar>`, `<x.y.z>`.
                    swc_ecma_ast::JSXElementName::JSXMemberExpr(d) => {
                        let mut ids = vec![];
                        let mut mem = d;
                        loop {
                            ids.push(mem.prop.sym.to_string());
                            match &mem.obj {
                                swc_ecma_ast::JSXObject::Ident(d) => {
                                    ids.push(d.sym.to_string());
                                    break;
                                }
                                swc_ecma_ast::JSXObject::JSXMemberExpr(d) => {
                                    mem = d;
                                }
                            }
                        }
                        ids.reverse();
                        let primary_id = ids.first().unwrap().clone();
                        let in_scope = self.in_scope(&primary_id);

                        if !in_scope {
                            let info_mut = self.current_top_level_info_mut().unwrap();

                            // To do: add positional info.
                            let mut index = 1;
                            while index <= ids.len() {
                                let full_id = ids[0..index].join(".");
                                let component = index == ids.len();
                                if let Some(reference) =
                                    info_mut.references.iter_mut().find(|d| d.0 == full_id)
                                {
                                    if component {
                                        reference.1 = true;
                                    }
                                } else {
                                    info_mut.references.push((full_id, component))
                                }
                                index += 1;
                            }

                            if !info_mut.objects.contains(&primary_id) {
                                info_mut.objects.push(primary_id);
                            }
                        }
                    }
                    // `<foo>`, `<Foo>`, `<$>`, `<_bar>`, `<a_b>`.
                    swc_ecma_ast::JSXElementName::Ident(d) => {
                        // If the name is a valid ES identifier, and it doesn’t
                        // start with a lowercase letter, it’s a component.
                        // For example, `$foo`, `_bar`, `Baz` are all component
                        // names.
                        // But `foo` and `b-ar` are tag names.
                        let id = d.sym.to_string();

                        if is_literal_name(&id) {
                            // To do: ignore explicit JSX?

                            let mut invalid = None;

                            let name = if is_identifier_name(&id) {
                                swc_ecma_ast::JSXElementName::JSXMemberExpr(
                                    swc_ecma_ast::JSXMemberExpr {
                                        obj: swc_ecma_ast::JSXObject::Ident(create_ident(
                                            "_components",
                                        )),
                                        prop: create_ident(&id),
                                    },
                                )
                            } else {
                                let name = if let Some(invalid_ref) =
                                    info.aliases.iter().find(|d| d.0 == id)
                                {
                                    invalid_ref.1.clone()
                                } else {
                                    let name = format!("_component{}", info.aliases.len());
                                    invalid = Some((id.clone(), name.clone()));
                                    name
                                };

                                swc_ecma_ast::JSXElementName::Ident(create_ident(&name))
                            };

                            let info_mut = self.current_top_level_info_mut().unwrap();

                            if !info_mut.tags.contains(&id) {
                                info_mut.tags.push(id);
                            }

                            if let Some(invalid) = invalid {
                                info_mut.aliases.push(invalid)
                            }

                            if let Some(closing) = node.closing.as_mut() {
                                closing.name = name.clone();
                            }

                            node.opening.name = name;
                        } else {
                            let mut is_layout = false;

                            // The MDXLayout is wrapped in a
                            if let Some(name) = &info.name {
                                if name == "MDXContent" && id == "MDXLayout" {
                                    is_layout = true;
                                }
                            }

                            if !self.in_scope(&id) {
                                let info_mut = self.current_top_level_info_mut().unwrap();

                                if !is_layout {
                                    if let Some(reference) =
                                        info_mut.references.iter_mut().find(|d| d.0 == id)
                                    {
                                        reference.1 = true;
                                    } else {
                                        info_mut.references.push((id.clone(), true))
                                    }
                                }

                                if !info_mut.components.contains(&id) {
                                    info_mut.components.push(id);
                                }
                            }
                        }
                    }
                    // `<xml:thing>`.
                    swc_ecma_ast::JSXElementName::JSXNamespacedName(_) => {
                        // Ignore.
                    }
                }
            }
        }

        node.visit_mut_children_with(self);
    }

    /// Add specifiers of import declarations.
    fn visit_mut_import_decl(&mut self, node: &mut swc_ecma_ast::ImportDecl) {
        let mut index = 0;
        while index < node.specifiers.len() {
            let ident = match &node.specifiers[index] {
                swc_ecma_ast::ImportSpecifier::Default(x) => &x.local.sym,
                swc_ecma_ast::ImportSpecifier::Namespace(x) => &x.local.sym,
                swc_ecma_ast::ImportSpecifier::Named(x) => &x.local.sym,
            };
            self.add_id(ident.to_string(), false);
            index += 1;
        }

        node.visit_mut_children_with(self);
    }

    /// Add patterns of variable declarations.
    fn visit_mut_var_decl(&mut self, node: &mut swc_ecma_ast::VarDecl) {
        let block = node.kind != swc_ecma_ast::VarDeclKind::Var;
        let mut index = 0;
        while index < node.decls.len() {
            self.add_pat(&node.decls[index].name, block);
            index += 1;
        }
        node.visit_mut_children_with(self);
    }

    /// Add identifier of class declaration.
    fn visit_mut_class_decl(&mut self, node: &mut swc_ecma_ast::ClassDecl) {
        self.add_id(node.ident.sym.to_string(), false);
        node.visit_mut_children_with(self);
    }

    /// On function declarations, add name, create scope, add parameters.
    fn visit_mut_fn_decl(&mut self, node: &mut swc_ecma_ast::FnDecl) {
        let id = node.ident.sym.to_string();
        self.add_id(id.clone(), false);
        self.enter(Some(Info {
            name: Some(id),
            ..Default::default()
        }));
        let mut index = 0;
        while index < node.function.params.len() {
            self.add_pat(&node.function.params[index].pat, false);
            index += 1;
        }
        node.visit_mut_children_with(self);
        // Rewrite.
        self.exit_func(Func::Decl(node));
    }

    /// On function expressions, add name, create scope, add parameters.
    fn visit_mut_fn_expr(&mut self, node: &mut swc_ecma_ast::FnExpr) {
        // Note: `periscopic` adds the ID to the newly generated scope, for
        // fn expressions.
        // That seems wrong?
        let name = if let Some(ident) = &node.ident {
            let id = ident.sym.to_string();
            self.add_id(id.clone(), false);
            Some(id)
        } else {
            None
        };

        self.enter(Some(Info {
            name,
            ..Default::default()
        }));
        let mut index = 0;
        while index < node.function.params.len() {
            self.add_pat(&node.function.params[index].pat, false);
            index += 1;
        }
        node.visit_mut_children_with(self);
        self.exit_func(Func::Expr(node));
    }

    /// On arrow functions, create scope, add parameters.
    fn visit_mut_arrow_expr(&mut self, node: &mut swc_ecma_ast::ArrowExpr) {
        self.enter(Some(Info::default()));
        let mut index = 0;
        while index < node.params.len() {
            self.add_pat(&node.params[index], false);
            index += 1;
        }
        node.visit_mut_children_with(self);
        self.exit_func(Func::Arrow(node));
    }

    // Blocks.
    // Not sure why `periscopic` only does `For`/`ForIn`/`ForOf`/`Block`.
    // I added `While`/`DoWhile` here just to be sure.
    // But there are more.
    /// On for statements, create scope.
    fn visit_mut_for_stmt(&mut self, node: &mut swc_ecma_ast::ForStmt) {
        self.enter(None);
        node.visit_mut_children_with(self);
        self.exit();
    }
    /// On for/in statements, create scope.
    fn visit_mut_for_in_stmt(&mut self, node: &mut swc_ecma_ast::ForInStmt) {
        self.enter(None);
        node.visit_mut_children_with(self);
        self.exit();
    }
    /// On for/of statements, create scope.
    fn visit_mut_for_of_stmt(&mut self, node: &mut swc_ecma_ast::ForOfStmt) {
        self.enter(None);
        node.visit_mut_children_with(self);
        self.exit();
    }
    /// On while statements, create scope.
    fn visit_mut_while_stmt(&mut self, node: &mut swc_ecma_ast::WhileStmt) {
        self.enter(None);
        node.visit_mut_children_with(self);
        self.exit();
    }
    /// On do/while statements, create scope.
    fn visit_mut_do_while_stmt(&mut self, node: &mut swc_ecma_ast::DoWhileStmt) {
        self.enter(None);
        node.visit_mut_children_with(self);
        self.exit();
    }
    /// On block statements, create scope.
    fn visit_mut_block_stmt(&mut self, node: &mut swc_ecma_ast::BlockStmt) {
        self.enter(None);
        node.visit_mut_children_with(self);
        self.exit();
    }

    /// On catch clauses, create scope, add param.
    fn visit_mut_catch_clause(&mut self, node: &mut swc_ecma_ast::CatchClause) {
        self.enter(None);
        if let Some(pat) = &node.param {
            self.add_pat(pat, true);
        }
        node.visit_mut_children_with(self);
        self.exit();
    }
}

/// Generate an import provider.
///
/// ```js
/// import { useMDXComponents as _provideComponents } from "x"
/// ```
fn create_import_provider(source: &str) -> swc_ecma_ast::ModuleItem {
    swc_ecma_ast::ModuleItem::ModuleDecl(swc_ecma_ast::ModuleDecl::Import(
        swc_ecma_ast::ImportDecl {
            specifiers: vec![swc_ecma_ast::ImportSpecifier::Named(
                swc_ecma_ast::ImportNamedSpecifier {
                    local: create_ident("_provideComponents"),
                    imported: Some(swc_ecma_ast::ModuleExportName::Ident(create_ident(
                        "useMDXComponents",
                    ))),
                    span: swc_common::DUMMY_SP,
                    is_type_only: false,
                },
            )],
            src: Box::new(swc_ecma_ast::Str {
                value: source.into(),
                span: swc_common::DUMMY_SP,
                raw: None,
            }),
            type_only: false,
            asserts: None,
            span: swc_common::DUMMY_SP,
        },
    ))
}

/// Generate an error helper.
///
/// ```js
/// function _missingMdxReference(id, component) {
///   throw new Error("Expected " + (component ? "component" : "object") + " `" + id + "` to be defined: you likely forgot to import, pass, or provide it.");
/// }
/// ```
fn create_error_helper() -> swc_ecma_ast::ModuleItem {
    let parameters = vec![
        swc_ecma_ast::Param {
            pat: swc_ecma_ast::Pat::Ident(swc_ecma_ast::BindingIdent {
                id: create_ident("id"),
                type_ann: None,
            }),
            decorators: vec![],
            span: swc_common::DUMMY_SP,
        },
        swc_ecma_ast::Param {
            pat: swc_ecma_ast::Pat::Ident(swc_ecma_ast::BindingIdent {
                id: create_ident("component"),
                type_ann: None,
            }),
            decorators: vec![],
            span: swc_common::DUMMY_SP,
        },
    ];

    let message = vec![
        swc_ecma_ast::Expr::Lit(swc_ecma_ast::Lit::Str(swc_ecma_ast::Str {
            value: "Expected ".into(),
            span: swc_common::DUMMY_SP,
            raw: None,
        })),
        // `component ? "component" : "object"`
        swc_ecma_ast::Expr::Paren(swc_ecma_ast::ParenExpr {
            expr: Box::new(swc_ecma_ast::Expr::Cond(swc_ecma_ast::CondExpr {
                test: Box::new(create_ident_expression("component")),
                cons: Box::new(swc_ecma_ast::Expr::Lit(swc_ecma_ast::Lit::Str(
                    swc_ecma_ast::Str {
                        value: "component".into(),
                        span: swc_common::DUMMY_SP,
                        raw: None,
                    },
                ))),
                alt: Box::new(swc_ecma_ast::Expr::Lit(swc_ecma_ast::Lit::Str(
                    swc_ecma_ast::Str {
                        value: "object".into(),
                        span: swc_common::DUMMY_SP,
                        raw: None,
                    },
                ))),
                span: swc_common::DUMMY_SP,
            })),
            span: swc_common::DUMMY_SP,
        }),
        swc_ecma_ast::Expr::Lit(swc_ecma_ast::Lit::Str(swc_ecma_ast::Str {
            value: " `".into(),
            span: swc_common::DUMMY_SP,
            raw: None,
        })),
        create_ident_expression("id"),
        swc_ecma_ast::Expr::Lit(swc_ecma_ast::Lit::Str(swc_ecma_ast::Str {
            value: "` to be defined: you likely forgot to import, pass, or provide it.".into(),
            span: swc_common::DUMMY_SP,
            raw: None,
        })),
    ];

    // To do: in development, add `place` param, and use the positional info.
    // Also, then, add file path.

    swc_ecma_ast::ModuleItem::Stmt(swc_ecma_ast::Stmt::Decl(swc_ecma_ast::Decl::Fn(
        swc_ecma_ast::FnDecl {
            ident: create_ident("_missingMdxReference"),
            declare: false,
            function: Box::new(swc_ecma_ast::Function {
                params: parameters,
                decorators: vec![],
                body: Some(swc_ecma_ast::BlockStmt {
                    stmts: vec![swc_ecma_ast::Stmt::Throw(swc_ecma_ast::ThrowStmt {
                        arg: Box::new(swc_ecma_ast::Expr::New(swc_ecma_ast::NewExpr {
                            callee: Box::new(create_ident_expression("Error")),
                            args: Some(vec![swc_ecma_ast::ExprOrSpread {
                                spread: None,
                                expr: Box::new(create_binary_expression(
                                    message,
                                    swc_ecma_ast::BinaryOp::Add,
                                )),
                            }]),
                            span: swc_common::DUMMY_SP,
                            type_args: None,
                        })),
                        span: swc_common::DUMMY_SP,
                    })],
                    span: swc_common::DUMMY_SP,
                }),
                is_generator: false,
                is_async: false,
                type_params: None,
                return_type: None,
                span: swc_common::DUMMY_SP,
            }),
        },
    )))
}

/// Generate a binary expression.
///
/// ```js
/// a + b + c
/// a || b
/// ```
fn create_binary_expression(
    mut exprs: Vec<swc_ecma_ast::Expr>,
    op: swc_ecma_ast::BinaryOp,
) -> swc_ecma_ast::Expr {
    exprs.reverse();

    let mut left = None;

    while let Some(right_expr) = exprs.pop() {
        left = Some(if let Some(left_expr) = left {
            swc_ecma_ast::Expr::Bin(swc_ecma_ast::BinExpr {
                left: Box::new(left_expr),
                right: Box::new(right_expr),
                op,
                span: swc_common::DUMMY_SP,
            })
        } else {
            right_expr
        });
    }

    left.expect("expected one or more expressions")
}

/// Generate a member expression.
///
/// ```js
/// a.b
/// a
/// ```
fn create_member_expression(name: &str) -> swc_ecma_ast::Expr {
    let bytes = name.as_bytes();
    let mut index = 0;
    let mut start = 0;
    let mut parts = vec![];

    while index < bytes.len() {
        if bytes[index] == b'.' {
            parts.push(&name[start..index]);
            start = index + 1;
        }

        index += 1;
    }

    if parts.len() > 1 {
        let mut member = swc_ecma_ast::MemberExpr {
            obj: Box::new(create_ident_expression(parts[0])),
            prop: swc_ecma_ast::MemberProp::Ident(create_ident(parts[1])),
            span: swc_common::DUMMY_SP,
        };
        let mut index = 2;
        while index < parts.len() {
            member = swc_ecma_ast::MemberExpr {
                obj: Box::new(swc_ecma_ast::Expr::Member(member)),
                prop: swc_ecma_ast::MemberProp::Ident(create_ident(parts[1])),
                span: swc_common::DUMMY_SP,
            };
            index += 1;
        }
        swc_ecma_ast::Expr::Member(member)
    } else {
        create_ident_expression(name)
    }
}

/// Generate an ident expression.
///
/// ```js
/// a
/// ```
fn create_ident_expression(sym: &str) -> swc_ecma_ast::Expr {
    swc_ecma_ast::Expr::Ident(create_ident(sym))
}

/// Generate an ident.
///
/// ```js
/// a
/// ```
fn create_ident(sym: &str) -> swc_ecma_ast::Ident {
    swc_ecma_ast::Ident {
        sym: sym.into(),
        optional: false,
        span: swc_common::DUMMY_SP,
    }
}

/// Check if this function is a props receiving component: it’s one of ours.
fn is_props_receiving_fn(name: &Option<String>) -> bool {
    if let Some(name) = name {
        name == "_createMdxContent" || name == "MDXContent"
    } else {
        false
    }
}

/// Check if a name is a literal tag name or an identifier to a component.
fn is_literal_name(name: &str) -> bool {
    matches!(name.as_bytes().first(), Some(b'a'..=b'z')) || !is_identifier_name(name)
}

// Check if a name is a valid identifier name.
fn is_identifier_name(name: &str) -> bool {
    for (index, char) in name.chars().enumerate() {
        if if index == 0 {
            !id_start(char)
        } else {
            !id_cont(char, false)
        } {
            return false;
        }
    }

    true
}
