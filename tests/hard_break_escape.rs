extern crate micromark;
use micromark::{micromark};

#[test]
fn hard_break_escape() {
    // To do: trailing.
    // assert_eq!(
    //     micromark("foo  \nbaz"),
    //     "<p>foo<br />\nbaz</p>",
    //     "should support two trailing spaces to form a hard break"
    // );

    assert_eq!(
        micromark("foo\\\nbaz"),
        "<p>foo<br />\nbaz</p>",
        "should support a backslash to form a hard break"
    );

    // To do: trailing.
    // assert_eq!(
    //     micromark("foo       \nbaz"),
    //     "<p>foo<br />\nbaz</p>",
    //     "should support multiple trailing spaces"
    // );

    // To do: trailing.
    // assert_eq!(
    //     micromark("foo  \n     bar"),
    //     "<p>foo<br />\nbar</p>",
    //     "should support leading spaces after a trailing hard break"
    // );

    // To do: trim paragraph whitespace.
    // assert_eq!(
    //     micromark("foo\\\n     bar"),
    //     "<p>foo<br />\nbar</p>",
    //     "should support leading spaces after an escape hard break"
    // );

    // To do: trailing, attention.
    // assert_eq!(
    //     micromark("*foo  \nbar*"),
    //     "<p><em>foo<br />\nbar</em></p>",
    //     "should support trailing hard breaks in emphasis"
    // );

    // To do: attention.
    // assert_eq!(
    //     micromark("*foo\\\nbar*"),
    //     "<p><em>foo<br />\nbar</em></p>",
    //     "should support escape hard breaks in emphasis"
    // );

    assert_eq!(
        micromark("`code  \ntext`"),
        "<p><code>code   text</code></p>",
        "should not support trailing hard breaks in code"
    );

    assert_eq!(
        micromark("``code\\\ntext``"),
        "<p><code>code\\ text</code></p>",
        "should not support escape hard breaks in code"
    );

    // To do: paragraph trimming.
    // assert_eq!(
    //     micromark("foo  "),
    //     "<p>foo</p>",
    //     "should not support trailing hard breaks at the end of a paragraph"
    // );

    assert_eq!(
        micromark("foo\\"),
        "<p>foo\\</p>",
        "should not support escape hard breaks at the end of a paragraph"
    );

    assert_eq!(
        micromark("### foo\\"),
        "<h3>foo\\</h3>",
        "should not support escape hard breaks at the end of a heading"
    );

    assert_eq!(
        micromark("### foo  "),
        "<h3>foo</h3>",
        "should not support trailing hard breaks at the end of a heading"
    );

    // To do: paragraph trimming.
    // assert_eq!(
    //     micromark("aaa  \t\nbb"),
    //     "<p>aaa\nbb</p>",
    //     "should support a mixed line suffix (1)"
    // );

    // To do: paragraph trimming.
    // assert_eq!(
    //     micromark("aaa\t  \nbb"),
    //     "<p>aaa\nbb</p>",
    //     "should support a mixed line suffix (2)"
    // );

    // To do: paragraph trimming.
    // assert_eq!(
    //     micromark("aaa  \t  \nbb"),
    //     "<p>aaa\nbb</p>",
    //     "should support a mixed line suffix (3)"
    // );

    // To do: trailing.
    // assert_eq!(
    //     micromark("aaa\0  \nbb"),
    //     "<p>aaa�<br />\nbb</p>",
    //     "should support a hard break after a replacement character"
    // );

    // To do: trailing.
    // assert_eq!(
    //     micromark("aaa\0\t\nbb"),
    //     "<p>aaa�\nbb</p>",
    //     "should support a line suffix after a replacement character"
    // );

    // To do: attention, trailing.
    // assert_eq!(
    //     micromark("*a*  \nbb"),
    //     "<p><em>a</em><br />\nbb</p>",
    //     "should support a hard break after a span"
    // );

    // To do: attention, trailing.
    // assert_eq!(
    //     micromark("*a*\t\nbb"),
    //     "<p><em>a</em>\nbb</p>",
    //     "should support a line suffix after a span"
    // );

    // To do: attention, trailing.
    // assert_eq!(
    //     micromark("*a*  \t\nbb"),
    //     "<p><em>a</em>\nbb</p>",
    //     "should support a mixed line suffix after a span (1)"
    // );

    // To do: attention, trailing.
    // assert_eq!(
    //     micromark("*a*\t  \nbb"),
    //     "<p><em>a</em>\nbb</p>",
    //     "should support a mixed line suffix after a span (2)"
    // );

    // To do: attention, trailing.
    // assert_eq!(
    //     micromark("*a*  \t  \nbb"),
    //     "<p><em>a</em>\nbb</p>",
    //     "should support a mixed line suffix after a span (3)"
    // );

    // // To do: turning off things.
    // assert_eq!(
    //   micromark("a\\\nb", {extensions: [{disable: {null: ["hardBreakEscape"]}}]}),
    //   "<p>a\\\nb</p>",
    //   "should support turning off hard break (escape)"
    // );
}
