extern crate markdown;
use markdown::{to_html, to_html_with_options, CompileOptions, Constructs, Options, ParseOptions};
use pretty_assertions::assert_eq;

#[test]
fn fuzz() -> Result<(), String> {
    assert_eq!(
        to_html("[\n~\na\n-\n\n"),
        "<h2>[\n~\na</h2>\n",
        "1: label, blank lines, and code"
    );

    // The first link is stopped by the `+` (so it’s `a@b.c`), but the next
    // link overlaps it (`b.c+d@e.f`).
    assert_eq!(
        to_html_with_options(
            "a@b.c+d@e.f",
            &Options {
                parse: ParseOptions {
                    constructs: Constructs::gfm(),
                    ..ParseOptions::default()
                },
                compile: CompileOptions {
                    gfm_tagfilter: true,
                    ..CompileOptions::default()
                }
            }
        )?,
        "<p><a href=\"mailto:a@b.c\">a@b.c</a><a href=\"mailto:+d@e.f\">+d@e.f</a></p>",
        "2: gfm: email autolink literals running into each other"
    );

    assert_eq!(
        to_html("    x\n*    "),
        "<pre><code>x\n</code></pre>\n<ul>\n<li></li>\n</ul>",
        "3-a: containers should not pierce into indented code"
    );

    assert_eq!(
        to_html("    a\n*     b"),
        "<pre><code>a\n</code></pre>\n<ul>\n<li>\n<pre><code>b\n</code></pre>\n</li>\n</ul>",
        "3-b: containers should not pierce into indented code"
    );

    Ok(())
}
