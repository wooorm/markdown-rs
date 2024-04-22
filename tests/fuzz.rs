use markdown::{mdast, message, to_html, to_html_with_options, to_mdast, Options};
use pretty_assertions::assert_eq;

#[test]
fn fuzz() -> Result<(), message::Message> {
    assert_eq!(
        to_html("[\n~\na\n-\n\n"),
        "<h2>[\n~\na</h2>\n",
        "1: label, blank lines, and code"
    );

    // The first link is stopped by the `+` (so it’s `a@b.c`), but the next
    // link overlaps it (`b.c+d@e.f`).
    assert_eq!(
        to_html_with_options("a@b.c+d@e.f", &Options::gfm())?,
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

    assert_eq!(
        to_html("a * "),
        "<p>a *</p>",
        "4-a: trailing whitespace and broken data"
    );

    assert_eq!(
        to_html("_  "),
        "<p>_</p>",
        "4-b: trailing whitespace and broken data (GH-13)"
    );

    assert_eq!(
        to_html_with_options("a ~ ", &Options::gfm())?,
        "<p>a ~</p>",
        "4-c: trailing whitespace and broken data (GH-14)"
    );

    assert!(
        matches!(
            to_mdast("123456789. ok", &Default::default()),
            Ok(mdast::Node::Root(_))
        ),
        "5: lists should support high start numbers (GH-17)"
    );

    assert_eq!(
        to_html("> ```\n"),
        "<blockquote>\n<pre><code>\n</code></pre>\n</blockquote>",
        "6-a: container close after unclosed fenced code, with eol (block quote, GH-16)"
    );

    assert_eq!(
        to_html("- ```\n"),
        "<ul>\n<li>\n<pre><code>\n</code></pre>\n</li>\n</ul>",
        "6-b: container close after unclosed fenced code, with eol (list, GH-16)"
    );

    assert_eq!(
        to_html_with_options("> x\n``", &Options::gfm()),
        Ok("<blockquote>\n<p>x</p>\n</blockquote>\n<p>``</p>".into()),
        "7: lazy container lines almost starting fenced code (GH-19)"
    );

    assert_eq!(
        to_html_with_options("a\tb@c.d", &Options::gfm()),
        Ok("<p>a\t<a href=\"mailto:b@c.d\">b@c.d</a></p>".into()),
        "8-a: autolink literals after tabs (GH-18)"
    );

    assert_eq!(
        to_html_with_options("aa\tb@c.d", &Options::gfm()),
        Ok("<p>aa\t<a href=\"mailto:b@c.d\">b@c.d</a></p>".into()),
        "8-b: autolink literals after tabs (GH-18)"
    );

    assert_eq!(
        to_html_with_options("aaa\tb@c.d", &Options::gfm()),
        Ok("<p>aaa\t<a href=\"mailto:b@c.d\">b@c.d</a></p>".into()),
        "8-c: autolink literals after tabs (GH-18)"
    );

    assert_eq!(
        to_html_with_options("aaaa\tb@c.d", &Options::gfm()),
        Ok("<p>aaaa\t<a href=\"mailto:b@c.d\">b@c.d</a></p>".into()),
        "8-d: autolink literals after tabs (GH-18)"
    );

    assert_eq!(
        to_html_with_options("| a |\n| - |\n| www.a|", &Options::gfm()),
        Ok("<table>\n<thead>\n<tr>\n<th>a</th>\n</tr>\n</thead>\n<tbody>\n<tr>\n<td><a href=\"http://www.a\">www.a</a></td>\n</tr>\n</tbody>\n</table>".into()),
        "9: autolink literals that end in table cell delimiter (GH-20)"
    );

    assert_eq!(
        to_html_with_options("[*]() [*]()", &Options::gfm()),
        Ok("<p><a href=\"\">*</a> <a href=\"\">*</a></p>".into()),
        "10: attention in different links (GH-21)"
    );

    assert!(
        matches!(
            to_mdast("* [ ]\na", &Default::default()),
            Ok(mdast::Node::Root(_))
        ),
        "11: gfm task list items followed by eols (GH-24)"
    );

    assert_eq!(
        markdown::to_html_with_options(
            "<",
            &markdown::Options {
                parse: markdown::ParseOptions::mdx(),
                ..Default::default()
            }
        ),
        Ok("<p>&lt;</p>".to_string()),
        "12: mdx: handle invalid mdx without panic (GH-26)"
    );

    Ok(())
}
