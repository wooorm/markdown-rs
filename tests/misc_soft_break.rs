extern crate micromark;
use micromark::micromark;
use pretty_assertions::assert_eq;

#[test]
fn soft_break() {
    assert_eq!(
        micromark("foo\nbaz"),
        "<p>foo\nbaz</p>",
        "should support line endings"
    );

    assert_eq!(
        micromark("foo \n baz"),
        "<p>foo\nbaz</p>",
        "should trim spaces around line endings"
    );
}
