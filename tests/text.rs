use markdown::to_html;
use pretty_assertions::assert_eq;

#[test]
fn text() {
    assert_eq!(
        to_html("hello $.;'there"),
        "<p>hello $.;'there</p>",
        "should support ascii text"
    );

    assert_eq!(
        to_html("Foo χρῆν"),
        "<p>Foo χρῆν</p>",
        "should support unicode text"
    );

    assert_eq!(
        to_html("Multiple     spaces"),
        "<p>Multiple     spaces</p>",
        "should preserve internal spaces verbatim"
    );
}
