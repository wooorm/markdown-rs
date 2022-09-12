extern crate micromark;
use micromark::micromark;
use pretty_assertions::assert_eq;

#[test]
fn fuzz() -> Result<(), String> {
    assert_eq!(
        micromark("[\n~\na\n-\n\n"),
        "<p>[\n~\na</p>\n<ul>\n<li></li>\n</ul>\n",
        "1"
    );

    Ok(())
}
