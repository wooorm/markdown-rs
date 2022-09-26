extern crate micromark;
use micromark::{
    mdast::{
        Definition, Image, ImageReference, Node, Paragraph, Position, ReferenceKind, Root, Text,
    },
    micromark, micromark_to_mdast, micromark_with_options, Constructs, Options,
};
use pretty_assertions::assert_eq;

#[test]
fn image() -> Result<(), String> {
    assert_eq!(
        micromark("[link](/uri \"title\")"),
        "<p><a href=\"/uri\" title=\"title\">link</a></p>",
        "should support links"
    );
    assert_eq!(
        micromark("![foo](/url \"title\")"),
        "<p><img src=\"/url\" alt=\"foo\" title=\"title\" /></p>",
        "should support image w/ resource"
    );

    assert_eq!(
        micromark("[foo *bar*]: train.jpg \"train & tracks\"\n\n![foo *bar*]"),
        "<p><img src=\"train.jpg\" alt=\"foo bar\" title=\"train &amp; tracks\" /></p>",
        "should support image as shortcut reference"
    );

    assert_eq!(
        micromark("![foo ![bar](/url)](/url2)"),
        "<p><img src=\"/url2\" alt=\"foo bar\" /></p>",
        "should “support” images in images"
    );

    assert_eq!(
        micromark("![foo [bar](/url)](/url2)"),
        "<p><img src=\"/url2\" alt=\"foo bar\" /></p>",
        "should “support” links in images"
    );

    assert_eq!(
        micromark("[foo *bar*]: train.jpg \"train & tracks\"\n\n![foo *bar*][]"),
        "<p><img src=\"train.jpg\" alt=\"foo bar\" title=\"train &amp; tracks\" /></p>",
        "should support “content” in images"
    );

    assert_eq!(
        micromark("[FOOBAR]: train.jpg \"train & tracks\"\n\n![foo *bar*][foobar]"),
        "<p><img src=\"train.jpg\" alt=\"foo bar\" title=\"train &amp; tracks\" /></p>",
        "should support “content” in images"
    );

    assert_eq!(
        micromark("![foo](train.jpg)"),
        "<p><img src=\"train.jpg\" alt=\"foo\" /></p>",
        "should support images w/o title"
    );

    assert_eq!(
        micromark("My ![foo bar](/path/to/train.jpg  \"title\"   )"),
        "<p>My <img src=\"/path/to/train.jpg\" alt=\"foo bar\" title=\"title\" /></p>",
        "should support images w/ lots of whitespace"
    );

    assert_eq!(
        micromark("![foo](<url>)"),
        "<p><img src=\"url\" alt=\"foo\" /></p>",
        "should support images w/ enclosed destinations"
    );

    assert_eq!(
        micromark("![](/url)"),
        "<p><img src=\"/url\" alt=\"\" /></p>",
        "should support images w/ empty labels"
    );

    assert_eq!(
        micromark("[bar]: /url\n\n![foo][bar]"),
        "<p><img src=\"/url\" alt=\"foo\" /></p>",
        "should support full references (1)"
    );

    assert_eq!(
        micromark("[BAR]: /url\n\n![foo][bar]"),
        "<p><img src=\"/url\" alt=\"foo\" /></p>",
        "should support full references (2)"
    );

    assert_eq!(
        micromark("[foo]: /url \"title\"\n\n![foo][]"),
        "<p><img src=\"/url\" alt=\"foo\" title=\"title\" /></p>",
        "should support collapsed references (1)"
    );

    assert_eq!(
        micromark("[*foo* bar]: /url \"title\"\n\n![*foo* bar][]"),
        "<p><img src=\"/url\" alt=\"foo bar\" title=\"title\" /></p>",
        "should support collapsed references (2)"
    );

    assert_eq!(
        micromark("[foo]: /url \"title\"\n\n![Foo][]"),
        "<p><img src=\"/url\" alt=\"Foo\" title=\"title\" /></p>",
        "should support case-insensitive labels"
    );

    assert_eq!(
        micromark("[foo]: /url \"title\"\n\n![foo] \n[]"),
        "<p><img src=\"/url\" alt=\"foo\" title=\"title\" />\n[]</p>",
        "should not support whitespace between sets of brackets"
    );

    assert_eq!(
        micromark("[foo]: /url \"title\"\n\n![foo]"),
        "<p><img src=\"/url\" alt=\"foo\" title=\"title\" /></p>",
        "should support shortcut references (1)"
    );

    assert_eq!(
        micromark("[*foo* bar]: /url \"title\"\n\n![*foo* bar]"),
        "<p><img src=\"/url\" alt=\"foo bar\" title=\"title\" /></p>",
        "should support shortcut references (2)"
    );

    assert_eq!(
        micromark("[[foo]]: /url \"title\"\n\n![[foo]]"),
        "<p>[[foo]]: /url &quot;title&quot;</p>\n<p>![[foo]]</p>",
        "should not support link labels w/ unescaped brackets"
    );

    assert_eq!(
        micromark("[foo]: /url \"title\"\n\n![Foo]"),
        "<p><img src=\"/url\" alt=\"Foo\" title=\"title\" /></p>",
        "should support case-insensitive label matching"
    );

    assert_eq!(
        micromark("[foo]: /url \"title\"\n\n!\\[foo]"),
        "<p>![foo]</p>",
        "should “support” an escaped bracket instead of an image"
    );

    assert_eq!(
        micromark("[foo]: /url \"title\"\n\n\\![foo]"),
        "<p>!<a href=\"/url\" title=\"title\">foo</a></p>",
        "should support an escaped bang instead of an image, but still have a link"
    );

    // Extra
    assert_eq!(
        micromark("![foo]()"),
        "<p><img src=\"\" alt=\"foo\" /></p>",
        "should support images w/o destination"
    );

    assert_eq!(
        micromark("![foo](<>)"),
        "<p><img src=\"\" alt=\"foo\" /></p>",
        "should support images w/ explicit empty destination"
    );

    assert_eq!(
        micromark("![](example.png)"),
        "<p><img src=\"example.png\" alt=\"\" /></p>",
        "should support images w/o alt"
    );

    assert_eq!(
        micromark("![alpha](bravo.png \"\")"),
        "<p><img src=\"bravo.png\" alt=\"alpha\" /></p>",
        "should support images w/ empty title (1)"
    );

    assert_eq!(
        micromark("![alpha](bravo.png '')"),
        "<p><img src=\"bravo.png\" alt=\"alpha\" /></p>",
        "should support images w/ empty title (2)"
    );

    assert_eq!(
        micromark("![alpha](bravo.png ())"),
        "<p><img src=\"bravo.png\" alt=\"alpha\" /></p>",
        "should support images w/ empty title (3)"
    );

    assert_eq!(
    micromark("![&amp;&copy;&](example.com/&amp;&copy;& \"&amp;&copy;&\")"),
    "<p><img src=\"example.com/&amp;%C2%A9&amp;\" alt=\"&amp;©&amp;\" title=\"&amp;©&amp;\" /></p>",
    "should support character references in images"
  );

    // Extra
    // See: <https://github.com/commonmark/commonmark.js/issues/192>
    assert_eq!(
        micromark("![](<> \"\")"),
        "<p><img src=\"\" alt=\"\" /></p>",
        "should ignore an empty title"
    );

    assert_eq!(
        micromark_with_options(
            "![x]()",
            &Options {
                constructs: Constructs {
                    label_start_image: false,
                    ..Constructs::default()
                },
                ..Options::default()
            }
        )?,
        "<p>!<a href=\"\">x</a></p>",
        "should support turning off label start (image)"
    );

    assert_eq!(
        micromark("![](javascript:alert(1))"),
        "<p><img src=\"\" alt=\"\" /></p>",
        "should ignore non-http protocols by default"
    );

    assert_eq!(
        micromark_with_options(
            "![](javascript:alert(1))",
            &Options {
                allow_dangerous_protocol: true,
                ..Options::default()
            }
        )?,
        "<p><img src=\"javascript:alert(1)\" alt=\"\" /></p>",
        "should allow non-http protocols w/ `allowDangerousProtocol`"
    );

    assert_eq!(
        micromark_to_mdast(
            "a ![alpha]() b ![bravo](charlie 'delta') c.",
            &Options::default()
        )?,
        Node::Root(Root {
            children: vec![Node::Paragraph(Paragraph {
                children: vec![
                    Node::Text(Text {
                        value: "a ".to_string(),
                        position: Some(Position::new(1, 1, 0, 1, 3, 2))
                    }),
                    Node::Image(Image {
                        alt: "alpha".to_string(),
                        url: String::new(),
                        title: None,
                        position: Some(Position::new(1, 3, 2, 1, 13, 12))
                    }),
                    Node::Text(Text {
                        value: " b ".to_string(),
                        position: Some(Position::new(1, 13, 12, 1, 16, 15))
                    }),
                    Node::Image(Image {
                        alt: "bravo".to_string(),
                        url: "charlie".to_string(),
                        title: Some("delta".to_string()),
                        position: Some(Position::new(1, 16, 15, 1, 41, 40))
                    }),
                    Node::Text(Text {
                        value: " c.".to_string(),
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
        micromark_to_mdast(
            "[x]: y\n\na ![x] b ![x][] c ![d][x] e.",
            &Options::default()
        )?,
        Node::Root(Root {
            children: vec![
                Node::Definition(Definition {
                    identifier: "x".to_string(),
                    label: Some("x".to_string()),
                    url: "y".to_string(),
                    title: None,
                    position: Some(Position::new(1, 1, 0, 1, 7, 6))
                }),
                Node::Paragraph(Paragraph {
                    children: vec![
                        Node::Text(Text {
                            value: "a ".to_string(),
                            position: Some(Position::new(3, 1, 8, 3, 3, 10))
                        }),
                        Node::ImageReference(ImageReference {
                            reference_kind: ReferenceKind::Shortcut,
                            identifier: "x".to_string(),
                            label: Some("x".to_string()),
                            alt: "x".to_string(),
                            position: Some(Position::new(3, 3, 10, 3, 7, 14))
                        }),
                        Node::Text(Text {
                            value: " b ".to_string(),
                            position: Some(Position::new(3, 7, 14, 3, 10, 17))
                        }),
                        Node::ImageReference(ImageReference {
                            reference_kind: ReferenceKind::Collapsed,
                            identifier: "x".to_string(),
                            label: Some("x".to_string()),
                            alt: "x".to_string(),
                            position: Some(Position::new(3, 10, 17, 3, 16, 23))
                        }),
                        Node::Text(Text {
                            value: " c ".to_string(),
                            position: Some(Position::new(3, 16, 23, 3, 19, 26))
                        }),
                        Node::ImageReference(ImageReference {
                            reference_kind: ReferenceKind::Full,
                            identifier: "x".to_string(),
                            label: Some("x".to_string()),
                            alt: "d".to_string(),
                            position: Some(Position::new(3, 19, 26, 3, 26, 33))
                        }),
                        Node::Text(Text {
                            value: " e.".to_string(),
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
