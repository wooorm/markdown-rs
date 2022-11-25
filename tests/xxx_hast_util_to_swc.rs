mod test_utils;
use pretty_assertions::assert_eq;
use test_utils::{
    hast,
    hast_util_to_swc::{hast_util_to_swc, Program},
    swc::serialize,
};

#[test]
fn hast_util_to_swc_test() -> Result<(), String> {
    let comment_ast = hast_util_to_swc(
        &hast::Node::Comment(hast::Comment {
            value: "a".into(),
            position: None,
        }),
        None,
        None,
    )?;

    assert_eq!(
        comment_ast,
        Program {
            path: None,
            module: swc_ecma_ast::Module {
                shebang: None,
                body: vec![swc_ecma_ast::ModuleItem::Stmt(swc_ecma_ast::Stmt::Expr(
                    swc_ecma_ast::ExprStmt {
                        expr: Box::new(swc_ecma_ast::Expr::JSXFragment(
                            swc_ecma_ast::JSXFragment {
                                opening: swc_ecma_ast::JSXOpeningFragment {
                                    span: swc_common::DUMMY_SP,
                                },
                                closing: swc_ecma_ast::JSXClosingFragment {
                                    span: swc_common::DUMMY_SP,
                                },
                                children: vec![swc_ecma_ast::JSXElementChild::JSXExprContainer(
                                    swc_ecma_ast::JSXExprContainer {
                                        expr: swc_ecma_ast::JSXExpr::JSXEmptyExpr(
                                            swc_ecma_ast::JSXEmptyExpr {
                                                span: swc_common::DUMMY_SP,
                                            }
                                        ),
                                        span: swc_common::DUMMY_SP,
                                    },
                                )],
                                span: swc_common::DUMMY_SP,
                            }
                        )),
                        span: swc_common::DUMMY_SP,
                    },
                ))],
                span: swc_common::DUMMY_SP,
            },
            comments: vec![swc_common::comments::Comment {
                kind: swc_common::comments::CommentKind::Block,
                text: "a".into(),
                span: swc_common::DUMMY_SP,
            }],
        },
        "should support a `Comment`",
    );

    assert_eq!(
        serialize(&comment_ast.module),
        // To do: comment should be in this.
        "<>{}</>;\n",
        "should support a `Comment` (serialize)",
    );

    let element_ast = hast_util_to_swc(
        &hast::Node::Element(hast::Element {
            tag_name: "a".into(),
            properties: vec![(
                "className".into(),
                hast::PropertyValue::SpaceSeparated(vec!["b".into()]),
            )],
            children: vec![],
            position: None,
        }),
        None,
        None,
    )?;

    assert_eq!(
        element_ast,
        Program {
            path: None,
            module: swc_ecma_ast::Module {
                shebang: None,
                body: vec![swc_ecma_ast::ModuleItem::Stmt(swc_ecma_ast::Stmt::Expr(
                    swc_ecma_ast::ExprStmt {
                        expr: Box::new(swc_ecma_ast::Expr::JSXElement(Box::new(
                            swc_ecma_ast::JSXElement {
                                opening: swc_ecma_ast::JSXOpeningElement {
                                    name: swc_ecma_ast::JSXElementName::Ident(
                                        swc_ecma_ast::Ident {
                                            span: swc_common::DUMMY_SP,
                                            sym: "a".into(),
                                            optional: false,
                                        }
                                    ),
                                    attrs: vec![swc_ecma_ast::JSXAttrOrSpread::JSXAttr(
                                        swc_ecma_ast::JSXAttr {
                                            name: swc_ecma_ast::JSXAttrName::Ident(
                                                swc_ecma_ast::Ident {
                                                    sym: "className".into(),
                                                    span: swc_common::DUMMY_SP,
                                                    optional: false,
                                                }
                                            ),
                                            value: Some(swc_ecma_ast::JSXAttrValue::Lit(
                                                swc_ecma_ast::Lit::Str(swc_ecma_ast::Str {
                                                    value: "b".into(),
                                                    span: swc_common::DUMMY_SP,
                                                    raw: None,
                                                })
                                            )),
                                            span: swc_common::DUMMY_SP,
                                        },
                                    )],
                                    self_closing: true,
                                    type_args: None,
                                    span: swc_common::DUMMY_SP,
                                },
                                closing: None,
                                children: vec![],
                                span: swc_common::DUMMY_SP,
                            }
                        ))),
                        span: swc_common::DUMMY_SP,
                    },
                ))],
                span: swc_common::DUMMY_SP,
            },
            comments: vec![],
        },
        "should support an `Element`",
    );

    assert_eq!(
        serialize(&element_ast.module),
        "<a className=\"b\"/>;\n",
        "should support an `Element` (serialize)",
    );

    assert_eq!(
        serialize(
            &hast_util_to_swc(
                &hast::Node::Element(hast::Element {
                    tag_name: "a".into(),
                    properties: vec![],
                    children: vec![hast::Node::Text(hast::Text {
                        value: "a".into(),
                        position: None,
                    })],
                    position: None,
                }),
                None,
                None
            )?
            .module
        ),
        "<a >{\"a\"}</a>;\n",
        "should support an `Element` w/ children",
    );

    assert_eq!(
        serialize(
            &hast_util_to_swc(
                &hast::Node::Element(hast::Element {
                    tag_name: "a".into(),
                    properties: vec![("b".into(), hast::PropertyValue::String("c".into()),)],
                    children: vec![],
                    position: None,
                }),
                None,
                None
            )?
            .module
        ),
        "<a b=\"c\"/>;\n",
        "should support an `Element` w/ a string attribute",
    );

    assert_eq!(
        serialize(
            &hast_util_to_swc(
                &hast::Node::Element(hast::Element {
                    tag_name: "a".into(),
                    properties: vec![("b".into(), hast::PropertyValue::Boolean(true),)],
                    children: vec![],
                    position: None,
                }),
                None,
                None
            )?
            .module
        ),
        "<a b/>;\n",
        "should support an `Element` w/ a boolean (true) attribute",
    );

    assert_eq!(
        serialize(
            &hast_util_to_swc(
                &hast::Node::Element(hast::Element {
                    tag_name: "a".into(),
                    properties: vec![("b".into(), hast::PropertyValue::Boolean(false),)],
                    children: vec![],
                    position: None,
                }),
                None,
                None
            )?
            .module
        ),
        "<a />;\n",
        "should support an `Element` w/ a boolean (false) attribute",
    );

    assert_eq!(
        serialize(
            &hast_util_to_swc(
                &hast::Node::Element(hast::Element {
                    tag_name: "a".into(),
                    properties: vec![(
                        "b".into(),
                        hast::PropertyValue::CommaSeparated(vec!["c".into(), "d".into()]),
                    )],
                    children: vec![],
                    position: None,
                }),
                None,
                None
            )?
            .module
        ),
        "<a b=\"c, d\"/>;\n",
        "should support an `Element` w/ a comma-separated attribute",
    );

    assert_eq!(
        serialize(
            &hast_util_to_swc(
                &hast::Node::Element(hast::Element {
                    tag_name: "a".into(),
                    properties: vec![
                        ("data123".into(), hast::PropertyValue::Boolean(true),),
                        ("dataFoo".into(), hast::PropertyValue::Boolean(true),),
                        ("dataBAR".into(), hast::PropertyValue::Boolean(true),)
                    ],
                    children: vec![],
                    position: None,
                }),
                None,
                None
            )?
            .module
        ),
        "<a data-123 data-foo data-b-a-r/>;\n",
        "should support an `Element` w/ data attributes",
    );

    assert_eq!(
        serialize(
            &hast_util_to_swc(
                &hast::Node::Element(hast::Element {
                    tag_name: "a".into(),
                    properties: vec![
                        ("role".into(), hast::PropertyValue::Boolean(true),),
                        ("ariaValueNow".into(), hast::PropertyValue::Boolean(true),),
                        ("ariaDescribedBy".into(), hast::PropertyValue::Boolean(true),)
                    ],
                    children: vec![],
                    position: None,
                }),
                None,
                None
            )?
            .module
        ),
        "<a role aria-valuenow aria-describedby/>;\n",
        "should support an `Element` w/ aria attributes",
    );

    let mdx_element_ast = hast_util_to_swc(
        &hast::Node::MdxJsxElement(hast::MdxJsxElement {
            name: None,
            attributes: vec![],
            children: vec![],
            position: None,
        }),
        None,
        None,
    )?;

    assert_eq!(
        mdx_element_ast,
        Program {
            path: None,
            module: swc_ecma_ast::Module {
                shebang: None,
                body: vec![swc_ecma_ast::ModuleItem::Stmt(swc_ecma_ast::Stmt::Expr(
                    swc_ecma_ast::ExprStmt {
                        expr: Box::new(swc_ecma_ast::Expr::JSXFragment(
                            swc_ecma_ast::JSXFragment {
                                opening: swc_ecma_ast::JSXOpeningFragment {
                                    span: swc_common::DUMMY_SP,
                                },
                                closing: swc_ecma_ast::JSXClosingFragment {
                                    span: swc_common::DUMMY_SP,
                                },
                                children: vec![],
                                span: swc_common::DUMMY_SP,
                            }
                        )),
                        span: swc_common::DUMMY_SP,
                    },
                ))],
                span: swc_common::DUMMY_SP,
            },
            comments: vec![],
        },
        "should support an `MdxElement` (fragment)",
    );

    assert_eq!(
        serialize(&mdx_element_ast.module),
        "<></>;\n",
        "should support an `MdxElement` (fragment, serialize)",
    );

    assert_eq!(
        serialize(
            &hast_util_to_swc(
                &hast::Node::MdxJsxElement(hast::MdxJsxElement {
                    name: Some("a".into()),
                    attributes: vec![],
                    children: vec![],
                    position: None,
                }),
                None,
                None
            )?
            .module
        ),
        "<a />;\n",
        "should support an `MdxElement` (element, no children)",
    );

    assert_eq!(
        serialize(
            &hast_util_to_swc(
                &hast::Node::MdxJsxElement(hast::MdxJsxElement {
                    name: Some("a:b".into()),
                    attributes: vec![],
                    children: vec![],
                    position: None,
                }),
                None,
                None
            )?
            .module
        ),
        "<a:b />;\n",
        "should support an `MdxElement` (element, namespace id)",
    );

    assert_eq!(
        serialize(
            &hast_util_to_swc(
                &hast::Node::MdxJsxElement(hast::MdxJsxElement {
                    name: Some("a.b.c".into()),
                    attributes: vec![],
                    children: vec![],
                    position: None,
                }),
                None,
                None
            )?
            .module
        ),
        "<a.b.c />;\n",
        "should support an `MdxElement` (element, member expression)",
    );

    assert_eq!(
        serialize(
            &hast_util_to_swc(
                &hast::Node::MdxJsxElement(hast::MdxJsxElement {
                    name: Some("a".into()),
                    attributes: vec![],
                    children: vec![hast::Node::Text(hast::Text {
                        value: "b".into(),
                        position: None,
                    })],
                    position: None,
                }),
                None,
                None
            )?
            .module
        ),
        "<a >{\"b\"}</a>;\n",
        "should support an `MdxElement` (element, children)",
    );

    assert_eq!(
        serialize(
            &hast_util_to_swc(
                &hast::Node::MdxJsxElement(hast::MdxJsxElement {
                    name: Some("a".into()),
                    attributes: vec![hast::AttributeContent::Property(hast::MdxJsxAttribute {
                        name: "b".into(),
                        value: None
                    })],
                    children: vec![],
                    position: None,
                }),
                None,
                None
            )?
            .module
        ),
        "<a b/>;\n",
        "should support an `MdxElement` (element, boolean attribute)",
    );

    assert_eq!(
        serialize(
            &hast_util_to_swc(
                &hast::Node::MdxJsxElement(hast::MdxJsxElement {
                    name: Some("a".into()),
                    attributes: vec![hast::AttributeContent::Property(hast::MdxJsxAttribute {
                        name: "b".into(),
                        value: Some(hast::AttributeValue::Literal("c".into()))
                    })],
                    children: vec![],
                    position: None,
                }),
                None,
                None
            )?
            .module
        ),
        "<a b=\"c\"/>;\n",
        "should support an `MdxElement` (element, attribute w/ literal value)",
    );

    assert_eq!(
        serialize(
            &hast_util_to_swc(
                &hast::Node::MdxJsxElement(hast::MdxJsxElement {
                    name: Some("a".into()),
                    attributes: vec![hast::AttributeContent::Property(hast::MdxJsxAttribute {
                        name: "b".into(),
                        value: Some(hast::AttributeValue::Expression("c".into(), vec![]))
                    })],
                    children: vec![],
                    position: None,
                }),
                None,
                None
            )?
            .module
        ),
        "<a b={c}/>;\n",
        "should support an `MdxElement` (element, attribute w/ expression value)",
    );

    assert_eq!(
        serialize(
            &hast_util_to_swc(
                &hast::Node::MdxJsxElement(hast::MdxJsxElement {
                    name: Some("a".into()),
                    attributes: vec![hast::AttributeContent::Expression("...c".into(), vec![])],
                    children: vec![],
                    position: None,
                }),
                None,
                None
            )?
            .module
        ),
        "<a {...c}/>;\n",
        "should support an `MdxElement` (element, expression attribute)",
    );

    let mdx_expression_ast = hast_util_to_swc(
        &hast::Node::MdxExpression(hast::MdxExpression {
            value: "a".into(),
            position: None,
            stops: vec![],
        }),
        None,
        None,
    )?;

    assert_eq!(
        mdx_expression_ast,
        Program {
            path: None,
            module: swc_ecma_ast::Module {
                shebang: None,
                body: vec![swc_ecma_ast::ModuleItem::Stmt(swc_ecma_ast::Stmt::Expr(
                    swc_ecma_ast::ExprStmt {
                        expr: Box::new(swc_ecma_ast::Expr::JSXFragment(
                            swc_ecma_ast::JSXFragment {
                                opening: swc_ecma_ast::JSXOpeningFragment {
                                    span: swc_common::DUMMY_SP,
                                },
                                closing: swc_ecma_ast::JSXClosingFragment {
                                    span: swc_common::DUMMY_SP,
                                },
                                children: vec![swc_ecma_ast::JSXElementChild::JSXExprContainer(
                                    swc_ecma_ast::JSXExprContainer {
                                        expr: swc_ecma_ast::JSXExpr::Expr(Box::new(
                                            swc_ecma_ast::Expr::Ident(swc_ecma_ast::Ident {
                                                sym: "a".into(),
                                                span: swc_common::DUMMY_SP,
                                                optional: false,
                                            })
                                        )),
                                        span: swc_common::DUMMY_SP,
                                    },
                                )],
                                span: swc_common::DUMMY_SP,
                            }
                        )),
                        span: swc_common::DUMMY_SP,
                    },
                ))],
                span: swc_common::DUMMY_SP,
            },
            comments: vec![],
        },
        "should support an `MdxExpression`",
    );

    assert_eq!(
        serialize(&mdx_expression_ast.module),
        "<>{a}</>;\n",
        "should support an `MdxExpression` (serialize)",
    );

    let mdxjs_esm_ast = hast_util_to_swc(
        &hast::Node::MdxjsEsm(hast::MdxjsEsm {
            value: "import a from 'b'".into(),
            position: None,
            stops: vec![],
        }),
        None,
        None,
    )?;

    assert_eq!(
        mdxjs_esm_ast,
        Program {
            path: None,
            module: swc_ecma_ast::Module {
                shebang: None,
                body: vec![swc_ecma_ast::ModuleItem::ModuleDecl(
                    swc_ecma_ast::ModuleDecl::Import(swc_ecma_ast::ImportDecl {
                        specifiers: vec![swc_ecma_ast::ImportSpecifier::Default(
                            swc_ecma_ast::ImportDefaultSpecifier {
                                local: swc_ecma_ast::Ident {
                                    sym: "a".into(),
                                    optional: false,
                                    span: swc_common::DUMMY_SP,
                                },
                                span: swc_common::DUMMY_SP,
                            }
                        )],
                        src: Box::new(swc_ecma_ast::Str {
                            value: "b".into(),
                            span: swc_common::DUMMY_SP,
                            raw: Some("\'b\'".into()),
                        }),
                        type_only: false,
                        asserts: None,
                        span: swc_common::DUMMY_SP,
                    })
                )],
                span: swc_common::DUMMY_SP,
            },
            comments: vec![],
        },
        "should support an `MdxjsEsm`",
    );

    assert_eq!(
        serialize(&mdxjs_esm_ast.module),
        "import a from 'b';\n",
        "should support an `MdxjsEsm` (serialize)",
    );

    let root_ast = hast_util_to_swc(
        &hast::Node::Root(hast::Root {
            children: vec![hast::Node::Text(hast::Text {
                value: "a".into(),
                position: None,
            })],
            position: None,
        }),
        None,
        None,
    )?;

    assert_eq!(
        root_ast,
        Program {
            path: None,
            module: swc_ecma_ast::Module {
                shebang: None,
                body: vec![swc_ecma_ast::ModuleItem::Stmt(swc_ecma_ast::Stmt::Expr(
                    swc_ecma_ast::ExprStmt {
                        expr: Box::new(swc_ecma_ast::Expr::JSXFragment(
                            swc_ecma_ast::JSXFragment {
                                opening: swc_ecma_ast::JSXOpeningFragment {
                                    span: swc_common::DUMMY_SP,
                                },
                                closing: swc_ecma_ast::JSXClosingFragment {
                                    span: swc_common::DUMMY_SP,
                                },
                                children: vec![swc_ecma_ast::JSXElementChild::JSXExprContainer(
                                    swc_ecma_ast::JSXExprContainer {
                                        expr: swc_ecma_ast::JSXExpr::Expr(Box::new(
                                            swc_ecma_ast::Expr::Lit(swc_ecma_ast::Lit::Str(
                                                swc_ecma_ast::Str {
                                                    value: "a".into(),
                                                    span: swc_common::DUMMY_SP,
                                                    raw: None,
                                                }
                                            ),)
                                        )),
                                        span: swc_common::DUMMY_SP,
                                    },
                                )],
                                span: swc_common::DUMMY_SP,
                            }
                        )),
                        span: swc_common::DUMMY_SP,
                    },
                ))],
                span: swc_common::DUMMY_SP,
            },
            comments: vec![],
        },
        "should support a `Root`",
    );

    assert_eq!(
        serialize(&root_ast.module),
        "<>{\"a\"}</>;\n",
        "should support a `Root` (serialize)",
    );

    let text_ast = hast_util_to_swc(
        &hast::Node::Text(hast::Text {
            value: "a".into(),
            position: None,
        }),
        None,
        None,
    )?;

    assert_eq!(
        text_ast,
        Program {
            path: None,
            module: swc_ecma_ast::Module {
                shebang: None,
                body: vec![swc_ecma_ast::ModuleItem::Stmt(swc_ecma_ast::Stmt::Expr(
                    swc_ecma_ast::ExprStmt {
                        expr: Box::new(swc_ecma_ast::Expr::JSXFragment(
                            swc_ecma_ast::JSXFragment {
                                opening: swc_ecma_ast::JSXOpeningFragment {
                                    span: swc_common::DUMMY_SP,
                                },
                                closing: swc_ecma_ast::JSXClosingFragment {
                                    span: swc_common::DUMMY_SP,
                                },
                                children: vec![swc_ecma_ast::JSXElementChild::JSXExprContainer(
                                    swc_ecma_ast::JSXExprContainer {
                                        expr: swc_ecma_ast::JSXExpr::Expr(Box::new(
                                            swc_ecma_ast::Expr::Lit(swc_ecma_ast::Lit::Str(
                                                swc_ecma_ast::Str {
                                                    value: "a".into(),
                                                    span: swc_common::DUMMY_SP,
                                                    raw: None,
                                                }
                                            ),)
                                        )),
                                        span: swc_common::DUMMY_SP,
                                    },
                                )],
                                span: swc_common::DUMMY_SP,
                            }
                        )),
                        span: swc_common::DUMMY_SP,
                    },
                ))],
                span: swc_common::DUMMY_SP,
            },
            comments: vec![],
        },
        "should support a `Text`",
    );

    assert_eq!(
        serialize(&text_ast.module),
        "<>{\"a\"}</>;\n",
        "should support a `Text` (serialize)",
    );

    Ok(())
}
