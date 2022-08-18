extern crate micromark;
use micromark::{micromark, micromark_with_options, LineEnding, Options};
use pretty_assertions::assert_eq;

#[test]
fn default_line_ending() {
    assert_eq!(
        micromark("> a"),
        "<blockquote>\n<p>a</p>\n</blockquote>",
        "should use `\\n` default"
    );

    assert_eq!(
        micromark("> a\n"),
        "<blockquote>\n<p>a</p>\n</blockquote>\n",
        "should infer the first line ending (1)"
    );

    assert_eq!(
        micromark("> a\r"),
        "<blockquote>\r<p>a</p>\r</blockquote>\r",
        "should infer the first line ending (2)"
    );

    assert_eq!(
        micromark("> a\r\n"),
        "<blockquote>\r\n<p>a</p>\r\n</blockquote>\r\n",
        "should infer the first line ending (3)"
    );

    assert_eq!(
        micromark_with_options(
            "> a",
            &Options {
                default_line_ending: LineEnding::CarriageReturn,
                ..Options::default()
            }
        ),
        "<blockquote>\r<p>a</p>\r</blockquote>",
        "should support the given line ending"
    );

    assert_eq!(
        micromark_with_options(
            "> a\n",
            &Options {
                default_line_ending: LineEnding::CarriageReturn,
                ..Options::default()
            }
        ),
        // To do: is this a bug in `micromark.js` that it uses `\r` for earlier line endings?
        // "<blockquote>\r<p>a</p>\r</blockquote>\n",
        "<blockquote>\n<p>a</p>\n</blockquote>\n",
        "should support the given line ending, even if line endings exist"
    );
}
