extern crate micromark;
use micromark::{micromark, micromark_with_options, CompileOptions};

const DANGER: &CompileOptions = &CompileOptions {
    allow_dangerous_html: true,
    allow_dangerous_protocol: true,
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
