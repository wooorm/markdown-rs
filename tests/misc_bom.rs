extern crate micromark;
use micromark::micromark;
use pretty_assertions::assert_eq;

#[test]
fn bom() -> Result<(), String> {
    assert_eq!(micromark("\u{FEFF}"), "", "should ignore just a bom");

    assert_eq!(
        micromark("\u{FEFF}# hea\u{FEFF}ding"),
        "<h1>hea\u{FEFF}ding</h1>",
        "should ignore a bom"
    );

    Ok(())
}
