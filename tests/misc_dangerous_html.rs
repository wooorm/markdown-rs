extern crate micromark;
use micromark::{micromark, micromark_with_options, Options};

const DANGER: &Options = &Options {
    allow_dangerous_html: true,
    allow_dangerous_protocol: true,
    default_line_ending: None,
};

#[test]
fn dangerous_html() {
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
        micromark_with_options("<x>", DANGER),
        "<x>",
        "should be unsafe w/ `allowDangerousHtml`"
    );
}
