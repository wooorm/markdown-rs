use markdown::{
    mdast::{
        Definition, MdxFlowExpression, MdxTextExpression, MdxjsEsm, Node, Paragraph, Root, Text,
    },
    to_mdast as from, ParseOptions,
};
use mdast_util_to_markdown::to_markdown as to;
use pretty_assertions::assert_eq;

#[test]
fn mdx() {
    assert_eq!(
        to(&Node::Root(Root {
            children: vec![
                Node::MdxFlowExpression(MdxFlowExpression {
                    value: "a + b".to_string(),
                    position: None,
                    stops: vec![]
                }),
                Node::MdxFlowExpression(MdxFlowExpression {
                    value: String::from("\nc +\n1\n"),
                    position: None,
                    stops: vec![]
                }),
                Node::MdxFlowExpression(MdxFlowExpression {
                    value: String::new(),
                    position: None,
                    stops: vec![]
                }),
                Node::Paragraph(Paragraph {
                    children: vec![Node::Text(Text {
                        value: "d".to_string(),
                        position: None
                    })],
                    position: None
                })
            ],
            position: None
        }))
        .unwrap(),
        "{a + b}\n\n{\n  c +\n  1\n}\n\n{}\n\nd\n",
        "should serialize flow expressions"
    );

    assert_eq!(
        to(&Node::Paragraph(Paragraph {
            children: vec![
                Node::Text(Text {
                    value: "a ".to_string(),
                    position: None
                }),
                Node::MdxTextExpression(MdxTextExpression {
                    value: "b + c".to_string(),
                    position: None,
                    stops: vec![]
                }),
                Node::Text(Text {
                    value: ", d ".to_string(),
                    position: None
                }),
                Node::MdxTextExpression(MdxTextExpression {
                    value: "e + 1".to_string(),
                    position: None,
                    stops: vec![]
                }),
                Node::Text(Text {
                    value: ", f ".to_string(),
                    position: None
                }),
                Node::MdxTextExpression(MdxTextExpression {
                    value: String::new(),
                    position: None,
                    stops: vec![]
                }),
                Node::Text(Text {
                    value: ".".to_string(),
                    position: None
                }),
            ],
            position: None
        }))
        .unwrap(),
        "a {b + c}, d {e + 1}, f {}.\n",
        "should serialize text expressions"
    );

    assert_eq!(
        to(&Node::Paragraph(Paragraph {
            children: vec![Node::Text(Text {
                value: "a { b".to_string(),
                position: None
            })],
            position: None
        }))
        .unwrap(),
        "a \\{ b\n",
        "should escape `{{` in text"
    );

    assert_eq!(
        to(&Node::Definition(Definition {
            position: None,
            url: "x".to_string(),
            title: "a\n{\nb".to_string().into(),
            identifier: "a".to_string(),
            label: None
        }))
        .unwrap(),
        "[a]: x \"a\n\\{\nb\"\n",
        "should escape `{{` at the start of a line"
    );

    assert_eq!(
        to(&from("  {`\n a\n `}", &ParseOptions::mdx()).unwrap()).unwrap(),
        "{`\n  a\n  `}\n",
        "should strip superfluous whitespace depending on the opening prefix, when roundtripping expressions (flow)"
    );

    // This require changing to match the js tests when https://github.com/wooorm/markdown-rs/issues/150 is resolved
    //assert_eq!(
    //    to(&from("  {`\n    a\n  `}", &ParseOptions::mdx()).unwrap()).unwrap(),
    //    "{`\n  a\n  `}\n",
    //    "should *not* strip superfluous whitespace (if there is more) when roundtripping expressions (flow)"
    //);

    //// This require changing to match the js tests when https://github.com/wooorm/markdown-rs/issues/150 is resolved
    //assert_eq!(
    //    to(&from("a {`\n    b\n  `} c", &ParseOptions::mdx()).unwrap()).unwrap(),
    //    "a {`\n  b\n  `} c\n",
    //    "should not strip consecutive lines in expressions (text)"
    //);

    assert_eq!(
        to(&Node::Root(Root {
            children: vec![
                Node::MdxjsEsm(MdxjsEsm {
                    value: String::from("import a from \"b\"\nexport var c = \"\""),
                    position: None,
                    stops: Vec::new()
                }),
                Node::Paragraph(Paragraph {
                    children: vec![Node::Text(Text {
                        value: "d".to_string(),
                        position: None
                    })],
                    position: None
                })
            ],
            position: None
        }))
        .unwrap(),
        "import a from \"b\"\nexport var c = \"\"\n\nd\n"
    )
}
