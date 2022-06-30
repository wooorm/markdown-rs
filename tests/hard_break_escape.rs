extern crate micromark;
use micromark::micromark;

#[test]
fn hard_break_escape() {
    assert_eq!(
        micromark("foo\\\nbaz"),
        "<p>foo<br />\nbaz</p>",
        "should support a backslash to form a hard break"
    );

    assert_eq!(
        micromark("foo\\\n     bar"),
        "<p>foo<br />\nbar</p>",
        "should support leading spaces after an escape hard break"
    );

    // To do: attention.
    // assert_eq!(
    //     micromark("*foo\\\nbar*"),
    //     "<p><em>foo<br />\nbar</em></p>",
    //     "should support escape hard breaks in emphasis"
    // );

    assert_eq!(
        micromark("``code\\\ntext``"),
        "<p><code>code\\ text</code></p>",
        "should not support escape hard breaks in code"
    );

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

    // // To do: turning off things.
    // assert_eq!(
    //   micromark("a\\\nb", {extensions: [{disable: {null: ["hardBreakEscape"]}}]}),
    //   "<p>a\\\nb</p>",
    //   "should support turning off hard break (escape)"
    // );
}
