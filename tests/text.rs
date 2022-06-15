extern crate micromark;
use micromark::micromark;

#[test]
fn text() {
    assert_eq!(
        micromark("hello $.;'there"),
        "<p>hello $.;'there</p>",
        "should support ascii text"
    );

    assert_eq!(
        micromark("Foo χρῆν"),
        "<p>Foo χρῆν</p>",
        "should support unicode text"
    );

    assert_eq!(
        micromark("Multiple     spaces"),
        "<p>Multiple     spaces</p>",
        "should preserve internal spaces verbatim"
    );
}
