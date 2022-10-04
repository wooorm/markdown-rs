extern crate swc_common;
extern crate swc_ecma_ast;
use crate::test_utils::to_swc::Program;

/// JSX runtimes.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub enum JsxRuntime {
    /// Automatic runtime.
    ///
    /// With the automatic runtime, some module is expected to exist somewhere.
    /// That modules is expected to expose a certain API.
    /// The compiler adds an import of that module and compiles JSX away to
    /// function calls that use that API.
    #[default]
    Automatic,
    /// Classic runtime.
    ///
    /// With the classic runtime, you define two values yourself in each file,
    /// which are expected to work a certain way.
    /// The compiler compiles JSX away to function calls using those two values.
    Classic,
}

/// Configuration.
#[derive(Debug, PartialEq, Eq)]
pub struct Options {
    /// Pragma for JSX (used in classic runtime).
    ///
    /// Default: `React.createElement`.
    pub pragma: Option<String>,
    /// Pragma for JSX fragments (used in classic runtime).
    ///
    /// Default: `React.Fragment`.
    pub pragma_frag: Option<String>,
    /// Where to import the identifier of `pragma` from (used in classic runtime).
    ///
    /// Default: `react`.
    pub pragma_import_source: Option<String>,
    /// Place to import automatic JSX runtimes from (used in automatic runtime).
    ///
    /// Default: `react`.
    pub jsx_import_source: Option<String>,
    /// JSX runtime to use.
    ///
    /// Default: `automatic`.
    pub jsx_runtime: Option<JsxRuntime>,
}

impl Default for Options {
    /// Use the automatic JSX runtime with React.
    fn default() -> Self {
        Self {
            pragma: None,
            pragma_frag: None,
            pragma_import_source: None,
            jsx_import_source: None,
            jsx_runtime: Some(JsxRuntime::default()),
        }
    }
}

#[allow(dead_code)]
pub fn to_document(mut program: Program, options: &Options) -> Result<Program, String> {
    // New body children.
    let mut replacements = vec![];

    // Inject JSX configuration comment.
    if let Some(runtime) = &options.jsx_runtime {
        let mut pragmas = vec![];
        let react = &"react".into();
        let create_element = &"React.createElement".into();
        let fragment = &"React.Fragment".into();

        if *runtime == JsxRuntime::Automatic {
            pragmas.push("@jsxRuntime automatic".into());
            pragmas.push(format!(
                "@jsxImportSource {}",
                if let Some(jsx_import_source) = &options.jsx_import_source {
                    jsx_import_source
                } else {
                    react
                }
            ));
        } else {
            pragmas.push("@jsxRuntime classic".into());
            pragmas.push(format!(
                "@jsx {}",
                if let Some(pragma) = &options.pragma {
                    pragma
                } else {
                    create_element
                }
            ));
            pragmas.push(format!(
                "@jsxFrag {}",
                if let Some(pragma_frag) = &options.pragma_frag {
                    pragma_frag
                } else {
                    fragment
                }
            ));
        }

        if !pragmas.is_empty() {
            program.comments.insert(
                0,
                swc_common::comments::Comment {
                    kind: swc_common::comments::CommentKind::Block,
                    text: pragmas.join(" ").into(),
                    span: swc_common::DUMMY_SP,
                },
            );
        }
    }

    // Inject an import in the classic runtime for the pragma (and presumably,
    // fragment).
    if options.jsx_runtime == Some(JsxRuntime::Classic) {
        let pragma = if let Some(pragma) = &options.pragma {
            pragma
        } else {
            "React"
        };
        let sym = pragma.split('.').next().expect("first item always exists");

        replacements.push(swc_ecma_ast::ModuleItem::ModuleDecl(
            swc_ecma_ast::ModuleDecl::Import(swc_ecma_ast::ImportDecl {
                specifiers: vec![swc_ecma_ast::ImportSpecifier::Named(
                    swc_ecma_ast::ImportNamedSpecifier {
                        local: swc_ecma_ast::Ident {
                            sym: sym.into(),
                            optional: false,
                            span: swc_common::DUMMY_SP,
                        },
                        imported: None,
                        span: swc_common::DUMMY_SP,
                        is_type_only: false,
                    },
                )],
                src: Box::new(swc_ecma_ast::Str {
                    value: (if let Some(source) = &options.pragma_import_source {
                        source.clone()
                    } else {
                        "react".into()
                    })
                    .into(),
                    span: swc_common::DUMMY_SP,
                    raw: None,
                }),
                type_only: false,
                asserts: None,
                span: swc_common::DUMMY_SP,
            }),
        ));
    }

    // Find the `export default`, the JSX expression, and leave the rest as it
    // is.
    let mut input = program.module.body.split_off(0);
    input.reverse();
    // To do: place position in this.
    let mut layout = false;
    let content = true;

    while let Some(module_item) = input.pop() {
        match module_item {
            // ```js
            // export default props => <>{props.children}</>
            // ```
            //
            // Treat it as an inline layout declaration.
            //
            // In estree, the below two are the same node (`ExportDefault`).
            swc_ecma_ast::ModuleItem::ModuleDecl(swc_ecma_ast::ModuleDecl::ExportDefaultDecl(
                decl,
            )) => {
                // To do: use positional info.
                if layout {
                    return Err("Cannot specify multiple layouts".into());
                }

                // To do: set positional info.
                layout = true;
                replacements.push(create_layout_decl(match decl.decl {
                    swc_ecma_ast::DefaultDecl::Class(cls) => swc_ecma_ast::Expr::Class(cls),
                    swc_ecma_ast::DefaultDecl::Fn(func) => swc_ecma_ast::Expr::Fn(func),
                    swc_ecma_ast::DefaultDecl::TsInterfaceDecl(_) => {
                        // To do: improved error? Not sure what a real example of this is?
                        unreachable!(
                            "Cannot use TypeScript interface declarations as default export in MDX"
                        )
                    }
                }));
            }
            swc_ecma_ast::ModuleItem::ModuleDecl(swc_ecma_ast::ModuleDecl::ExportDefaultExpr(
                expr,
            )) => {
                // To do: use positional info.
                if layout {
                    return Err("Cannot specify multiple layouts".into());
                }

                // To do: set positional info.
                layout = true;
                replacements.push(create_layout_decl(*expr.expr));
            }
            // ```js
            // export {a, b as c} from 'd'
            // export {a, b as c}
            // ```
            swc_ecma_ast::ModuleItem::ModuleDecl(swc_ecma_ast::ModuleDecl::ExportNamed(
                mut named_export,
            )) => {
                // SWC is currently crashing when generating code, w/o source
                // map, if an actual location is set on this node.
                named_export.span = swc_common::DUMMY_SP;

                let mut index = 0;
                let mut id = None;

                while index < named_export.specifiers.len() {
                    let mut take = false;
                    // Note: the `swc_ecma_ast::ExportSpecifier::Default`
                    // branch of this looks interesting, but as far as I
                    // understand it *is not* valid ES.
                    // `export a from 'b'` is a syntax error, even in SWC.
                    if let swc_ecma_ast::ExportSpecifier::Named(named) =
                        &named_export.specifiers[index]
                    {
                        if let Some(swc_ecma_ast::ModuleExportName::Ident(ident)) = &named.exported
                        {
                            if ident.sym.as_ref() == "default" {
                                // For some reason the AST supports strings
                                // instead of identifiers.
                                // Looks like some TC39 proposal. Ignore for now
                                // and only do things if this is an ID.
                                if let swc_ecma_ast::ModuleExportName::Ident(ident) = &named.orig {
                                    // To do: use positional info.
                                    if layout {
                                        return Err("Cannot specify multiple layouts".into());
                                    }
                                    // To do: set positional info.
                                    layout = true;
                                    take = true;
                                    id = Some(ident.clone());
                                }
                            }
                        }
                    }

                    if take {
                        named_export.specifiers.remove(index);
                    } else {
                        index += 1;
                    }
                }

                if let Some(id) = id {
                    let source = named_export.src.clone();

                    // If there was just a default export, we can drop the original node.
                    if !named_export.specifiers.is_empty() {
                        // Pass through.
                        replacements.push(swc_ecma_ast::ModuleItem::ModuleDecl(
                            swc_ecma_ast::ModuleDecl::ExportNamed(named_export),
                        ));
                    }

                    // It’s an `export {x} from 'y'`, so generate an import.
                    if let Some(source) = source {
                        replacements.push(swc_ecma_ast::ModuleItem::ModuleDecl(
                            swc_ecma_ast::ModuleDecl::Import(swc_ecma_ast::ImportDecl {
                                specifiers: vec![swc_ecma_ast::ImportSpecifier::Named(
                                    swc_ecma_ast::ImportNamedSpecifier {
                                        local: swc_ecma_ast::Ident {
                                            sym: "MDXLayout".into(),
                                            optional: false,
                                            span: swc_common::DUMMY_SP,
                                        },
                                        imported: Some(swc_ecma_ast::ModuleExportName::Ident(id)),
                                        span: swc_common::DUMMY_SP,
                                        is_type_only: false,
                                    },
                                )],
                                src: source,
                                type_only: false,
                                asserts: None,
                                span: swc_common::DUMMY_SP,
                            }),
                        ))
                    }
                    // It’s an `export {x}`, so generate a variable declaration.
                    else {
                        replacements.push(create_layout_decl(swc_ecma_ast::Expr::Ident(id)));
                    }
                } else {
                    // Pass through.
                    replacements.push(swc_ecma_ast::ModuleItem::ModuleDecl(
                        swc_ecma_ast::ModuleDecl::ExportNamed(named_export),
                    ));
                }
            }
            swc_ecma_ast::ModuleItem::ModuleDecl(swc_ecma_ast::ModuleDecl::Import(_))
            | swc_ecma_ast::ModuleItem::ModuleDecl(swc_ecma_ast::ModuleDecl::ExportDecl(_))
            | swc_ecma_ast::ModuleItem::ModuleDecl(swc_ecma_ast::ModuleDecl::ExportAll(_))
            | swc_ecma_ast::ModuleItem::ModuleDecl(swc_ecma_ast::ModuleDecl::TsImportEquals(_))
            | swc_ecma_ast::ModuleItem::ModuleDecl(swc_ecma_ast::ModuleDecl::TsExportAssignment(
                _,
            ))
            | swc_ecma_ast::ModuleItem::ModuleDecl(swc_ecma_ast::ModuleDecl::TsNamespaceExport(
                _,
            )) => {
                // Pass through.
                replacements.push(module_item);
            }
            swc_ecma_ast::ModuleItem::Stmt(swc_ecma_ast::Stmt::Expr(expr_stmt)) => {
                match *expr_stmt.expr {
                    swc_ecma_ast::Expr::JSXElement(elem) => {
                        replacements.append(&mut create_mdx_content(
                            Some(swc_ecma_ast::Expr::JSXElement(elem)),
                            layout,
                        ));
                    }
                    swc_ecma_ast::Expr::JSXFragment(mut frag) => {
                        // Unwrap if possible.
                        if frag.children.len() == 1 {
                            let item = frag.children.pop().unwrap();

                            if let swc_ecma_ast::JSXElementChild::JSXElement(elem) = item {
                                replacements.append(&mut create_mdx_content(
                                    Some(swc_ecma_ast::Expr::JSXElement(elem)),
                                    layout,
                                ));
                                continue;
                            }

                            frag.children.push(item)
                        }

                        replacements.append(&mut create_mdx_content(
                            Some(swc_ecma_ast::Expr::JSXFragment(frag)),
                            layout,
                        ));
                    }
                    _ => {
                        // Pass through.
                        replacements.push(swc_ecma_ast::ModuleItem::Stmt(
                            swc_ecma_ast::Stmt::Expr(expr_stmt),
                        ));
                    }
                }
            }
            swc_ecma_ast::ModuleItem::Stmt(stmt) => {
                replacements.push(swc_ecma_ast::ModuleItem::Stmt(stmt));
            }
        }
    }

    // Generate an empty component.
    if !content {
        replacements.append(&mut create_mdx_content(None, layout));
    }

    // ```jsx
    // export default MDXContent
    // ```
    replacements.push(swc_ecma_ast::ModuleItem::ModuleDecl(
        swc_ecma_ast::ModuleDecl::ExportDefaultExpr(swc_ecma_ast::ExportDefaultExpr {
            expr: Box::new(swc_ecma_ast::Expr::Ident(swc_ecma_ast::Ident {
                sym: "MDXContent".into(),
                optional: false,
                span: swc_common::DUMMY_SP,
            })),
            span: swc_common::DUMMY_SP,
        }),
    ));

    program.module.body = replacements;

    Ok(program)
}

/// Create a content component.
fn create_mdx_content(
    expr: Option<swc_ecma_ast::Expr>,
    has_internal_layout: bool,
) -> Vec<swc_ecma_ast::ModuleItem> {
    // ```jsx
    // <MDXLayout {...props}>xxx</MDXLayout>
    // ```
    let mut result = swc_ecma_ast::Expr::JSXElement(Box::new(swc_ecma_ast::JSXElement {
        opening: swc_ecma_ast::JSXOpeningElement {
            name: swc_ecma_ast::JSXElementName::Ident(swc_ecma_ast::Ident {
                sym: "MDXLayout".into(),
                optional: false,
                span: swc_common::DUMMY_SP,
            }),
            attrs: vec![swc_ecma_ast::JSXAttrOrSpread::SpreadElement(
                swc_ecma_ast::SpreadElement {
                    dot3_token: swc_common::DUMMY_SP,
                    expr: Box::new(swc_ecma_ast::Expr::Ident(swc_ecma_ast::Ident {
                        sym: "props".into(),
                        optional: false,
                        span: swc_common::DUMMY_SP,
                    })),
                },
            )],
            self_closing: false,
            type_args: None,
            span: swc_common::DUMMY_SP,
        },
        closing: Some(swc_ecma_ast::JSXClosingElement {
            name: swc_ecma_ast::JSXElementName::Ident(swc_ecma_ast::Ident {
                sym: "MDXLayout".into(),
                optional: false,
                span: swc_common::DUMMY_SP,
            }),
            span: swc_common::DUMMY_SP,
        }),
        // ```jsx
        // <_createMdxContent {...props} />
        // ```
        children: vec![swc_ecma_ast::JSXElementChild::JSXElement(Box::new(
            swc_ecma_ast::JSXElement {
                opening: swc_ecma_ast::JSXOpeningElement {
                    name: swc_ecma_ast::JSXElementName::Ident(swc_ecma_ast::Ident {
                        sym: "_createMdxContent".into(),
                        optional: false,
                        span: swc_common::DUMMY_SP,
                    }),
                    attrs: vec![swc_ecma_ast::JSXAttrOrSpread::SpreadElement(
                        swc_ecma_ast::SpreadElement {
                            dot3_token: swc_common::DUMMY_SP,
                            expr: Box::new(swc_ecma_ast::Expr::Ident(swc_ecma_ast::Ident {
                                sym: "props".into(),
                                optional: false,
                                span: swc_common::DUMMY_SP,
                            })),
                        },
                    )],
                    self_closing: true,
                    type_args: None,
                    span: swc_common::DUMMY_SP,
                },
                closing: None,
                children: vec![],
                span: swc_common::DUMMY_SP,
            },
        ))],
        span: swc_common::DUMMY_SP,
    }));

    if !has_internal_layout {
        // ```jsx
        // MDXLayout ? <MDXLayout>xxx</MDXLayout> : _createMdxContent(props)
        // ```
        result = swc_ecma_ast::Expr::Cond(swc_ecma_ast::CondExpr {
            test: Box::new(swc_ecma_ast::Expr::Ident(swc_ecma_ast::Ident {
                sym: "MDXLayout".into(),
                optional: false,
                span: swc_common::DUMMY_SP,
            })),
            cons: Box::new(result),
            alt: Box::new(swc_ecma_ast::Expr::Call(swc_ecma_ast::CallExpr {
                callee: swc_ecma_ast::Callee::Expr(Box::new(swc_ecma_ast::Expr::Ident(
                    swc_ecma_ast::Ident {
                        sym: "_createMdxContent".into(),
                        optional: false,
                        span: swc_common::DUMMY_SP,
                    },
                ))),
                args: vec![swc_ecma_ast::ExprOrSpread {
                    spread: None,
                    expr: Box::new(swc_ecma_ast::Expr::Ident(swc_ecma_ast::Ident {
                        sym: "props".into(),
                        optional: false,
                        span: swc_common::DUMMY_SP,
                    })),
                }],
                type_args: None,
                span: swc_common::DUMMY_SP,
            })),
            span: swc_common::DUMMY_SP,
        });
    }

    // ```jsx
    // function _createMdxContent(props) {
    //   return xxx
    // }
    // ```
    let create_mdx_content = swc_ecma_ast::ModuleItem::Stmt(swc_ecma_ast::Stmt::Decl(
        swc_ecma_ast::Decl::Fn(swc_ecma_ast::FnDecl {
            ident: swc_ecma_ast::Ident {
                sym: "_createMdxContent".into(),
                optional: false,
                span: swc_common::DUMMY_SP,
            },
            declare: false,
            function: Box::new(swc_ecma_ast::Function {
                params: vec![swc_ecma_ast::Param {
                    pat: swc_ecma_ast::Pat::Ident(swc_ecma_ast::BindingIdent {
                        id: swc_ecma_ast::Ident {
                            sym: "props".into(),
                            optional: false,
                            span: swc_common::DUMMY_SP,
                        },
                        type_ann: None,
                    }),
                    decorators: vec![],
                    span: swc_common::DUMMY_SP,
                }],
                decorators: vec![],
                body: Some(swc_ecma_ast::BlockStmt {
                    stmts: vec![swc_ecma_ast::Stmt::Return(swc_ecma_ast::ReturnStmt {
                        arg: Some(Box::new(expr.unwrap_or({
                            swc_ecma_ast::Expr::Lit(swc_ecma_ast::Lit::Null(swc_ecma_ast::Null {
                                span: swc_common::DUMMY_SP,
                            }))
                        }))),
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
        }),
    ));

    // ```jsx
    // function MDXContent(props = {}) {
    //   return <MDXLayout>xxx</MDXLayout>
    // }
    // ```
    let mdx_content = swc_ecma_ast::ModuleItem::Stmt(swc_ecma_ast::Stmt::Decl(
        swc_ecma_ast::Decl::Fn(swc_ecma_ast::FnDecl {
            ident: swc_ecma_ast::Ident {
                sym: "MDXContent".into(),
                optional: false,
                span: swc_common::DUMMY_SP,
            },
            declare: false,
            function: Box::new(swc_ecma_ast::Function {
                params: vec![swc_ecma_ast::Param {
                    pat: swc_ecma_ast::Pat::Assign(swc_ecma_ast::AssignPat {
                        left: Box::new(swc_ecma_ast::Pat::Ident(swc_ecma_ast::BindingIdent {
                            id: swc_ecma_ast::Ident {
                                sym: "props".into(),
                                optional: false,
                                span: swc_common::DUMMY_SP,
                            },
                            type_ann: None,
                        })),
                        right: Box::new(swc_ecma_ast::Expr::Object(swc_ecma_ast::ObjectLit {
                            props: vec![],
                            span: swc_common::DUMMY_SP,
                        })),
                        span: swc_common::DUMMY_SP,
                        type_ann: None,
                    }),
                    decorators: vec![],
                    span: swc_common::DUMMY_SP,
                }],
                decorators: vec![],
                body: Some(swc_ecma_ast::BlockStmt {
                    stmts: vec![swc_ecma_ast::Stmt::Return(swc_ecma_ast::ReturnStmt {
                        arg: Some(Box::new(result)),
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
        }),
    ));

    vec![create_mdx_content, mdx_content]
}

/// Create a layout, inside the document.
fn create_layout_decl(expr: swc_ecma_ast::Expr) -> swc_ecma_ast::ModuleItem {
    // ```jsx
    // const MDXLayout = xxx
    // ```
    swc_ecma_ast::ModuleItem::Stmt(swc_ecma_ast::Stmt::Decl(swc_ecma_ast::Decl::Var(Box::new(
        swc_ecma_ast::VarDecl {
            kind: swc_ecma_ast::VarDeclKind::Const,
            declare: false,
            decls: vec![swc_ecma_ast::VarDeclarator {
                name: swc_ecma_ast::Pat::Ident(swc_ecma_ast::BindingIdent {
                    id: swc_ecma_ast::Ident {
                        sym: "MDXLayout".into(),
                        optional: false,
                        span: swc_common::DUMMY_SP,
                    },
                    type_ann: None,
                }),
                init: Some(Box::new(expr)),
                span: swc_common::DUMMY_SP,
                definite: false,
            }],
            span: swc_common::DUMMY_SP,
        },
    ))))
}
