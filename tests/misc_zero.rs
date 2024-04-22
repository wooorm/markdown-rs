use markdown::{
    mdast::{Node, Root},
    message, to_html, to_mdast,
    unist::Position,
};
use pretty_assertions::assert_eq;

#[test]
fn zero() -> Result<(), message::Message> {
    assert_eq!(to_html(""), "", "should support no markdown");

    assert_eq!(
        to_html("asd\0asd"),
        "<p>asd�asd</p>",
        "should replace `\\0` w/ a replacement characters (`�`)"
    );

    assert_eq!(
        to_html("&#0;"),
        "<p>�</p>",
        "should replace NUL in a character reference"
    );

    // This doesn’t make sense in markdown, as character escapes only work on
    // ascii punctuation, but it’s good to demonstrate the behavior.
    assert_eq!(
        to_html("\\0"),
        "<p>\\0</p>",
        "should not support NUL in a character escape"
    );

    assert_eq!(
        to_mdast("", &Default::default())?,
        Node::Root(Root {
            children: vec![],
            position: Some(Position::new(1, 1, 0, 1, 1, 0))
        }),
        "should support no markdown (ast)"
    );

    Ok(())
}
