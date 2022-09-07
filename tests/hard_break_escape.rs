extern crate micromark;
use micromark::{micromark, micromark_with_options, Constructs, Options};
use pretty_assertions::assert_eq;

#[test]
fn hard_break_escape() -> Result<(), String> {
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

    assert_eq!(
        micromark("*foo\\\nbar*"),
        "<p><em>foo<br />\nbar</em></p>",
        "should support escape hard breaks in emphasis"
    );

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

    assert_eq!(
        micromark_with_options(
            "a\\\nb",
            &Options {
                constructs: Constructs {
                    hard_break_escape: false,
                    ..Constructs::default()
                },
                ..Options::default()
            }
        )?,
        "<p>a\\\nb</p>",
        "should support turning off hard break (escape)"
    );

    Ok(())
}
