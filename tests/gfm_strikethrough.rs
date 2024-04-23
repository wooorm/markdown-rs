#![allow(clippy::needless_raw_string_hashes)]

// To do: clippy introduced this in 1.72 but breaks when it fixes it.
// Remove when solved.

use markdown::{
    mdast::{Delete, Node, Paragraph, Root, Text},
    message, to_html, to_html_with_options, to_mdast,
    unist::Position,
    Options, ParseOptions,
};
use pretty_assertions::assert_eq;

#[test]
fn gfm_strikethrough() -> Result<(), message::Message> {
    assert_eq!(
        to_html("a ~b~ c"),
        "<p>a ~b~ c</p>",
        "should ignore strikethrough by default"
    );

    assert_eq!(
        to_html_with_options("a ~b~", &Options::gfm())?,
        "<p>a <del>b</del></p>",
        "should support strikethrough w/ one tilde"
    );

    assert_eq!(
        to_html_with_options("a ~~b~~", &Options::gfm())?,
        "<p>a <del>b</del></p>",
        "should support strikethrough w/ two tildes"
    );

    assert_eq!(
        to_html_with_options("a ~~~b~~~", &Options::gfm())?,
        "<p>a ~~~b~~~</p>",
        "should not support strikethrough w/ three tildes"
    );

    assert_eq!(
        to_html_with_options("a \\~~~b~~ c", &Options::gfm())?,
        "<p>a ~<del>b</del> c</p>",
        "should support strikethrough after an escaped tilde"
    );

    assert_eq!(
        to_html_with_options("a ~~b ~~c~~ d~~ e", &Options::gfm())?,
        "<p>a <del>b <del>c</del> d</del> e</p>",
        "should support nested strikethrough"
    );

    assert_eq!(
        to_html_with_options("a ~-1~ b", &Options::gfm())?,
        "<p>a <del>-1</del> b</p>",
        "should open if preceded by whitespace and followed by punctuation"
    );

    assert_eq!(
        to_html_with_options("a ~b.~ c", &Options::gfm())?,
        "<p>a <del>b.</del> c</p>",
        "should close if preceded by punctuation and followed by whitespace"
    );

    assert_eq!(
        to_html_with_options("~b.~.", &Options::gfm())?,
        "<p><del>b.</del>.</p>",
        "should close if preceded and followed by punctuation"
    );

    assert_eq!(
        to_html_with_options(
            r###"
# Balanced

a ~one~ b

a ~~two~~ b

a ~~~three~~~ b

a ~~~~four~~~~ b

# Unbalanced

a ~one/two~~ b

a ~one/three~~~ b

a ~one/four~~~~ b

***

a ~~two/one~ b

a ~~two/three~~~ b

a ~~two/four~~~~ b

***

a ~~~three/one~ b

a ~~~three/two~~ b

a ~~~three/four~~~~ b

***

a ~~~~four/one~ b

a ~~~~four/two~~ b

a ~~~~four/three~~~ b

## Multiple

a ~one b one~ c one~ d

a ~one b two~~ c one~ d

a ~one b one~ c two~~ d

a ~~two b two~~ c two~~ d

a ~~two b one~ c two~~ d

a ~~two b two~~ c one~ d
"###,
            &Options::gfm()
        )?,
        r###"<h1>Balanced</h1>
<p>a <del>one</del> b</p>
<p>a <del>two</del> b</p>
<p>a ~~~three~~~ b</p>
<p>a ~~~~four~~~~ b</p>
<h1>Unbalanced</h1>
<p>a ~one/two~~ b</p>
<p>a ~one/three~~~ b</p>
<p>a ~one/four~~~~ b</p>
<hr />
<p>a ~~two/one~ b</p>
<p>a ~~two/three~~~ b</p>
<p>a ~~two/four~~~~ b</p>
<hr />
<p>a ~~~three/one~ b</p>
<p>a ~~~three/two~~ b</p>
<p>a ~~~three/four~~~~ b</p>
<hr />
<p>a ~~~~four/one~ b</p>
<p>a ~~~~four/two~~ b</p>
<p>a ~~~~four/three~~~ b</p>
<h2>Multiple</h2>
<p>a <del>one b one</del> c one~ d</p>
<p>a <del>one b two~~ c one</del> d</p>
<p>a <del>one b one</del> c two~~ d</p>
<p>a <del>two b two</del> c two~~ d</p>
<p>a <del>two b one~ c two</del> d</p>
<p>a <del>two b two</del> c one~ d</p>
"###,
        "should handle balance like GitHub"
    );

    assert_eq!(
        to_html_with_options(
            r###"
# Flank

a oneRight~ b oneRight~ c oneRight~ d

a oneRight~ b oneRight~ c ~oneLeft d

a oneRight~ b ~oneLeft c oneRight~ d

a ~oneLeft b oneRight~ c oneRight~ d

a ~oneLeft b oneRight~ c ~oneLeft d

a ~oneLeft b ~oneLeft c oneRight~ d

a ~oneLeft b ~oneLeft c ~oneLeft d

***

a twoRight~~ b twoRight~~ c twoRight~~ d

a twoRight~~ b twoRight~~ c ~~twoLeft d

a twoRight~~ b ~~twoLeft c twoRight~~ d

a ~~twoLeft b twoRight~~ c twoRight~~ d

a ~~twoLeft b twoRight~~ c ~~twoLeft d

a ~~twoLeft b ~~twoLeft c twoRight~~ d

a ~~twoLeft b ~~twoLeft c ~~twoLeft d
"###,
            &Options::gfm()
        )?,
        r###"<h1>Flank</h1>
<p>a oneRight~ b oneRight~ c oneRight~ d</p>
<p>a oneRight~ b oneRight~ c ~oneLeft d</p>
<p>a oneRight~ b <del>oneLeft c oneRight</del> d</p>
<p>a <del>oneLeft b oneRight</del> c oneRight~ d</p>
<p>a <del>oneLeft b oneRight</del> c ~oneLeft d</p>
<p>a ~oneLeft b <del>oneLeft c oneRight</del> d</p>
<p>a ~oneLeft b ~oneLeft c ~oneLeft d</p>
<hr />
<p>a twoRight~~ b twoRight~~ c twoRight~~ d</p>
<p>a twoRight~~ b twoRight~~ c ~~twoLeft d</p>
<p>a twoRight~~ b <del>twoLeft c twoRight</del> d</p>
<p>a <del>twoLeft b twoRight</del> c twoRight~~ d</p>
<p>a <del>twoLeft b twoRight</del> c ~~twoLeft d</p>
<p>a ~~twoLeft b <del>twoLeft c twoRight</del> d</p>
<p>a ~~twoLeft b ~~twoLeft c ~~twoLeft d</p>
"###,
        "should handle flanking like GitHub"
    );

    assert_eq!(
        to_html_with_options(
            r###"
# Interlpay

## Interleave with attention

a ~~two *emphasis* two~~ b

a ~~two **strong** two~~ b

a *marker ~~two marker* two~~ b

a ~~two *marker two~~ marker* b

## Interleave with links

a ~~two [resource](#) two~~ b

a ~~two [reference][#] two~~ b

a [label start ~~two label end](#) two~~ b

a ~~two [label start two~~ label end](#) b

a ~~two [label start ~one one~ label end](#) two~~ b

a ~one [label start ~~two two~~ label end](#) one~ b

a ~one [label start ~one one~ label end](#) one~ b

a ~~two [label start ~~two two~~ label end](#) two~~ b

[#]: #

## Interleave with code (text)

a ~~two `code` two~~ b

a ~~two `code two~~` b

a `code start ~~two code end` two~~ b

a ~~two `code start two~~ code end` b

a ~~two `code start ~one one~ code end` two~~ b

a ~one `code start ~~two two~~ code end` one~ b

a ~one `code start ~one one~ code end` one~ b

a ~~two `code start ~~two two~~ code end` two~~ b

## Emphasis/strong/strikethrough interplay

a ***~~xxx~~*** zzz

b ***xxx***zzz

c **xxx**zzz

d *xxx*zzz

e ***~~xxx~~***yyy

f **~~xxx~~**yyy

g *~~xxx~~*yyy

h ***~~xxx~~*** zzz

i **~~xxx~~** zzz

j *~~xxx~~* zzz

k ~~~**xxx**~~~ zzz

l ~~~xxx~~~zzz

m ~~xxx~~zzz

n ~xxx~zzz

o ~~~**xxx**~~~yyy

p ~~**xxx**~~yyy

r ~**xxx**~yyy

s ~~~**xxx**~~~ zzz

t ~~**xxx**~~ zzz

u ~**xxx**~ zzz
"###,
            &Options::gfm()
        )?,
        r###"<h1>Interlpay</h1>
<h2>Interleave with attention</h2>
<p>a <del>two <em>emphasis</em> two</del> b</p>
<p>a <del>two <strong>strong</strong> two</del> b</p>
<p>a <em>marker ~~two marker</em> two~~ b</p>
<p>a <del>two *marker two</del> marker* b</p>
<h2>Interleave with links</h2>
<p>a <del>two <a href="#">resource</a> two</del> b</p>
<p>a <del>two <a href="#">reference</a> two</del> b</p>
<p>a <a href="#">label start ~~two label end</a> two~~ b</p>
<p>a ~~two <a href="#">label start two~~ label end</a> b</p>
<p>a <del>two <a href="#">label start <del>one one</del> label end</a> two</del> b</p>
<p>a <del>one <a href="#">label start <del>two two</del> label end</a> one</del> b</p>
<p>a <del>one <a href="#">label start <del>one one</del> label end</a> one</del> b</p>
<p>a <del>two <a href="#">label start <del>two two</del> label end</a> two</del> b</p>
<h2>Interleave with code (text)</h2>
<p>a <del>two <code>code</code> two</del> b</p>
<p>a ~~two <code>code two~~</code> b</p>
<p>a <code>code start ~~two code end</code> two~~ b</p>
<p>a ~~two <code>code start two~~ code end</code> b</p>
<p>a <del>two <code>code start ~one one~ code end</code> two</del> b</p>
<p>a <del>one <code>code start ~~two two~~ code end</code> one</del> b</p>
<p>a <del>one <code>code start ~one one~ code end</code> one</del> b</p>
<p>a <del>two <code>code start ~~two two~~ code end</code> two</del> b</p>
<h2>Emphasis/strong/strikethrough interplay</h2>
<p>a <em><strong><del>xxx</del></strong></em> zzz</p>
<p>b <em><strong>xxx</strong></em>zzz</p>
<p>c <strong>xxx</strong>zzz</p>
<p>d <em>xxx</em>zzz</p>
<p>e <em><strong><del>xxx</del></strong></em>yyy</p>
<p>f <strong><del>xxx</del></strong>yyy</p>
<p>g <em><del>xxx</del></em>yyy</p>
<p>h <em><strong><del>xxx</del></strong></em> zzz</p>
<p>i <strong><del>xxx</del></strong> zzz</p>
<p>j <em><del>xxx</del></em> zzz</p>
<p>k ~~~<strong>xxx</strong>~~~ zzz</p>
<p>l ~~~xxx~~~zzz</p>
<p>m <del>xxx</del>zzz</p>
<p>n <del>xxx</del>zzz</p>
<p>o ~~~<strong>xxx</strong>~~~yyy</p>
<p>p ~~<strong>xxx</strong>~~yyy</p>
<p>r ~<strong>xxx</strong>~yyy</p>
<p>s ~~~<strong>xxx</strong>~~~ zzz</p>
<p>t <del><strong>xxx</strong></del> zzz</p>
<p>u <del><strong>xxx</strong></del> zzz</p>
"###,
        "should handle interplay like GitHub"
    );

    assert_eq!(
        to_html_with_options("a*~b~*c\n\na*.b.*c", &Options::gfm())?,
        "<p>a<em><del>b</del></em>c</p>\n<p>a*.b.*c</p>",
        "should handle interplay w/ other attention markers (GFM)"
    );

    assert_eq!(
        to_html("a*~b~*c\n\na*.b.*c"),
        "<p>a*~b~*c</p>\n<p>a*.b.*c</p>",
        "should handle interplay w/ other attention markers (CM reference)"
    );

    assert_eq!(
        to_html_with_options(
            "a ~b~ ~~c~~ d",
            &Options {
                parse: ParseOptions {
                    gfm_strikethrough_single_tilde: false,
                    ..ParseOptions::gfm()
                },
                ..Options::gfm()
            }
        )?,
        "<p>a ~b~ <del>c</del> d</p>",
        "should not support strikethrough w/ one tilde if `singleTilde: false`"
    );

    assert_eq!(
        to_html_with_options(
            "a ~b~ ~~c~~ d",
            &Options {
                parse: ParseOptions {
                    gfm_strikethrough_single_tilde: true,
                    ..ParseOptions::gfm()
                },
                ..Options::gfm()
            }
        )?,
        "<p>a <del>b</del> <del>c</del> d</p>",
        "should support strikethrough w/ one tilde if `singleTilde: true`"
    );

    assert_eq!(
        to_mdast("a ~~alpha~~ b.", &ParseOptions::gfm())?,
        Node::Root(Root {
            children: vec![Node::Paragraph(Paragraph {
                children: vec![
                    Node::Text(Text {
                        value: "a ".into(),
                        position: Some(Position::new(1, 1, 0, 1, 3, 2))
                    }),
                    Node::Delete(Delete {
                        children: vec![Node::Text(Text {
                            value: "alpha".into(),
                            position: Some(Position::new(1, 5, 4, 1, 10, 9))
                        }),],
                        position: Some(Position::new(1, 3, 2, 1, 12, 11))
                    }),
                    Node::Text(Text {
                        value: " b.".into(),
                        position: Some(Position::new(1, 12, 11, 1, 15, 14))
                    }),
                ],
                position: Some(Position::new(1, 1, 0, 1, 15, 14))
            })],
            position: Some(Position::new(1, 1, 0, 1, 15, 14))
        }),
        "should support GFM strikethrough as `Delete`s in mdast"
    );

    Ok(())
}
