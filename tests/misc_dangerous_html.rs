extern crate micromark;
use micromark::{micromark, micromark_with_options, Options};
use pretty_assertions::assert_eq;

#[test]
fn dangerous_html() {
    let danger = &Options {
        allow_dangerous_html: true,
        allow_dangerous_protocol: true,
        ..Options::default()
    };

    assert_eq!(
        micromark("<x>"),
        "&lt;x&gt;",
        "should be safe by default for flow"
    );

    assert_eq!(
        micromark("a<b>"),
        "<p>a&lt;b&gt;</p>",
        "should be safe by default for text"
    );

    assert_eq!(
        micromark_with_options("<x>", danger),
        "<x>",
        "should be unsafe w/ `allowDangerousHtml`"
    );
}
