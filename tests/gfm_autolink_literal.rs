#![allow(clippy::needless_raw_string_hashes)]

// To do: clippy introduced this in 1.72 but breaks when it fixes it.
// Remove when solved.

use markdown::{
    mdast::{Link, Node, Paragraph, Root, Text},
    message, to_html, to_html_with_options, to_mdast,
    unist::Position,
    Options, ParseOptions,
};
use pretty_assertions::assert_eq;

#[test]
fn gfm_autolink_literal() -> Result<(), message::Message> {
    assert_eq!(
        to_html("https://example.com"),
        "<p>https://example.com</p>",
        "should ignore protocol urls by default"
    );
    assert_eq!(
        to_html("www.example.com"),
        "<p>www.example.com</p>",
        "should ignore www urls by default"
    );
    assert_eq!(
        to_html("user@example.com"),
        "<p>user@example.com</p>",
        "should ignore email urls by default"
    );

    assert_eq!(
        to_html_with_options("https://example.com", &Options::gfm())?,
        "<p><a href=\"https://example.com\">https://example.com</a></p>",
        "should support protocol urls if enabled"
    );
    assert_eq!(
        to_html_with_options("www.example.com", &Options::gfm())?,
        "<p><a href=\"http://www.example.com\">www.example.com</a></p>",
        "should support www urls if enabled"
    );
    assert_eq!(
        to_html_with_options("user@example.com", &Options::gfm())?,
        "<p><a href=\"mailto:user@example.com\">user@example.com</a></p>",
        "should support email urls if enabled"
    );

    assert_eq!(
        to_html_with_options("[https://example.com](xxx)", &Options::gfm())?,
        "<p><a href=\"xxx\">https://example.com</a></p>",
        "should not link protocol urls in links"
    );
    assert_eq!(
        to_html_with_options("[www.example.com](xxx)", &Options::gfm())?,
        "<p><a href=\"xxx\">www.example.com</a></p>",
        "should not link www urls in links"
    );
    assert_eq!(
        to_html_with_options("[user@example.com](xxx)", &Options::gfm())?,
        "<p><a href=\"xxx\">user@example.com</a></p>",
        "should not link email urls in links"
    );

    assert_eq!(
        to_html_with_options("user@example.com", &Options::gfm())?,
        "<p><a href=\"mailto:user@example.com\">user@example.com</a></p>",
        "should support a closing paren at TLD (email)"
    );

    assert_eq!(
        to_html_with_options("www.a.)", &Options::gfm())?,
        "<p><a href=\"http://www.a\">www.a</a>.)</p>",
        "should support a closing paren at TLD (www)"
    );

    assert_eq!(
        to_html_with_options("www.a b", &Options::gfm())?,
        "<p><a href=\"http://www.a\">www.a</a> b</p>",
        "should support no TLD"
    );

    assert_eq!(
        to_html_with_options("www.a/b c", &Options::gfm())?,
        "<p><a href=\"http://www.a/b\">www.a/b</a> c</p>",
        "should support a path instead of TLD"
    );

    assert_eq!(
        to_html_with_options("www.�a", &Options::gfm())?,
        "<p><a href=\"http://www.%EF%BF%BDa\">www.�a</a></p>",
        "should support a replacement character in a domain"
    );

    assert_eq!(
        to_html_with_options("http://點看.com", &Options::gfm())?,
        "<p><a href=\"http://%E9%BB%9E%E7%9C%8B.com\">http://點看.com</a></p>",
        "should support non-ascii characters in a domain (http)"
    );

    assert_eq!(
        to_html_with_options("www.點看.com", &Options::gfm())?,
        "<p><a href=\"http://www.%E9%BB%9E%E7%9C%8B.com\">www.點看.com</a></p>",
        "should support non-ascii characters in a domain (www)"
    );

    assert_eq!(
        to_html_with_options("點看@example.com", &Options::gfm())?,
        "<p>點看@example.com</p>",
        "should *not* support non-ascii characters in atext (email)"
    );

    assert_eq!(
        to_html_with_options("example@點看.com", &Options::gfm())?,
        "<p>example@點看.com</p>",
        "should *not* support non-ascii characters in a domain (email)"
    );

    assert_eq!(
        to_html_with_options("www.a.com/點看", &Options::gfm())?,
        "<p><a href=\"http://www.a.com/%E9%BB%9E%E7%9C%8B\">www.a.com/點看</a></p>",
        "should support non-ascii characters in a path"
    );

    assert_eq!(
        to_html_with_options("www.-a.b", &Options::gfm())?,
        "<p><a href=\"http://www.-a.b\">www.-a.b</a></p>",
        "should support a dash to start a domain"
    );

    assert_eq!(
        to_html_with_options("www.$", &Options::gfm())?,
        "<p><a href=\"http://www.$\">www.$</a></p>",
        "should support a dollar as a domain name"
    );

    assert_eq!(
        to_html_with_options("www.a..b.c", &Options::gfm())?,
        "<p><a href=\"http://www.a..b.c\">www.a..b.c</a></p>",
        "should support adjacent dots in a domain name"
    );

    assert_eq!(
        to_html_with_options("www.a&a;", &Options::gfm())?,
        "<p><a href=\"http://www.a\">www.a</a>&amp;a;</p>",
        "should support named character references in domains"
    );

    assert_eq!(
        to_html_with_options("https://a.bc/d/e/).", &Options::gfm())?,
        "<p><a href=\"https://a.bc/d/e/\">https://a.bc/d/e/</a>).</p>",
        "should support a closing paren and period after a path"
    );

    assert_eq!(
        to_html_with_options("https://a.bc/d/e/.)", &Options::gfm())?,
        "<p><a href=\"https://a.bc/d/e/\">https://a.bc/d/e/</a>.)</p>",
        "should support a period and closing paren after a path"
    );

    assert_eq!(
        to_html_with_options("https://a.bc).", &Options::gfm())?,
        "<p><a href=\"https://a.bc\">https://a.bc</a>).</p>",
        "should support a closing paren and period after a domain"
    );

    assert_eq!(
        to_html_with_options("https://a.bc.)", &Options::gfm())?,
        "<p><a href=\"https://a.bc\">https://a.bc</a>.)</p>",
        "should support a period and closing paren after a domain"
    );

    assert_eq!(
        to_html_with_options("https://a.bc).d", &Options::gfm())?,
        "<p><a href=\"https://a.bc).d\">https://a.bc).d</a></p>",
        "should support a closing paren and period in a path"
    );

    assert_eq!(
        to_html_with_options("https://a.bc.)d", &Options::gfm())?,
        "<p><a href=\"https://a.bc.)d\">https://a.bc.)d</a></p>",
        "should support a period and closing paren in a path"
    );

    assert_eq!(
        to_html_with_options("https://a.bc/))d", &Options::gfm())?,
        "<p><a href=\"https://a.bc/))d\">https://a.bc/))d</a></p>",
        "should support two closing parens in a path"
    );

    assert_eq!(
        to_html_with_options("ftp://a/b/c.txt", &Options::gfm())?,
        "<p>ftp://a/b/c.txt</p>",
        "should not support ftp links"
    );

    // Note: GH comments/issues/PRs do not link this, but Gists/readmes do.
    // Fixing it would mean deviating from `cmark-gfm`:
    // Source: <https://github.com/github/cmark-gfm/blob/ef1cfcb/extensions/autolink.c#L156>.
    // assert_eq!(
    //     to_html_with_options("，www.example.com", &Options::gfm())?,
    //     "<p>，<a href=\"http://www.example.com\">www.example.com</a></p>",
    //     "should support www links after Unicode punctuation",
    // );

    assert_eq!(
        to_html_with_options("，https://example.com", &Options::gfm())?,
        "<p>，<a href=\"https://example.com\">https://example.com</a></p>",
        "should support http links after Unicode punctuation"
    );

    assert_eq!(
        to_html_with_options("，example@example.com", &Options::gfm())?,
        "<p>，<a href=\"mailto:example@example.com\">example@example.com</a></p>",
        "should support email links after Unicode punctuation"
    );

    assert_eq!(
        to_html_with_options(
            "http&#x3A;//user:password@host:port/path?key=value#fragment",
            &Options::gfm()
        )?,
        "<p>http://user:password@host:port/path?key=value#fragment</p>",
        "should not link character reference for `:`"
    );

    assert_eq!(
        to_html_with_options("http://example.com/ab<cd", &Options::gfm())?,
        "<p><a href=\"http://example.com/ab\">http://example.com/ab</a>&lt;cd</p>",
        "should stop domains/paths at `<`"
    );

    assert_eq!(
        to_html_with_options(
            r###"
mailto:scyther@pokemon.com

This is a mailto:scyther@pokemon.com

mailto:scyther@pokemon.com.

mmmmailto:scyther@pokemon.com

mailto:scyther@pokemon.com/

mailto:scyther@pokemon.com/message

mailto:scyther@pokemon.com/mailto:beedrill@pokemon.com

xmpp:scyther@pokemon.com

xmpp:scyther@pokemon.com.

xmpp:scyther@pokemon.com/message

xmpp:scyther@pokemon.com/message.

Email me at:scyther@pokemon.com"###,
            &Options::gfm()
        )?,
        r###"<p><a href="mailto:scyther@pokemon.com">mailto:scyther@pokemon.com</a></p>
<p>This is a <a href="mailto:scyther@pokemon.com">mailto:scyther@pokemon.com</a></p>
<p><a href="mailto:scyther@pokemon.com">mailto:scyther@pokemon.com</a>.</p>
<p>mmmmailto:<a href="mailto:scyther@pokemon.com">scyther@pokemon.com</a></p>
<p><a href="mailto:scyther@pokemon.com">mailto:scyther@pokemon.com</a>/</p>
<p><a href="mailto:scyther@pokemon.com">mailto:scyther@pokemon.com</a>/message</p>
<p><a href="mailto:scyther@pokemon.com">mailto:scyther@pokemon.com</a>/<a href="mailto:beedrill@pokemon.com">mailto:beedrill@pokemon.com</a></p>
<p><a href="xmpp:scyther@pokemon.com">xmpp:scyther@pokemon.com</a></p>
<p><a href="xmpp:scyther@pokemon.com">xmpp:scyther@pokemon.com</a>.</p>
<p><a href="xmpp:scyther@pokemon.com/message">xmpp:scyther@pokemon.com/message</a></p>
<p><a href="xmpp:scyther@pokemon.com/message">xmpp:scyther@pokemon.com/message</a>.</p>
<p>Email me at:<a href="mailto:scyther@pokemon.com">scyther@pokemon.com</a></p>"###,
        "should support `mailto:` and `xmpp:` protocols"
    );

    assert_eq!(
        to_html_with_options(
            r###"
a www.example.com&xxx;b c

a www.example.com&xxx;. b

a www.example.com&xxxxxxxxx;. b

a www.example.com&xxxxxxxxxx;. b

a www.example.com&xxxxxxxxxxx;. b

a www.example.com&xxx. b

a www.example.com&#123. b

a www.example.com&123. b

a www.example.com&x. b

a www.example.com&#1. b

a www.example.com&1. b

a www.example.com&. b

a www.example.com& b
"###,
            &Options::gfm()
        )?,
        r###"<p>a <a href="http://www.example.com&amp;xxx;b">www.example.com&amp;xxx;b</a> c</p>
<p>a <a href="http://www.example.com">www.example.com</a>&amp;xxx;. b</p>
<p>a <a href="http://www.example.com">www.example.com</a>&amp;xxxxxxxxx;. b</p>
<p>a <a href="http://www.example.com">www.example.com</a>&amp;xxxxxxxxxx;. b</p>
<p>a <a href="http://www.example.com">www.example.com</a>&amp;xxxxxxxxxxx;. b</p>
<p>a <a href="http://www.example.com&amp;xxx">www.example.com&amp;xxx</a>. b</p>
<p>a <a href="http://www.example.com&amp;#123">www.example.com&amp;#123</a>. b</p>
<p>a <a href="http://www.example.com&amp;123">www.example.com&amp;123</a>. b</p>
<p>a <a href="http://www.example.com&amp;x">www.example.com&amp;x</a>. b</p>
<p>a <a href="http://www.example.com&amp;#1">www.example.com&amp;#1</a>. b</p>
<p>a <a href="http://www.example.com&amp;1">www.example.com&amp;1</a>. b</p>
<p>a <a href="http://www.example.com&amp;">www.example.com&amp;</a>. b</p>
<p>a <a href="http://www.example.com&amp;">www.example.com&amp;</a> b</p>
"###,
        "should match “character references” like GitHub does"
    );

    // Note: this deviates from GFM, as <https://github.com/github/cmark-gfm/issues/278> is fixed.
    assert_eq!(
        to_html_with_options(
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
            &Options::gfm()
        )?,
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
        "should match interplay with brackets, links, and images, like GitHub does (but without the bugs)"
    );

    assert_eq!(
        to_html_with_options(
            r###"
www.example.com/?=a(b)cccccc

www.example.com/?=a(b(c)ccccc

www.example.com/?=a(b(c)c)cccc

www.example.com/?=a(b(c)c)c)ccc

www.example.com/?q=a(business)

www.example.com/?q=a(business)))

(www.example.com/?q=a(business))

(www.example.com/?q=a(business)

www.example.com/?q=a(business)".

www.example.com/?q=a(business)))

(www.example.com/?q=a(business))".

(www.example.com/?q=a(business)".)

(www.example.com/?q=a(business)".
"###,
            &Options::gfm()
        )?,
        r###"<p><a href="http://www.example.com/?=a(b)cccccc">www.example.com/?=a(b)cccccc</a></p>
<p><a href="http://www.example.com/?=a(b(c)ccccc">www.example.com/?=a(b(c)ccccc</a></p>
<p><a href="http://www.example.com/?=a(b(c)c)cccc">www.example.com/?=a(b(c)c)cccc</a></p>
<p><a href="http://www.example.com/?=a(b(c)c)c)ccc">www.example.com/?=a(b(c)c)c)ccc</a></p>
<p><a href="http://www.example.com/?q=a(business)">www.example.com/?q=a(business)</a></p>
<p><a href="http://www.example.com/?q=a(business)">www.example.com/?q=a(business)</a>))</p>
<p>(<a href="http://www.example.com/?q=a(business)">www.example.com/?q=a(business)</a>)</p>
<p>(<a href="http://www.example.com/?q=a(business)">www.example.com/?q=a(business)</a></p>
<p><a href="http://www.example.com/?q=a(business)">www.example.com/?q=a(business)</a>&quot;.</p>
<p><a href="http://www.example.com/?q=a(business)">www.example.com/?q=a(business)</a>))</p>
<p>(<a href="http://www.example.com/?q=a(business)">www.example.com/?q=a(business)</a>)&quot;.</p>
<p>(<a href="http://www.example.com/?q=a(business)">www.example.com/?q=a(business)</a>&quot;.)</p>
<p>(<a href="http://www.example.com/?q=a(business)">www.example.com/?q=a(business)</a>&quot;.</p>
"###,
        "should match parens like GitHub does"
    );

    // Note: this deviates from GFM.
    // Here, the following issues are fixed:
    // - <https://github.com/github/cmark-gfm/issues/280>
    assert_eq!(
        to_html_with_options(
            r###"
# Literal autolinks

## WWW autolinks

w.commonmark.org

ww.commonmark.org

www.commonmark.org

Www.commonmark.org

wWw.commonmark.org

wwW.commonmark.org

WWW.COMMONMARK.ORG

Visit www.commonmark.org/help for more information.

Visit www.commonmark.org.

Visit www.commonmark.org/a.b.

www.aaa.bbb.ccc_ccc

www.aaa_bbb.ccc

www.aaa.bbb.ccc.ddd_ddd

www.aaa.bbb.ccc_ccc.ddd

www.aaa.bbb_bbb.ccc.ddd

www.aaa_aaa.bbb.ccc.ddd

Visit www.commonmark.org.

Visit www.commonmark.org/a.b.

www.google.com/search?q=Markup+(business)

www.google.com/search?q=Markup+(business)))

(www.google.com/search?q=Markup+(business))

(www.google.com/search?q=Markup+(business)

www.google.com/search?q=(business))+ok

www.google.com/search?q=commonmark&hl=en

www.google.com/search?q=commonmark&hl;en

www.google.com/search?q=commonmark&hl;

www.commonmark.org/he<lp

## HTTP autolinks

hexample.com

htexample.com

httexample.com

httpexample.com

http:example.com

http:/example.com

https:/example.com

http://example.com

https://example.com

https://example

http://commonmark.org

(Visit https://encrypted.google.com/search?q=Markup+(business))

## Email autolinks

No dot: foo@barbaz

No dot: foo@barbaz.

foo@bar.baz

hello@mail+xyz.example isn’t valid, but hello+xyz@mail.example is.

a.b-c_d@a.b

a.b-c_d@a.b.

a.b-c_d@a.b-

a.b-c_d@a.b_

a@a_b.c

a@a-b.c

Can’t end in an underscore followed by a period: aaa@a.b_.

Can contain an underscore followed by a period: aaa@a.b_.c

## Link text should not be expanded

[Visit www.example.com](http://www.example.com) please.

[Visit http://www.example.com](http://www.example.com) please.

[Mail example@example.com](mailto:example@example.com) please.

[link]() <http://autolink> should still be expanded.
"###,
            &Options::gfm()
        )?,
        r###"<h1>Literal autolinks</h1>
<h2>WWW autolinks</h2>
<p>w.commonmark.org</p>
<p>ww.commonmark.org</p>
<p><a href="http://www.commonmark.org">www.commonmark.org</a></p>
<p><a href="http://Www.commonmark.org">Www.commonmark.org</a></p>
<p><a href="http://wWw.commonmark.org">wWw.commonmark.org</a></p>
<p><a href="http://wwW.commonmark.org">wwW.commonmark.org</a></p>
<p><a href="http://WWW.COMMONMARK.ORG">WWW.COMMONMARK.ORG</a></p>
<p>Visit <a href="http://www.commonmark.org/help">www.commonmark.org/help</a> for more information.</p>
<p>Visit <a href="http://www.commonmark.org">www.commonmark.org</a>.</p>
<p>Visit <a href="http://www.commonmark.org/a.b">www.commonmark.org/a.b</a>.</p>
<p>www.aaa.bbb.ccc_ccc</p>
<p>www.aaa_bbb.ccc</p>
<p>www.aaa.bbb.ccc.ddd_ddd</p>
<p>www.aaa.bbb.ccc_ccc.ddd</p>
<p><a href="http://www.aaa.bbb_bbb.ccc.ddd">www.aaa.bbb_bbb.ccc.ddd</a></p>
<p><a href="http://www.aaa_aaa.bbb.ccc.ddd">www.aaa_aaa.bbb.ccc.ddd</a></p>
<p>Visit <a href="http://www.commonmark.org">www.commonmark.org</a>.</p>
<p>Visit <a href="http://www.commonmark.org/a.b">www.commonmark.org/a.b</a>.</p>
<p><a href="http://www.google.com/search?q=Markup+(business)">www.google.com/search?q=Markup+(business)</a></p>
<p><a href="http://www.google.com/search?q=Markup+(business)">www.google.com/search?q=Markup+(business)</a>))</p>
<p>(<a href="http://www.google.com/search?q=Markup+(business)">www.google.com/search?q=Markup+(business)</a>)</p>
<p>(<a href="http://www.google.com/search?q=Markup+(business)">www.google.com/search?q=Markup+(business)</a></p>
<p><a href="http://www.google.com/search?q=(business))+ok">www.google.com/search?q=(business))+ok</a></p>
<p><a href="http://www.google.com/search?q=commonmark&amp;hl=en">www.google.com/search?q=commonmark&amp;hl=en</a></p>
<p><a href="http://www.google.com/search?q=commonmark&amp;hl;en">www.google.com/search?q=commonmark&amp;hl;en</a></p>
<p><a href="http://www.google.com/search?q=commonmark">www.google.com/search?q=commonmark</a>&amp;hl;</p>
<p><a href="http://www.commonmark.org/he">www.commonmark.org/he</a>&lt;lp</p>
<h2>HTTP autolinks</h2>
<p>hexample.com</p>
<p>htexample.com</p>
<p>httexample.com</p>
<p>httpexample.com</p>
<p>http:example.com</p>
<p>http:/example.com</p>
<p>https:/example.com</p>
<p><a href="http://example.com">http://example.com</a></p>
<p><a href="https://example.com">https://example.com</a></p>
<p><a href="https://example">https://example</a></p>
<p><a href="http://commonmark.org">http://commonmark.org</a></p>
<p>(Visit <a href="https://encrypted.google.com/search?q=Markup+(business)">https://encrypted.google.com/search?q=Markup+(business)</a>)</p>
<h2>Email autolinks</h2>
<p>No dot: foo@barbaz</p>
<p>No dot: foo@barbaz.</p>
<p><a href="mailto:foo@bar.baz">foo@bar.baz</a></p>
<p>hello@mail+xyz.example isn’t valid, but <a href="mailto:hello+xyz@mail.example">hello+xyz@mail.example</a> is.</p>
<p><a href="mailto:a.b-c_d@a.b">a.b-c_d@a.b</a></p>
<p><a href="mailto:a.b-c_d@a.b">a.b-c_d@a.b</a>.</p>
<p>a.b-c_d@a.b-</p>
<p>a.b-c_d@a.b_</p>
<p><a href="mailto:a@a_b.c">a@a_b.c</a></p>
<p><a href="mailto:a@a-b.c">a@a-b.c</a></p>
<p>Can’t end in an underscore followed by a period: aaa@a.b_.</p>
<p>Can contain an underscore followed by a period: <a href="mailto:aaa@a.b_.c">aaa@a.b_.c</a></p>
<h2>Link text should not be expanded</h2>
<p><a href="http://www.example.com">Visit www.example.com</a> please.</p>
<p><a href="http://www.example.com">Visit http://www.example.com</a> please.</p>
<p><a href="mailto:example@example.com">Mail example@example.com</a> please.</p>
<p><a href="">link</a> <a href="http://autolink">http://autolink</a> should still be expanded.</p>
"###,
        "should match base like GitHub does"
    );

    assert_eq!(
        to_html_with_options(
            r###"H0.

[https://a.com&copy;b

[www.a.com&copy;b

H1.

[]https://a.com&copy;b

[]www.a.com&copy;b

H2.

[] https://a.com&copy;b

[] www.a.com&copy;b

H3.

[[https://a.com&copy;b

[[www.a.com&copy;b

H4.

[[]https://a.com&copy;b

[[]www.a.com&copy;b

H5.

[[]]https://a.com&copy;b

[[]]www.a.com&copy;b
"###,
            &Options::gfm()
        )?,
        r###"<p>H0.</p>
<p>[<a href="https://a.com&amp;copy;b">https://a.com&amp;copy;b</a></p>
<p>[<a href="http://www.a.com&amp;copy;b">www.a.com&amp;copy;b</a></p>
<p>H1.</p>
<p>[]<a href="https://a.com&amp;copy;b">https://a.com&amp;copy;b</a></p>
<p>[]<a href="http://www.a.com&amp;copy;b">www.a.com&amp;copy;b</a></p>
<p>H2.</p>
<p>[] <a href="https://a.com&amp;copy;b">https://a.com&amp;copy;b</a></p>
<p>[] <a href="http://www.a.com&amp;copy;b">www.a.com&amp;copy;b</a></p>
<p>H3.</p>
<p>[[<a href="https://a.com&amp;copy;b">https://a.com&amp;copy;b</a></p>
<p>[[<a href="http://www.a.com&amp;copy;b">www.a.com&amp;copy;b</a></p>
<p>H4.</p>
<p>[[]<a href="https://a.com&amp;copy;b">https://a.com&amp;copy;b</a></p>
<p>[[]<a href="http://www.a.com&amp;copy;b">www.a.com&amp;copy;b</a></p>
<p>H5.</p>
<p>[[]]<a href="https://a.com&amp;copy;b">https://a.com&amp;copy;b</a></p>
<p>[[]]<a href="http://www.a.com&amp;copy;b">www.a.com&amp;copy;b</a></p>
"###,
        "should match brackets like GitHub does (except for the bracket bug)"
    );

    assert_eq!(
        to_html_with_options(r###"Image start.

![https://a.com

![http://a.com

![www.a.com

![a@b.c

Image start and label end.

![https://a.com]

![http://a.com]

![www.a.com]

![a@b.c]

Image label with reference (note: GH cleans hashes here, but we keep them in).

![https://a.com][x]

![http://a.com][x]

![www.a.com][x]

![a@b.c][x]

[x]: #

Image label with resource.

![https://a.com]()

![http://a.com]()

![www.a.com]()

![a@b.c]()

Autolink literal after image.

![a]() https://a.com

![a]() http://a.com

![a]() www.a.com

![a]() a@b.c
"###, &Options::gfm())?,
        r###"<p>Image start.</p>
<p>![<a href="https://a.com">https://a.com</a></p>
<p>![<a href="http://a.com">http://a.com</a></p>
<p>![<a href="http://www.a.com">www.a.com</a></p>
<p>![<a href="mailto:a@b.c">a@b.c</a></p>
<p>Image start and label end.</p>
<p>![<a href="https://a.com">https://a.com</a>]</p>
<p>![<a href="http://a.com">http://a.com</a>]</p>
<p>![<a href="http://www.a.com">www.a.com</a>]</p>
<p>![<a href="mailto:a@b.c">a@b.c</a>]</p>
<p>Image label with reference (note: GH cleans hashes here, but we keep them in).</p>
<p><img src="#" alt="https://a.com" /></p>
<p><img src="#" alt="http://a.com" /></p>
<p><img src="#" alt="www.a.com" /></p>
<p><img src="#" alt="a@b.c" /></p>
<p>Image label with resource.</p>
<p><img src="" alt="https://a.com" /></p>
<p><img src="" alt="http://a.com" /></p>
<p><img src="" alt="www.a.com" /></p>
<p><img src="" alt="a@b.c" /></p>
<p>Autolink literal after image.</p>
<p><img src="" alt="a" /> <a href="https://a.com">https://a.com</a></p>
<p><img src="" alt="a" /> <a href="http://a.com">http://a.com</a></p>
<p><img src="" alt="a" /> <a href="http://www.a.com">www.a.com</a></p>
<p><img src="" alt="a" /> <a href="mailto:a@b.c">a@b.c</a></p>
"###,
        "should match autolink literals combined w/ images like GitHub does (except for the bracket bug)"
    );

    assert_eq!(
        to_html_with_options(r###"Link start.

[https://a.com

[http://a.com

[www.a.com

[a@b.c

Label end.

https://a.com]

http://a.com]

www.a.com]

a@b.c]

Link start and label end.

[https://a.com]

[http://a.com]

[www.a.com]

[a@b.c]

What naïvely seems like a label end (A).

https://a.com`]`

http://a.com`]`

www.a.com`]`

a@b.c`]`

Link start and what naïvely seems like a balanced brace (B).

[https://a.com`]`

[http://a.com`]`

[www.a.com`]`

[a@b.c`]`

What naïvely seems like a label end (C).

https://a.com `]`

http://a.com `]`

www.a.com `]`

a@b.c `]`

Link start and what naïvely seems like a balanced brace (D).

[https://a.com `]`

[http://a.com `]`

[www.a.com `]`

[a@b.c `]`

Link label with reference.

[https://a.com][x]

[http://a.com][x]

[www.a.com][x]

[a@b.c][x]

[x]: #

Link label with resource.

[https://a.com]()

[http://a.com]()

[www.a.com]()

[a@b.c]()

More in link.

[a https://b.com c]()

[a http://b.com c]()

[a www.b.com c]()

[a b@c.d e]()

Autolink literal after link.

[a]() https://a.com

[a]() http://a.com

[a]() www.a.com

[a]() a@b.c
"###, &Options::gfm())?,
        r###"<p>Link start.</p>
<p>[<a href="https://a.com">https://a.com</a></p>
<p>[<a href="http://a.com">http://a.com</a></p>
<p>[<a href="http://www.a.com">www.a.com</a></p>
<p>[<a href="mailto:a@b.c">a@b.c</a></p>
<p>Label end.</p>
<p><a href="https://a.com">https://a.com</a>]</p>
<p><a href="http://a.com">http://a.com</a>]</p>
<p><a href="http://www.a.com">www.a.com</a>]</p>
<p><a href="mailto:a@b.c">a@b.c</a>]</p>
<p>Link start and label end.</p>
<p>[<a href="https://a.com">https://a.com</a>]</p>
<p>[<a href="http://a.com">http://a.com</a>]</p>
<p>[<a href="http://www.a.com">www.a.com</a>]</p>
<p>[<a href="mailto:a@b.c">a@b.c</a>]</p>
<p>What naïvely seems like a label end (A).</p>
<p><a href="https://a.com%60%5D%60">https://a.com`]`</a></p>
<p><a href="http://a.com%60%5D%60">http://a.com`]`</a></p>
<p><a href="http://www.a.com%60%5D%60">www.a.com`]`</a></p>
<p><a href="mailto:a@b.c">a@b.c</a><code>]</code></p>
<p>Link start and what naïvely seems like a balanced brace (B).</p>
<p>[<a href="https://a.com%60%5D%60">https://a.com`]`</a></p>
<p>[<a href="http://a.com%60%5D%60">http://a.com`]`</a></p>
<p>[<a href="http://www.a.com%60%5D%60">www.a.com`]`</a></p>
<p>[<a href="mailto:a@b.c">a@b.c</a><code>]</code></p>
<p>What naïvely seems like a label end (C).</p>
<p><a href="https://a.com">https://a.com</a> <code>]</code></p>
<p><a href="http://a.com">http://a.com</a> <code>]</code></p>
<p><a href="http://www.a.com">www.a.com</a> <code>]</code></p>
<p><a href="mailto:a@b.c">a@b.c</a> <code>]</code></p>
<p>Link start and what naïvely seems like a balanced brace (D).</p>
<p>[<a href="https://a.com">https://a.com</a> <code>]</code></p>
<p>[<a href="http://a.com">http://a.com</a> <code>]</code></p>
<p>[<a href="http://www.a.com">www.a.com</a> <code>]</code></p>
<p>[<a href="mailto:a@b.c">a@b.c</a> <code>]</code></p>
<p>Link label with reference.</p>
<p><a href="#">https://a.com</a></p>
<p><a href="#">http://a.com</a></p>
<p><a href="#">www.a.com</a></p>
<p><a href="#">a@b.c</a></p>
<p>Link label with resource.</p>
<p><a href="">https://a.com</a></p>
<p><a href="">http://a.com</a></p>
<p><a href="">www.a.com</a></p>
<p><a href="">a@b.c</a></p>
<p>More in link.</p>
<p><a href="">a https://b.com c</a></p>
<p><a href="">a http://b.com c</a></p>
<p><a href="">a www.b.com c</a></p>
<p><a href="">a b@c.d e</a></p>
<p>Autolink literal after link.</p>
<p><a href="">a</a> <a href="https://a.com">https://a.com</a></p>
<p><a href="">a</a> <a href="http://a.com">http://a.com</a></p>
<p><a href="">a</a> <a href="http://www.a.com">www.a.com</a></p>
<p><a href="">a</a> <a href="mailto:a@b.c">a@b.c</a></p>
"###,
        "should match autolink literals combined w/ links like GitHub does (except for the bracket bug)"
    );

    assert_eq!(
        to_html_with_options(
            r###"# “character reference”

www.a&b (space)

www.a&b!

www.a&b"

www.a&b#

www.a&b$

www.a&b%

www.a&b&

www.a&b'

www.a&b(

www.a&b)

www.a&b*

www.a&b+

www.a&b,

www.a&b-

www.a&b

www.a&b.

www.a&b/

www.a&b:

www.a&b;

www.a&b<

www.a&b=

www.a&b>

www.a&b?

www.a&b@

www.a&b[

www.a&b\

www.a&b]

www.a&b^

www.a&b_

www.a&b`

www.a&b{

www.a&b|

www.a&b}

www.a&b~
"###,
            &Options::gfm()
        )?,
        r###"<h1>“character reference”</h1>
<p><a href="http://www.a&amp;b">www.a&amp;b</a> (space)</p>
<p><a href="http://www.a&amp;b">www.a&amp;b</a>!</p>
<p><a href="http://www.a&amp;b">www.a&amp;b</a>&quot;</p>
<p><a href="http://www.a&amp;b#">www.a&amp;b#</a></p>
<p><a href="http://www.a&amp;b$">www.a&amp;b$</a></p>
<p><a href="http://www.a&amp;b%25">www.a&amp;b%</a></p>
<p><a href="http://www.a&amp;b&amp;">www.a&amp;b&amp;</a></p>
<p><a href="http://www.a&amp;b">www.a&amp;b</a>'</p>
<p><a href="http://www.a&amp;b(">www.a&amp;b(</a></p>
<p><a href="http://www.a&amp;b">www.a&amp;b</a>)</p>
<p><a href="http://www.a&amp;b">www.a&amp;b</a>*</p>
<p><a href="http://www.a&amp;b+">www.a&amp;b+</a></p>
<p><a href="http://www.a&amp;b">www.a&amp;b</a>,</p>
<p><a href="http://www.a&amp;b-">www.a&amp;b-</a></p>
<p><a href="http://www.a&amp;b">www.a&amp;b</a></p>
<p><a href="http://www.a&amp;b">www.a&amp;b</a>.</p>
<p><a href="http://www.a&amp;b/">www.a&amp;b/</a></p>
<p><a href="http://www.a&amp;b">www.a&amp;b</a>:</p>
<p><a href="http://www.a">www.a</a>&amp;b;</p>
<p><a href="http://www.a&amp;b">www.a&amp;b</a>&lt;</p>
<p><a href="http://www.a&amp;b=">www.a&amp;b=</a></p>
<p><a href="http://www.a&amp;b%3E">www.a&amp;b&gt;</a></p>
<p><a href="http://www.a&amp;b">www.a&amp;b</a>?</p>
<p><a href="http://www.a&amp;b@">www.a&amp;b@</a></p>
<p><a href="http://www.a&amp;b%5B">www.a&amp;b[</a></p>
<p><a href="http://www.a&amp;b%5C">www.a&amp;b\</a></p>
<p><a href="http://www.a&amp;b">www.a&amp;b</a>]</p>
<p><a href="http://www.a&amp;b%5E">www.a&amp;b^</a></p>
<p><a href="http://www.a&amp;b">www.a&amp;b</a>_</p>
<p><a href="http://www.a&amp;b%60">www.a&amp;b`</a></p>
<p><a href="http://www.a&amp;b%7B">www.a&amp;b{</a></p>
<p><a href="http://www.a&amp;b%7C">www.a&amp;b|</a></p>
<p><a href="http://www.a&amp;b%7D">www.a&amp;b}</a></p>
<p><a href="http://www.a&amp;b">www.a&amp;b</a>~</p>
"###,
        "should match “character references (named)” like GitHub does (except for the bracket bug)"
    );

    assert_eq!(
        to_html_with_options(r###"# “character reference”

www.a&#35 (space)

www.a&#35!

www.a&#35"

www.a&#35#

www.a&#35$

www.a&#35%

www.a&#35&

www.a&#35'

www.a&#35(

www.a&#35)

www.a&#35*

www.a&#35+

www.a&#35,

www.a&#35-

www.a&#35

www.a&#35.

www.a&#35/

www.a&#35:

www.a&#35;

www.a&#35<

www.a&#35=

www.a&#35>

www.a&#35?

www.a&#35@

www.a&#35[

www.a&#35\

www.a&#35]

www.a&#35^

www.a&#35_

www.a&#35`

www.a&#35{

www.a&#35|

www.a&#35}

www.a&#35~
"###, &Options::gfm())?,
        r###"<h1>“character reference”</h1>
<p><a href="http://www.a&amp;#35">www.a&amp;#35</a> (space)</p>
<p><a href="http://www.a&amp;#35">www.a&amp;#35</a>!</p>
<p><a href="http://www.a&amp;#35">www.a&amp;#35</a>&quot;</p>
<p><a href="http://www.a&amp;#35#">www.a&amp;#35#</a></p>
<p><a href="http://www.a&amp;#35$">www.a&amp;#35$</a></p>
<p><a href="http://www.a&amp;#35%25">www.a&amp;#35%</a></p>
<p><a href="http://www.a&amp;#35&amp;">www.a&amp;#35&amp;</a></p>
<p><a href="http://www.a&amp;#35">www.a&amp;#35</a>'</p>
<p><a href="http://www.a&amp;#35(">www.a&amp;#35(</a></p>
<p><a href="http://www.a&amp;#35">www.a&amp;#35</a>)</p>
<p><a href="http://www.a&amp;#35">www.a&amp;#35</a>*</p>
<p><a href="http://www.a&amp;#35+">www.a&amp;#35+</a></p>
<p><a href="http://www.a&amp;#35">www.a&amp;#35</a>,</p>
<p><a href="http://www.a&amp;#35-">www.a&amp;#35-</a></p>
<p><a href="http://www.a&amp;#35">www.a&amp;#35</a></p>
<p><a href="http://www.a&amp;#35">www.a&amp;#35</a>.</p>
<p><a href="http://www.a&amp;#35/">www.a&amp;#35/</a></p>
<p><a href="http://www.a&amp;#35">www.a&amp;#35</a>:</p>
<p><a href="http://www.a&amp;#35">www.a&amp;#35</a>;</p>
<p><a href="http://www.a&amp;#35">www.a&amp;#35</a>&lt;</p>
<p><a href="http://www.a&amp;#35=">www.a&amp;#35=</a></p>
<p><a href="http://www.a&amp;#35%3E">www.a&amp;#35&gt;</a></p>
<p><a href="http://www.a&amp;#35">www.a&amp;#35</a>?</p>
<p><a href="http://www.a&amp;#35@">www.a&amp;#35@</a></p>
<p><a href="http://www.a&amp;#35%5B">www.a&amp;#35[</a></p>
<p><a href="http://www.a&amp;#35%5C">www.a&amp;#35\</a></p>
<p><a href="http://www.a&amp;#35">www.a&amp;#35</a>]</p>
<p><a href="http://www.a&amp;#35%5E">www.a&amp;#35^</a></p>
<p><a href="http://www.a&amp;#35">www.a&amp;#35</a>_</p>
<p><a href="http://www.a&amp;#35%60">www.a&amp;#35`</a></p>
<p><a href="http://www.a&amp;#35%7B">www.a&amp;#35{</a></p>
<p><a href="http://www.a&amp;#35%7C">www.a&amp;#35|</a></p>
<p><a href="http://www.a&amp;#35%7D">www.a&amp;#35}</a></p>
<p><a href="http://www.a&amp;#35">www.a&amp;#35</a>~</p>
"###,
        "should match “character references (numeric)” like GitHub does (except for the bracket bug)"
    );

    assert_eq!(
        to_html_with_options(
            r###"a@0.0

a@0.b

a@a.29

a@a.b

a@0.0.c

react@0.11.1

react@0.12.0-rc1

react@0.14.0-alpha1

react@16.7.0-alpha.2

react@0.0.0-experimental-aae83a4b9

[ react@0.11.1

[ react@0.12.0-rc1

[ react@0.14.0-alpha1

[ react@16.7.0-alpha.2

[ react@0.0.0-experimental-aae83a4b9
"###,
            &Options::gfm()
        )?,
        r###"<p>a@0.0</p>
<p><a href="mailto:a@0.b">a@0.b</a></p>
<p>a@a.29</p>
<p><a href="mailto:a@a.b">a@a.b</a></p>
<p><a href="mailto:a@0.0.c">a@0.0.c</a></p>
<p>react@0.11.1</p>
<p>react@0.12.0-rc1</p>
<p>react@0.14.0-alpha1</p>
<p>react@16.7.0-alpha.2</p>
<p>react@0.0.0-experimental-aae83a4b9</p>
<p>[ react@0.11.1</p>
<p>[ react@0.12.0-rc1</p>
<p>[ react@0.14.0-alpha1</p>
<p>[ react@16.7.0-alpha.2</p>
<p>[ react@0.0.0-experimental-aae83a4b9</p>
"###,
        "should match email TLD digits like GitHub does"
    );

    assert_eq!(
        to_html_with_options(
            r###"# httpshhh? (2)

http://a (space)

http://a!

http://a"

http://a#

http://a$

http://a%

http://a&

http://a'

http://a(

http://a)

http://a*

http://a+

http://a,

http://a-

http://a

http://a.

http://a/

http://a:

http://a;

http://a<

http://a=

http://a>

http://a?

http://a@

http://a[

http://a\

http://a]

http://a^

http://a_

http://a`

http://a{

http://a|

http://a}

http://a~
"###,
            &Options::gfm()
        )?,
        r###"<h1>httpshhh? (2)</h1>
<p><a href="http://a">http://a</a> (space)</p>
<p><a href="http://a">http://a</a>!</p>
<p><a href="http://a">http://a</a>&quot;</p>
<p><a href="http://a#">http://a#</a></p>
<p><a href="http://a$">http://a$</a></p>
<p><a href="http://a%25">http://a%</a></p>
<p><a href="http://a&amp;">http://a&amp;</a></p>
<p><a href="http://a">http://a</a>'</p>
<p><a href="http://a(">http://a(</a></p>
<p><a href="http://a">http://a</a>)</p>
<p><a href="http://a">http://a</a>*</p>
<p><a href="http://a+">http://a+</a></p>
<p><a href="http://a">http://a</a>,</p>
<p><a href="http://a-">http://a-</a></p>
<p><a href="http://a">http://a</a></p>
<p><a href="http://a">http://a</a>.</p>
<p><a href="http://a/">http://a/</a></p>
<p><a href="http://a">http://a</a>:</p>
<p><a href="http://a">http://a</a>;</p>
<p><a href="http://a">http://a</a>&lt;</p>
<p><a href="http://a=">http://a=</a></p>
<p><a href="http://a%3E">http://a&gt;</a></p>
<p><a href="http://a">http://a</a>?</p>
<p><a href="http://a@">http://a@</a></p>
<p><a href="http://a%5B">http://a[</a></p>
<p><a href="http://a%5C">http://a\</a></p>
<p><a href="http://a">http://a</a>]</p>
<p><a href="http://a%5E">http://a^</a></p>
<p><a href="http://a">http://a</a>_</p>
<p><a href="http://a%60">http://a`</a></p>
<p><a href="http://a%7B">http://a{</a></p>
<p><a href="http://a%7C">http://a|</a></p>
<p><a href="http://a%7D">http://a}</a></p>
<p><a href="http://a">http://a</a>~</p>
"###,
        "should match protocol domain continue like GitHub does"
    );

    assert_eq!(
        to_html_with_options(
            r###"# httpshhh? (1)

http:// (space)

http://!

http://"

http://#

http://$

http://%

http://&

http://'

http://(

http://)

http://*

http://+

http://,

http://-

http://

http://.

http:///

http://:

http://;

http://<

http://=

http://>

http://?

http://@

http://[

http://\

http://]

http://^

http://_

http://`

http://{

http://|

http://}

http://~
"###,
            &Options::gfm()
        )?,
        r###"<h1>httpshhh? (1)</h1>
<p>http:// (space)</p>
<p>http://!</p>
<p>http://&quot;</p>
<p>http://#</p>
<p>http://$</p>
<p>http://%</p>
<p>http://&amp;</p>
<p>http://'</p>
<p>http://(</p>
<p>http://)</p>
<p>http://*</p>
<p>http://+</p>
<p>http://,</p>
<p>http://-</p>
<p>http://</p>
<p>http://.</p>
<p>http:///</p>
<p>http://:</p>
<p>http://;</p>
<p>http://&lt;</p>
<p>http://=</p>
<p>http://&gt;</p>
<p>http://?</p>
<p>http://@</p>
<p>http://[</p>
<p>http://\</p>
<p>http://]</p>
<p>http://^</p>
<p>http://_</p>
<p>http://`</p>
<p>http://{</p>
<p>http://|</p>
<p>http://}</p>
<p>http://~</p>
"###,
        "should match protocol domain start like GitHub does"
    );

    assert_eq!(
        to_html_with_options(
            r###"# httpshhh? (4)

http://a/b (space)

http://a/b!

http://a/b"

http://a/b#

http://a/b$

http://a/b%

http://a/b&

http://a/b'

http://a/b(

http://a/b)

http://a/b*

http://a/b+

http://a/b,

http://a/b-

http://a/b

http://a/b.

http://a/b/

http://a/b:

http://a/b;

http://a/b<

http://a/b=

http://a/b>

http://a/b?

http://a/b@

http://a/b[

http://a/b\

http://a/b]

http://a/b^

http://a/b_

http://a/b`

http://a/b{

http://a/b|

http://a/b}

http://a/b~
"###,
            &Options::gfm()
        )?,
        r###"<h1>httpshhh? (4)</h1>
<p><a href="http://a/b">http://a/b</a> (space)</p>
<p><a href="http://a/b">http://a/b</a>!</p>
<p><a href="http://a/b">http://a/b</a>&quot;</p>
<p><a href="http://a/b#">http://a/b#</a></p>
<p><a href="http://a/b$">http://a/b$</a></p>
<p><a href="http://a/b%25">http://a/b%</a></p>
<p><a href="http://a/b&amp;">http://a/b&amp;</a></p>
<p><a href="http://a/b">http://a/b</a>'</p>
<p><a href="http://a/b(">http://a/b(</a></p>
<p><a href="http://a/b">http://a/b</a>)</p>
<p><a href="http://a/b">http://a/b</a>*</p>
<p><a href="http://a/b+">http://a/b+</a></p>
<p><a href="http://a/b">http://a/b</a>,</p>
<p><a href="http://a/b-">http://a/b-</a></p>
<p><a href="http://a/b">http://a/b</a></p>
<p><a href="http://a/b">http://a/b</a>.</p>
<p><a href="http://a/b/">http://a/b/</a></p>
<p><a href="http://a/b">http://a/b</a>:</p>
<p><a href="http://a/b">http://a/b</a>;</p>
<p><a href="http://a/b">http://a/b</a>&lt;</p>
<p><a href="http://a/b=">http://a/b=</a></p>
<p><a href="http://a/b%3E">http://a/b&gt;</a></p>
<p><a href="http://a/b">http://a/b</a>?</p>
<p><a href="http://a/b@">http://a/b@</a></p>
<p><a href="http://a/b%5B">http://a/b[</a></p>
<p><a href="http://a/b%5C">http://a/b\</a></p>
<p><a href="http://a/b">http://a/b</a>]</p>
<p><a href="http://a/b%5E">http://a/b^</a></p>
<p><a href="http://a/b">http://a/b</a>_</p>
<p><a href="http://a/b%60">http://a/b`</a></p>
<p><a href="http://a/b%7B">http://a/b{</a></p>
<p><a href="http://a/b%7C">http://a/b|</a></p>
<p><a href="http://a/b%7D">http://a/b}</a></p>
<p><a href="http://a/b">http://a/b</a>~</p>
"###,
        "should match protocol path continue like GitHub does"
    );

    assert_eq!(
        to_html_with_options(
            r###"# httpshhh? (3)

http://a/ (space)

http://a/!

http://a/"

http://a/#

http://a/$

http://a/%

http://a/&

http://a/'

http://a/(

http://a/)

http://a/*

http://a/+

http://a/,

http://a/-

http://a/

http://a/.

http://a//

http://a/:

http://a/;

http://a/<

http://a/=

http://a/>

http://a/?

http://a/@

http://a/[

http://a/\

http://a/]

http://a/^

http://a/_

http://a/`

http://a/{

http://a/|

http://a/}

http://a/~
"###,
            &Options::gfm()
        )?,
        r###"<h1>httpshhh? (3)</h1>
<p><a href="http://a/">http://a/</a> (space)</p>
<p><a href="http://a/">http://a/</a>!</p>
<p><a href="http://a/">http://a/</a>&quot;</p>
<p><a href="http://a/#">http://a/#</a></p>
<p><a href="http://a/$">http://a/$</a></p>
<p><a href="http://a/%25">http://a/%</a></p>
<p><a href="http://a/&amp;">http://a/&amp;</a></p>
<p><a href="http://a/">http://a/</a>'</p>
<p><a href="http://a/(">http://a/(</a></p>
<p><a href="http://a/">http://a/</a>)</p>
<p><a href="http://a/">http://a/</a>*</p>
<p><a href="http://a/+">http://a/+</a></p>
<p><a href="http://a/">http://a/</a>,</p>
<p><a href="http://a/-">http://a/-</a></p>
<p><a href="http://a/">http://a/</a></p>
<p><a href="http://a/">http://a/</a>.</p>
<p><a href="http://a//">http://a//</a></p>
<p><a href="http://a/">http://a/</a>:</p>
<p><a href="http://a/">http://a/</a>;</p>
<p><a href="http://a/">http://a/</a>&lt;</p>
<p><a href="http://a/=">http://a/=</a></p>
<p><a href="http://a/%3E">http://a/&gt;</a></p>
<p><a href="http://a/">http://a/</a>?</p>
<p><a href="http://a/@">http://a/@</a></p>
<p><a href="http://a/%5B">http://a/[</a></p>
<p><a href="http://a/%5C">http://a/\</a></p>
<p><a href="http://a/">http://a/</a>]</p>
<p><a href="http://a/%5E">http://a/^</a></p>
<p><a href="http://a/">http://a/</a>_</p>
<p><a href="http://a/%60">http://a/`</a></p>
<p><a href="http://a/%7B">http://a/{</a></p>
<p><a href="http://a/%7C">http://a/|</a></p>
<p><a href="http://a/%7D">http://a/}</a></p>
<p><a href="http://a/">http://a/</a>~</p>
"###,
        "should match protocol path start like GitHub does"
    );

    assert_eq!(
        to_html_with_options(
            r###"[www.example.com/a&copy;](#)

www.example.com/a&copy;

[www.example.com/a&bogus;](#)

www.example.com/a&bogus;

[www.example.com/a\.](#)

www.example.com/a\.
"###,
            &Options::gfm()
        )?,
        r###"<p><a href="#">www.example.com/a©</a></p>
<p><a href="http://www.example.com/a">www.example.com/a</a>©</p>
<p><a href="#">www.example.com/a&amp;bogus;</a></p>
<p><a href="http://www.example.com/a">www.example.com/a</a>&amp;bogus;</p>
<p><a href="#">www.example.com/a\.</a></p>
<p><a href="http://www.example.com/a%5C">www.example.com/a\</a>.</p>
"###,
        "should match links, autolink literals, and characters like GitHub does"
    );

    assert_eq!(
        to_html_with_options(
            r###"# “character reference”

www.a/b&c (space)

www.a/b&c!

www.a/b&c"

www.a/b&c#

www.a/b&c$

www.a/b&c%

www.a/b&c&

www.a/b&c'

www.a/b&c(

www.a/b&c)

www.a/b&c*

www.a/b&c+

www.a/b&c,

www.a/b&c-

www.a/b&c

www.a/b&c.

www.a/b&c/

www.a/b&c:

www.a/b&c;

www.a/b&c<

www.a/b&c=

www.a/b&c>

www.a/b&c?

www.a/b&c@

www.a/b&c[

www.a/b&c\

www.a/b&c]

www.a/b&c^

www.a/b&c_

www.a/b&c`

www.a/b&c{

www.a/b&c|

www.a/b&c}

www.a/b&c~
"###,
            &Options::gfm()
        )?,
        r###"<h1>“character reference”</h1>
<p><a href="http://www.a/b&amp;c">www.a/b&amp;c</a> (space)</p>
<p><a href="http://www.a/b&amp;c">www.a/b&amp;c</a>!</p>
<p><a href="http://www.a/b&amp;c">www.a/b&amp;c</a>&quot;</p>
<p><a href="http://www.a/b&amp;c#">www.a/b&amp;c#</a></p>
<p><a href="http://www.a/b&amp;c$">www.a/b&amp;c$</a></p>
<p><a href="http://www.a/b&amp;c%25">www.a/b&amp;c%</a></p>
<p><a href="http://www.a/b&amp;c&amp;">www.a/b&amp;c&amp;</a></p>
<p><a href="http://www.a/b&amp;c">www.a/b&amp;c</a>'</p>
<p><a href="http://www.a/b&amp;c(">www.a/b&amp;c(</a></p>
<p><a href="http://www.a/b&amp;c">www.a/b&amp;c</a>)</p>
<p><a href="http://www.a/b&amp;c">www.a/b&amp;c</a>*</p>
<p><a href="http://www.a/b&amp;c+">www.a/b&amp;c+</a></p>
<p><a href="http://www.a/b&amp;c">www.a/b&amp;c</a>,</p>
<p><a href="http://www.a/b&amp;c-">www.a/b&amp;c-</a></p>
<p><a href="http://www.a/b&amp;c">www.a/b&amp;c</a></p>
<p><a href="http://www.a/b&amp;c">www.a/b&amp;c</a>.</p>
<p><a href="http://www.a/b&amp;c/">www.a/b&amp;c/</a></p>
<p><a href="http://www.a/b&amp;c">www.a/b&amp;c</a>:</p>
<p><a href="http://www.a/b">www.a/b</a>&amp;c;</p>
<p><a href="http://www.a/b&amp;c">www.a/b&amp;c</a>&lt;</p>
<p><a href="http://www.a/b&amp;c=">www.a/b&amp;c=</a></p>
<p><a href="http://www.a/b&amp;c%3E">www.a/b&amp;c&gt;</a></p>
<p><a href="http://www.a/b&amp;c">www.a/b&amp;c</a>?</p>
<p><a href="http://www.a/b&amp;c@">www.a/b&amp;c@</a></p>
<p><a href="http://www.a/b&amp;c%5B">www.a/b&amp;c[</a></p>
<p><a href="http://www.a/b&amp;c%5C">www.a/b&amp;c\</a></p>
<p><a href="http://www.a/b&amp;c">www.a/b&amp;c</a>]</p>
<p><a href="http://www.a/b&amp;c%5E">www.a/b&amp;c^</a></p>
<p><a href="http://www.a/b&amp;c">www.a/b&amp;c</a>_</p>
<p><a href="http://www.a/b&amp;c%60">www.a/b&amp;c`</a></p>
<p><a href="http://www.a/b&amp;c%7B">www.a/b&amp;c{</a></p>
<p><a href="http://www.a/b&amp;c%7C">www.a/b&amp;c|</a></p>
<p><a href="http://www.a/b&amp;c%7D">www.a/b&amp;c}</a></p>
<p><a href="http://www.a/b&amp;c">www.a/b&amp;c</a>~</p>
"###,
        "should match character reference-like (named) things in paths like GitHub does"
    );

    assert_eq!(
        to_html_with_options(
            r###"# “character reference”

www.a/b&#35 (space)

www.a/b&#35!

www.a/b&#35"

www.a/b&#35#

www.a/b&#35$

www.a/b&#35%

www.a/b&#35&

www.a/b&#35'

www.a/b&#35(

www.a/b&#35)

www.a/b&#35*

www.a/b&#35+

www.a/b&#35,

www.a/b&#35-

www.a/b&#35

www.a/b&#35.

www.a/b&#35/

www.a/b&#35:

www.a/b&#35;

www.a/b&#35<

www.a/b&#35=

www.a/b&#35>

www.a/b&#35?

www.a/b&#35@

www.a/b&#35[

www.a/b&#35\

www.a/b&#35]

www.a/b&#35^

www.a/b&#35_

www.a/b&#35`

www.a/b&#35{

www.a/b&#35|

www.a/b&#35}

www.a/b&#35~
"###,
            &Options::gfm()
        )?,
        r###"<h1>“character reference”</h1>
<p><a href="http://www.a/b&amp;#35">www.a/b&amp;#35</a> (space)</p>
<p><a href="http://www.a/b&amp;#35">www.a/b&amp;#35</a>!</p>
<p><a href="http://www.a/b&amp;#35">www.a/b&amp;#35</a>&quot;</p>
<p><a href="http://www.a/b&amp;#35#">www.a/b&amp;#35#</a></p>
<p><a href="http://www.a/b&amp;#35$">www.a/b&amp;#35$</a></p>
<p><a href="http://www.a/b&amp;#35%25">www.a/b&amp;#35%</a></p>
<p><a href="http://www.a/b&amp;#35&amp;">www.a/b&amp;#35&amp;</a></p>
<p><a href="http://www.a/b&amp;#35">www.a/b&amp;#35</a>'</p>
<p><a href="http://www.a/b&amp;#35(">www.a/b&amp;#35(</a></p>
<p><a href="http://www.a/b&amp;#35">www.a/b&amp;#35</a>)</p>
<p><a href="http://www.a/b&amp;#35">www.a/b&amp;#35</a>*</p>
<p><a href="http://www.a/b&amp;#35+">www.a/b&amp;#35+</a></p>
<p><a href="http://www.a/b&amp;#35">www.a/b&amp;#35</a>,</p>
<p><a href="http://www.a/b&amp;#35-">www.a/b&amp;#35-</a></p>
<p><a href="http://www.a/b&amp;#35">www.a/b&amp;#35</a></p>
<p><a href="http://www.a/b&amp;#35">www.a/b&amp;#35</a>.</p>
<p><a href="http://www.a/b&amp;#35/">www.a/b&amp;#35/</a></p>
<p><a href="http://www.a/b&amp;#35">www.a/b&amp;#35</a>:</p>
<p><a href="http://www.a/b&amp;#35">www.a/b&amp;#35</a>;</p>
<p><a href="http://www.a/b&amp;#35">www.a/b&amp;#35</a>&lt;</p>
<p><a href="http://www.a/b&amp;#35=">www.a/b&amp;#35=</a></p>
<p><a href="http://www.a/b&amp;#35%3E">www.a/b&amp;#35&gt;</a></p>
<p><a href="http://www.a/b&amp;#35">www.a/b&amp;#35</a>?</p>
<p><a href="http://www.a/b&amp;#35@">www.a/b&amp;#35@</a></p>
<p><a href="http://www.a/b&amp;#35%5B">www.a/b&amp;#35[</a></p>
<p><a href="http://www.a/b&amp;#35%5C">www.a/b&amp;#35\</a></p>
<p><a href="http://www.a/b&amp;#35">www.a/b&amp;#35</a>]</p>
<p><a href="http://www.a/b&amp;#35%5E">www.a/b&amp;#35^</a></p>
<p><a href="http://www.a/b&amp;#35">www.a/b&amp;#35</a>_</p>
<p><a href="http://www.a/b&amp;#35%60">www.a/b&amp;#35`</a></p>
<p><a href="http://www.a/b&amp;#35%7B">www.a/b&amp;#35{</a></p>
<p><a href="http://www.a/b&amp;#35%7C">www.a/b&amp;#35|</a></p>
<p><a href="http://www.a/b&amp;#35%7D">www.a/b&amp;#35}</a></p>
<p><a href="http://www.a/b&amp;#35">www.a/b&amp;#35</a>~</p>
"###,
        "should match character reference-like (numeric) things in paths like GitHub does"
    );

    assert_eq!(
        to_html_with_options(
            r###"In autolink literal path or link end?

[https://a.com/d]()

[http://a.com/d]()

[www.a.com/d]()

https://a.com/d]()

http://a.com/d]()

www.a.com/d]()

In autolink literal search or link end?

[https://a.com?d]()

[http://a.com?d]()

[www.a.com?d]()

https://a.com?d]()

http://a.com?d]()

www.a.com?d]()

In autolink literal hash or link end?

[https://a.com#d]()

[http://a.com#d]()

[www.a.com#d]()

https://a.com#d]()

http://a.com#d]()

www.a.com#d]()
"###,
            &Options::gfm()
        )?,
        r###"<p>In autolink literal path or link end?</p>
<p><a href="">https://a.com/d</a></p>
<p><a href="">http://a.com/d</a></p>
<p><a href="">www.a.com/d</a></p>
<p><a href="https://a.com/d">https://a.com/d</a>]()</p>
<p><a href="http://a.com/d">http://a.com/d</a>]()</p>
<p><a href="http://www.a.com/d">www.a.com/d</a>]()</p>
<p>In autolink literal search or link end?</p>
<p><a href="">https://a.com?d</a></p>
<p><a href="">http://a.com?d</a></p>
<p><a href="">www.a.com?d</a></p>
<p><a href="https://a.com?d">https://a.com?d</a>]()</p>
<p><a href="http://a.com?d">http://a.com?d</a>]()</p>
<p><a href="http://www.a.com?d">www.a.com?d</a>]()</p>
<p>In autolink literal hash or link end?</p>
<p><a href="">https://a.com#d</a></p>
<p><a href="">http://a.com#d</a></p>
<p><a href="">www.a.com#d</a></p>
<p><a href="https://a.com#d">https://a.com#d</a>]()</p>
<p><a href="http://a.com#d">http://a.com#d</a>]()</p>
<p><a href="http://www.a.com#d">www.a.com#d</a>]()</p>
"###,
        "should match path or link end like GitHub does (except for the bracket bug)"
    );

    assert_eq!(
        to_html_with_options(
            r###"Last non-markdown ASCII whitespace (FF): noreply@example.com, http://example.com, https://example.com, www.example.com

Last non-whitespace ASCII control (US): noreply@example.com, http://example.com, https://example.com, www.example.com

First punctuation after controls: !noreply@example.com, !http://example.com, !https://example.com, !www.example.com

Last punctuation before digits: /noreply@example.com, /http://example.com, /https://example.com, /www.example.com

First digit: 0noreply@example.com, 0http://example.com, 0https://example.com, 0www.example.com

First punctuation after digits: :noreply@example.com, :http://example.com, :https://example.com, :www.example.com

Last punctuation before caps: @noreply@example.com, @http://example.com, @https://example.com, @www.example.com

First uppercase: Anoreply@example.com, Ahttp://example.com, Ahttps://example.com, Awww.example.com

Punctuation after uppercase: \noreply@example.com, \http://example.com, \https://example.com, \www.example.com

Last punctuation before lowercase (1): `noreply@example.com;

(2) `http://example.com;

(3) `https://example.com;

(4) `www.example.com; (broken up to prevent code from forming)

First lowercase: anoreply@example.com, ahttp://example.com, ahttps://example.com, awww.example.com

First punctuation after lowercase: {noreply@example.com, {http://example.com, {https://example.com, {www.example.com

Last punctuation: ~noreply@example.com, ~http://example.com, ~https://example.com, ~www.example.com

First non-ASCII unicode whitespace (0x80): noreply@example.com, http://example.com, https://example.com, www.example.com

Last non-ASCII unicode whitespace (0x3000): 　noreply@example.com, 　http://example.com, 　https://example.com, 　www.example.com

First non-ASCII punctuation: ¡noreply@example.com, ¡http://example.com, ¡https://example.com, ¡www.example.com

Last non-ASCII punctuation: ･noreply@example.com, ･http://example.com, ･https://example.com, ･www.example.com

Some non-ascii: 中noreply@example.com, 中http://example.com, 中https://example.com, 中www.example.com

Some more non-ascii: 🤷‍noreply@example.com, 🤷‍http://example.com, 🤷‍https://example.com, 🤷‍www.example.com
"###,
            &Options::gfm()
        )?,
        r###"<p>Last non-markdown ASCII whitespace (FF): <a href="mailto:noreply@example.com">noreply@example.com</a>, <a href="http://example.com">http://example.com</a>, <a href="https://example.com">https://example.com</a>, www.example.com</p>
<p>Last non-whitespace ASCII control (US): <a href="mailto:noreply@example.com">noreply@example.com</a>, <a href="http://example.com">http://example.com</a>, <a href="https://example.com">https://example.com</a>, www.example.com</p>
<p>First punctuation after controls: !<a href="mailto:noreply@example.com">noreply@example.com</a>, !<a href="http://example.com">http://example.com</a>, !<a href="https://example.com">https://example.com</a>, !www.example.com</p>
<p>Last punctuation before digits: /noreply@example.com, /<a href="http://example.com">http://example.com</a>, /<a href="https://example.com">https://example.com</a>, /www.example.com</p>
<p>First digit: <a href="mailto:0noreply@example.com">0noreply@example.com</a>, 0<a href="http://example.com">http://example.com</a>, 0<a href="https://example.com">https://example.com</a>, 0www.example.com</p>
<p>First punctuation after digits: :<a href="mailto:noreply@example.com">noreply@example.com</a>, :<a href="http://example.com">http://example.com</a>, :<a href="https://example.com">https://example.com</a>, :www.example.com</p>
<p>Last punctuation before caps: @<a href="mailto:noreply@example.com">noreply@example.com</a>, @<a href="http://example.com">http://example.com</a>, @<a href="https://example.com">https://example.com</a>, @www.example.com</p>
<p>First uppercase: <a href="mailto:Anoreply@example.com">Anoreply@example.com</a>, Ahttp://example.com, Ahttps://example.com, Awww.example.com</p>
<p>Punctuation after uppercase: \<a href="mailto:noreply@example.com">noreply@example.com</a>, \<a href="http://example.com">http://example.com</a>, \<a href="https://example.com">https://example.com</a>, \www.example.com</p>
<p>Last punctuation before lowercase (1): `<a href="mailto:noreply@example.com">noreply@example.com</a>;</p>
<p>(2) `<a href="http://example.com">http://example.com</a>;</p>
<p>(3) `<a href="https://example.com">https://example.com</a>;</p>
<p>(4) `www.example.com; (broken up to prevent code from forming)</p>
<p>First lowercase: <a href="mailto:anoreply@example.com">anoreply@example.com</a>, ahttp://example.com, ahttps://example.com, awww.example.com</p>
<p>First punctuation after lowercase: {<a href="mailto:noreply@example.com">noreply@example.com</a>, {<a href="http://example.com">http://example.com</a>, {<a href="https://example.com">https://example.com</a>, {www.example.com</p>
<p>Last punctuation: ~<a href="mailto:noreply@example.com">noreply@example.com</a>, ~<a href="http://example.com">http://example.com</a>, ~<a href="https://example.com">https://example.com</a>, ~<a href="http://www.example.com">www.example.com</a></p>
<p>First non-ASCII unicode whitespace (0x80): <a href="mailto:noreply@example.com">noreply@example.com</a>, <a href="http://example.com">http://example.com</a>, <a href="https://example.com">https://example.com</a>, www.example.com</p>
<p>Last non-ASCII unicode whitespace (0x3000): 　<a href="mailto:noreply@example.com">noreply@example.com</a>, 　<a href="http://example.com">http://example.com</a>, 　<a href="https://example.com">https://example.com</a>, 　www.example.com</p>
<p>First non-ASCII punctuation: ¡<a href="mailto:noreply@example.com">noreply@example.com</a>, ¡<a href="http://example.com">http://example.com</a>, ¡<a href="https://example.com">https://example.com</a>, ¡www.example.com</p>
<p>Last non-ASCII punctuation: ･<a href="mailto:noreply@example.com">noreply@example.com</a>, ･<a href="http://example.com">http://example.com</a>, ･<a href="https://example.com">https://example.com</a>, ･www.example.com</p>
<p>Some non-ascii: 中<a href="mailto:noreply@example.com">noreply@example.com</a>, 中<a href="http://example.com">http://example.com</a>, 中<a href="https://example.com">https://example.com</a>, 中www.example.com</p>
<p>Some more non-ascii: 🤷‍<a href="mailto:noreply@example.com">noreply@example.com</a>, 🤷‍<a href="http://example.com">http://example.com</a>, 🤷‍<a href="https://example.com">https://example.com</a>, 🤷‍www.example.com</p>
"###,
        "should match previous (complex) like GitHub does"
    );

    assert_eq!(
        to_html_with_options(
            r###"# HTTP

https://a.b can start after EOF

Can start after EOL:
https://a.b

Can start after tab:	https://a.b.

Can start after space: https://a.b.

Can start after left paren (https://a.b.

Can start after asterisk *https://a.b.

Can start after underscore *_https://a.b.

Can start after tilde ~https://a.b.

# www

www.a.b can start after EOF

Can start after EOL:
www.a.b

Can start after tab:	www.a.b.

Can start after space: www.a.b.

Can start after left paren (www.a.b.

Can start after asterisk *www.a.b.

Can start after underscore *_www.a.b.

Can start after tilde ~www.a.b.

# Email

## Correct character before

a@b.c can start after EOF

Can start after EOL:
a@b.c

Can start after tab:	a@b.c.

Can start after space: a@b.c.

Can start after left paren(a@b.c.

Can start after asterisk*a@b.c.

While theoretically it’s possible to start at an underscore, that underscore
is part of the email, so it’s in fact part of the link: _a@b.c.

Can start after tilde~a@b.c.

## Others characters before

While other characters before the email aren’t allowed by GFM, they work on
github.com: !a@b.c, "a@b.c, #a@b.c, $a@b.c, &a@b.c, 'a@b.c, )a@b.c, +a@b.c,
,a@b.c, -a@b.c, .a@b.c, /a@b.c, :a@b.c, ;a@b.c, <a@b.c, =a@b.c, >a@b.c, ?a@b.c,
@a@b.c, \a@b.c, ]a@b.c, ^a@b.c, `a@b.c, {a@b.c, }a@b.c.

## Commas

See `https://github.com/remarkjs/remark/discussions/678`.

,https://github.com

[ ,https://github.com

[asd] ,https://github.com
"###,
            &Options::gfm()
        )?,
        r###"<h1>HTTP</h1>
<p><a href="https://a.b">https://a.b</a> can start after EOF</p>
<p>Can start after EOL:
<a href="https://a.b">https://a.b</a></p>
<p>Can start after tab:	<a href="https://a.b">https://a.b</a>.</p>
<p>Can start after space: <a href="https://a.b">https://a.b</a>.</p>
<p>Can start after left paren (<a href="https://a.b">https://a.b</a>.</p>
<p>Can start after asterisk *<a href="https://a.b">https://a.b</a>.</p>
<p>Can start after underscore *_<a href="https://a.b">https://a.b</a>.</p>
<p>Can start after tilde ~<a href="https://a.b">https://a.b</a>.</p>
<h1>www</h1>
<p><a href="http://www.a.b">www.a.b</a> can start after EOF</p>
<p>Can start after EOL:
<a href="http://www.a.b">www.a.b</a></p>
<p>Can start after tab:	<a href="http://www.a.b">www.a.b</a>.</p>
<p>Can start after space: <a href="http://www.a.b">www.a.b</a>.</p>
<p>Can start after left paren (<a href="http://www.a.b">www.a.b</a>.</p>
<p>Can start after asterisk *<a href="http://www.a.b">www.a.b</a>.</p>
<p>Can start after underscore *_<a href="http://www.a.b">www.a.b</a>.</p>
<p>Can start after tilde ~<a href="http://www.a.b">www.a.b</a>.</p>
<h1>Email</h1>
<h2>Correct character before</h2>
<p><a href="mailto:a@b.c">a@b.c</a> can start after EOF</p>
<p>Can start after EOL:
<a href="mailto:a@b.c">a@b.c</a></p>
<p>Can start after tab:	<a href="mailto:a@b.c">a@b.c</a>.</p>
<p>Can start after space: <a href="mailto:a@b.c">a@b.c</a>.</p>
<p>Can start after left paren(<a href="mailto:a@b.c">a@b.c</a>.</p>
<p>Can start after asterisk*<a href="mailto:a@b.c">a@b.c</a>.</p>
<p>While theoretically it’s possible to start at an underscore, that underscore
is part of the email, so it’s in fact part of the link: <a href="mailto:_a@b.c">_a@b.c</a>.</p>
<p>Can start after tilde~<a href="mailto:a@b.c">a@b.c</a>.</p>
<h2>Others characters before</h2>
<p>While other characters before the email aren’t allowed by GFM, they work on
github.com: !<a href="mailto:a@b.c">a@b.c</a>, &quot;<a href="mailto:a@b.c">a@b.c</a>, #<a href="mailto:a@b.c">a@b.c</a>, $<a href="mailto:a@b.c">a@b.c</a>, &amp;<a href="mailto:a@b.c">a@b.c</a>, '<a href="mailto:a@b.c">a@b.c</a>, )<a href="mailto:a@b.c">a@b.c</a>, <a href="mailto:+a@b.c">+a@b.c</a>,
,<a href="mailto:a@b.c">a@b.c</a>, <a href="mailto:-a@b.c">-a@b.c</a>, <a href="mailto:.a@b.c">.a@b.c</a>, /a@b.c, :<a href="mailto:a@b.c">a@b.c</a>, ;<a href="mailto:a@b.c">a@b.c</a>, &lt;<a href="mailto:a@b.c">a@b.c</a>, =<a href="mailto:a@b.c">a@b.c</a>, &gt;<a href="mailto:a@b.c">a@b.c</a>, ?<a href="mailto:a@b.c">a@b.c</a>,
@<a href="mailto:a@b.c">a@b.c</a>, \<a href="mailto:a@b.c">a@b.c</a>, ]<a href="mailto:a@b.c">a@b.c</a>, ^<a href="mailto:a@b.c">a@b.c</a>, `<a href="mailto:a@b.c">a@b.c</a>, {<a href="mailto:a@b.c">a@b.c</a>, }<a href="mailto:a@b.c">a@b.c</a>.</p>
<h2>Commas</h2>
<p>See <code>https://github.com/remarkjs/remark/discussions/678</code>.</p>
<p>,<a href="https://github.com">https://github.com</a></p>
<p>[ ,<a href="https://github.com">https://github.com</a></p>
<p>[asd] ,<a href="https://github.com">https://github.com</a></p>
"###,
        "should match previous like GitHub does"
    );

    assert_eq!(
        to_html_with_options(
            r###"# wwwtf 2?

www.a (space)

www.a!

www.a"

www.a#

www.a$

www.a%

www.a&

www.a'

www.a(

www.a)

www.a*

www.a+

www.a,

www.a-

www.a

www.a.

www.a/

www.a:

www.a;

www.a<

www.a=

www.a>

www.a?

www.a@

www.a[

www.a\

www.a]

www.a^

www.a_

www.a`

www.a{

www.a|

www.a}

www.a~
"###,
            &Options::gfm()
        )?,
        r###"<h1>wwwtf 2?</h1>
<p><a href="http://www.a">www.a</a> (space)</p>
<p><a href="http://www.a">www.a</a>!</p>
<p><a href="http://www.a">www.a</a>&quot;</p>
<p><a href="http://www.a#">www.a#</a></p>
<p><a href="http://www.a$">www.a$</a></p>
<p><a href="http://www.a%25">www.a%</a></p>
<p><a href="http://www.a&amp;">www.a&amp;</a></p>
<p><a href="http://www.a">www.a</a>'</p>
<p><a href="http://www.a(">www.a(</a></p>
<p><a href="http://www.a">www.a</a>)</p>
<p><a href="http://www.a">www.a</a>*</p>
<p><a href="http://www.a+">www.a+</a></p>
<p><a href="http://www.a">www.a</a>,</p>
<p><a href="http://www.a-">www.a-</a></p>
<p><a href="http://www.a">www.a</a></p>
<p><a href="http://www.a">www.a</a>.</p>
<p><a href="http://www.a/">www.a/</a></p>
<p><a href="http://www.a">www.a</a>:</p>
<p><a href="http://www.a">www.a</a>;</p>
<p><a href="http://www.a">www.a</a>&lt;</p>
<p><a href="http://www.a=">www.a=</a></p>
<p><a href="http://www.a%3E">www.a&gt;</a></p>
<p><a href="http://www.a">www.a</a>?</p>
<p><a href="http://www.a@">www.a@</a></p>
<p><a href="http://www.a%5B">www.a[</a></p>
<p><a href="http://www.a%5C">www.a\</a></p>
<p><a href="http://www.a">www.a</a>]</p>
<p><a href="http://www.a%5E">www.a^</a></p>
<p><a href="http://www.a">www.a</a>_</p>
<p><a href="http://www.a%60">www.a`</a></p>
<p><a href="http://www.a%7B">www.a{</a></p>
<p><a href="http://www.a%7C">www.a|</a></p>
<p><a href="http://www.a%7D">www.a}</a></p>
<p><a href="http://www.a">www.a</a>~</p>
"###,
        "should match www (domain continue) like GitHub does (except for the bracket bug)"
    );

    assert_eq!(
        to_html_with_options(
            r###"# wwwtf 5?

www.a. (space)

www.a.!

www.a."

www.a.#

www.a.$

www.a.%

www.a.&

www.a.'

www.a.(

www.a.)

www.a.*

www.a.+

www.a.,

www.a.-

www.a.

www.a..

www.a./

www.a.:

www.a.;

www.a.<

www.a.=

www.a.>

www.a.?

www.a.@

www.a.[

www.a.\

www.a.]

www.a.^

www.a._

www.a.`

www.a.{

www.a.|

www.a.}

www.a.~
"###,
            &Options::gfm()
        )?,
        r###"<h1>wwwtf 5?</h1>
<p><a href="http://www.a">www.a</a>. (space)</p>
<p><a href="http://www.a">www.a</a>.!</p>
<p><a href="http://www.a">www.a</a>.&quot;</p>
<p><a href="http://www.a.#">www.a.#</a></p>
<p><a href="http://www.a.$">www.a.$</a></p>
<p><a href="http://www.a.%25">www.a.%</a></p>
<p><a href="http://www.a.&amp;">www.a.&amp;</a></p>
<p><a href="http://www.a">www.a</a>.'</p>
<p><a href="http://www.a.(">www.a.(</a></p>
<p><a href="http://www.a">www.a</a>.)</p>
<p><a href="http://www.a">www.a</a>.*</p>
<p><a href="http://www.a.+">www.a.+</a></p>
<p><a href="http://www.a">www.a</a>.,</p>
<p><a href="http://www.a.-">www.a.-</a></p>
<p><a href="http://www.a">www.a</a>.</p>
<p><a href="http://www.a">www.a</a>..</p>
<p><a href="http://www.a./">www.a./</a></p>
<p><a href="http://www.a">www.a</a>.:</p>
<p><a href="http://www.a">www.a</a>.;</p>
<p><a href="http://www.a">www.a</a>.&lt;</p>
<p><a href="http://www.a.=">www.a.=</a></p>
<p><a href="http://www.a.%3E">www.a.&gt;</a></p>
<p><a href="http://www.a">www.a</a>.?</p>
<p><a href="http://www.a.@">www.a.@</a></p>
<p><a href="http://www.a.%5B">www.a.[</a></p>
<p><a href="http://www.a.%5C">www.a.\</a></p>
<p><a href="http://www.a">www.a</a>.]</p>
<p><a href="http://www.a.%5E">www.a.^</a></p>
<p><a href="http://www.a">www.a</a>._</p>
<p><a href="http://www.a.%60">www.a.`</a></p>
<p><a href="http://www.a.%7B">www.a.{</a></p>
<p><a href="http://www.a.%7C">www.a.|</a></p>
<p><a href="http://www.a.%7D">www.a.}</a></p>
<p><a href="http://www.a">www.a</a>.~</p>
"###,
        "should match www (domain dot) like GitHub does (except for the bracket bug)"
    );

    assert_eq!(
        to_html_with_options(
            r###"# wwwtf?

www. (space)

www.!

www."

www.#

www.$

www.%

www.&

www.'

www.(

www.)

www.*

www.+

www.,

www.-

www.

www..

www./

www.:

www.;

www.<

www.=

www.>

www.?

www.@

www.[

www.\

www.]

www.^

www._

www.`

www.{

www.|

www.}

www.~
"###,
            &Options::gfm()
        )?,
        r###"<h1>wwwtf?</h1>
<p><a href="http://www">www</a>. (space)</p>
<p><a href="http://www">www</a>.!</p>
<p><a href="http://www">www</a>.&quot;</p>
<p><a href="http://www.#">www.#</a></p>
<p><a href="http://www.$">www.$</a></p>
<p><a href="http://www.%25">www.%</a></p>
<p><a href="http://www.&amp;">www.&amp;</a></p>
<p><a href="http://www">www</a>.'</p>
<p><a href="http://www.(">www.(</a></p>
<p><a href="http://www">www</a>.)</p>
<p><a href="http://www">www</a>.*</p>
<p><a href="http://www.+">www.+</a></p>
<p><a href="http://www">www</a>.,</p>
<p><a href="http://www.-">www.-</a></p>
<p>www.</p>
<p><a href="http://www">www</a>..</p>
<p><a href="http://www./">www./</a></p>
<p><a href="http://www">www</a>.:</p>
<p><a href="http://www">www</a>.;</p>
<p><a href="http://www">www</a>.&lt;</p>
<p><a href="http://www.=">www.=</a></p>
<p><a href="http://www.%3E">www.&gt;</a></p>
<p><a href="http://www">www</a>.?</p>
<p><a href="http://www.@">www.@</a></p>
<p><a href="http://www.%5B">www.[</a></p>
<p><a href="http://www.%5C">www.\</a></p>
<p><a href="http://www">www</a>.]</p>
<p><a href="http://www.%5E">www.^</a></p>
<p><a href="http://www">www</a>._</p>
<p><a href="http://www.%60">www.`</a></p>
<p><a href="http://www.%7B">www.{</a></p>
<p><a href="http://www.%7C">www.|</a></p>
<p><a href="http://www.%7D">www.}</a></p>
<p><a href="http://www">www</a>.~</p>
"###,
        "should match www (domain start) like GitHub does"
    );

    assert_eq!(
        to_html_with_options(
            r###"# wwwtf? (4)

www.a/b (space)

www.a/b!

www.a/b"

www.a/b#

www.a/b$

www.a/b%

www.a/b&

www.a/b'

www.a/b(

www.a/b)

www.a/b*

www.a/b+

www.a/b,

www.a/b-

www.a/b

www.a/b.

www.a/b/

www.a/b:

www.a/b;

www.a/b<

www.a/b=

www.a/b>

www.a/b?

www.a/b@

www.a/b[

www.a/b\

www.a/b]

www.a/b^

www.a/b_

www.a/b`

www.a/b{

www.a/b|

www.a/b}

www.a/b~
"###,
            &Options::gfm()
        )?,
        r###"<h1>wwwtf? (4)</h1>
<p><a href="http://www.a/b">www.a/b</a> (space)</p>
<p><a href="http://www.a/b">www.a/b</a>!</p>
<p><a href="http://www.a/b">www.a/b</a>&quot;</p>
<p><a href="http://www.a/b#">www.a/b#</a></p>
<p><a href="http://www.a/b$">www.a/b$</a></p>
<p><a href="http://www.a/b%25">www.a/b%</a></p>
<p><a href="http://www.a/b&amp;">www.a/b&amp;</a></p>
<p><a href="http://www.a/b">www.a/b</a>'</p>
<p><a href="http://www.a/b(">www.a/b(</a></p>
<p><a href="http://www.a/b">www.a/b</a>)</p>
<p><a href="http://www.a/b">www.a/b</a>*</p>
<p><a href="http://www.a/b+">www.a/b+</a></p>
<p><a href="http://www.a/b">www.a/b</a>,</p>
<p><a href="http://www.a/b-">www.a/b-</a></p>
<p><a href="http://www.a/b">www.a/b</a></p>
<p><a href="http://www.a/b">www.a/b</a>.</p>
<p><a href="http://www.a/b/">www.a/b/</a></p>
<p><a href="http://www.a/b">www.a/b</a>:</p>
<p><a href="http://www.a/b">www.a/b</a>;</p>
<p><a href="http://www.a/b">www.a/b</a>&lt;</p>
<p><a href="http://www.a/b=">www.a/b=</a></p>
<p><a href="http://www.a/b%3E">www.a/b&gt;</a></p>
<p><a href="http://www.a/b">www.a/b</a>?</p>
<p><a href="http://www.a/b@">www.a/b@</a></p>
<p><a href="http://www.a/b%5B">www.a/b[</a></p>
<p><a href="http://www.a/b%5C">www.a/b\</a></p>
<p><a href="http://www.a/b">www.a/b</a>]</p>
<p><a href="http://www.a/b%5E">www.a/b^</a></p>
<p><a href="http://www.a/b">www.a/b</a>_</p>
<p><a href="http://www.a/b%60">www.a/b`</a></p>
<p><a href="http://www.a/b%7B">www.a/b{</a></p>
<p><a href="http://www.a/b%7C">www.a/b|</a></p>
<p><a href="http://www.a/b%7D">www.a/b}</a></p>
<p><a href="http://www.a/b">www.a/b</a>~</p>
"###,
        "should match www (path continue) like GitHub does (except for the bracket bug)"
    );

    assert_eq!(
        to_html_with_options(
            r###"# wwwtf? (3)

www.a/ (space)

www.a/!

www.a/"

www.a/#

www.a/$

www.a/%

www.a/&

www.a/'

www.a/(

www.a/)

www.a/*

www.a/+

www.a/,

www.a/-

www.a/

www.a/.

www.a//

www.a/:

www.a/;

www.a/<

www.a/=

www.a/>

www.a/?

www.a/@

www.a/[

www.a/\

www.a/]

www.a/^

www.a/_

www.a/`

www.a/{

www.a/|

www.a/}

www.a/~
"###,
            &Options::gfm()
        )?,
        r###"<h1>wwwtf? (3)</h1>
<p><a href="http://www.a/">www.a/</a> (space)</p>
<p><a href="http://www.a/">www.a/</a>!</p>
<p><a href="http://www.a/">www.a/</a>&quot;</p>
<p><a href="http://www.a/#">www.a/#</a></p>
<p><a href="http://www.a/$">www.a/$</a></p>
<p><a href="http://www.a/%25">www.a/%</a></p>
<p><a href="http://www.a/&amp;">www.a/&amp;</a></p>
<p><a href="http://www.a/">www.a/</a>'</p>
<p><a href="http://www.a/(">www.a/(</a></p>
<p><a href="http://www.a/">www.a/</a>)</p>
<p><a href="http://www.a/">www.a/</a>*</p>
<p><a href="http://www.a/+">www.a/+</a></p>
<p><a href="http://www.a/">www.a/</a>,</p>
<p><a href="http://www.a/-">www.a/-</a></p>
<p><a href="http://www.a/">www.a/</a></p>
<p><a href="http://www.a/">www.a/</a>.</p>
<p><a href="http://www.a//">www.a//</a></p>
<p><a href="http://www.a/">www.a/</a>:</p>
<p><a href="http://www.a/">www.a/</a>;</p>
<p><a href="http://www.a/">www.a/</a>&lt;</p>
<p><a href="http://www.a/=">www.a/=</a></p>
<p><a href="http://www.a/%3E">www.a/&gt;</a></p>
<p><a href="http://www.a/">www.a/</a>?</p>
<p><a href="http://www.a/@">www.a/@</a></p>
<p><a href="http://www.a/%5B">www.a/[</a></p>
<p><a href="http://www.a/%5C">www.a/\</a></p>
<p><a href="http://www.a/">www.a/</a>]</p>
<p><a href="http://www.a/%5E">www.a/^</a></p>
<p><a href="http://www.a/">www.a/</a>_</p>
<p><a href="http://www.a/%60">www.a/`</a></p>
<p><a href="http://www.a/%7B">www.a/{</a></p>
<p><a href="http://www.a/%7C">www.a/|</a></p>
<p><a href="http://www.a/%7D">www.a/}</a></p>
<p><a href="http://www.a/">www.a/</a>~</p>
"###,
        "should match www (path start) like GitHub does (except for the bracket bug)"
    );

    assert_eq!(
        to_mdast(
            "a https://alpha.com b bravo@charlie.com c www.delta.com d xmpp:echo@foxtrot.com e mailto:golf@hotel.com f.",
            &ParseOptions::gfm()
        )?,
        Node::Root(Root {
            children: vec![Node::Paragraph(Paragraph {
                children: vec![
                    Node::Text(Text {
                        value: "a ".into(),
                        position: Some(Position::new(1, 1, 0, 1, 3, 2))
                    }),
                    Node::Link(Link {
                        url: "https://alpha.com".into(),
                        title: None,
                        children: vec![Node::Text(Text {
                            value: "https://alpha.com".into(),
                            position: Some(Position::new(1, 3, 2, 1, 20, 19))
                        }),],
                        position: Some(Position::new(1, 3, 2, 1, 20, 19))
                    }),
                    Node::Text(Text {
                        value: " b ".into(),
                        position: Some(Position::new(1, 20, 19, 1, 23, 22))
                    }),
                    Node::Link(Link {
                        url: "mailto:bravo@charlie.com".into(),
                        title: None,
                        children: vec![Node::Text(Text {
                            value: "bravo@charlie.com".into(),
                            position: Some(Position::new(1, 23, 22, 1, 40, 39))
                        }),],
                        position: Some(Position::new(1, 23, 22, 1, 40, 39))
                    }),
                    Node::Text(Text {
                        value: " c ".into(),
                        position: Some(Position::new(1, 40, 39, 1, 43, 42))
                    }),
                    Node::Link(Link {
                        url: "http://www.delta.com".into(),
                        title: None,
                        children: vec![Node::Text(Text {
                            value: "www.delta.com".into(),
                            position: Some(Position::new(1, 43, 42, 1, 56, 55))
                        }),],
                        position: Some(Position::new(1, 43, 42, 1, 56, 55))
                    }),
                    Node::Text(Text {
                        value: " d ".into(),
                        position: Some(Position::new(1, 56, 55, 1, 59, 58))
                    }),
                    Node::Link(Link {
                        url: "xmpp:echo@foxtrot.com".into(),
                        title: None,
                        children: vec![Node::Text(Text {
                            value: "xmpp:echo@foxtrot.com".into(),
                            position: Some(Position::new(1, 59, 58, 1, 80, 79))
                        }),],
                        position: Some(Position::new(1, 59, 58, 1, 80, 79))
                    }),
                    Node::Text(Text {
                        value: " e ".into(),
                        position: Some(Position::new(1, 80, 79, 1, 83, 82))
                    }),
                    Node::Link(Link {
                        url: "mailto:golf@hotel.com".into(),
                        title: None,
                        children: vec![Node::Text(Text {
                            value: "mailto:golf@hotel.com".into(),
                            position: Some(Position::new(1, 83, 82, 1, 104, 103))
                        }),],
                        position: Some(Position::new(1, 83, 82, 1, 104, 103))
                    }),
                    Node::Text(Text {
                        value: " f.".into(),
                        position: Some(Position::new(1, 104, 103, 1, 107, 106))
                    })
                ],
                position: Some(Position::new(1, 1, 0, 1, 107, 106))
            })],
            position: Some(Position::new(1, 1, 0, 1, 107, 106))
        }),
        "should support GFM autolink literals as `Link`s in mdast"
    );

    Ok(())
}
