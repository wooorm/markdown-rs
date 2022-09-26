extern crate micromark;
use micromark::{
    mdast::{List, ListItem, MdxJsxFlowElement, Node, Paragraph, Position, Root, Text},
    micromark_to_mdast, micromark_with_options, Constructs, Options,
};
use pretty_assertions::assert_eq;

#[test]
fn mdx_jsx_flow_agnostic() -> Result<(), String> {
    let mdx = Options {
        constructs: Constructs::mdx(),
        ..Options::default()
    };

    assert_eq!(
        micromark_with_options("<a />", &mdx)?,
        "",
        "should support a self-closing element"
    );

    assert_eq!(
        micromark_with_options("<a></a>", &mdx)?,
        "",
        "should support a closed element"
    );

    assert_eq!(
        micromark_with_options("<a>\nb\n</a>", &mdx)?,
        "<p>b</p>\n",
        "should support an element w/ content"
    );

    assert_eq!(
        micromark_with_options("<a>\n- b\n</a>", &mdx)?,
        "<ul>\n<li>b</li>\n</ul>\n",
        "should support an element w/ containers as content"
    );

    assert_eq!(
        micromark_with_options("<a b c:d e=\"\" f={/* g */} {...h} />", &mdx)?,
        "",
        "should support attributes"
    );

    Ok(())
}

// Flow is mostly the same as `text`, so we only test the relevant
// differences.
#[test]
fn mdx_jsx_flow_essence() -> Result<(), String> {
    let mdx = Options {
        constructs: Constructs::mdx(),
        ..Options::default()
    };

    assert_eq!(
        micromark_with_options("<a />", &mdx)?,
        "",
        "should support an element"
    );

    assert_eq!(
        micromark_with_options("<a>\n- b\n</a>", &mdx)?,
        "<ul>\n<li>b</li>\n</ul>\n",
        "should support an element around a container"
    );

    assert_eq!(
        micromark_with_options("<x\n  y\n>  \nb\n  </x>", &mdx)?,
        "<p>b</p>\n",
        "should support a dangling `>` in a tag (not a block quote)"
    );

    assert_eq!(
        micromark_with_options("<a>  \nb\n  </a>", &mdx)?,
        "<p>b</p>\n",
        "should support trailing initial and final whitespace around tags"
    );

    assert_eq!(
        micromark_with_options("<a> <b>\t\nc\n  </b> </a>", &mdx)?,
        "<p>c</p>\n",
        "should support tags after tags"
    );

    assert_eq!(
        micromark_with_options("> <X\n/>", &mdx).err().unwrap(),
        "2:1: Unexpected lazy line in jsx in container, expected line to be prefixed with `>` when in a block quote, whitespace when in a list, etc",
        "should not support lazy flow (1)"
    );

    assert_eq!(
        micromark_with_options("> a\n> <X\n/>", &mdx)
            .err()
            .unwrap(),
        "3:1: Unexpected lazy line in jsx in container, expected line to be prefixed with `>` when in a block quote, whitespace when in a list, etc",
        "should not support lazy flow (2)"
    );

    assert_eq!(
        micromark_with_options("> <a b='\nc'/>", &mdx)
            .err()
            .unwrap(),
        "2:1: Unexpected lazy line in jsx in container, expected line to be prefixed with `>` when in a block quote, whitespace when in a list, etc",
        "should not support lazy flow (3)"
    );

    assert_eq!(
        micromark_with_options("> <a b='c\n'/>", &mdx)
            .err()
            .unwrap(),
        "2:1: Unexpected lazy line in jsx in container, expected line to be prefixed with `>` when in a block quote, whitespace when in a list, etc",
        "should not support lazy flow (4)"
    );

    assert_eq!(
        micromark_with_options("> <a b='c\nd'/>", &mdx)
            .err()
            .unwrap(),
        "2:1: Unexpected lazy line in jsx in container, expected line to be prefixed with `>` when in a block quote, whitespace when in a list, etc",
        "should not support lazy flow (4)"
    );

    assert_eq!(
        micromark_with_options("> <a b={c\nd}/>", &mdx)
            .err()
            .unwrap(),
        "2:1: Unexpected lazy line in expression in container, expected line to be prefixed with `>` when in a block quote, whitespace when in a list, etc",
        "should not support lazy flow (5)"
    );

    assert_eq!(
        micromark_with_options("> <a {b\nc}/>", &mdx)
            .err()
            .unwrap(),
        "2:1: Unexpected lazy line in expression in container, expected line to be prefixed with `>` when in a block quote, whitespace when in a list, etc",
        "should not support lazy flow (6)"
    );

    assert_eq!(
        micromark_with_options("> a\n<X />", &mdx)?,
        "<blockquote>\n<p>a</p>\n</blockquote>\n",
        "should not support lazy flow (7)"
    );

    assert_eq!(
        micromark_to_mdast("<>\n  * a\n</>", &mdx)?,
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
                                value: "a".to_string(),
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
