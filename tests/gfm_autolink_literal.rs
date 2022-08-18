extern crate micromark;
use micromark::{micromark, micromark_with_options, Constructs, Options};
use pretty_assertions::assert_eq;

#[test]
fn gfm_autolink_literal() {
    let gfm = Options {
        constructs: Constructs::gfm(),
        ..Options::default()
    };

    assert_eq!(
        micromark("https://example.com"),
        "<p>https://example.com</p>",
        "should ignore protocol urls by default"
    );
    assert_eq!(
        micromark("www.example.com"),
        "<p>www.example.com</p>",
        "should ignore www urls by default"
    );
    assert_eq!(
        micromark("user@example.com"),
        "<p>user@example.com</p>",
        "should ignore email urls by default"
    );

    assert_eq!(
        micromark_with_options("https://example.com", &gfm),
        "<p><a href=\"https://example.com\">https://example.com</a></p>",
        "should support protocol urls if enabled"
    );
    assert_eq!(
        micromark_with_options("www.example.com", &gfm),
        "<p><a href=\"http://www.example.com\">www.example.com</a></p>",
        "should support www urls if enabled"
    );
    assert_eq!(
        micromark_with_options("user@example.com", &gfm),
        "<p><a href=\"mailto:user@example.com\">user@example.com</a></p>",
        "should support email urls if enabled"
    );

    assert_eq!(
        micromark_with_options("user@example.com", &gfm),
        "<p><a href=\"mailto:user@example.com\">user@example.com</a></p>",
        "should support a closing paren at TLD (email)"
    );

    assert_eq!(
        micromark_with_options("www.a.)", &gfm),
        "<p><a href=\"http://www.a\">www.a</a>.)</p>",
        "should support a closing paren at TLD (www)"
    );

    assert_eq!(
        micromark_with_options("www.a b", &gfm),
        "<p><a href=\"http://www.a\">www.a</a> b</p>",
        "should support no TLD"
    );

    assert_eq!(
        micromark_with_options("www.a/b c", &gfm),
        "<p><a href=\"http://www.a/b\">www.a/b</a> c</p>",
        "should support a path instead of TLD"
    );

    assert_eq!(
        micromark_with_options("www.�a", &gfm),
        "<p><a href=\"http://www.%EF%BF%BDa\">www.�a</a></p>",
        "should support a replacement character in a domain"
    );

    assert_eq!(
        micromark_with_options("http://點看.com", &gfm),
        "<p><a href=\"http://%E9%BB%9E%E7%9C%8B.com\">http://點看.com</a></p>",
        "should support non-ascii characters in a domain (http)"
    );

    assert_eq!(
        micromark_with_options("www.點看.com", &gfm),
        "<p><a href=\"http://www.%E9%BB%9E%E7%9C%8B.com\">www.點看.com</a></p>",
        "should support non-ascii characters in a domain (www)"
    );

    assert_eq!(
        micromark_with_options("點看@example.com", &gfm),
        "<p>點看@example.com</p>",
        "should *not* support non-ascii characters in atext (email)"
    );

    assert_eq!(
        micromark_with_options("example@點看.com", &gfm),
        "<p>example@點看.com</p>",
        "should *not* support non-ascii characters in a domain (email)"
    );

    assert_eq!(
        micromark_with_options("www.a.com/點看", &gfm),
        "<p><a href=\"http://www.a.com/%E9%BB%9E%E7%9C%8B\">www.a.com/點看</a></p>",
        "should support non-ascii characters in a path"
    );

    assert_eq!(
        micromark_with_options("www.-a.b", &gfm),
        "<p><a href=\"http://www.-a.b\">www.-a.b</a></p>",
        "should support a dash to start a domain"
    );

    assert_eq!(
        micromark_with_options("www.$", &gfm),
        "<p><a href=\"http://www.$\">www.$</a></p>",
        "should support a dollar as a domain name"
    );

    assert_eq!(
        micromark_with_options("www.a..b.c", &gfm),
        "<p><a href=\"http://www.a..b.c\">www.a..b.c</a></p>",
        "should support adjacent dots in a domain name"
    );

    assert_eq!(
        micromark_with_options("www.a&a;", &gfm),
        "<p><a href=\"http://www.a\">www.a</a>&amp;a;</p>",
        "should support named character references in domains"
    );

    assert_eq!(
        micromark_with_options("https://a.bc/d/e/).", &gfm),
        "<p><a href=\"https://a.bc/d/e/\">https://a.bc/d/e/</a>).</p>",
        "should support a closing paren and period after a path"
    );

    assert_eq!(
        micromark_with_options("https://a.bc/d/e/.)", &gfm),
        "<p><a href=\"https://a.bc/d/e/\">https://a.bc/d/e/</a>.)</p>",
        "should support a period and closing paren after a path"
    );

    assert_eq!(
        micromark_with_options("https://a.bc).", &gfm),
        "<p><a href=\"https://a.bc\">https://a.bc</a>).</p>",
        "should support a closing paren and period after a domain"
    );

    assert_eq!(
        micromark_with_options("https://a.bc.)", &gfm),
        "<p><a href=\"https://a.bc\">https://a.bc</a>.)</p>",
        "should support a period and closing paren after a domain"
    );

    assert_eq!(
        micromark_with_options("https://a.bc).d", &gfm),
        "<p><a href=\"https://a.bc).d\">https://a.bc).d</a></p>",
        "should support a closing paren and period in a path"
    );

    assert_eq!(
        micromark_with_options("https://a.bc.)d", &gfm),
        "<p><a href=\"https://a.bc.)d\">https://a.bc.)d</a></p>",
        "should support a period and closing paren in a path"
    );

    assert_eq!(
        micromark_with_options("https://a.bc/))d", &gfm),
        "<p><a href=\"https://a.bc/))d\">https://a.bc/))d</a></p>",
        "should support two closing parens in a path"
    );

    assert_eq!(
        micromark_with_options("ftp://a/b/c.txt", &gfm),
        "<p>ftp://a/b/c.txt</p>",
        "should not support ftp links"
    );

    // Note: GH comments/issues/PRs do not link this, but Gists/readmes do.
    // Fixing it would mean defiating from `cmark-gfm`:
    // Source: <https://github.com/github/cmark-gfm/blob/ef1cfcb/extensions/autolink.c#L156>.
    // assert_eq!(
    //     micromark_with_options("，www.example.com", &gfm),
    //     "<p>，<a href=\"http://www.example.com\">www.example.com</a></p>",
    //     "should support www links after Unicode punctuation",
    // );

    assert_eq!(
        micromark_with_options("，https://example.com", &gfm),
        "<p>，<a href=\"https://example.com\">https://example.com</a></p>",
        "should support http links after Unicode punctuation"
    );

    assert_eq!(
        micromark_with_options("，example@example.com", &gfm),
        "<p>，<a href=\"mailto:example@example.com\">example@example.com</a></p>",
        "should support email links after Unicode punctuation"
    );

    assert_eq!(
        micromark_with_options(
            "http&#x3A;//user:password@host:port/path?key=value#fragment",
            &gfm
        ),
        "<p>http://user:password@host:port/path?key=value#fragment</p>",
        "should not link character reference for `:`"
    );

    assert_eq!(
        micromark_with_options("http://example.com/ab<cd", &gfm),
        "<p><a href=\"http://example.com/ab\">http://example.com/ab</a>&lt;cd</p>",
        "should stop domains/paths at `<`"
    );

    assert_eq!(
        micromark_with_options(
            r###"
[ www.example.com

[ https://example.com

[ contact@example.com

[ www.example.com ]

[ https://example.com ]

[ contact@example.com ]

[ www.example.com ](#)

[ https://example.com ](#)

[ contact@example.com ](#)

![ www.example.com ](#)

![ https://example.com ](#)

![ contact@example.com ](#)
"###,
            &gfm
        ),
        r###"<p>[ <a href="http://www.example.com">www.example.com</a></p>
<p>[ <a href="https://example.com">https://example.com</a></p>
<p>[ <a href="mailto:contact@example.com">contact@example.com</a></p>
<p>[ <a href="http://www.example.com">www.example.com</a> ]</p>
<p>[ <a href="https://example.com">https://example.com</a> ]</p>
<p>[ <a href="mailto:contact@example.com">contact@example.com</a> ]</p>
<p><a href="#"> www.example.com </a></p>
<p><a href="#"> https://example.com </a></p>
<p><a href="#"> contact@example.com </a></p>
<p><img src="#" alt=" www.example.com " /></p>
<p><img src="#" alt=" https://example.com " /></p>
<p><img src="#" alt=" contact@example.com " /></p>
"###,
        "should interplay with brackets, links, and images"
    );
}
