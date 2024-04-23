use markdown::{
    mdast::{FootnoteDefinition, FootnoteReference, Node, Paragraph, Root, Text},
    message, to_html, to_html_with_options, to_mdast,
    unist::Position,
    CompileOptions, Options, ParseOptions,
};
use pretty_assertions::assert_eq;

#[test]
fn gfm_footnote() -> Result<(), message::Message> {
    assert_eq!(
        to_html("A call.[^a]\n\n[^a]: whatevs"),
        "<p>A call.<a href=\"whatevs\">^a</a></p>\n",
        "should ignore footnotes by default"
    );

    assert_eq!(
        to_html_with_options("A call.[^a]\n\n[^a]: whatevs", &Options::gfm())?,
        "<p>A call.<sup><a href=\"#user-content-fn-a\" id=\"user-content-fnref-a\" data-footnote-ref=\"\" aria-describedby=\"footnote-label\">1</a></sup></p>
<section data-footnotes=\"\" class=\"footnotes\"><h2 id=\"footnote-label\" class=\"sr-only\">Footnotes</h2>
<ol>
<li id=\"user-content-fn-a\">
<p>whatevs <a href=\"#user-content-fnref-a\" data-footnote-backref=\"\" aria-label=\"Back to content\" class=\"data-footnote-backref\">↩</a></p>
</li>
</ol>
</section>
",
        "should support footnotes"
    );

    assert_eq!(
        to_html_with_options(
            "Noot.[^a]\n\n[^a]: dingen",
            &Options {
                parse: ParseOptions::gfm(),
                compile: CompileOptions {
                    gfm_footnote_label: Some("Voetnoten".into()),
                    gfm_footnote_back_label: Some("Terug naar de inhoud".into()),
                    ..CompileOptions::gfm()
                }
            }
        )?,
        "<p>Noot.<sup><a href=\"#user-content-fn-a\" id=\"user-content-fnref-a\" data-footnote-ref=\"\" aria-describedby=\"footnote-label\">1</a></sup></p>
<section data-footnotes=\"\" class=\"footnotes\"><h2 id=\"footnote-label\" class=\"sr-only\">Voetnoten</h2>
<ol>
<li id=\"user-content-fn-a\">
<p>dingen <a href=\"#user-content-fnref-a\" data-footnote-backref=\"\" aria-label=\"Terug naar de inhoud\" class=\"data-footnote-backref\">↩</a></p>
</li>
</ol>
</section>
",
        "should support `options.gfm_footnote_label`, `options.gfm_footnote_back_label`"
    );

    assert_eq!(
        to_html_with_options(
            "[^a]\n\n[^a]: b",
            &Options {
                parse: ParseOptions::gfm(),
                compile: CompileOptions {
                    gfm_footnote_label_tag_name: Some("h1".into()),
                    ..CompileOptions::gfm()
                }
            }
        )?,
        "<p><sup><a href=\"#user-content-fn-a\" id=\"user-content-fnref-a\" data-footnote-ref=\"\" aria-describedby=\"footnote-label\">1</a></sup></p>
<section data-footnotes=\"\" class=\"footnotes\"><h1 id=\"footnote-label\" class=\"sr-only\">Footnotes</h1>
<ol>
<li id=\"user-content-fn-a\">
<p>b <a href=\"#user-content-fnref-a\" data-footnote-backref=\"\" aria-label=\"Back to content\" class=\"data-footnote-backref\">↩</a></p>
</li>
</ol>
</section>
",
        "should support `options.gfm_footnote_label_tag_name`"
    );

    assert_eq!(
        to_html_with_options(
            "[^a]\n\n[^a]: b",
            &Options {
                parse: ParseOptions::gfm(),
                compile: CompileOptions {
                    gfm_footnote_label_attributes: Some("class=\"footnote-heading\"".into()),
                    ..CompileOptions::gfm()
                }
            }
        )?,
        "<p><sup><a href=\"#user-content-fn-a\" id=\"user-content-fnref-a\" data-footnote-ref=\"\" aria-describedby=\"footnote-label\">1</a></sup></p>
<section data-footnotes=\"\" class=\"footnotes\"><h2 id=\"footnote-label\" class=\"footnote-heading\">Footnotes</h2>
<ol>
<li id=\"user-content-fn-a\">
<p>b <a href=\"#user-content-fnref-a\" data-footnote-backref=\"\" aria-label=\"Back to content\" class=\"data-footnote-backref\">↩</a></p>
</li>
</ol>
</section>
",
        "should support `options.gfm_footnote_label_attributes`"
    );

    assert_eq!(
        to_html_with_options(
            "[^a]\n\n[^a]: b",
            &Options {
                parse: ParseOptions::gfm(),
                compile: CompileOptions {
                    gfm_footnote_clobber_prefix: Some("".into()),
                    ..CompileOptions::gfm()
                }
            }
        )?,
        "<p><sup><a href=\"#fn-a\" id=\"fnref-a\" data-footnote-ref=\"\" aria-describedby=\"footnote-label\">1</a></sup></p>
<section data-footnotes=\"\" class=\"footnotes\"><h2 id=\"footnote-label\" class=\"sr-only\">Footnotes</h2>
<ol>
<li id=\"fn-a\">
<p>b <a href=\"#fnref-a\" data-footnote-backref=\"\" aria-label=\"Back to content\" class=\"data-footnote-backref\">↩</a></p>
</li>
</ol>
</section>
",
        "should support `options.gfm_footnote_clobber_prefix`"
    );

    assert_eq!(
        to_html_with_options("A paragraph.\n\n[^a]: whatevs", &Options::gfm())?,
        "<p>A paragraph.</p>\n",
        "should ignore definitions w/o calls"
    );

    assert_eq!(
        to_html_with_options("a[^b]", &Options::gfm())?,
        "<p>a[^b]</p>",
        "should ignore calls w/o definitions"
    );

    assert_eq!(
        to_html_with_options("a[^b]\n\n[^b]: c\n[^b]: d", &Options::gfm())?,
        "<p>a<sup><a href=\"#user-content-fn-b\" id=\"user-content-fnref-b\" data-footnote-ref=\"\" aria-describedby=\"footnote-label\">1</a></sup></p>
<section data-footnotes=\"\" class=\"footnotes\"><h2 id=\"footnote-label\" class=\"sr-only\">Footnotes</h2>
<ol>
<li id=\"user-content-fn-b\">
<p>c <a href=\"#user-content-fnref-b\" data-footnote-backref=\"\" aria-label=\"Back to content\" class=\"data-footnote-backref\">↩</a></p>
</li>
</ol>
</section>
",
        "should use the first of duplicate definitions"
    );

    assert_eq!(
        to_html_with_options("a[^b], c[^b]\n\n[^b]: d", &Options::gfm())?,
        "<p>a<sup><a href=\"#user-content-fn-b\" id=\"user-content-fnref-b\" data-footnote-ref=\"\" aria-describedby=\"footnote-label\">1</a></sup>, c<sup><a href=\"#user-content-fn-b\" id=\"user-content-fnref-b-2\" data-footnote-ref=\"\" aria-describedby=\"footnote-label\">1</a></sup></p>
<section data-footnotes=\"\" class=\"footnotes\"><h2 id=\"footnote-label\" class=\"sr-only\">Footnotes</h2>
<ol>
<li id=\"user-content-fn-b\">
<p>d <a href=\"#user-content-fnref-b\" data-footnote-backref=\"\" aria-label=\"Back to content\" class=\"data-footnote-backref\">↩</a> <a href=\"#user-content-fnref-b-2\" data-footnote-backref=\"\" aria-label=\"Back to content\" class=\"data-footnote-backref\">↩<sup>2</sup></a></p>
</li>
</ol>
</section>
",
        "should supports multiple calls to the same definition"
    );

    assert_eq!(
        to_html_with_options("![^a](b)", &Options::gfm())?,
        "<p>!<a href=\"b\">^a</a></p>",
        "should not support images starting w/ `^` (but see it as a link?!, 1)"
    );

    assert_eq!(
        to_html_with_options("![^a][b]\n\n[b]: c", &Options::gfm())?,
        "<p>!<a href=\"c\">^a</a></p>\n",
        "should not support images starting w/ `^` (but see it as a link?!, 2)"
    );

    assert_eq!(
        to_html_with_options("[^]()", &Options::gfm())?,
        "<p><a href=\"\">^</a></p>",
        "should support an empty link with caret"
    );

    assert_eq!(
        to_html_with_options("![^]()", &Options::gfm())?,
        "<p>!<a href=\"\">^</a></p>",
        "should support an empty image with caret (as link)"
    );

    // <https://github.com/github/cmark-gfm/issues/239>
    assert_eq!(
        to_html_with_options("Call.[^a\\+b].\n\n[^a\\+b]: y", &Options::gfm())?,
        "<p>Call.<sup><a href=\"#user-content-fn-a%5C+b\" id=\"user-content-fnref-a%5C+b\" data-footnote-ref=\"\" aria-describedby=\"footnote-label\">1</a></sup>.</p>
<section data-footnotes=\"\" class=\"footnotes\"><h2 id=\"footnote-label\" class=\"sr-only\">Footnotes</h2>
<ol>
<li id=\"user-content-fn-a%5C+b\">
<p>y <a href=\"#user-content-fnref-a%5C+b\" data-footnote-backref=\"\" aria-label=\"Back to content\" class=\"data-footnote-backref\">↩</a></p>
</li>
</ol>
</section>
",
        "should support a character escape in a call / definition"
    );

    assert_eq!(
        to_html_with_options("Call.[^a&copy;b].\n\n[^a&copy;b]: y", &Options::gfm())?,
        "<p>Call.<sup><a href=\"#user-content-fn-a&amp;copy;b\" id=\"user-content-fnref-a&amp;copy;b\" data-footnote-ref=\"\" aria-describedby=\"footnote-label\">1</a></sup>.</p>
<section data-footnotes=\"\" class=\"footnotes\"><h2 id=\"footnote-label\" class=\"sr-only\">Footnotes</h2>
<ol>
<li id=\"user-content-fn-a&amp;copy;b\">
<p>y <a href=\"#user-content-fnref-a&amp;copy;b\" data-footnote-backref=\"\" aria-label=\"Back to content\" class=\"data-footnote-backref\">↩</a></p>
</li>
</ol>
</section>
",
        "should support a character reference in a call / definition"
    );

    // <https://github.com/github/cmark-gfm/issues/239>
    // <https://github.com/github/cmark-gfm/issues/240>
    assert_eq!(
        to_html_with_options("Call.[^a\\]b].\n\n[^a\\]b]: y", &Options::gfm())?,
        "<p>Call.<sup><a href=\"#user-content-fn-a%5C%5Db\" id=\"user-content-fnref-a%5C%5Db\" data-footnote-ref=\"\" aria-describedby=\"footnote-label\">1</a></sup>.</p>
<section data-footnotes=\"\" class=\"footnotes\"><h2 id=\"footnote-label\" class=\"sr-only\">Footnotes</h2>
<ol>
<li id=\"user-content-fn-a%5C%5Db\">
<p>y <a href=\"#user-content-fnref-a%5C%5Db\" data-footnote-backref=\"\" aria-label=\"Back to content\" class=\"data-footnote-backref\">↩</a></p>
</li>
</ol>
</section>
",
        "should support a useful character escape in a call / definition"
    );

    assert_eq!(
        to_html_with_options("Call.[^a&#91;b].\n\n[^a&#91;b]: y", &Options::gfm())?,
        "<p>Call.<sup><a href=\"#user-content-fn-a&amp;#91;b\" id=\"user-content-fnref-a&amp;#91;b\" data-footnote-ref=\"\" aria-describedby=\"footnote-label\">1</a></sup>.</p>
<section data-footnotes=\"\" class=\"footnotes\"><h2 id=\"footnote-label\" class=\"sr-only\">Footnotes</h2>
<ol>
<li id=\"user-content-fn-a&amp;#91;b\">
<p>y <a href=\"#user-content-fnref-a&amp;#91;b\" data-footnote-backref=\"\" aria-label=\"Back to content\" class=\"data-footnote-backref\">↩</a></p>
</li>
</ol>
</section>
",
        "should support a useful character reference in a call / definition"
    );

    assert_eq!(
        to_html_with_options("Call.[^a\\+b].\n\n[^a+b]: y", &Options::gfm())?,
        "<p>Call.[^a+b].</p>\n",
        "should match calls to definitions on the source of the label, not on resolved escapes"
    );

    assert_eq!(
        to_html_with_options("Call.[^a&#91;b].\n\n[^a\\[b]: y", &Options::gfm())?,
        "<p>Call.[^a[b].</p>\n",
        "should match calls to definitions on the source of the label, not on resolved references"
    );

    assert_eq!(
        to_html_with_options("[^1].\n\n[^1]: a\nb", &Options::gfm())?,
        "<p><sup><a href=\"#user-content-fn-1\" id=\"user-content-fnref-1\" data-footnote-ref=\"\" aria-describedby=\"footnote-label\">1</a></sup>.</p>
<section data-footnotes=\"\" class=\"footnotes\"><h2 id=\"footnote-label\" class=\"sr-only\">Footnotes</h2>
<ol>
<li id=\"user-content-fn-1\">
<p>a
b <a href=\"#user-content-fnref-1\" data-footnote-backref=\"\" aria-label=\"Back to content\" class=\"data-footnote-backref\">↩</a></p>
</li>
</ol>
</section>
",
        "should support lazyness (1)"
    );

    assert_eq!(
        to_html_with_options("[^1].\n\n> [^1]: a\nb", &Options::gfm())?,
        "<p><sup><a href=\"#user-content-fn-1\" id=\"user-content-fnref-1\" data-footnote-ref=\"\" aria-describedby=\"footnote-label\">1</a></sup>.</p>
<blockquote>
</blockquote>
<section data-footnotes=\"\" class=\"footnotes\"><h2 id=\"footnote-label\" class=\"sr-only\">Footnotes</h2>
<ol>
<li id=\"user-content-fn-1\">
<p>a
b <a href=\"#user-content-fnref-1\" data-footnote-backref=\"\" aria-label=\"Back to content\" class=\"data-footnote-backref\">↩</a></p>
</li>
</ol>
</section>
",
        "should support lazyness (2)"
    );

    assert_eq!(
        to_html_with_options("[^1].\n\n> [^1]: a\n> b", &Options::gfm())?,
        "<p><sup><a href=\"#user-content-fn-1\" id=\"user-content-fnref-1\" data-footnote-ref=\"\" aria-describedby=\"footnote-label\">1</a></sup>.</p>
<blockquote>
</blockquote>
<section data-footnotes=\"\" class=\"footnotes\"><h2 id=\"footnote-label\" class=\"sr-only\">Footnotes</h2>
<ol>
<li id=\"user-content-fn-1\">
<p>a
b <a href=\"#user-content-fnref-1\" data-footnote-backref=\"\" aria-label=\"Back to content\" class=\"data-footnote-backref\">↩</a></p>
</li>
</ol>
</section>
",
        "should support lazyness (3)"
    );

    assert_eq!(
        to_html_with_options("[^1].\n\n[^1]: a\n\n    > b", &Options::gfm())?,
        "<p><sup><a href=\"#user-content-fn-1\" id=\"user-content-fnref-1\" data-footnote-ref=\"\" aria-describedby=\"footnote-label\">1</a></sup>.</p>
<section data-footnotes=\"\" class=\"footnotes\"><h2 id=\"footnote-label\" class=\"sr-only\">Footnotes</h2>
<ol>
<li id=\"user-content-fn-1\">
<p>a</p>
<blockquote>
<p>b</p>
</blockquote>
<a href=\"#user-content-fnref-1\" data-footnote-backref=\"\" aria-label=\"Back to content\" class=\"data-footnote-backref\">↩</a>
</li>
</ol>
</section>
",
        "should support lazyness (4)"
    );

    // 999 `x` characters.
    let max = "x".repeat(999);

    assert_eq!(
        to_html_with_options(format!("Call.[^{}].\n\n[^{}]: y", max, max).as_str(), &Options::gfm())?,
        format!("<p>Call.<sup><a href=\"#user-content-fn-{}\" id=\"user-content-fnref-{}\" data-footnote-ref=\"\" aria-describedby=\"footnote-label\">1</a></sup>.</p>
<section data-footnotes=\"\" class=\"footnotes\"><h2 id=\"footnote-label\" class=\"sr-only\">Footnotes</h2>
<ol>
<li id=\"user-content-fn-{}\">
<p>y <a href=\"#user-content-fnref-{}\" data-footnote-backref=\"\" aria-label=\"Back to content\" class=\"data-footnote-backref\">↩</a></p>
</li>
</ol>
</section>
", max, max, max, max),
        "should support 999 characters in a call / definition"
    );

    assert_eq!(
        to_html_with_options(
            format!("Call.[^a{}].\n\n[^a{}]: y", max, max).as_str(),
            &Options::gfm()
        )?,
        format!("<p>Call.[^a{}].</p>\n<p>[^a{}]: y</p>", max, max),
        "should not support 1000 characters in a call / definition"
    );

    assert_eq!(
        to_html_with_options(
            "[^a]\n\n[^a]: b\n  \n    c",
            &Options::gfm()
        )?,
        "<p><sup><a href=\"#user-content-fn-a\" id=\"user-content-fnref-a\" data-footnote-ref=\"\" aria-describedby=\"footnote-label\">1</a></sup></p>
<section data-footnotes=\"\" class=\"footnotes\"><h2 id=\"footnote-label\" class=\"sr-only\">Footnotes</h2>
<ol>
<li id=\"user-content-fn-a\">
<p>b</p>
<p>c <a href=\"#user-content-fnref-a\" data-footnote-backref=\"\" aria-label=\"Back to content\" class=\"data-footnote-backref\">↩</a></p>
</li>
</ol>
</section>\n",
        "should support blank lines in footnote definitions"
    );

    assert_eq!(
        to_html_with_options(
            r"a![i](#)
a\![i](#)
a![i][]
a![^1]
[^1]
^1]

[^1]: b

[i]: c",
            &Options::gfm()
        )?,
        r##"<p>a<img src="#" alt="i" />
a!<a href="#">i</a>
a<img src="c" alt="i" />
a!<sup><a href="#user-content-fn-1" id="user-content-fnref-1" data-footnote-ref="" aria-describedby="footnote-label">1</a></sup>
<sup><a href="#user-content-fn-1" id="user-content-fnref-1-2" data-footnote-ref="" aria-describedby="footnote-label">1</a></sup>
^1]</p>
<section data-footnotes="" class="footnotes"><h2 id="footnote-label" class="sr-only">Footnotes</h2>
<ol>
<li id="user-content-fn-1">
<p>b <a href="#user-content-fnref-1" data-footnote-backref="" aria-label="Back to content" class="data-footnote-backref">↩</a> <a href="#user-content-fnref-1-2" data-footnote-backref="" aria-label="Back to content" class="data-footnote-backref">↩<sup>2</sup></a></p>
</li>
</ol>
</section>
"##,
        "should match bang/caret interplay like GitHub"
    );

    assert_eq!(
        to_html_with_options("a![^1]", &Options::gfm())?,
        "<p>a![^1]</p>",
        "should match bang/caret interplay (undefined) like GitHub"
    );

    assert_eq!(
        to_html_with_options(
            r###"a![^1]

[^1]: b
"###,
            &Options::gfm()
        )?,
        r##"<p>a!<sup><a href="#user-content-fn-1" id="user-content-fnref-1" data-footnote-ref="" aria-describedby="footnote-label">1</a></sup></p>
<section data-footnotes="" class="footnotes"><h2 id="footnote-label" class="sr-only">Footnotes</h2>
<ol>
<li id="user-content-fn-1">
<p>b <a href="#user-content-fnref-1" data-footnote-backref="" aria-label="Back to content" class="data-footnote-backref">↩</a></p>
</li>
</ol>
</section>
"##,
        "should match bang/caret like GitHub"
    );

    assert_eq!(
        to_html_with_options(
            r###"Calls may not be empty: [^].

Calls cannot contain whitespace only: [^ ].

Calls cannot contain whitespace at all: [^ ], [^	], [^
].

Calls can contain other characters, such as numbers [^1234567890], or [^^]
even another caret.

[^]: empty

[^ ]: space

[^	]: tab

[^
]&#x3A; line feed

[^1234567890]: numbers

[^^]: caret
"###,
            &Options::gfm()
        )?,
        r##"<p>Calls may not be empty: <a href="empty">^</a>.</p>
<p>Calls cannot contain whitespace only: <a href="empty">^ </a>.</p>
<p>Calls cannot contain whitespace at all: <a href="empty">^ </a>, <a href="empty">^	</a>, <a href="empty">^
</a>.</p>
<p>Calls can contain other characters, such as numbers <sup><a href="#user-content-fn-1234567890" id="user-content-fnref-1234567890" data-footnote-ref="" aria-describedby="footnote-label">1</a></sup>, or <sup><a href="#user-content-fn-%5E" id="user-content-fnref-%5E" data-footnote-ref="" aria-describedby="footnote-label">2</a></sup>
even another caret.</p>
<p><a href="empty">^
</a>: line feed</p>
<section data-footnotes="" class="footnotes"><h2 id="footnote-label" class="sr-only">Footnotes</h2>
<ol>
<li id="user-content-fn-1234567890">
<p>numbers <a href="#user-content-fnref-1234567890" data-footnote-backref="" aria-label="Back to content" class="data-footnote-backref">↩</a></p>
</li>
<li id="user-content-fn-%5E">
<p>caret <a href="#user-content-fnref-%5E" data-footnote-backref="" aria-label="Back to content" class="data-footnote-backref">↩</a></p>
</li>
</ol>
</section>
"##,
        "should match calls like GitHub"
    );

    // Note:
    // * GH does not support line ending in call.
    //   See: <https://github.com/github/cmark-gfm/issues/282>
    //   Here line endings don’t make text disappear.
    assert_eq!(
        to_html_with_options(
            r###"[^a]: # b

[^c d]: # e

[^f	g]: # h

[^i
j]: # k

[^ l]: # l

[^m ]: # m

xxx[^a], [^c d], [^f	g], [^i
j], [^ l], [^m ]

---

Some calls.[^ w][^x ][^y][^z]

[^w]: # w

[^x]: # x

[^ y]: # y

[^x ]: # z
"###,
            &Options::gfm()
        )?,
        r##"<p>[^c d]: # e</p>
<p>[^f	g]: # h</p>
<p>[^i
j]: # k</p>
<p>[^ l]: # l</p>
<p>[^m ]: # m</p>
<p>xxx<sup><a href="#user-content-fn-a" id="user-content-fnref-a" data-footnote-ref="" aria-describedby="footnote-label">1</a></sup>, [^c d], [^f	g], [^i
j], [^ l], [^m ]</p>
<hr />
<p>Some calls.<sup><a href="#user-content-fn-w" id="user-content-fnref-w" data-footnote-ref="" aria-describedby="footnote-label">2</a></sup><sup><a href="#user-content-fn-x" id="user-content-fnref-x" data-footnote-ref="" aria-describedby="footnote-label">3</a></sup>[^y][^z]</p>
<p>[^ y]: # y</p>
<p><sup><a href="#user-content-fn-x" id="user-content-fnref-x-2" data-footnote-ref="" aria-describedby="footnote-label">3</a></sup>: # z</p>
<section data-footnotes="" class="footnotes"><h2 id="footnote-label" class="sr-only">Footnotes</h2>
<ol>
<li id="user-content-fn-a">
<h1>b</h1>
<a href="#user-content-fnref-a" data-footnote-backref="" aria-label="Back to content" class="data-footnote-backref">↩</a>
</li>
<li id="user-content-fn-w">
<h1>w</h1>
<a href="#user-content-fnref-w" data-footnote-backref="" aria-label="Back to content" class="data-footnote-backref">↩</a>
</li>
<li id="user-content-fn-x">
<h1>x</h1>
<a href="#user-content-fnref-x" data-footnote-backref="" aria-label="Back to content" class="data-footnote-backref">↩</a> <a href="#user-content-fnref-x-2" data-footnote-backref="" aria-label="Back to content" class="data-footnote-backref">↩<sup>2</sup></a>
</li>
</ol>
</section>
"##,
        "should match whitespace in calls like GitHub (except for the bugs)"
    );

    assert_eq!(
        to_html_with_options(
            r###"[^*emphasis*]

[^**strong**]

[^`code`]

[^www.example.com]

[^https://example.com]

[^://example.com]

[^[link](#)]

[^![image](#)]

[^*emphasis*]: a

[^**strong**]: a

[^`code`]: a

[^www.example.com]: a

[^https://example.com]: a

[^://example.com]: a

[^[link](#)]: a

[^![image](#)]: a
"###,
            &Options::gfm()
        )?,
        // Note:
        // * GH does not support colons.
        //   See: <https://github.com/github/cmark-gfm/issues/250>
        //   Here identifiers that include colons *do* work (so they’re added below).
        // * GH does not support footnote-like brackets around an image.
        //   See: <https://github.com/github/cmark-gfm/issues/275>
        //   Here images are fine.
        r##"<p><sup><a href="#user-content-fn-*emphasis*" id="user-content-fnref-*emphasis*" data-footnote-ref="" aria-describedby="footnote-label">1</a></sup></p>
<p><sup><a href="#user-content-fn-**strong**" id="user-content-fnref-**strong**" data-footnote-ref="" aria-describedby="footnote-label">2</a></sup></p>
<p><sup><a href="#user-content-fn-%60code%60" id="user-content-fnref-%60code%60" data-footnote-ref="" aria-describedby="footnote-label">3</a></sup></p>
<p><sup><a href="#user-content-fn-www.example.com" id="user-content-fnref-www.example.com" data-footnote-ref="" aria-describedby="footnote-label">4</a></sup></p>
<p><sup><a href="#user-content-fn-https://example.com" id="user-content-fnref-https://example.com" data-footnote-ref="" aria-describedby="footnote-label">5</a></sup></p>
<p><sup><a href="#user-content-fn-://example.com" id="user-content-fnref-://example.com" data-footnote-ref="" aria-describedby="footnote-label">6</a></sup></p>
<p>[^<a href="#">link</a>]</p>
<p>[^<img src="#" alt="image" />]</p>
<p>[^<a href="#">link</a>]: a</p>
<p>[^<img src="#" alt="image" />]: a</p>
<section data-footnotes="" class="footnotes"><h2 id="footnote-label" class="sr-only">Footnotes</h2>
<ol>
<li id="user-content-fn-*emphasis*">
<p>a <a href="#user-content-fnref-*emphasis*" data-footnote-backref="" aria-label="Back to content" class="data-footnote-backref">↩</a></p>
</li>
<li id="user-content-fn-**strong**">
<p>a <a href="#user-content-fnref-**strong**" data-footnote-backref="" aria-label="Back to content" class="data-footnote-backref">↩</a></p>
</li>
<li id="user-content-fn-%60code%60">
<p>a <a href="#user-content-fnref-%60code%60" data-footnote-backref="" aria-label="Back to content" class="data-footnote-backref">↩</a></p>
</li>
<li id="user-content-fn-www.example.com">
<p>a <a href="#user-content-fnref-www.example.com" data-footnote-backref="" aria-label="Back to content" class="data-footnote-backref">↩</a></p>
</li>
<li id="user-content-fn-https://example.com">
<p>a <a href="#user-content-fnref-https://example.com" data-footnote-backref="" aria-label="Back to content" class="data-footnote-backref">↩</a></p>
</li>
<li id="user-content-fn-://example.com">
<p>a <a href="#user-content-fnref-://example.com" data-footnote-backref="" aria-label="Back to content" class="data-footnote-backref">↩</a></p>
</li>
</ol>
</section>
"##,
        "should match construct identifiers like GitHub (except for its bugs)"
    );

    assert_eq!(
        to_html_with_options(
            r###"Call[^1][^2][^3][^4]

> [^1]: Defined in a block quote.
>
> More.
[^2]: Directly after a block quote.

* [^3]: Defined in a list item.

  More.
[^4]: Directly after a list item.
"###,
            &Options::gfm()
        )?,
        r##"<p>Call<sup><a href="#user-content-fn-1" id="user-content-fnref-1" data-footnote-ref="" aria-describedby="footnote-label">1</a></sup><sup><a href="#user-content-fn-2" id="user-content-fnref-2" data-footnote-ref="" aria-describedby="footnote-label">2</a></sup><sup><a href="#user-content-fn-3" id="user-content-fnref-3" data-footnote-ref="" aria-describedby="footnote-label">3</a></sup><sup><a href="#user-content-fn-4" id="user-content-fnref-4" data-footnote-ref="" aria-describedby="footnote-label">4</a></sup></p>
<blockquote>
<p>More.</p>
</blockquote>
<ul>
<li>
<p>More.</p>
</li>
</ul>
<section data-footnotes="" class="footnotes"><h2 id="footnote-label" class="sr-only">Footnotes</h2>
<ol>
<li id="user-content-fn-1">
<p>Defined in a block quote. <a href="#user-content-fnref-1" data-footnote-backref="" aria-label="Back to content" class="data-footnote-backref">↩</a></p>
</li>
<li id="user-content-fn-2">
<p>Directly after a block quote. <a href="#user-content-fnref-2" data-footnote-backref="" aria-label="Back to content" class="data-footnote-backref">↩</a></p>
</li>
<li id="user-content-fn-3">
<p>Defined in a list item. <a href="#user-content-fnref-3" data-footnote-backref="" aria-label="Back to content" class="data-footnote-backref">↩</a></p>
</li>
<li id="user-content-fn-4">
<p>Directly after a list item. <a href="#user-content-fnref-4" data-footnote-backref="" aria-label="Back to content" class="data-footnote-backref">↩</a></p>
</li>
</ol>
</section>
"##,
        "should match containers like GitHub"
    );

    assert_eq!(
        to_html_with_options(
            r###"[^1][^2][^3][^4]

[^1]: Paragraph
…continuation

# Heading

[^2]: Paragraph
…continuation

    “code”, which is paragraphs…

    …because of the indent!

[^3]: Paragraph
…continuation

> block quote

[^4]: Paragraph
…continuation

- list
"###,
            &Options::gfm()
        )?,
        r##"<p><sup><a href="#user-content-fn-1" id="user-content-fnref-1" data-footnote-ref="" aria-describedby="footnote-label">1</a></sup><sup><a href="#user-content-fn-2" id="user-content-fnref-2" data-footnote-ref="" aria-describedby="footnote-label">2</a></sup><sup><a href="#user-content-fn-3" id="user-content-fnref-3" data-footnote-ref="" aria-describedby="footnote-label">3</a></sup><sup><a href="#user-content-fn-4" id="user-content-fnref-4" data-footnote-ref="" aria-describedby="footnote-label">4</a></sup></p>
<h1>Heading</h1>
<blockquote>
<p>block quote</p>
</blockquote>
<ul>
<li>list</li>
</ul>
<section data-footnotes="" class="footnotes"><h2 id="footnote-label" class="sr-only">Footnotes</h2>
<ol>
<li id="user-content-fn-1">
<p>Paragraph
…continuation <a href="#user-content-fnref-1" data-footnote-backref="" aria-label="Back to content" class="data-footnote-backref">↩</a></p>
</li>
<li id="user-content-fn-2">
<p>Paragraph
…continuation</p>
<p>“code”, which is paragraphs…</p>
<p>…because of the indent! <a href="#user-content-fnref-2" data-footnote-backref="" aria-label="Back to content" class="data-footnote-backref">↩</a></p>
</li>
<li id="user-content-fn-3">
<p>Paragraph
…continuation <a href="#user-content-fnref-3" data-footnote-backref="" aria-label="Back to content" class="data-footnote-backref">↩</a></p>
</li>
<li id="user-content-fn-4">
<p>Paragraph
…continuation <a href="#user-content-fnref-4" data-footnote-backref="" aria-label="Back to content" class="data-footnote-backref">↩</a></p>
</li>
</ol>
</section>
"##,
        "should match continuation like GitHub"
    );

    assert_eq!(
        to_html_with_options(
            r###"Call[^1][^2][^3][^4][^5].

[^1]:
    ---

[^2]:
    Paragraph.

[^3]:
Lazy?

[^4]:

    Another blank.

[^5]:

Lazy!
"###,
            &Options::gfm()
        )?,
        r##"<p>Call<sup><a href="#user-content-fn-1" id="user-content-fnref-1" data-footnote-ref="" aria-describedby="footnote-label">1</a></sup><sup><a href="#user-content-fn-2" id="user-content-fnref-2" data-footnote-ref="" aria-describedby="footnote-label">2</a></sup><sup><a href="#user-content-fn-3" id="user-content-fnref-3" data-footnote-ref="" aria-describedby="footnote-label">3</a></sup><sup><a href="#user-content-fn-4" id="user-content-fnref-4" data-footnote-ref="" aria-describedby="footnote-label">4</a></sup><sup><a href="#user-content-fn-5" id="user-content-fnref-5" data-footnote-ref="" aria-describedby="footnote-label">5</a></sup>.</p>
<p>Lazy?</p>
<p>Lazy!</p>
<section data-footnotes="" class="footnotes"><h2 id="footnote-label" class="sr-only">Footnotes</h2>
<ol>
<li id="user-content-fn-1">
<hr />
<a href="#user-content-fnref-1" data-footnote-backref="" aria-label="Back to content" class="data-footnote-backref">↩</a>
</li>
<li id="user-content-fn-2">
<p>Paragraph. <a href="#user-content-fnref-2" data-footnote-backref="" aria-label="Back to content" class="data-footnote-backref">↩</a></p>
</li>
<li id="user-content-fn-3">
<a href="#user-content-fnref-3" data-footnote-backref="" aria-label="Back to content" class="data-footnote-backref">↩</a>
</li>
<li id="user-content-fn-4">
<p>Another blank. <a href="#user-content-fnref-4" data-footnote-backref="" aria-label="Back to content" class="data-footnote-backref">↩</a></p>
</li>
<li id="user-content-fn-5">
<a href="#user-content-fnref-5" data-footnote-backref="" aria-label="Back to content" class="data-footnote-backref">↩</a>
</li>
</ol>
</section>
"##,
        "should match definitions initial blank like GitHub"
    );

    assert_eq!(
        to_html_with_options(
            r###"Note![^0][^1][^2][^3][^4][^5][^6][^7][^8][^9][^10]

[^0]: alpha

[^1]: bravo

[^2]: charlie
    indented delta

[^3]:    echo

[^4]:     foxtrot

[^5]:> golf

[^6]:    > hotel

[^7]:     > india

[^8]: # juliett

[^9]: ---

[^10]:- - - kilo
"###,
            &Options::gfm()
        )?,
        r##"<p>Note!<sup><a href="#user-content-fn-0" id="user-content-fnref-0" data-footnote-ref="" aria-describedby="footnote-label">1</a></sup><sup><a href="#user-content-fn-1" id="user-content-fnref-1" data-footnote-ref="" aria-describedby="footnote-label">2</a></sup><sup><a href="#user-content-fn-2" id="user-content-fnref-2" data-footnote-ref="" aria-describedby="footnote-label">3</a></sup><sup><a href="#user-content-fn-3" id="user-content-fnref-3" data-footnote-ref="" aria-describedby="footnote-label">4</a></sup><sup><a href="#user-content-fn-4" id="user-content-fnref-4" data-footnote-ref="" aria-describedby="footnote-label">5</a></sup><sup><a href="#user-content-fn-5" id="user-content-fnref-5" data-footnote-ref="" aria-describedby="footnote-label">6</a></sup><sup><a href="#user-content-fn-6" id="user-content-fnref-6" data-footnote-ref="" aria-describedby="footnote-label">7</a></sup><sup><a href="#user-content-fn-7" id="user-content-fnref-7" data-footnote-ref="" aria-describedby="footnote-label">8</a></sup><sup><a href="#user-content-fn-8" id="user-content-fnref-8" data-footnote-ref="" aria-describedby="footnote-label">9</a></sup><sup><a href="#user-content-fn-9" id="user-content-fnref-9" data-footnote-ref="" aria-describedby="footnote-label">10</a></sup><sup><a href="#user-content-fn-10" id="user-content-fnref-10" data-footnote-ref="" aria-describedby="footnote-label">11</a></sup></p>
<section data-footnotes="" class="footnotes"><h2 id="footnote-label" class="sr-only">Footnotes</h2>
<ol>
<li id="user-content-fn-0">
<p>alpha <a href="#user-content-fnref-0" data-footnote-backref="" aria-label="Back to content" class="data-footnote-backref">↩</a></p>
</li>
<li id="user-content-fn-1">
<p>bravo <a href="#user-content-fnref-1" data-footnote-backref="" aria-label="Back to content" class="data-footnote-backref">↩</a></p>
</li>
<li id="user-content-fn-2">
<p>charlie
indented delta <a href="#user-content-fnref-2" data-footnote-backref="" aria-label="Back to content" class="data-footnote-backref">↩</a></p>
</li>
<li id="user-content-fn-3">
<p>echo <a href="#user-content-fnref-3" data-footnote-backref="" aria-label="Back to content" class="data-footnote-backref">↩</a></p>
</li>
<li id="user-content-fn-4">
<p>foxtrot <a href="#user-content-fnref-4" data-footnote-backref="" aria-label="Back to content" class="data-footnote-backref">↩</a></p>
</li>
<li id="user-content-fn-5">
<blockquote>
<p>golf</p>
</blockquote>
<a href="#user-content-fnref-5" data-footnote-backref="" aria-label="Back to content" class="data-footnote-backref">↩</a>
</li>
<li id="user-content-fn-6">
<blockquote>
<p>hotel</p>
</blockquote>
<a href="#user-content-fnref-6" data-footnote-backref="" aria-label="Back to content" class="data-footnote-backref">↩</a>
</li>
<li id="user-content-fn-7">
<blockquote>
<p>india</p>
</blockquote>
<a href="#user-content-fnref-7" data-footnote-backref="" aria-label="Back to content" class="data-footnote-backref">↩</a>
</li>
<li id="user-content-fn-8">
<h1>juliett</h1>
<a href="#user-content-fnref-8" data-footnote-backref="" aria-label="Back to content" class="data-footnote-backref">↩</a>
</li>
<li id="user-content-fn-9">
<hr />
<a href="#user-content-fnref-9" data-footnote-backref="" aria-label="Back to content" class="data-footnote-backref">↩</a>
</li>
<li id="user-content-fn-10">
<ul>
<li>
<ul>
<li>
<ul>
<li>kilo</li>
</ul>
</li>
</ul>
</li>
</ul>
<a href="#user-content-fnref-10" data-footnote-backref="" aria-label="Back to content" class="data-footnote-backref">↩</a>
</li>
</ol>
</section>
"##,
        "should match definitions like GitHub"
    );

    assert_eq!(
        to_html_with_options(
            r###"Call[^1][^1]

[^1]: Recursion[^1][^1]
"###,
            &Options::gfm()
        )?,
        r##"<p>Call<sup><a href="#user-content-fn-1" id="user-content-fnref-1" data-footnote-ref="" aria-describedby="footnote-label">1</a></sup><sup><a href="#user-content-fn-1" id="user-content-fnref-1-2" data-footnote-ref="" aria-describedby="footnote-label">1</a></sup></p>
<section data-footnotes="" class="footnotes"><h2 id="footnote-label" class="sr-only">Footnotes</h2>
<ol>
<li id="user-content-fn-1">
<p>Recursion<sup><a href="#user-content-fn-1" id="user-content-fnref-1-3" data-footnote-ref="" aria-describedby="footnote-label">1</a></sup><sup><a href="#user-content-fn-1" id="user-content-fnref-1-4" data-footnote-ref="" aria-describedby="footnote-label">1</a></sup> <a href="#user-content-fnref-1" data-footnote-backref="" aria-label="Back to content" class="data-footnote-backref">↩</a> <a href="#user-content-fnref-1-2" data-footnote-backref="" aria-label="Back to content" class="data-footnote-backref">↩<sup>2</sup></a> <a href="#user-content-fnref-1-3" data-footnote-backref="" aria-label="Back to content" class="data-footnote-backref">↩<sup>3</sup></a> <a href="#user-content-fnref-1-4" data-footnote-backref="" aria-label="Back to content" class="data-footnote-backref">↩<sup>4</sup></a></p>
</li>
</ol>
</section>
"##,
        "should match duplicate calls and recursion like GitHub"
    );

    assert_eq!(
        to_html_with_options(
            r###"Call[^1]

[^1]: a

[^1]: b
"###,
            &Options::gfm()
        )?,
        r##"<p>Call<sup><a href="#user-content-fn-1" id="user-content-fnref-1" data-footnote-ref="" aria-describedby="footnote-label">1</a></sup></p>
<section data-footnotes="" class="footnotes"><h2 id="footnote-label" class="sr-only">Footnotes</h2>
<ol>
<li id="user-content-fn-1">
<p>a <a href="#user-content-fnref-1" data-footnote-backref="" aria-label="Back to content" class="data-footnote-backref">↩</a></p>
</li>
</ol>
</section>
"##,
        "should match duplicate definitions like GitHub"
    );

    // Note:
    // * GH “supports” footnotes inside links.
    //   This breaks an HTML parser, as it is not allowed.
    //   See: <https://github.com/github/cmark-gfm/issues/275>
    //   CommonMark has mechanisms in place to prevent links in links.
    //   These mechanisms are in place here too.
    assert_eq!(
        to_html_with_options(
            r###"*emphasis[^1]*

**strong[^2]**

`code[^3]`

![image[^4]](#)

[link[^5]](#)

[^1]: a

[^2]: b

[^3]: c

[^4]: d

[^5]: e
"###,
            &Options::gfm()
        )?,
        r##"<p><em>emphasis<sup><a href="#user-content-fn-1" id="user-content-fnref-1" data-footnote-ref="" aria-describedby="footnote-label">1</a></sup></em></p>
<p><strong>strong<sup><a href="#user-content-fn-2" id="user-content-fnref-2" data-footnote-ref="" aria-describedby="footnote-label">2</a></sup></strong></p>
<p><code>code[^3]</code></p>
<p><img src="#" alt="image" /></p>
<p>[link<sup><a href="#user-content-fn-5" id="user-content-fnref-5" data-footnote-ref="" aria-describedby="footnote-label">4</a></sup>](#)</p>
<section data-footnotes="" class="footnotes"><h2 id="footnote-label" class="sr-only">Footnotes</h2>
<ol>
<li id="user-content-fn-1">
<p>a <a href="#user-content-fnref-1" data-footnote-backref="" aria-label="Back to content" class="data-footnote-backref">↩</a></p>
</li>
<li id="user-content-fn-2">
<p>b <a href="#user-content-fnref-2" data-footnote-backref="" aria-label="Back to content" class="data-footnote-backref">↩</a></p>
</li>
<li id="user-content-fn-4">
<p>d <a href="#user-content-fnref-4" data-footnote-backref="" aria-label="Back to content" class="data-footnote-backref">↩</a></p>
</li>
<li id="user-content-fn-5">
<p>e <a href="#user-content-fnref-5" data-footnote-backref="" aria-label="Back to content" class="data-footnote-backref">↩</a></p>
</li>
</ol>
</section>
"##,
        "should match footnotes in constructs like GitHub (without the bugs)"
    );

    assert_eq!(
        to_html_with_options(
            r###"What are these![^1], ![^2][], and ![this][^3].

[^1]: a

[^2]: b

[^3]: c
"###,
            &Options::gfm()
        )?,
        r##"<p>What are these!<sup><a href="#user-content-fn-1" id="user-content-fnref-1" data-footnote-ref="" aria-describedby="footnote-label">1</a></sup>, !<sup><a href="#user-content-fn-2" id="user-content-fnref-2" data-footnote-ref="" aria-describedby="footnote-label">2</a></sup>[], and ![this]<sup><a href="#user-content-fn-3" id="user-content-fnref-3" data-footnote-ref="" aria-describedby="footnote-label">3</a></sup>.</p>
<section data-footnotes="" class="footnotes"><h2 id="footnote-label" class="sr-only">Footnotes</h2>
<ol>
<li id="user-content-fn-1">
<p>a <a href="#user-content-fnref-1" data-footnote-backref="" aria-label="Back to content" class="data-footnote-backref">↩</a></p>
</li>
<li id="user-content-fn-2">
<p>b <a href="#user-content-fnref-2" data-footnote-backref="" aria-label="Back to content" class="data-footnote-backref">↩</a></p>
</li>
<li id="user-content-fn-3">
<p>c <a href="#user-content-fnref-3" data-footnote-backref="" aria-label="Back to content" class="data-footnote-backref">↩</a></p>
</li>
</ol>
</section>
"##,
        "should match images/footnotes like GitHub"
    );

    assert_eq!(
        to_html_with_options(
            r###"[^0][^1][^2][^3][^4][^5]

[^0]: Paragraph
…continuation
[^1]: Another
[^2]: Paragraph
…continuation
# Heading
[^3]: Paragraph
…continuation
    “code”, which is paragraphs…

    …because of the indent!
[^4]: Paragraph
…continuation
> block quote
[^5]: Paragraph
…continuation
*   list
"###,
            &Options::gfm()
        )?,
        r##"<p><sup><a href="#user-content-fn-0" id="user-content-fnref-0" data-footnote-ref="" aria-describedby="footnote-label">1</a></sup><sup><a href="#user-content-fn-1" id="user-content-fnref-1" data-footnote-ref="" aria-describedby="footnote-label">2</a></sup><sup><a href="#user-content-fn-2" id="user-content-fnref-2" data-footnote-ref="" aria-describedby="footnote-label">3</a></sup><sup><a href="#user-content-fn-3" id="user-content-fnref-3" data-footnote-ref="" aria-describedby="footnote-label">4</a></sup><sup><a href="#user-content-fn-4" id="user-content-fnref-4" data-footnote-ref="" aria-describedby="footnote-label">5</a></sup><sup><a href="#user-content-fn-5" id="user-content-fnref-5" data-footnote-ref="" aria-describedby="footnote-label">6</a></sup></p>
<h1>Heading</h1>
<blockquote>
<p>block quote</p>
</blockquote>
<ul>
<li>list</li>
</ul>
<section data-footnotes="" class="footnotes"><h2 id="footnote-label" class="sr-only">Footnotes</h2>
<ol>
<li id="user-content-fn-0">
<p>Paragraph
…continuation <a href="#user-content-fnref-0" data-footnote-backref="" aria-label="Back to content" class="data-footnote-backref">↩</a></p>
</li>
<li id="user-content-fn-1">
<p>Another <a href="#user-content-fnref-1" data-footnote-backref="" aria-label="Back to content" class="data-footnote-backref">↩</a></p>
</li>
<li id="user-content-fn-2">
<p>Paragraph
…continuation <a href="#user-content-fnref-2" data-footnote-backref="" aria-label="Back to content" class="data-footnote-backref">↩</a></p>
</li>
<li id="user-content-fn-3">
<p>Paragraph
…continuation
“code”, which is paragraphs…</p>
<p>…because of the indent! <a href="#user-content-fnref-3" data-footnote-backref="" aria-label="Back to content" class="data-footnote-backref">↩</a></p>
</li>
<li id="user-content-fn-4">
<p>Paragraph
…continuation <a href="#user-content-fnref-4" data-footnote-backref="" aria-label="Back to content" class="data-footnote-backref">↩</a></p>
</li>
<li id="user-content-fn-5">
<p>Paragraph
…continuation <a href="#user-content-fnref-5" data-footnote-backref="" aria-label="Back to content" class="data-footnote-backref">↩</a></p>
</li>
</ol>
</section>
"##,
        "should match interrupt like GitHub"
    );

    assert_eq!(
        to_html_with_options(
            r###"What are these[^1], [^2][], and [this][^3].

[^1]: a

[^2]: b

[^3]: c
"###,
            &Options::gfm()
        )?,
        r##"<p>What are these<sup><a href="#user-content-fn-1" id="user-content-fnref-1" data-footnote-ref="" aria-describedby="footnote-label">1</a></sup>, <sup><a href="#user-content-fn-2" id="user-content-fnref-2" data-footnote-ref="" aria-describedby="footnote-label">2</a></sup>[], and [this]<sup><a href="#user-content-fn-3" id="user-content-fnref-3" data-footnote-ref="" aria-describedby="footnote-label">3</a></sup>.</p>
<section data-footnotes="" class="footnotes"><h2 id="footnote-label" class="sr-only">Footnotes</h2>
<ol>
<li id="user-content-fn-1">
<p>a <a href="#user-content-fnref-1" data-footnote-backref="" aria-label="Back to content" class="data-footnote-backref">↩</a></p>
</li>
<li id="user-content-fn-2">
<p>b <a href="#user-content-fnref-2" data-footnote-backref="" aria-label="Back to content" class="data-footnote-backref">↩</a></p>
</li>
<li id="user-content-fn-3">
<p>c <a href="#user-content-fnref-3" data-footnote-backref="" aria-label="Back to content" class="data-footnote-backref">↩</a></p>
</li>
</ol>
</section>
"##,
        "should match links/footnotes like GitHub"
    );

    assert_eq!(
        to_html_with_options(
            r###"[^1][^2][^3][^4]

[^1]: Paragraph


# Heading


[^2]: Paragraph


    “code”, which is paragraphs…


    …because of the indent!


[^3]: Paragraph


> block quote


[^4]: Paragraph


- list
"###,
            &Options::gfm()
        )?,
        r##"<p><sup><a href="#user-content-fn-1" id="user-content-fnref-1" data-footnote-ref="" aria-describedby="footnote-label">1</a></sup><sup><a href="#user-content-fn-2" id="user-content-fnref-2" data-footnote-ref="" aria-describedby="footnote-label">2</a></sup><sup><a href="#user-content-fn-3" id="user-content-fnref-3" data-footnote-ref="" aria-describedby="footnote-label">3</a></sup><sup><a href="#user-content-fn-4" id="user-content-fnref-4" data-footnote-ref="" aria-describedby="footnote-label">4</a></sup></p>
<h1>Heading</h1>
<blockquote>
<p>block quote</p>
</blockquote>
<ul>
<li>list</li>
</ul>
<section data-footnotes="" class="footnotes"><h2 id="footnote-label" class="sr-only">Footnotes</h2>
<ol>
<li id="user-content-fn-1">
<p>Paragraph <a href="#user-content-fnref-1" data-footnote-backref="" aria-label="Back to content" class="data-footnote-backref">↩</a></p>
</li>
<li id="user-content-fn-2">
<p>Paragraph</p>
<p>“code”, which is paragraphs…</p>
<p>…because of the indent! <a href="#user-content-fnref-2" data-footnote-backref="" aria-label="Back to content" class="data-footnote-backref">↩</a></p>
</li>
<li id="user-content-fn-3">
<p>Paragraph <a href="#user-content-fnref-3" data-footnote-backref="" aria-label="Back to content" class="data-footnote-backref">↩</a></p>
</li>
<li id="user-content-fn-4">
<p>Paragraph <a href="#user-content-fnref-4" data-footnote-backref="" aria-label="Back to content" class="data-footnote-backref">↩</a></p>
</li>
</ol>
</section>
"##,
        "should match many blank lines/no indent like GitHub"
    );

    assert_eq!(
        to_html_with_options(
            r###"[^1][^2][^3][^4]

[^1]: Paragraph


    # Heading


[^2]: Paragraph


        code


        more code


[^3]: Paragraph


    > block quote


[^4]: Paragraph


    - list
"###,
            &Options::gfm()
        )?,
        r##"<p><sup><a href="#user-content-fn-1" id="user-content-fnref-1" data-footnote-ref="" aria-describedby="footnote-label">1</a></sup><sup><a href="#user-content-fn-2" id="user-content-fnref-2" data-footnote-ref="" aria-describedby="footnote-label">2</a></sup><sup><a href="#user-content-fn-3" id="user-content-fnref-3" data-footnote-ref="" aria-describedby="footnote-label">3</a></sup><sup><a href="#user-content-fn-4" id="user-content-fnref-4" data-footnote-ref="" aria-describedby="footnote-label">4</a></sup></p>
<section data-footnotes="" class="footnotes"><h2 id="footnote-label" class="sr-only">Footnotes</h2>
<ol>
<li id="user-content-fn-1">
<p>Paragraph</p>
<h1>Heading</h1>
<a href="#user-content-fnref-1" data-footnote-backref="" aria-label="Back to content" class="data-footnote-backref">↩</a>
</li>
<li id="user-content-fn-2">
<p>Paragraph</p>
<pre><code>code


more code
</code></pre>
<a href="#user-content-fnref-2" data-footnote-backref="" aria-label="Back to content" class="data-footnote-backref">↩</a>
</li>
<li id="user-content-fn-3">
<p>Paragraph</p>
<blockquote>
<p>block quote</p>
</blockquote>
<a href="#user-content-fnref-3" data-footnote-backref="" aria-label="Back to content" class="data-footnote-backref">↩</a>
</li>
<li id="user-content-fn-4">
<p>Paragraph</p>
<ul>
<li>list</li>
</ul>
<a href="#user-content-fnref-4" data-footnote-backref="" aria-label="Back to content" class="data-footnote-backref">↩</a>
</li>
</ol>
</section>
"##,
        "should match many blank lines like GitHub"
    );

    assert_eq!(
        to_html_with_options(
            r###"Note![^1][^2][^3][^4]

- [^1]: Paragraph

> [^2]: Paragraph

[^3]: [^4]: Paragraph
"###,
            &Options::gfm()
        )?,
        r##"<p>Note!<sup><a href="#user-content-fn-1" id="user-content-fnref-1" data-footnote-ref="" aria-describedby="footnote-label">1</a></sup><sup><a href="#user-content-fn-2" id="user-content-fnref-2" data-footnote-ref="" aria-describedby="footnote-label">2</a></sup><sup><a href="#user-content-fn-3" id="user-content-fnref-3" data-footnote-ref="" aria-describedby="footnote-label">3</a></sup><sup><a href="#user-content-fn-4" id="user-content-fnref-4" data-footnote-ref="" aria-describedby="footnote-label">4</a></sup></p>
<ul>
<li></li>
</ul>
<blockquote>
</blockquote>
<section data-footnotes="" class="footnotes"><h2 id="footnote-label" class="sr-only">Footnotes</h2>
<ol>
<li id="user-content-fn-1">
<p>Paragraph <a href="#user-content-fnref-1" data-footnote-backref="" aria-label="Back to content" class="data-footnote-backref">↩</a></p>
</li>
<li id="user-content-fn-2">
<p>Paragraph <a href="#user-content-fnref-2" data-footnote-backref="" aria-label="Back to content" class="data-footnote-backref">↩</a></p>
</li>
<li id="user-content-fn-3">
<a href="#user-content-fnref-3" data-footnote-backref="" aria-label="Back to content" class="data-footnote-backref">↩</a>
</li>
<li id="user-content-fn-4">
<p>Paragraph <a href="#user-content-fnref-4" data-footnote-backref="" aria-label="Back to content" class="data-footnote-backref">↩</a></p>
</li>
</ol>
</section>
"##,
        "should match nest like GitHub"
    );

    assert_eq!(
        to_html_with_options(
            r###"[^1][^2][^3][^4]

[^1]: Paragraph

# Heading

[^2]: Paragraph

    “code”, which is paragraphs…

    …because of the indent!

[^3]: Paragraph

> block quote

[^4]: Paragraph

- list
"###,
            &Options::gfm()
        )?,
        r##"<p><sup><a href="#user-content-fn-1" id="user-content-fnref-1" data-footnote-ref="" aria-describedby="footnote-label">1</a></sup><sup><a href="#user-content-fn-2" id="user-content-fnref-2" data-footnote-ref="" aria-describedby="footnote-label">2</a></sup><sup><a href="#user-content-fn-3" id="user-content-fnref-3" data-footnote-ref="" aria-describedby="footnote-label">3</a></sup><sup><a href="#user-content-fn-4" id="user-content-fnref-4" data-footnote-ref="" aria-describedby="footnote-label">4</a></sup></p>
<h1>Heading</h1>
<blockquote>
<p>block quote</p>
</blockquote>
<ul>
<li>list</li>
</ul>
<section data-footnotes="" class="footnotes"><h2 id="footnote-label" class="sr-only">Footnotes</h2>
<ol>
<li id="user-content-fn-1">
<p>Paragraph <a href="#user-content-fnref-1" data-footnote-backref="" aria-label="Back to content" class="data-footnote-backref">↩</a></p>
</li>
<li id="user-content-fn-2">
<p>Paragraph</p>
<p>“code”, which is paragraphs…</p>
<p>…because of the indent! <a href="#user-content-fnref-2" data-footnote-backref="" aria-label="Back to content" class="data-footnote-backref">↩</a></p>
</li>
<li id="user-content-fn-3">
<p>Paragraph <a href="#user-content-fnref-3" data-footnote-backref="" aria-label="Back to content" class="data-footnote-backref">↩</a></p>
</li>
<li id="user-content-fn-4">
<p>Paragraph <a href="#user-content-fnref-4" data-footnote-backref="" aria-label="Back to content" class="data-footnote-backref">↩</a></p>
</li>
</ol>
</section>
"##,
        "should match normal blank lines/no indent like GitHub"
    );

    assert_eq!(
        to_html_with_options(
            r###"[^1][^2][^3][^4]

[^1]: Paragraph

    # Heading

[^2]: Paragraph

        code

        more code

[^3]: Paragraph

    > block quote

[^4]: Paragraph

    - list
"###,
            &Options::gfm()
        )?,
        r##"<p><sup><a href="#user-content-fn-1" id="user-content-fnref-1" data-footnote-ref="" aria-describedby="footnote-label">1</a></sup><sup><a href="#user-content-fn-2" id="user-content-fnref-2" data-footnote-ref="" aria-describedby="footnote-label">2</a></sup><sup><a href="#user-content-fn-3" id="user-content-fnref-3" data-footnote-ref="" aria-describedby="footnote-label">3</a></sup><sup><a href="#user-content-fn-4" id="user-content-fnref-4" data-footnote-ref="" aria-describedby="footnote-label">4</a></sup></p>
<section data-footnotes="" class="footnotes"><h2 id="footnote-label" class="sr-only">Footnotes</h2>
<ol>
<li id="user-content-fn-1">
<p>Paragraph</p>
<h1>Heading</h1>
<a href="#user-content-fnref-1" data-footnote-backref="" aria-label="Back to content" class="data-footnote-backref">↩</a>
</li>
<li id="user-content-fn-2">
<p>Paragraph</p>
<pre><code>code

more code
</code></pre>
<a href="#user-content-fnref-2" data-footnote-backref="" aria-label="Back to content" class="data-footnote-backref">↩</a>
</li>
<li id="user-content-fn-3">
<p>Paragraph</p>
<blockquote>
<p>block quote</p>
</blockquote>
<a href="#user-content-fnref-3" data-footnote-backref="" aria-label="Back to content" class="data-footnote-backref">↩</a>
</li>
<li id="user-content-fn-4">
<p>Paragraph</p>
<ul>
<li>list</li>
</ul>
<a href="#user-content-fnref-4" data-footnote-backref="" aria-label="Back to content" class="data-footnote-backref">↩</a>
</li>
</ol>
</section>
"##,
        "should match normal blank lines like GitHub"
    );

    assert_eq!(
        to_html_with_options(
            r###"Here is a footnote reference,[^1] and another.[^longnote]

[^1]: Here is the footnote.

[^longnote]: Here’s one with multiple blocks.

    Subsequent paragraphs are indented to show that they
belong to the previous footnote.

        { some.code }

    The whole paragraph can be indented, or just the first
    line.  In this way, multi-paragraph footnotes work like
    multi-paragraph list items.

This paragraph won’t be part of the note, because it
isn’t indented.
"###,
            &Options::gfm()
        )?,
        r##"<p>Here is a footnote reference,<sup><a href="#user-content-fn-1" id="user-content-fnref-1" data-footnote-ref="" aria-describedby="footnote-label">1</a></sup> and another.<sup><a href="#user-content-fn-longnote" id="user-content-fnref-longnote" data-footnote-ref="" aria-describedby="footnote-label">2</a></sup></p>
<p>This paragraph won’t be part of the note, because it
isn’t indented.</p>
<section data-footnotes="" class="footnotes"><h2 id="footnote-label" class="sr-only">Footnotes</h2>
<ol>
<li id="user-content-fn-1">
<p>Here is the footnote. <a href="#user-content-fnref-1" data-footnote-backref="" aria-label="Back to content" class="data-footnote-backref">↩</a></p>
</li>
<li id="user-content-fn-longnote">
<p>Here’s one with multiple blocks.</p>
<p>Subsequent paragraphs are indented to show that they
belong to the previous footnote.</p>
<pre><code>{ some.code }
</code></pre>
<p>The whole paragraph can be indented, or just the first
line.  In this way, multi-paragraph footnotes work like
multi-paragraph list items. <a href="#user-content-fnref-longnote" data-footnote-backref="" aria-label="Back to content" class="data-footnote-backref">↩</a></p>
</li>
</ol>
</section>
"##,
        "should match pandoc like GitHub"
    );

    assert_eq!(
        to_html_with_options(
            r###"Call[^1][^2][^3][^4][^5][^6][^7][^8][^9][^10][^11][^12].

     [^1]: 5

    [^2]: 4

   [^3]: 3

  [^4]: 2

 [^5]: 1

[^6]: 0

***

   [^7]: 3

     5

   [^8]: 3

    4

   [^9]: 3

   3

  [^10]: 2

     5

  [^11]: 2

    4

  [^12]: 2

   3
"###,
            &Options::gfm()
        )?,
        r##"<p>Call[^1][^2]<sup><a href="#user-content-fn-3" id="user-content-fnref-3" data-footnote-ref="" aria-describedby="footnote-label">1</a></sup><sup><a href="#user-content-fn-4" id="user-content-fnref-4" data-footnote-ref="" aria-describedby="footnote-label">2</a></sup><sup><a href="#user-content-fn-5" id="user-content-fnref-5" data-footnote-ref="" aria-describedby="footnote-label">3</a></sup><sup><a href="#user-content-fn-6" id="user-content-fnref-6" data-footnote-ref="" aria-describedby="footnote-label">4</a></sup><sup><a href="#user-content-fn-7" id="user-content-fnref-7" data-footnote-ref="" aria-describedby="footnote-label">5</a></sup><sup><a href="#user-content-fn-8" id="user-content-fnref-8" data-footnote-ref="" aria-describedby="footnote-label">6</a></sup><sup><a href="#user-content-fn-9" id="user-content-fnref-9" data-footnote-ref="" aria-describedby="footnote-label">7</a></sup><sup><a href="#user-content-fn-10" id="user-content-fnref-10" data-footnote-ref="" aria-describedby="footnote-label">8</a></sup><sup><a href="#user-content-fn-11" id="user-content-fnref-11" data-footnote-ref="" aria-describedby="footnote-label">9</a></sup><sup><a href="#user-content-fn-12" id="user-content-fnref-12" data-footnote-ref="" aria-describedby="footnote-label">10</a></sup>.</p>
<pre><code> [^1]: 5

[^2]: 4
</code></pre>
<hr />
<p>3</p>
<p>3</p>
<section data-footnotes="" class="footnotes"><h2 id="footnote-label" class="sr-only">Footnotes</h2>
<ol>
<li id="user-content-fn-3">
<p>3 <a href="#user-content-fnref-3" data-footnote-backref="" aria-label="Back to content" class="data-footnote-backref">↩</a></p>
</li>
<li id="user-content-fn-4">
<p>2 <a href="#user-content-fnref-4" data-footnote-backref="" aria-label="Back to content" class="data-footnote-backref">↩</a></p>
</li>
<li id="user-content-fn-5">
<p>1 <a href="#user-content-fnref-5" data-footnote-backref="" aria-label="Back to content" class="data-footnote-backref">↩</a></p>
</li>
<li id="user-content-fn-6">
<p>0 <a href="#user-content-fnref-6" data-footnote-backref="" aria-label="Back to content" class="data-footnote-backref">↩</a></p>
</li>
<li id="user-content-fn-7">
<p>3</p>
<p>5 <a href="#user-content-fnref-7" data-footnote-backref="" aria-label="Back to content" class="data-footnote-backref">↩</a></p>
</li>
<li id="user-content-fn-8">
<p>3</p>
<p>4 <a href="#user-content-fnref-8" data-footnote-backref="" aria-label="Back to content" class="data-footnote-backref">↩</a></p>
</li>
<li id="user-content-fn-9">
<p>3 <a href="#user-content-fnref-9" data-footnote-backref="" aria-label="Back to content" class="data-footnote-backref">↩</a></p>
</li>
<li id="user-content-fn-10">
<p>2</p>
<p>5 <a href="#user-content-fnref-10" data-footnote-backref="" aria-label="Back to content" class="data-footnote-backref">↩</a></p>
</li>
<li id="user-content-fn-11">
<p>2</p>
<p>4 <a href="#user-content-fnref-11" data-footnote-backref="" aria-label="Back to content" class="data-footnote-backref">↩</a></p>
</li>
<li id="user-content-fn-12">
<p>2 <a href="#user-content-fnref-12" data-footnote-backref="" aria-label="Back to content" class="data-footnote-backref">↩</a></p>
</li>
</ol>
</section>
"##,
        "should match prefix before like GitHub"
    );

    assert_eq!(
        to_html_with_options(
            r###"Call[^1][^2][^3][^4][^5][^6][^7][^8][^9].

[^1]: a

        8

[^2]: a

       7

[^3]: a

      6

[^4]: a

     5

[^5]: a

    4

[^6]: a

   3

[^7]: a

  2

[^8]: a

 1

[^9]: a

0
"###,
            &Options::gfm()
        )?,
        r##"<p>Call<sup><a href="#user-content-fn-1" id="user-content-fnref-1" data-footnote-ref="" aria-describedby="footnote-label">1</a></sup><sup><a href="#user-content-fn-2" id="user-content-fnref-2" data-footnote-ref="" aria-describedby="footnote-label">2</a></sup><sup><a href="#user-content-fn-3" id="user-content-fnref-3" data-footnote-ref="" aria-describedby="footnote-label">3</a></sup><sup><a href="#user-content-fn-4" id="user-content-fnref-4" data-footnote-ref="" aria-describedby="footnote-label">4</a></sup><sup><a href="#user-content-fn-5" id="user-content-fnref-5" data-footnote-ref="" aria-describedby="footnote-label">5</a></sup><sup><a href="#user-content-fn-6" id="user-content-fnref-6" data-footnote-ref="" aria-describedby="footnote-label">6</a></sup><sup><a href="#user-content-fn-7" id="user-content-fnref-7" data-footnote-ref="" aria-describedby="footnote-label">7</a></sup><sup><a href="#user-content-fn-8" id="user-content-fnref-8" data-footnote-ref="" aria-describedby="footnote-label">8</a></sup><sup><a href="#user-content-fn-9" id="user-content-fnref-9" data-footnote-ref="" aria-describedby="footnote-label">9</a></sup>.</p>
<p>3</p>
<p>2</p>
<p>1</p>
<p>0</p>
<section data-footnotes="" class="footnotes"><h2 id="footnote-label" class="sr-only">Footnotes</h2>
<ol>
<li id="user-content-fn-1">
<p>a</p>
<pre><code>8
</code></pre>
<a href="#user-content-fnref-1" data-footnote-backref="" aria-label="Back to content" class="data-footnote-backref">↩</a>
</li>
<li id="user-content-fn-2">
<p>a</p>
<p>7 <a href="#user-content-fnref-2" data-footnote-backref="" aria-label="Back to content" class="data-footnote-backref">↩</a></p>
</li>
<li id="user-content-fn-3">
<p>a</p>
<p>6 <a href="#user-content-fnref-3" data-footnote-backref="" aria-label="Back to content" class="data-footnote-backref">↩</a></p>
</li>
<li id="user-content-fn-4">
<p>a</p>
<p>5 <a href="#user-content-fnref-4" data-footnote-backref="" aria-label="Back to content" class="data-footnote-backref">↩</a></p>
</li>
<li id="user-content-fn-5">
<p>a</p>
<p>4 <a href="#user-content-fnref-5" data-footnote-backref="" aria-label="Back to content" class="data-footnote-backref">↩</a></p>
</li>
<li id="user-content-fn-6">
<p>a <a href="#user-content-fnref-6" data-footnote-backref="" aria-label="Back to content" class="data-footnote-backref">↩</a></p>
</li>
<li id="user-content-fn-7">
<p>a <a href="#user-content-fnref-7" data-footnote-backref="" aria-label="Back to content" class="data-footnote-backref">↩</a></p>
</li>
<li id="user-content-fn-8">
<p>a <a href="#user-content-fnref-8" data-footnote-backref="" aria-label="Back to content" class="data-footnote-backref">↩</a></p>
</li>
<li id="user-content-fn-9">
<p>a <a href="#user-content-fnref-9" data-footnote-backref="" aria-label="Back to content" class="data-footnote-backref">↩</a></p>
</li>
</ol>
</section>
"##,
        "should match prefix like GitHub"
    );

    assert_eq!(
        to_html_with_options(
            r###"Here is a short reference,[1], a collapsed one,[2][], and a full [one][3].

[1]: a

[2]: b

[3]: c
"###,
            &Options::gfm()
        )?,
        r#"<p>Here is a short reference,<a href="a">1</a>, a collapsed one,<a href="b">2</a>, and a full <a href="c">one</a>.</p>
"#,
        "should match references and definitions like GitHub"
    );

    assert_eq!(
        to_mdast("[^a]: b\n\tc\n\nd [^a] e.", &ParseOptions::gfm())?,
        Node::Root(Root {
            children: vec![
                Node::FootnoteDefinition(FootnoteDefinition {
                    children: vec![Node::Paragraph(Paragraph {
                        children: vec![Node::Text(Text {
                            value: "b\nc".into(),
                            position: Some(Position::new(1, 7, 6, 2, 6, 10))
                        })],
                        position: Some(Position::new(1, 7, 6, 2, 6, 10))
                    })],
                    identifier: "a".into(),
                    label: Some("a".into()),
                    position: Some(Position::new(1, 1, 0, 3, 1, 11))
                }),
                Node::Paragraph(Paragraph {
                    children: vec![
                        Node::Text(Text {
                            value: "d ".into(),
                            position: Some(Position::new(4, 1, 12, 4, 3, 14))
                        }),
                        Node::FootnoteReference(FootnoteReference {
                            identifier: "a".into(),
                            label: Some("a".into()),
                            position: Some(Position::new(4, 3, 14, 4, 7, 18))
                        }),
                        Node::Text(Text {
                            value: " e.".into(),
                            position: Some(Position::new(4, 7, 18, 4, 10, 21))
                        })
                    ],
                    position: Some(Position::new(4, 1, 12, 4, 10, 21))
                })
            ],
            position: Some(Position::new(1, 1, 0, 4, 10, 21))
        }),
        "should support GFM footnotes as `FootnoteDefinition`, `FootnoteReference`s in mdast"
    );

    Ok(())
}
