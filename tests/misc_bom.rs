use markdown::to_html;
use pretty_assertions::assert_eq;

#[test]
fn bom() {
    assert_eq!(to_html("\u{FEFF}"), "", "should ignore just a bom");

    assert_eq!(
        to_html("\u{FEFF}# hea\u{FEFF}ding"),
        "<h1>hea\u{FEFF}ding</h1>",
        "should ignore a bom"
    );
}
