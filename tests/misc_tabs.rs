extern crate micromark;
use micromark::{micromark, micromark_with_options, Options};

#[test]
fn tabs_flow() {
    let danger = &Options {
        allow_dangerous_html: true,
        ..Options::default()
    };

    assert_eq!(
        micromark("    x"),
        "<pre><code>x\n</code></pre>",
        "should support a 4*SP to start code"
    );

    assert_eq!(
        micromark("\tx"),
        "<pre><code>x\n</code></pre>",
        "should support a HT to start code"
    );

    assert_eq!(
        micromark(" \tx"),
        "<pre><code>x\n</code></pre>",
        "should support a SP + HT to start code"
    );

    assert_eq!(
        micromark("  \tx"),
        "<pre><code>x\n</code></pre>",
        "should support a 2*SP + HT to start code"
    );

    assert_eq!(
        micromark("   \tx"),
        "<pre><code>x\n</code></pre>",
        "should support a 3*SP + HT to start code"
    );

    assert_eq!(
        micromark("    \tx"),
        "<pre><code>\tx\n</code></pre>",
        "should support a 4*SP to start code, and leave the next HT as code data"
    );

    assert_eq!(
        micromark("   \t# x"),
        "<pre><code># x\n</code></pre>",
        "should not support a 3*SP + HT to start an ATX heading"
    );

    assert_eq!(
        micromark("   \t> x"),
        "<pre><code>&gt; x\n</code></pre>",
        "should not support a 3*SP + HT to start a block quote"
    );

    assert_eq!(
        micromark("   \t- x"),
        "<pre><code>- x\n</code></pre>",
        "should not support a 3*SP + HT to start a list item"
    );

    assert_eq!(
        micromark("   \t---"),
        "<pre><code>---\n</code></pre>",
        "should not support a 3*SP + HT to start a thematic break"
    );

    assert_eq!(
        micromark("   \t---"),
        "<pre><code>---\n</code></pre>",
        "should not support a 3*SP + HT to start a thematic break"
    );

    assert_eq!(
        micromark("   \t```"),
        "<pre><code>```\n</code></pre>",
        "should not support a 3*SP + HT to start a fenced code"
    );

    assert_eq!(
        micromark("   \t<div>"),
        "<pre><code>&lt;div&gt;\n</code></pre>",
        "should not support a 3*SP + HT to start HTML"
    );

    assert_eq!(
        micromark("#\tx\t#\t"),
        "<h1>x</h1>",
        "should support tabs around ATX heading sequences"
    );

    assert_eq!(
        micromark("#\t\tx\t\t#\t\t"),
        "<h1>x</h1>",
        "should support arbitrary tabs around ATX heading sequences"
    );

    assert_eq!(
        micromark("```\tx\ty\t\n```\t"),
        "<pre><code class=\"language-x\"></code></pre>",
        "should support tabs around fenced code fences, info, and meta"
    );

    assert_eq!(
        micromark("```\t\tx\t\ty\t\t\n```\t\t"),
        "<pre><code class=\"language-x\"></code></pre>",
        "should support arbitrary tabs around fenced code fences, info, and meta"
    );

    assert_eq!(
        micromark("```x\n\t```"),
        "<pre><code class=\"language-x\">\t```\n</code></pre>\n",
        "should not support tabs before fenced code closing fences"
    );

    assert_eq!(
        micromark_with_options("<x\ty\tz\t=\t\"\tx\t\">", danger),
        "<x\ty\tz\t=\t\"\tx\t\">",
        "should support tabs in HTML (if whitespace is allowed)"
    );

    assert_eq!(
        micromark("*\t*\t*\t"),
        "<hr />",
        "should support tabs in thematic breaks"
    );

    assert_eq!(
        micromark("*\t\t*\t\t*\t\t"),
        "<hr />",
        "should support arbitrary tabs in thematic breaks"
    );
}

#[test]
fn tabs_text() {
    assert_eq!(
        micromark("<http:\t>"),
        "<p>&lt;http:\t&gt;</p>",
        "should not support a tab to start an autolink w/ protocol’s rest"
    );

    assert_eq!(
        micromark("<http:x\t>"),
        "<p>&lt;http:x\t&gt;</p>",
        "should not support a tab in an autolink w/ protocol’s rest"
    );

    assert_eq!(
        micromark("<example\t@x.com>"),
        "<p>&lt;example\t@x.com&gt;</p>",
        "should not support a tab in an email autolink’s local part"
    );

    assert_eq!(
        micromark("<example@x\ty.com>"),
        "<p>&lt;example@x\ty.com&gt;</p>",
        "should not support a tab in an email autolink’s label"
    );

    assert_eq!(
        micromark("\\\tx"),
        "<p>\\\tx</p>",
        "should not support character escaped tab"
    );

    assert_eq!(
        micromark("&#9;"),
        "<p>\t</p>",
        "should support character reference resolving to a tab"
    );

    assert_eq!(
        micromark("`\tx`"),
        "<p><code>\tx</code></p>",
        "should support a tab starting code"
    );

    assert_eq!(
        micromark("`x\t`"),
        "<p><code>x\t</code></p>",
        "should support a tab ending code"
    );

    assert_eq!(
        micromark("`\tx\t`"),
        "<p><code>\tx\t</code></p>",
        "should support tabs around code"
    );

    assert_eq!(
        micromark("`\tx `"),
        "<p><code>\tx </code></p>",
        "should support a tab starting, and a space ending, code"
    );

    assert_eq!(
        micromark("` x\t`"),
        "<p><code> x\t</code></p>",
        "should support a space starting, and a tab ending, code"
    );

    // Note: CM does not strip it in this case.
    // However, that should be a bug there: makes more sense to remove it like
    // trailing spaces.
    assert_eq!(
        micromark("x\t\ny"),
        "<p>x\ny</p>",
        "should support a trailing tab at a line ending in a paragraph"
    );

    assert_eq!(
        micromark("x\n\ty"),
        "<p>x\ny</p>",
        "should support an initial tab after a line ending in a paragraph"
    );

    assert_eq!(
        micromark("x[\ty](z)"),
        "<p>x<a href=\"z\">\ty</a></p>",
        "should support an initial tab in a link label"
    );

    assert_eq!(
        micromark("x[y\t](z)"),
        "<p>x<a href=\"z\">y\t</a></p>",
        "should support a final tab in a link label"
    );

    assert_eq!(
        micromark("[x\ty](z)"),
        "<p><a href=\"z\">x\ty</a></p>",
        "should support a tab in a link label"
    );

    // Note: CM.js bug, see: <https://github.com/commonmark/commonmark.js/issues/191>
    assert_eq!(
        micromark("[x](\ty)"),
        "<p><a href=\"y\">x</a></p>",
        "should support a tab starting a link resource"
    );

    assert_eq!(
        micromark("[x](y\t)"),
        "<p><a href=\"y\">x</a></p>",
        "should support a tab ending a link resource"
    );

    assert_eq!(
        micromark("[x](y\t\"z\")"),
        "<p><a href=\"y\" title=\"z\">x</a></p>",
        "should support a tab between a link destination and title"
    );
}

#[test]
fn tabs_virtual_spaces() {
    assert_eq!(
        micromark("```\n\tx"),
        "<pre><code>\tx\n</code></pre>\n",
        "should support a tab in fenced code"
    );

    assert_eq!(
        micromark(" ```\n\tx"),
        "<pre><code>   x\n</code></pre>\n",
        "should strip 1 space from an initial tab in fenced code if the opening fence is indented as such"
    );

    assert_eq!(
        micromark("  ```\n\tx"),
        "<pre><code>  x\n</code></pre>\n",
        "should strip 2 spaces from an initial tab in fenced code if the opening fence is indented as such"
    );

    assert_eq!(
        micromark("   ```\n\tx"),
        "<pre><code> x\n</code></pre>\n",
        "should strip 3 spaces from an initial tab in fenced code if the opening fence is indented as such"
    );
}
