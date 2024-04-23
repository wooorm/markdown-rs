use markdown::{message, to_html, to_html_with_options, CompileOptions, Options};
use pretty_assertions::assert_eq;

#[test]
fn dangerous_html() -> Result<(), message::Message> {
    let danger = &Options {
        compile: CompileOptions {
            allow_dangerous_html: true,
            allow_dangerous_protocol: true,
            ..Default::default()
        },
        ..Default::default()
    };

    assert_eq!(
        to_html("<x>"),
        "&lt;x&gt;",
        "should be safe by default for flow"
    );

    assert_eq!(
        to_html("a<b>"),
        "<p>a&lt;b&gt;</p>",
        "should be safe by default for text"
    );

    assert_eq!(
        to_html_with_options("<x>", danger)?,
        "<x>",
        "should be unsafe w/ `allowDangerousHtml`"
    );

    Ok(())
}
