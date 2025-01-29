mod test_utils;
use markdown::{
    mdast::{
        AttributeContent, AttributeValue, AttributeValueExpression, Blockquote, MdxFlowExpression,
        MdxJsxAttribute, MdxJsxTextElement, MdxTextExpression, Node, Paragraph, Root, Text,
    },
    message, to_html_with_options, to_mdast,
    unist::Position,
    Constructs, Options, ParseOptions,
};
use pretty_assertions::assert_eq;
use test_utils::swc::{parse_esm, parse_expression};

/// Note: these tests are also in `micromark/micromark-extension-mdx-expression`
/// at `tests/index.js`.
#[test]
fn mdx_expression_flow_agnostic() -> Result<(), message::Message> {
    let mdx = Options {
        parse: ParseOptions::mdx(),
        ..Default::default()
    };

    assert_eq!(
        to_html_with_options("{a}", &mdx)?,
        "",
        "should support an expression"
    );

    assert_eq!(
        to_html_with_options("{}", &mdx)?,
        "",
        "should support an empty expression"
    );

    // Note: in MDX, indented code is turned off:
    assert_eq!(
        to_html_with_options(
            "    {}",
            &Options {
                parse: ParseOptions {
                    constructs: Constructs {
                        mdx_expression_flow: true,
                        ..Default::default()
                    },
                    ..Default::default()
                },
                ..Default::default()
            }
        )?,
        "<pre><code>{}\n</code></pre>",
        "should prefer indented code over expressions if it’s enabled"
    );

    assert_eq!(
        to_html_with_options(
            "   {}",
            &Options {
                parse: ParseOptions {
                    constructs: Constructs {
                        mdx_expression_flow: true,
                        ..Default::default()
                    },
                    ..Default::default()
                },
                ..Default::default()
            }
        )?,
        "",
        "should support indented expressions if indented code is enabled"
    );

    assert_eq!(
        to_html_with_options("{a", &mdx).err().unwrap().to_string(),
        "1:3: Unexpected end of file in expression, expected a corresponding closing brace for `{` (markdown-rs:unexpected-eof)",
        "should crash if no closing brace is found (1)"
    );

    assert_eq!(
        to_html_with_options("{b { c }", &mdx)
            .err()
            .unwrap()
            .to_string(),
        "1:9: Unexpected end of file in expression, expected a corresponding closing brace for `{` (markdown-rs:unexpected-eof)",
        "should crash if no closing brace is found (2)"
    );

    assert_eq!(
        to_html_with_options("{\n}\na", &mdx)?,
        "<p>a</p>",
        "should support a line ending in an expression"
    );

    assert_eq!(
        to_html_with_options("{ a } \t\nb", &mdx)?,
        "<p>b</p>",
        "should support expressions followed by spaces"
    );

    assert_eq!(
        to_html_with_options("  { a }\nb", &mdx)?,
        "<p>b</p>",
        "should support expressions preceded by spaces"
    );

    assert_eq!(
        to_html_with_options("a\n\n* b", &mdx)?,
        "<p>a</p>\n<ul>\n<li>b</li>\n</ul>",
        "should support lists after non-expressions (wooorm/markdown-rs#11)"
    );

    assert_eq!(
        to_html_with_options("> {a\nb}", &mdx)
            .err()
            .unwrap().to_string(),
        "2:1: Unexpected lazy line in expression in container, expected line to be prefixed with `>` when in a block quote, whitespace when in a list, etc (markdown-rs:unexpected-lazy)",
        "should not support lazyness (1)"
    );

    assert_eq!(
        to_html_with_options("> a\n{b}", &mdx)?,
        "<blockquote>\n<p>a</p>\n</blockquote>\n",
        "should not support lazyness (2)"
    );

    assert_eq!(
        to_html_with_options("> {a}\nb", &mdx)?,
        "<blockquote>\n</blockquote>\n<p>b</p>",
        "should not support lazyness (3)"
    );

    assert_eq!(
        to_html_with_options("> {\n> a\nb}", &mdx)
            .err()
            .unwrap().to_string(),
        "3:1: Unexpected lazy line in expression in container, expected line to be prefixed with `>` when in a block quote, whitespace when in a list, etc (markdown-rs:unexpected-lazy)",
        "should not support lazyness (4)"
    );

    assert_eq!(
        to_mdast("{alpha +\nbravo}", &mdx.parse)?,
        Node::Root(Root {
            children: vec![Node::MdxFlowExpression(MdxFlowExpression {
                value: "alpha +\nbravo".into(),
                position: Some(Position::new(1, 1, 0, 2, 7, 15)),
                stops: vec![(0, 1), (7, 8), (8, 9)]
            })],
            position: Some(Position::new(1, 1, 0, 2, 7, 15))
        }),
        "should support mdx expressions (flow) as `MdxFlowExpression`s in mdast"
    );

    assert_eq!(
        to_mdast("  {`\n    a\n  `}", &mdx.parse)?,
        Node::Root(Root {
            children: vec![Node::MdxFlowExpression(MdxFlowExpression {
                value: "`\n  a\n`".into(),
                position: Some(Position::new(1, 3, 2, 3, 5, 15)),
                stops: vec![(0, 3), (1, 4), (2, 7), (5, 10), (6, 13)]
            })],
            position: Some(Position::new(1, 1, 0, 3, 5, 15))
        }),
        "should support indent in `MdxFlowExpression` in mdast"
    );

    Ok(())
}

/// Note: these tests are also in `micromark/micromark-extension-mdx-expression`
/// at `tests/index.js`.
#[test]
fn mdx_expression_flow_gnostic() -> Result<(), message::Message> {
    let swc = Options {
        parse: ParseOptions {
            constructs: Constructs::mdx(),
            mdx_esm_parse: Some(Box::new(parse_esm)),
            mdx_expression_parse: Some(Box::new(parse_expression)),
            ..Default::default()
        },
        ..Default::default()
    };

    assert_eq!(
        to_html_with_options("{a}", &swc)?,
        "",
        "should support an expression"
    );

    assert_eq!(
        to_html_with_options("{}", &swc)?,
        "",
        "should support an empty expression"
    );

    assert_eq!(
        to_html_with_options("{a", &swc).err().unwrap().to_string(),
        "1:3: Unexpected end of file in expression, expected a corresponding closing brace for `{` (markdown-rs:unexpected-eof)",
        "should crash if no closing brace is found (1)"
    );

    assert_eq!(
        to_html_with_options("{b { c }", &swc)
            .err()
            .unwrap()
            .to_string(),
        "1:9: Could not parse expression with swc: Unexpected content after expression (mdx:swc)",
        "should crash if no closing brace is found (2)"
    );

    assert_eq!(
        to_html_with_options("{\n}\na", &swc)?,
        "<p>a</p>",
        "should support a line ending in an expression"
    );

    assert_eq!(
        to_html_with_options("{ a } \t\nb", &swc)?,
        "<p>b</p>",
        "should support expressions followed by spaces"
    );

    assert_eq!(
        to_html_with_options("  { a }\nb", &swc)?,
        "<p>b</p>",
        "should support expressions preceded by spaces"
    );

    assert_eq!(
        to_html_with_options("  {`\n    a\n  `}", &swc)?,
        "",
        "should support indented expressions"
    );

    assert_eq!(
        to_html_with_options("a{(b)}c", &swc)?,
        "<p>ac</p>",
        "should support expressions padded w/ parens"
    );

    assert_eq!(
        to_html_with_options("a{/* b */ ( (c) /* d */ + (e) )}f", &swc)?,
        "<p>af</p>",
        "should support expressions padded w/ parens and comments"
    );

    assert_eq!(
        to_mdast("{`\n\t`}", &swc.parse)?,
        Node::Root(Root {
            children: vec![Node::MdxFlowExpression(MdxFlowExpression {
                value: "`\n  `".into(),
                position: Some(Position::new(1, 1, 0, 2, 7, 6)),
                stops: vec![(0, 1), (1, 2), (2, 3)]
            })],
            position: Some(Position::new(1, 1, 0, 2, 7, 6))
        }),
        "should use correct positional info when tabs are used (1, indent)"
    );

    assert_eq!(
        to_mdast("{`\nalpha\t`}", &swc.parse)?,
        Node::Root(Root {
            children: vec![Node::MdxFlowExpression(MdxFlowExpression {
                value: "`\nalpha\t`".into(),
                position: Some(Position::new(1, 1, 0, 2, 11, 11)),
                stops: vec![(0, 1), (1, 2), (2, 3)]
            })],
            position: Some(Position::new(1, 1, 0, 2, 11, 11))
        }),
        "should use correct positional info when tabs are used (2, content)"
    );

    assert_eq!(
        to_mdast(">  aaa <b c={`\n>      d\n>  `} /> eee", &swc.parse)?,
        Node::Root(Root {
            children: vec![Node::Blockquote(Blockquote {
                children: vec![Node::Paragraph(Paragraph {
                    children: vec![
                        Node::Text(Text {
                            value: "aaa ".into(),
                            position: Some(Position::new(1, 4, 3, 1, 8, 7))
                        }),
                        Node::MdxJsxTextElement(MdxJsxTextElement {
                            children: vec![],
                            name: Some("b".into()),
                            attributes: vec![AttributeContent::Property(MdxJsxAttribute {
                                name: "c".into(),
                                value: Some(AttributeValue::Expression(AttributeValueExpression {
                                    value: "`\n   d\n`".into(),
                                    stops: vec![(0, 13), (1, 14), (2, 19), (6, 23), (7, 27)]
                                }))
                            })],
                            position: Some(Position::new(1, 8, 7, 3, 9, 32))
                        }),
                        Node::Text(Text {
                            value: " eee".into(),
                            position: Some(Position::new(3, 9, 32, 3, 13, 36))
                        })
                    ],
                    position: Some(Position::new(1, 3, 2, 3, 13, 36))
                })],
                position: Some(Position::new(1, 1, 0, 3, 13, 36))
            })],
            position: Some(Position::new(1, 1, 0, 3, 13, 36))
        }),
        "should support template strings in JSX (text) in block quotes"
    );

    assert_eq!(
        to_mdast("> ab {`\n>\t`}", &swc.parse)?,
        Node::Root(Root {
            children: vec![Node::Blockquote(Blockquote {
                children: vec![Node::Paragraph(Paragraph {
                    children: vec![
                        Node::Text(Text {
                            value: "ab ".into(),
                            position: Some(Position::new(1, 3, 2, 1, 6, 5))
                        }),
                        Node::MdxTextExpression(MdxTextExpression {
                            value: "`\n`".into(),
                            stops: vec![(0, 6), (1, 7), (2, 10)],
                            position: Some(Position::new(1, 6, 5, 2, 7, 12))
                        })
                    ],
                    position: Some(Position::new(1, 3, 2, 2, 7, 12))
                })],
                position: Some(Position::new(1, 1, 0, 2, 7, 12))
            })],
            position: Some(Position::new(1, 1, 0, 2, 7, 12))
        }),
        "should use correct positional when there are virtual spaces due to a block quote"
    );

    assert_eq!(
        to_mdast(
            "> {`\n> alpha\n>  bravo\n>   charlie\n>    delta\n> `}",
            &swc.parse
        )?,
        Node::Root(Root {
            children: vec![Node::Blockquote(Blockquote {
                children: vec![Node::MdxFlowExpression(MdxFlowExpression {
                    value: "`\nalpha\nbravo\ncharlie\n delta\n`".into(),
                    position: Some(Position::new(1, 3, 2, 6, 5, 49)),
                    stops: vec![
                        (0, 3),
                        (1, 4),
                        (2, 7),
                        (7, 12),
                        (8, 16),
                        (13, 21),
                        (14, 26),
                        (21, 33),
                        (22, 38),
                        (28, 44),
                        (29, 47)
                    ]
                })],
                position: Some(Position::new(1, 1, 0, 6, 5, 49))
            })],
            position: Some(Position::new(1, 1, 0, 6, 5, 49))
        }),
        "should keep the correct number of spaces in a blockquote (flow)"
    );

    assert_eq!(
        to_mdast(
            "> {`\n> alpha\n>  bravo\n>   charlie\n>    delta\n> `}",
            &swc.parse
        )?,
        Node::Root(Root {
            children: vec![Node::Blockquote(Blockquote {
                children: vec![Node::MdxFlowExpression(MdxFlowExpression {
                    value: "`\nalpha\nbravo\ncharlie\n delta\n`".into(),
                    position: Some(Position::new(1, 3, 2, 6, 5, 49)),
                    stops: vec![
                        (0, 3),
                        (1, 4),
                        (2, 7),
                        (7, 12),
                        (8, 16),
                        (13, 21),
                        (14, 26),
                        (21, 33),
                        (22, 38),
                        (28, 44),
                        (29, 47)
                    ]
                })],
                position: Some(Position::new(1, 1, 0, 6, 5, 49))
            })],
            position: Some(Position::new(1, 1, 0, 6, 5, 49))
        }),
        "should keep the correct number of spaces in a blockquote (flow)"
    );

    // Note: the weird character test has to go in mdxjs-rs.

    Ok(())
}

/// Note: these tests are also in `micromark/micromark-extension-mdx-expression`
/// at `tests/index.js`.
/// This project includes *all* extensions which means that it can use JSX.
/// There we test something that does not exist in actual MDX but which is used
/// by the separate JSX extension.
#[test]
fn mdx_expression_spread() -> Result<(), message::Message> {
    let swc = Options {
        parse: ParseOptions {
            constructs: Constructs::mdx(),
            mdx_esm_parse: Some(Box::new(parse_esm)),
            mdx_expression_parse: Some(Box::new(parse_expression)),
            ..Default::default()
        },
        ..Default::default()
    };

    assert_eq!(
        to_html_with_options("<a {...b} />", &swc)?,
        "",
        "should support a spread"
    );

    assert_eq!(
        to_html_with_options("<a {b} />", &swc).err().unwrap().to_string(),
        "1:5: Unexpected prop in spread (such as `{x}`): only a spread is supported (such as `{...x}`) (mdx:swc)",
        "should crash if not a spread"
    );

    assert_eq!(
        to_html_with_options("<a {...?} />", &swc)
            .err()
            .unwrap()
            .to_string(),
        "1:13: Could not parse expression with swc: Expression expected (mdx:swc)",
        "should crash on an incorrect spread"
    );

    assert_eq!(
        to_html_with_options("<a {b=c}={} d>", &swc).err().unwrap().to_string(),
        "1:5: Unexpected prop in spread (such as `{x}`): only a spread is supported (such as `{...x}`) (mdx:swc)",
        "should crash on an incorrect spread that looks like an assignment"
    );

    assert_eq!(
        to_html_with_options("<a {...b,c} d>", &swc).err().unwrap().to_string(),
        "1:5: Unexpected extra content in spread (such as `{...x,y}`): only a single spread is supported (such as `{...x}`) (mdx:swc)",
        "should crash if a spread and other things"
    );

    assert_eq!(
        to_html_with_options("<a {b=c} />", &swc)
            .err()
            .unwrap()
            .to_string(),
        "1:12: Could not parse expression with swc: assignment property is invalid syntax (mdx:swc)",
        "should crash if not an identifier"
    );

    // Note: `markdown-rs` has no `allowEmpty`.
    assert_eq!(
        to_html_with_options("<a {} />", &swc).err().unwrap().to_string(),
        "1:9: Unexpected prop in spread (such as `{x}`): only a spread is supported (such as `{...x}`) (mdx:swc)",
        "should crash on an empty spread"
    );

    assert_eq!(
        to_html_with_options("<a {/* b */} />", &swc).err().unwrap().to_string(),
        "1:5: Unexpected prop in spread (such as `{x}`): only a spread is supported (such as `{...x}`) (mdx:swc)",
        "should crash on a comment spread"
    );

    Ok(())
}
