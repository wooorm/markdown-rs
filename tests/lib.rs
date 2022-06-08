extern crate micromark;
use micromark::micromark;

#[test]
fn basic() {
    assert_eq!(micromark("asd"), "<p>asd</p>", "should work");
    assert_eq!(micromark("1 < 3"), "<p>1 &lt; 3</p>", "should encode");
}
