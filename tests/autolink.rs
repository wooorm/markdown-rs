use markdown::{
    mdast::{Link, Node, Paragraph, Root, Text},
    message, to_html, to_html_with_options, to_mdast,
    unist::Position,
    CompileOptions, Constructs, Options, ParseOptions,
};
use pretty_assertions::assert_eq;

#[test]
fn autolink() -> Result<(), message::Message> {
    let danger = Options {
        compile: CompileOptions {
            allow_dangerous_html: true,
            allow_dangerous_protocol: true,
            ..Default::default()
        },
        ..Default::default()
    };

    assert_eq!(
        to_html("<http://foo.bar.baz>"),
        "<p><a href=\"http://foo.bar.baz\">http://foo.bar.baz</a></p>",
        "should support protocol autolinks (1)"
    );

    assert_eq!(
    to_html("<http://foo.bar.baz/test?q=hello&id=22&boolean>"),
    "<p><a href=\"http://foo.bar.baz/test?q=hello&amp;id=22&amp;boolean\">http://foo.bar.baz/test?q=hello&amp;id=22&amp;boolean</a></p>",
    "should support protocol autolinks (2)"
  );

    assert_eq!(
        to_html("<irc://foo.bar:2233/baz>"),
        "<p><a href=\"irc://foo.bar:2233/baz\">irc://foo.bar:2233/baz</a></p>",
        "should support protocol autolinks w/ non-HTTP schemes"
    );

    assert_eq!(
        to_html("<MAILTO:FOO@BAR.BAZ>"),
        "<p><a href=\"MAILTO:FOO@BAR.BAZ\">MAILTO:FOO@BAR.BAZ</a></p>",
        "should support protocol autolinks in uppercase"
    );

    assert_eq!(
        to_html("<a+b+c:d>"),
        "<p><a href=\"\">a+b+c:d</a></p>",
        "should support protocol autolinks w/ incorrect URIs (1, default)"
    );

    assert_eq!(
        to_html_with_options("<a+b+c:d>", &danger)?,
        "<p><a href=\"a+b+c:d\">a+b+c:d</a></p>",
        "should support protocol autolinks w/ incorrect URIs (1, danger)"
    );

    assert_eq!(
        to_html("<made-up-scheme://foo,bar>"),
        "<p><a href=\"\">made-up-scheme://foo,bar</a></p>",
        "should support protocol autolinks w/ incorrect URIs (2, default)"
    );

    assert_eq!(
        to_html_with_options("<made-up-scheme://foo,bar>", &danger)?,
        "<p><a href=\"made-up-scheme://foo,bar\">made-up-scheme://foo,bar</a></p>",
        "should support protocol autolinks w/ incorrect URIs (2, danger)"
    );

    assert_eq!(
        to_html("<http://../>"),
        "<p><a href=\"http://../\">http://../</a></p>",
        "should support protocol autolinks w/ incorrect URIs (3)"
    );

    assert_eq!(
        to_html_with_options("<localhost:5001/foo>", &danger)?,
        "<p><a href=\"localhost:5001/foo\">localhost:5001/foo</a></p>",
        "should support protocol autolinks w/ incorrect URIs (4)"
    );

    assert_eq!(
        to_html("<http://foo.bar/baz bim>"),
        "<p>&lt;http://foo.bar/baz bim&gt;</p>",
        "should not support protocol autolinks w/ spaces"
    );

    assert_eq!(
        to_html("<http://example.com/\\[\\>"),
        "<p><a href=\"http://example.com/%5C%5B%5C\">http://example.com/\\[\\</a></p>",
        "should not support character escapes in protocol autolinks"
    );

    assert_eq!(
        to_html("<foo@bar.example.com>"),
        "<p><a href=\"mailto:foo@bar.example.com\">foo@bar.example.com</a></p>",
        "should support email autolinks (1)"
    );

    assert_eq!(
        to_html("<foo+special@Bar.baz-bar0.com>"),
        "<p><a href=\"mailto:foo+special@Bar.baz-bar0.com\">foo+special@Bar.baz-bar0.com</a></p>",
        "should support email autolinks (2)"
    );

    assert_eq!(
        to_html("<a@b.c>"),
        "<p><a href=\"mailto:a@b.c\">a@b.c</a></p>",
        "should support email autolinks (3)"
    );

    assert_eq!(
        to_html("<foo\\+@bar.example.com>"),
        "<p>&lt;foo+@bar.example.com&gt;</p>",
        "should not support character escapes in email autolinks"
    );

    assert_eq!(
        to_html("<>"),
        "<p>&lt;&gt;</p>",
        "should not support empty autolinks"
    );

    assert_eq!(
        to_html("< http://foo.bar >"),
        "<p>&lt; http://foo.bar &gt;</p>",
        "should not support autolinks w/ space"
    );

    assert_eq!(
        to_html("<m:abc>"),
        "<p>&lt;m:abc&gt;</p>",
        "should not support autolinks w/ a single character for a scheme"
    );

    assert_eq!(
        to_html("<foo.bar.baz>"),
        "<p>&lt;foo.bar.baz&gt;</p>",
        "should not support autolinks w/o a colon or at sign"
    );

    assert_eq!(
        to_html("http://example.com"),
        "<p>http://example.com</p>",
        "should not support protocol autolinks w/o angle brackets"
    );

    assert_eq!(
        to_html("foo@bar.example.com"),
        "<p>foo@bar.example.com</p>",
        "should not support email autolinks w/o angle brackets"
    );

    // Extra:
    assert_eq!(
        to_html("<*@example.com>"),
        "<p><a href=\"mailto:*@example.com\">*@example.com</a></p>",
        "should support autolinks w/ atext (1)"
    );
    assert_eq!(
        to_html("<a*@example.com>"),
        "<p><a href=\"mailto:a*@example.com\">a*@example.com</a></p>",
        "should support autolinks w/ atext (2)"
    );
    assert_eq!(
        to_html("<aa*@example.com>"),
        "<p><a href=\"mailto:aa*@example.com\">aa*@example.com</a></p>",
        "should support autolinks w/ atext (3)"
    );

    assert_eq!(
        to_html("<aaa©@example.com>"),
        "<p>&lt;aaa©@example.com&gt;</p>",
        "should support non-atext in email autolinks local part (1)"
    );
    assert_eq!(
        to_html("<a*a©@example.com>"),
        "<p>&lt;a*a©@example.com&gt;</p>",
        "should support non-atext in email autolinks local part (2)"
    );

    assert_eq!(
        to_html("<asd@.example.com>"),
        "<p>&lt;asd@.example.com&gt;</p>",
        "should not support a dot after an at sign in email autolinks"
    );
    assert_eq!(
        to_html("<asd@e..xample.com>"),
        "<p>&lt;asd@e..xample.com&gt;</p>",
        "should not support a dot after another dot in email autolinks"
    );

    assert_eq!(
        to_html(
        "<asd@012345678901234567890123456789012345678901234567890123456789012>"),
        "<p><a href=\"mailto:asd@012345678901234567890123456789012345678901234567890123456789012\">asd@012345678901234567890123456789012345678901234567890123456789012</a></p>",
        "should support 63 character in email autolinks domains"
    );

    assert_eq!(
        to_html("<asd@0123456789012345678901234567890123456789012345678901234567890123>"),
        "<p>&lt;asd@0123456789012345678901234567890123456789012345678901234567890123&gt;</p>",
        "should not support 64 character in email autolinks domains"
    );

    assert_eq!(
        to_html(
            "<asd@012345678901234567890123456789012345678901234567890123456789012.a>"),
        "<p><a href=\"mailto:asd@012345678901234567890123456789012345678901234567890123456789012.a\">asd@012345678901234567890123456789012345678901234567890123456789012.a</a></p>",
        "should support a TLD after a 63 character domain in email autolinks"
    );

    assert_eq!(
        to_html("<asd@0123456789012345678901234567890123456789012345678901234567890123.a>"),
        "<p>&lt;asd@0123456789012345678901234567890123456789012345678901234567890123.a&gt;</p>",
        "should not support a TLD after a 64 character domain in email autolinks"
    );

    assert_eq!(
        to_html(
            "<asd@a.012345678901234567890123456789012345678901234567890123456789012>"),
        "<p><a href=\"mailto:asd@a.012345678901234567890123456789012345678901234567890123456789012\">asd@a.012345678901234567890123456789012345678901234567890123456789012</a></p>",
        "should support a 63 character TLD in email autolinks"
    );

    assert_eq!(
        to_html("<asd@a.0123456789012345678901234567890123456789012345678901234567890123>"),
        "<p>&lt;asd@a.0123456789012345678901234567890123456789012345678901234567890123&gt;</p>",
        "should not support a 64 character TLD in email autolinks"
    );

    assert_eq!(
        to_html("<asd@-example.com>"),
        "<p>&lt;asd@-example.com&gt;</p>",
        "should not support a dash after `@` in email autolinks"
    );

    assert_eq!(
        to_html("<asd@e-xample.com>"),
        "<p><a href=\"mailto:asd@e-xample.com\">asd@e-xample.com</a></p>",
        "should support a dash after other domain characters in email autolinks"
    );

    assert_eq!(
        to_html("<asd@e--xample.com>"),
        "<p><a href=\"mailto:asd@e--xample.com\">asd@e--xample.com</a></p>",
        "should support a dash after another dash in email autolinks"
    );

    assert_eq!(
        to_html("<asd@example-.com>"),
        "<p>&lt;asd@example-.com&gt;</p>",
        "should not support a dash before a dot in email autolinks"
    );

    assert_eq!(
        to_html("<@example.com>"),
        "<p>&lt;@example.com&gt;</p>",
        "should not support an at sign at the start of email autolinks"
    );

    assert_eq!(
        to_html_with_options(
            "<a@b.co>",
            &Options {
                parse: ParseOptions {
                    constructs: Constructs {
                        autolink: false,
                        ..Default::default()
                    },
                    ..Default::default()
                },
                ..Default::default()
            }
        )?,
        "<p>&lt;a@b.co&gt;</p>",
        "should support turning off autolinks"
    );

    assert_eq!(
        to_mdast(
            "a <https://alpha.com> b <bravo@charlie.com> c.",
            &Default::default()
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
                            position: Some(Position::new(1, 4, 3, 1, 21, 20))
                        }),],
                        position: Some(Position::new(1, 3, 2, 1, 22, 21))
                    }),
                    Node::Text(Text {
                        value: " b ".into(),
                        position: Some(Position::new(1, 22, 21, 1, 25, 24))
                    }),
                    Node::Link(Link {
                        url: "mailto:bravo@charlie.com".into(),
                        title: None,
                        children: vec![Node::Text(Text {
                            value: "bravo@charlie.com".into(),
                            position: Some(Position::new(1, 26, 25, 1, 43, 42))
                        }),],
                        position: Some(Position::new(1, 25, 24, 1, 44, 43))
                    }),
                    Node::Text(Text {
                        value: " c.".into(),
                        position: Some(Position::new(1, 44, 43, 1, 47, 46))
                    })
                ],
                position: Some(Position::new(1, 1, 0, 1, 47, 46))
            })],
            position: Some(Position::new(1, 1, 0, 1, 47, 46))
        }),
        "should support autolinks as `Link`s in mdast"
    );

    Ok(())
}
