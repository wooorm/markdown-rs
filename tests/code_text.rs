extern crate micromark;
use micromark::{micromark, micromark_with_options, CompileOptions};

const DANGER: &CompileOptions = &CompileOptions {
    allow_dangerous_html: true,
    allow_dangerous_protocol: false,
};

#[test]
fn code_text() {
    assert_eq!(
        micromark("`foo`"),
        "<p><code>foo</code></p>",
        "should support code"
    );

    assert_eq!(
        micromark("`` foo ` bar ``"),
        "<p><code>foo ` bar</code></p>",
        "should support code w/ more accents"
    );

    assert_eq!(
        micromark("` `` `"),
        "<p><code>``</code></p>",
        "should support code w/ fences inside, and padding"
    );

    assert_eq!(
        micromark("`  ``  `"),
        "<p><code> `` </code></p>",
        "should support code w/ extra padding"
    );

    assert_eq!(
        micromark("` a`"),
        "<p><code> a</code></p>",
        "should support code w/ unbalanced padding"
    );

    assert_eq!(
        micromark("`\u{a0}b\u{a0}`"),
        "<p><code>\u{a0}b\u{a0}</code></p>",
        "should support code w/ non-padding whitespace"
    );

    assert_eq!(
        micromark("` `\n`  `"),
        "<p><code> </code>\n<code>  </code></p>",
        "should support code w/o data"
    );

    assert_eq!(
        micromark("``\nfoo\nbar  \nbaz\n``"),
        "<p><code>foo bar   baz</code></p>",
        "should support code w/o line endings (1)"
    );

    assert_eq!(
        micromark("``\nfoo \n``"),
        "<p><code>foo </code></p>",
        "should support code w/o line endings (2)"
    );

    assert_eq!(
        micromark("`foo   bar \nbaz`"),
        "<p><code>foo   bar  baz</code></p>",
        "should not support whitespace collapsing"
    );

    assert_eq!(
        micromark("`foo\\`bar`"),
        "<p><code>foo\\</code>bar`</p>",
        "should not support character escapes"
    );

    assert_eq!(
        micromark("``foo`bar``"),
        "<p><code>foo`bar</code></p>",
        "should support more accents"
    );

    assert_eq!(
        micromark("` foo `` bar `"),
        "<p><code>foo `` bar</code></p>",
        "should support less accents"
    );

    assert_eq!(
        micromark("*foo`*`"),
        "<p>*foo<code>*</code></p>",
        "should precede over emphasis"
    );

    assert_eq!(
        micromark("[not a `link](/foo`)"),
        "<p>[not a <code>link](/foo</code>)</p>",
        "should precede over links"
    );

    assert_eq!(
        micromark("`<a href=\"`\">`"),
        "<p><code>&lt;a href=&quot;</code>&quot;&gt;`</p>",
        "should have same precedence as HTML (1)"
    );

    assert_eq!(
        micromark_with_options("<a href=\"`\">`", DANGER),
        "<p><a href=\"`\">`</p>",
        "should have same precedence as HTML (2)"
    );

    assert_eq!(
        micromark("`<http://foo.bar.`baz>`"),
        "<p><code>&lt;http://foo.bar.</code>baz&gt;`</p>",
        "should have same precedence as autolinks (1)"
    );

    assert_eq!(
        micromark("<http://foo.bar.`baz>`"),
        "<p><a href=\"http://foo.bar.%60baz\">http://foo.bar.`baz</a>`</p>",
        "should have same precedence as autolinks (2)"
    );

    assert_eq!(
        micromark("```foo``"),
        "<p>```foo``</p>",
        "should not support more accents before a fence"
    );

    assert_eq!(
        micromark("`foo"),
        "<p>`foo</p>",
        "should not support no closing fence (1)"
    );

    assert_eq!(
        micromark("`foo``bar``"),
        "<p>`foo<code>bar</code></p>",
        "should not support no closing fence (2)"
    );

    // Extra:
    assert_eq!(
        micromark("`foo\t\tbar`"),
        "<p><code>foo\t\tbar</code></p>",
        "should support tabs in code"
    );

    assert_eq!(
        micromark("\\``x`"),
        "<p>`<code>x</code></p>",
        "should support an escaped initial grave accent"
    );

    // To do: turning things off.
    // assert_eq!(
    //   micromark("`a`", {extensions: [{disable: {null: ["codeText"]}}]}),
    //   "<p>`a`</p>",
    //   "should support turning off code (text)"
    // );
}
