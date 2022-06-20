extern crate micromark;
use micromark::micromark;

#[test]
fn bom() {
    assert_eq!(micromark("\u{FEFF}"), "", "should ignore just a bom");

    assert_eq!(
        micromark("\u{FEFF}# hea\u{FEFF}ding"),
        "<h1>hea\u{FEFF}ding</h1>",
        "should ignore a bom"
    );
}
