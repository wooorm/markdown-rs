use markdown::{message, to_html, to_html_with_options, CompileOptions, LineEnding, Options};
use pretty_assertions::assert_eq;

#[test]
fn default_line_ending() -> Result<(), message::Message> {
    assert_eq!(
        to_html("> a"),
        "<blockquote>\n<p>a</p>\n</blockquote>",
        "should use `\\n` default"
    );

    assert_eq!(
        to_html("> a\n"),
        "<blockquote>\n<p>a</p>\n</blockquote>\n",
        "should infer the first line ending (1)"
    );

    assert_eq!(
        to_html("> a\r"),
        "<blockquote>\r<p>a</p>\r</blockquote>\r",
        "should infer the first line ending (2)"
    );

    assert_eq!(
        to_html("> a\r\n"),
        "<blockquote>\r\n<p>a</p>\r\n</blockquote>\r\n",
        "should infer the first line ending (3)"
    );

    assert_eq!(
        to_html_with_options(
            "> a",
            &Options {
                compile: CompileOptions {
                    default_line_ending: LineEnding::CarriageReturn,
                    ..Default::default()
                },
                ..Default::default()
            }
        )?,
        "<blockquote>\r<p>a</p>\r</blockquote>",
        "should support the given line ending"
    );

    assert_eq!(
        to_html_with_options(
            "> a\n",
            &Options {
                compile: CompileOptions {
                    default_line_ending: LineEnding::CarriageReturn,
                    ..Default::default()
                },
                ..Default::default()
            }
        )?,
        // To do: is this a bug in `to_html.js` that it uses `\r` for earlier line endings?
        // "<blockquote>\r<p>a</p>\r</blockquote>\n",
        "<blockquote>\n<p>a</p>\n</blockquote>\n",
        "should support the given line ending, even if line endings exist"
    );

    Ok(())
}
