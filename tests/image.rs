use markdown::{
    mdast::{Definition, Image, ImageReference, Node, Paragraph, ReferenceKind, Root, Text},
    message, to_html, to_html_with_options, to_mdast,
    unist::Position,
    CompileOptions, Constructs, Options, ParseOptions,
};
use pretty_assertions::assert_eq;

#[test]
fn image() -> Result<(), message::Message> {
    assert_eq!(
        to_html("[link](/uri \"title\")"),
        "<p><a href=\"/uri\" title=\"title\">link</a></p>",
        "should support links"
    );
    assert_eq!(
        to_html("![foo](/url \"title\")"),
        "<p><img src=\"/url\" alt=\"foo\" title=\"title\" /></p>",
        "should support image w/ resource"
    );

    assert_eq!(
        to_html("[foo *bar*]: train.jpg \"train & tracks\"\n\n![foo *bar*]"),
        "<p><img src=\"train.jpg\" alt=\"foo bar\" title=\"train &amp; tracks\" /></p>",
        "should support image as shortcut reference"
    );

    assert_eq!(
        to_html("![foo ![bar](/url)](/url2)"),
        "<p><img src=\"/url2\" alt=\"foo bar\" /></p>",
        "should “support” images in images"
    );

    assert_eq!(
        to_html("![foo [bar](/url)](/url2)"),
        "<p><img src=\"/url2\" alt=\"foo bar\" /></p>",
        "should “support” links in images"
    );

    assert_eq!(
        to_html("[foo *bar*]: train.jpg \"train & tracks\"\n\n![foo *bar*][]"),
        "<p><img src=\"train.jpg\" alt=\"foo bar\" title=\"train &amp; tracks\" /></p>",
        "should support “content” in images"
    );

    assert_eq!(
        to_html("[FOOBAR]: train.jpg \"train & tracks\"\n\n![foo *bar*][foobar]"),
        "<p><img src=\"train.jpg\" alt=\"foo bar\" title=\"train &amp; tracks\" /></p>",
        "should support “content” in images"
    );

    assert_eq!(
        to_html("![foo](train.jpg)"),
        "<p><img src=\"train.jpg\" alt=\"foo\" /></p>",
        "should support images w/o title"
    );

    assert_eq!(
        to_html("My ![foo bar](/path/to/train.jpg  \"title\"   )"),
        "<p>My <img src=\"/path/to/train.jpg\" alt=\"foo bar\" title=\"title\" /></p>",
        "should support images w/ lots of whitespace"
    );

    assert_eq!(
        to_html("![foo](<url>)"),
        "<p><img src=\"url\" alt=\"foo\" /></p>",
        "should support images w/ enclosed destinations"
    );

    assert_eq!(
        to_html("![](/url)"),
        "<p><img src=\"/url\" alt=\"\" /></p>",
        "should support images w/ empty labels"
    );

    assert_eq!(
        to_html("[bar]: /url\n\n![foo][bar]"),
        "<p><img src=\"/url\" alt=\"foo\" /></p>",
        "should support full references (1)"
    );

    assert_eq!(
        to_html("[BAR]: /url\n\n![foo][bar]"),
        "<p><img src=\"/url\" alt=\"foo\" /></p>",
        "should support full references (2)"
    );

    assert_eq!(
        to_html("[foo]: /url \"title\"\n\n![foo][]"),
        "<p><img src=\"/url\" alt=\"foo\" title=\"title\" /></p>",
        "should support collapsed references (1)"
    );

    assert_eq!(
        to_html("[*foo* bar]: /url \"title\"\n\n![*foo* bar][]"),
        "<p><img src=\"/url\" alt=\"foo bar\" title=\"title\" /></p>",
        "should support collapsed references (2)"
    );

    assert_eq!(
        to_html("[foo]: /url \"title\"\n\n![Foo][]"),
        "<p><img src=\"/url\" alt=\"Foo\" title=\"title\" /></p>",
        "should support case-insensitive labels"
    );

    assert_eq!(
        to_html("[foo]: /url \"title\"\n\n![foo] \n[]"),
        "<p><img src=\"/url\" alt=\"foo\" title=\"title\" />\n[]</p>",
        "should not support whitespace between sets of brackets"
    );

    assert_eq!(
        to_html("[foo]: /url \"title\"\n\n![foo]"),
        "<p><img src=\"/url\" alt=\"foo\" title=\"title\" /></p>",
        "should support shortcut references (1)"
    );

    assert_eq!(
        to_html("[*foo* bar]: /url \"title\"\n\n![*foo* bar]"),
        "<p><img src=\"/url\" alt=\"foo bar\" title=\"title\" /></p>",
        "should support shortcut references (2)"
    );

    assert_eq!(
        to_html("[[foo]]: /url \"title\"\n\n![[foo]]"),
        "<p>[[foo]]: /url &quot;title&quot;</p>\n<p>![[foo]]</p>",
        "should not support link labels w/ unescaped brackets"
    );

    assert_eq!(
        to_html("[foo]: /url \"title\"\n\n![Foo]"),
        "<p><img src=\"/url\" alt=\"Foo\" title=\"title\" /></p>",
        "should support case-insensitive label matching"
    );

    assert_eq!(
        to_html("[foo]: /url \"title\"\n\n!\\[foo]"),
        "<p>![foo]</p>",
        "should “support” an escaped bracket instead of an image"
    );

    assert_eq!(
        to_html("[foo]: /url \"title\"\n\n\\![foo]"),
        "<p>!<a href=\"/url\" title=\"title\">foo</a></p>",
        "should support an escaped bang instead of an image, but still have a link"
    );

    // Extra
    assert_eq!(
        to_html("![foo]()"),
        "<p><img src=\"\" alt=\"foo\" /></p>",
        "should support images w/o destination"
    );

    assert_eq!(
        to_html("![foo](<>)"),
        "<p><img src=\"\" alt=\"foo\" /></p>",
        "should support images w/ explicit empty destination"
    );

    assert_eq!(
        to_html("![](example.png)"),
        "<p><img src=\"example.png\" alt=\"\" /></p>",
        "should support images w/o alt"
    );

    assert_eq!(
        to_html("![alpha](bravo.png \"\")"),
        "<p><img src=\"bravo.png\" alt=\"alpha\" /></p>",
        "should support images w/ empty title (1)"
    );

    assert_eq!(
        to_html("![alpha](bravo.png '')"),
        "<p><img src=\"bravo.png\" alt=\"alpha\" /></p>",
        "should support images w/ empty title (2)"
    );

    assert_eq!(
        to_html("![alpha](bravo.png ())"),
        "<p><img src=\"bravo.png\" alt=\"alpha\" /></p>",
        "should support images w/ empty title (3)"
    );

    assert_eq!(
    to_html("![&amp;&copy;&](example.com/&amp;&copy;& \"&amp;&copy;&\")"),
    "<p><img src=\"example.com/&amp;%C2%A9&amp;\" alt=\"&amp;©&amp;\" title=\"&amp;©&amp;\" /></p>",
    "should support character references in images"
  );

    // Extra
    // See: <https://github.com/commonmark/commonmark.js/issues/192>
    assert_eq!(
        to_html("![](<> \"\")"),
        "<p><img src=\"\" alt=\"\" /></p>",
        "should ignore an empty title"
    );

    assert_eq!(
        to_html_with_options(
            "![x]()",
            &Options {
                parse: ParseOptions {
                    constructs: Constructs {
                        label_start_image: false,
                        ..Default::default()
                    },
                    ..Default::default()
                },
                ..Default::default()
            }
        )?,
        "<p>!<a href=\"\">x</a></p>",
        "should support turning off label start (image)"
    );

    assert_eq!(
        to_html("![](javascript:alert(1))"),
        "<p><img src=\"\" alt=\"\" /></p>",
        "should ignore non-http protocols by default"
    );

    assert_eq!(
        to_html_with_options(
            "![](javascript:alert(1))",
            &Options {
                compile: CompileOptions {
                    allow_dangerous_protocol: true,
                    ..Default::default()
                },
                ..Default::default()
            }
        )?,
        "<p><img src=\"javascript:alert(1)\" alt=\"\" /></p>",
        "should allow non-http protocols w/ `allowDangerousProtocol`"
    );

    assert_eq!(
        to_html_with_options(
            "![](javascript:alert(1))",
            &Options {
                compile: CompileOptions {
                    allow_any_img_src: true,
                    allow_dangerous_protocol: false,
                    ..Default::default()
                },
                ..Default::default()
            }
        )?,
        "<p><img src=\"javascript:alert(1)\" alt=\"\" /></p>",
        "should allow non-http protocols with the `allow_any_img_src` option"
    );

    assert_eq!(
        to_mdast(
            "a ![alpha]() b ![bravo](charlie 'delta') c.",
            &Default::default()
        )?,
        Node::Root(Root {
            children: vec![Node::Paragraph(Paragraph {
                children: vec![
                    Node::Text(Text {
                        value: "a ".into(),
                        position: Some(Position::new(1, 1, 0, 1, 3, 2))
                    }),
                    Node::Image(Image {
                        alt: "alpha".into(),
                        url: String::new(),
                        title: None,
                        position: Some(Position::new(1, 3, 2, 1, 13, 12))
                    }),
                    Node::Text(Text {
                        value: " b ".into(),
                        position: Some(Position::new(1, 13, 12, 1, 16, 15))
                    }),
                    Node::Image(Image {
                        alt: "bravo".into(),
                        url: "charlie".into(),
                        title: Some("delta".into()),
                        position: Some(Position::new(1, 16, 15, 1, 41, 40))
                    }),
                    Node::Text(Text {
                        value: " c.".into(),
                        position: Some(Position::new(1, 41, 40, 1, 44, 43))
                    })
                ],
                position: Some(Position::new(1, 1, 0, 1, 44, 43))
            })],
            position: Some(Position::new(1, 1, 0, 1, 44, 43))
        }),
        "should support image (resource) as `Image`s in mdast"
    );

    assert_eq!(
        to_mdast(
            "[x]: y\n\na ![x] b ![x][] c ![d][x] e.",
            &Default::default()
        )?,
        Node::Root(Root {
            children: vec![
                Node::Definition(Definition {
                    identifier: "x".into(),
                    label: Some("x".into()),
                    url: "y".into(),
                    title: None,
                    position: Some(Position::new(1, 1, 0, 1, 7, 6))
                }),
                Node::Paragraph(Paragraph {
                    children: vec![
                        Node::Text(Text {
                            value: "a ".into(),
                            position: Some(Position::new(3, 1, 8, 3, 3, 10))
                        }),
                        Node::ImageReference(ImageReference {
                            reference_kind: ReferenceKind::Shortcut,
                            identifier: "x".into(),
                            label: Some("x".into()),
                            alt: "x".into(),
                            position: Some(Position::new(3, 3, 10, 3, 7, 14))
                        }),
                        Node::Text(Text {
                            value: " b ".into(),
                            position: Some(Position::new(3, 7, 14, 3, 10, 17))
                        }),
                        Node::ImageReference(ImageReference {
                            reference_kind: ReferenceKind::Collapsed,
                            identifier: "x".into(),
                            label: Some("x".into()),
                            alt: "x".into(),
                            position: Some(Position::new(3, 10, 17, 3, 16, 23))
                        }),
                        Node::Text(Text {
                            value: " c ".into(),
                            position: Some(Position::new(3, 16, 23, 3, 19, 26))
                        }),
                        Node::ImageReference(ImageReference {
                            reference_kind: ReferenceKind::Full,
                            identifier: "x".into(),
                            label: Some("x".into()),
                            alt: "d".into(),
                            position: Some(Position::new(3, 19, 26, 3, 26, 33))
                        }),
                        Node::Text(Text {
                            value: " e.".into(),
                            position: Some(Position::new(3, 26, 33, 3, 29, 36))
                        }),
                    ],
                    position: Some(Position::new(3, 1, 8, 3, 29, 36))
                }),
            ],
            position: Some(Position::new(1, 1, 0, 3, 29, 36))
        }),
        "should support image (reference) as `ImageReference`s in mdast"
    );
    Ok(())
}
