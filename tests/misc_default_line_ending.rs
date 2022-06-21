extern crate micromark;
// use micromark::{micromark, micromark_with_options, Options};

#[test]
fn default_line_ending() {
    // To do: blockquote.
    // assert_eq!(
    //     micromark("> a"),
    //     "<blockquote>\n<p>a</p>\n</blockquote>",
    //     "should use `\\n` default"
    // );

    // assert_eq!(
    //     micromark("> a\n"),
    //     "<blockquote>\n<p>a</p>\n</blockquote>\n",
    //     "should infer the first line ending (1)"
    // );

    // assert_eq!(
    //     micromark("> a\r"),
    //     "<blockquote>\r<p>a</p>\r</blockquote>\r",
    //     "should infer the first line ending (2)"
    // );

    // assert_eq!(
    //     micromark("> a\r\n"),
    //     "<blockquote>\r\n<p>a</p>\r\n</blockquote>\r\n",
    //     "should infer the first line ending (3)"
    // );

    // assert_eq!(
    //     micromark_with_options(
    //         "> a",
    //         &Options {
    //             // default_line_ending: "\r",
    //             allow_dangerous_html: false,
    //             allow_dangerous_protocol: false
    //         }
    //     ),
    //     "<blockquote>\r<p>a</p>\r</blockquote>",
    //     "should support the given line ending"
    // );

    // assert_eq!(
    //     micromark_with_options(
    //         "> a\n",
    //         &Options {
    //             // default_line_ending: "\r",
    //             allow_dangerous_html: false,
    //             allow_dangerous_protocol: false
    //         }
    //     ),
    //     "<blockquote>\r<p>a</p>\r</blockquote>\n",
    //     "should support the given line ending, even if line endings exist"
    // );
}
