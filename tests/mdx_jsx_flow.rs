mod test_utils;
use markdown::{
    mdast::{List, ListItem, MdxJsxFlowElement, Node, Paragraph, Root, Text},
    message, to_html_with_options, to_mdast,
    unist::Position,
    Constructs, Options, ParseOptions,
};
use pretty_assertions::assert_eq;
use test_utils::swc::{parse_esm, parse_expression};

#[test]
fn mdx_jsx_flow_agnostic() -> Result<(), message::Message> {
    let mdx = Options {
        parse: ParseOptions::mdx(),
        ..Default::default()
    };

    assert_eq!(
        to_html_with_options("<a />", &mdx)?,
        "",
        "should support a self-closing element"
    );

    // Note: in MDX, indented code is turned off:
    assert_eq!(
        to_html_with_options(
            "    <a />",
            &Options {
                parse: ParseOptions {
                    constructs: Constructs {
                        html_flow: false,
                        mdx_jsx_flow: true,
                        ..Default::default()
                    },
                    ..Default::default()
                },
                ..Default::default()
            }
        )?,
        "<pre><code>&lt;a /&gt;\n</code></pre>",
        "should prefer indented code over jsx if it’s enabled"
    );

    assert_eq!(
        to_html_with_options(
            "   <a />",
            &Options {
                parse: ParseOptions {
                    constructs: Constructs {
                        html_flow: false,
                        mdx_jsx_flow: true,
                        ..Default::default()
                    },
                    ..Default::default()
                },
                ..Default::default()
            }
        )?,
        "",
        "should support indented jsx if indented code is enabled"
    );

    assert_eq!(
        to_html_with_options("<a></a>", &mdx)?,
        "",
        "should support a closed element"
    );

    assert_eq!(
        to_html_with_options("<a>\nb\n</a>", &mdx)?,
        "<p>b</p>\n",
        "should support an element w/ content"
    );

    assert_eq!(
        to_html_with_options("<a>\n- b\n</a>", &mdx)?,
        "<ul>\n<li>b</li>\n</ul>\n",
        "should support an element w/ containers as content"
    );

    assert_eq!(
        to_html_with_options("<a b c:d e=\"\" f={/* g */} {...h} />", &mdx)?,
        "",
        "should support attributes"
    );

    Ok(())
}

// Flow is mostly the same as `text`, so we only test the relevant
// differences.
#[test]
fn mdx_jsx_flow_essence() -> Result<(), message::Message> {
    let mdx = Options {
        parse: ParseOptions::mdx(),
        ..Default::default()
    };

    assert_eq!(
        to_html_with_options("<a />", &mdx)?,
        "",
        "should support an element"
    );

    assert_eq!(
        to_html_with_options("<a>\n- b\n</a>", &mdx)?,
        "<ul>\n<li>b</li>\n</ul>\n",
        "should support an element around a container"
    );

    assert_eq!(
        to_html_with_options("<x\n  y\n>  \nb\n  </x>", &mdx)?,
        "<p>b</p>\n",
        "should support a dangling `>` in a tag (not a block quote)"
    );

    assert_eq!(
        to_html_with_options("<a>  \nb\n  </a>", &mdx)?,
        "<p>b</p>\n",
        "should support trailing initial and final whitespace around tags"
    );

    assert_eq!(
        to_html_with_options("<a> <b>\t\nc\n  </b> </a>", &mdx)?,
        "<p>c</p>\n",
        "should support tags after tags"
    );

    // This is to make sure `document` passes errors through properly.
    assert_eq!(
        to_html_with_options("* <!a>\n1. b", &mdx)
            .err()
            .unwrap().to_string(),
        "1:4: Unexpected character `!` (U+0021) before name, expected a character that can start a name, such as a letter, `$`, or `_` (note: to create a comment in MDX, use `{/* text */}`) (markdown-rs:unexpected-character)",
        "should handle crash in containers gracefully"
    );

    assert_eq!(
         to_html_with_options("> <X\n/>", &mdx).err().unwrap().to_string(),
        "2:1: Unexpected lazy line in jsx in container, expected line to be prefixed with `>` when in a block quote, whitespace when in a list, etc (markdown-rs:unexpected-lazy)",
        "should not support lazy flow (1)"
    );

    assert_eq!(
        to_html_with_options("> a\n> <X\n/>", &mdx)
            .err()
            .unwrap().to_string(),
        "3:1: Unexpected lazy line in jsx in container, expected line to be prefixed with `>` when in a block quote, whitespace when in a list, etc (markdown-rs:unexpected-lazy)",
        "should not support lazy flow (2)"
    );

    assert_eq!(
        to_html_with_options("> <a b='\nc'/>", &mdx)
            .err()
            .unwrap().to_string(),
        "2:1: Unexpected lazy line in jsx in container, expected line to be prefixed with `>` when in a block quote, whitespace when in a list, etc (markdown-rs:unexpected-lazy)",
        "should not support lazy flow (3)"
    );

    assert_eq!(
        to_html_with_options("> <a b='c\n'/>", &mdx)
            .err()
            .unwrap().to_string(),
        "2:1: Unexpected lazy line in jsx in container, expected line to be prefixed with `>` when in a block quote, whitespace when in a list, etc (markdown-rs:unexpected-lazy)",
        "should not support lazy flow (4)"
    );

    assert_eq!(
        to_html_with_options("> <a b='c\nd'/>", &mdx)
            .err()
            .unwrap().to_string(),
        "2:1: Unexpected lazy line in jsx in container, expected line to be prefixed with `>` when in a block quote, whitespace when in a list, etc (markdown-rs:unexpected-lazy)",
        "should not support lazy flow (5)"
    );

    assert_eq!(
        to_html_with_options("> <a b={c\nd}/>", &mdx)
            .err()
            .unwrap().to_string(),
        "2:1: Unexpected lazy line in expression in container, expected line to be prefixed with `>` when in a block quote, whitespace when in a list, etc (markdown-rs:unexpected-lazy)",
        "should not support lazy flow (6)"
    );

    assert_eq!(
        to_html_with_options("> <a {b\nc}/>", &mdx)
            .err()
            .unwrap().to_string(),
        "2:1: Unexpected lazy line in expression in container, expected line to be prefixed with `>` when in a block quote, whitespace when in a list, etc (markdown-rs:unexpected-lazy)",
        "should not support lazy flow (7)"
    );

    assert_eq!(
        to_html_with_options("> a\n<X />", &mdx)?,
        "<blockquote>\n<p>a</p>\n</blockquote>\n",
        "should not support lazy flow (8)"
    );

    assert_eq!(
        to_mdast("<>\n  * a\n</>", &mdx.parse)?,
        Node::Root(Root {
            children: vec![Node::MdxJsxFlowElement(MdxJsxFlowElement {
                name: None,
                attributes: vec![],
                children: vec![Node::List(List {
                    ordered: false,
                    spread: false,
                    start: None,
                    children: vec![Node::ListItem(ListItem {
                        checked: None,
                        spread: false,
                        children: vec![Node::Paragraph(Paragraph {
                            children: vec![Node::Text(Text {
                                value: "a".into(),
                                position: Some(Position::new(2, 5, 7, 2, 6, 8))
                            }),],
                            position: Some(Position::new(2, 5, 7, 2, 6, 8))
                        })],
                        position: Some(Position::new(2, 1, 3, 2, 6, 8))
                    })],
                    position: Some(Position::new(2, 1, 3, 2, 6, 8))
                })],
                position: Some(Position::new(1, 1, 0, 3, 4, 12))
            })],
            position: Some(Position::new(1, 1, 0, 3, 4, 12))
        }),
        "should support mdx jsx (flow) as `MdxJsxFlowElement`s in mdast"
    );

    Ok(())
}

// Flow is mostly the same as `text`, so we only test the relevant
// differences.
#[test]
fn mdx_jsx_flow_interleaving_with_expressions() -> Result<(), message::Message> {
    let mdx = Options {
        parse: ParseOptions::mdx(),
        ..Default::default()
    };
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
        to_html_with_options("<div>\n{1}\n</div>", &mdx)?,
        "",
        "should support tags and expressions (unaware)"
    );

    assert_eq!(
        to_html_with_options("<div>\n{'}'}\n</div>", &swc)?,
        "",
        "should support tags and expressions (aware)"
    );

    assert_eq!(
        to_html_with_options("x<em>{1}</em>", &swc)?,
        "<p>x</p>",
        "should support tags and expressions with text before (text)"
    );

    assert_eq!(
        to_html_with_options("<em>x{1}</em>", &swc)?,
        "<p>x</p>",
        "should support tags and expressions with text between, early (text)"
    );

    assert_eq!(
        to_html_with_options("<em>{1}x</em>", &swc)?,
        "<p>x</p>",
        "should support tags and expressions with text between, late (text)"
    );

    assert_eq!(
        to_html_with_options("<em>{1}</em>x", &swc)?,
        "<p>x</p>",
        "should support tags and expressions with text after (text)"
    );

    assert_eq!(
        to_html_with_options("<x/>{1}", &swc)?,
        "",
        "should support a tag and then an expression (flow)"
    );

    assert_eq!(
        to_html_with_options("<x/>{1}x", &swc)?,
        "<p>x</p>",
        "should support a tag, an expression, then text (text)"
    );

    assert_eq!(
        to_html_with_options("x<x/>{1}", &swc)?,
        "<p>x</p>",
        "should support text, a tag, then an expression (text)"
    );

    assert_eq!(
        to_html_with_options("{1}<x/>", &swc)?,
        "",
        "should support an expression and then a tag (flow)"
    );

    assert_eq!(
        to_html_with_options("{1}<x/>x", &swc)?,
        "<p>x</p>",
        "should support an expression, a tag, then text (text)"
    );

    assert_eq!(
        to_html_with_options("x{1}<x/>", &swc)?,
        "<p>x</p>",
        "should support text, an expression, then a tag (text)"
    );

    assert_eq!(
        to_html_with_options("<x>{[\n'',\n{c:''}\n]}</x>", &swc)?,
        "",
        "should nicely interleaf (micromark/micromark-extension-mdx-jsx#9)"
    );

    assert_eq!(
        to_html_with_options(
            "
<style>{`
  .foo {}
  .bar {}
`}</style>
    ",
            &swc
        )?,
        "",
        "should nicely interleaf (mdx-js/mdx#1945)"
    );

    Ok(())
}
