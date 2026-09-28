use hast_util_to_html::{to_html, to_html_with_options, Options};
use pretty_assertions::assert_eq;

fn element(
    tag_name: &'static str,
    properties: Vec<(&'static str, hast::PropertyValue)>,
    children: Vec<hast::Node>,
) -> hast::Node {
    hast::Node::Element(hast::Element {
        tag_name: tag_name.into(),
        properties: properties.into_iter().map(|(k, v)| (k.into(), v)).collect(),
        children,
        position: None,
    })
}

fn text(value: &'static str) -> hast::Node {
    hast::Node::Text(hast::Text {
        value: value.into(),
        position: None,
    })
}

#[test]
fn serializes_void_elements() {
    assert_eq!(
        to_html_with_options(&element("br", vec![], vec![]), &Options::default()),
        "<br />",
        "should self-close void elements like `to_html` does"
    );
}

#[test]
fn maps_property_names_to_attributes() {
    let node = element(
        "a",
        vec![
            (
                "className",
                hast::PropertyValue::SpaceSeparated(vec!["b".into(), "c".into()]),
            ),
            ("htmlFor", hast::PropertyValue::String("d".into())),
            ("dataFootnoteRef", hast::PropertyValue::Boolean(true)),
            ("ariaDescribedBy", hast::PropertyValue::String("e".into())),
            ("hidden", hast::PropertyValue::Boolean(false)),
            (
                "accept",
                hast::PropertyValue::CommaSeparated(vec!["f".into(), "g".into()]),
            ),
        ],
        vec![],
    );

    assert_eq!(
        to_html_with_options(&node, &Options::default()),
        "<a class=\"b c\" for=\"d\" data-footnote-ref=\"\" aria-describedby=\"e\" accept=\"f, g\"></a>",
        "should map names, join lists, and drop false booleans"
    );
}

#[test]
fn encodes_text_and_attributes() {
    let node = element(
        "a",
        vec![("title", hast::PropertyValue::String("\"&".into()))],
        vec![text("<&>\"")],
    );

    assert_eq!(
        to_html_with_options(&node, &Options::default()),
        "<a title=\"&quot;&amp;\">&lt;&amp;&gt;&quot;</a>",
        "should encode `&`, `<`, `>`, and `\"`"
    );
}

#[test]
fn serializes_comments_and_doctypes() {
    let root = hast::Node::Root(hast::Root {
        children: vec![
            hast::Node::Doctype(hast::Doctype { position: None }),
            hast::Node::Comment(hast::Comment {
                value: " a --> b --!> c ".into(),
                position: None,
            }),
        ],
        position: None,
    });

    assert_eq!(
        to_html_with_options(&root, &Options::default()),
        "<!doctype html><!-- a --&gt; b --!&gt; c -->",
        "should keep comment values from closing the comment"
    );
}

#[test]
fn drops_dangerous_protocols_unless_allowed() {
    let node = element(
        "a",
        vec![(
            "href",
            hast::PropertyValue::String("javascript:alert(1)".into()),
        )],
        vec![element(
            "img",
            vec![("src", hast::PropertyValue::String("data:x".into()))],
            vec![],
        )],
    );

    assert_eq!(
        to_html_with_options(&node, &Options::default()),
        "<a href=\"\"><img src=\"\" /></a>",
        "should empty `href` and `src` with unsafe protocols by default"
    );
    assert_eq!(
        to_html_with_options(
            &node,
            &Options {
                allow_dangerous_protocol: true,
                ..Options::default()
            }
        ),
        "<a href=\"javascript:alert(1)\"><img src=\"data:x\" /></a>",
        "should keep any protocol when allowed"
    );
}

#[test]
fn keeps_comment_edges_from_closing_the_comment() {
    for (value, expected) in [
        (">a", "<!--&gt;a-->"),
        ("->a", "<!---&gt;a-->"),
        ("a<!-", "<!--a&lt;!--->"),
        ("a<!--b", "<!--a&lt;!--b-->"),
    ] {
        let node = hast::Node::Comment(hast::Comment {
            value: value.into(),
            position: None,
        });
        assert_eq!(
            to_html_with_options(&node, &Options::default()),
            expected,
            "should escape {:?}",
            value
        );
    }
}

#[test]
fn replaces_nul_and_keeps_other_characters() {
    assert_eq!(
        to_html_with_options(&text("é\0🦀&"), &Options::default()),
        "é\u{FFFD}🦀&amp;",
        "should replace NUL, and keep other characters"
    );
}

#[test]
fn writes_urls_as_given_but_checks_their_protocol() {
    let link = |href: &'static str| {
        element(
            "a",
            vec![("href", hast::PropertyValue::String(href.into()))],
            vec![],
        )
    };

    assert_eq!(
        to_html_with_options(&link("a b/é?c=1&d"), &Options::default()),
        "<a href=\"a b/é?c=1&amp;d\"></a>",
        "should encode a URL without normalizing it again"
    );
    for href in ["HTTPS://a", "a/b:c", "?a:b", "#a:b", "/wiki/Help:Contents"] {
        assert_eq!(
            to_html_with_options(&link(href), &Options::default()),
            format!("<a href=\"{}\"></a>", href),
            "should keep a safe protocol, or a colon after `/`, `?`, or `#`: {:?}",
            href
        );
    }
    for href in [
        " javascript:alert(1)",
        "java\tscript:alert(1)",
        "JAVASCRIPT:alert(1)",
    ] {
        assert_eq!(
            to_html_with_options(&link(href), &Options::default()),
            "<a href=\"\"></a>",
            "should drop an unsafe protocol in a URL that a transform added: {:?}",
            href
        );
    }
}

#[test]
fn defaults_to_safe_options() {
    let node = element(
        "a",
        vec![(
            "href",
            hast::PropertyValue::String("javascript:alert(1)".into()),
        )],
        vec![],
    );

    assert_eq!(
        to_html(&node),
        to_html_with_options(&node, &Options::default()),
        "should use the default options"
    );
}

#[test]
fn encodes_raw_unless_allowed() {
    let raw = hast::Node::Raw(hast::Raw {
        value: "<b>\0</b>".into(),
        position: None,
    });

    assert_eq!(
        to_html(&raw),
        "&lt;b&gt;\u{FFFD}&lt;/b&gt;",
        "should encode raw HTML by default"
    );
    assert_eq!(
        to_html_with_options(
            &raw,
            &Options {
                allow_dangerous_html: true,
                ..Options::default()
            }
        ),
        "<b>\u{FFFD}</b>",
        "should write raw HTML when allowed, with NUL replaced"
    );
}

#[test]
fn checks_the_protocol_of_src() {
    let image = |src: &'static str| {
        element(
            "img",
            vec![("src", hast::PropertyValue::String(src.into()))],
            vec![],
        )
    };

    assert_eq!(
        to_html(&image("https://a")),
        "<img src=\"https://a\" />",
        "should keep a safe protocol in `src`"
    );
    assert_eq!(
        to_html(&image("mailto:a")),
        "<img src=\"\" />",
        "should drop a protocol that `href` allows but `src` does not"
    );
}

#[test]
fn checks_protocols_without_regard_to_case() {
    for (name, value) in [("HREF", "javascript:alert(1)"), ("Src", "data:x")] {
        assert_eq!(
            to_html(&element(
                "a",
                vec![(name, hast::PropertyValue::String(value.into()))],
                vec![]
            )),
            format!("<a {}=\"\"></a>", name),
            "should check {:?}, which browsers read as a URL property",
            name
        );
    }
}

#[test]
fn writes_script_and_style_text_as_is() {
    assert_eq!(
        to_html(&element("style", vec![], vec![text("a > b {}")])),
        "<style>a > b {}</style>",
        "should write text in `style` as is, like `hast-util-to-html`"
    );
    assert_eq!(
        to_html(&element("script", vec![], vec![text("a && b")])),
        "<script>a && b</script>",
        "should write text in `script` as is"
    );
    assert_eq!(
        to_html(&element("p", vec![], vec![text("a > b")])),
        "<p>a &gt; b</p>",
        "should encode text in other elements"
    );
}

#[test]
fn maps_and_escapes_attribute_names() {
    for (name, expected) in [
        ("ariaDescribedBy", "aria-describedby"),
        ("aria-label", "aria-label"),
        ("dataFootnoteRef", "data-footnote-ref"),
        ("data-x", "data-x"),
        ("httpEquiv", "http-equiv"),
        ("acceptCharset", "accept-charset"),
        ("x onclick", "x&#x20;onclick"),
        ("a\"=<>'/&", "a&#x22;&#x3D;&#x3C;&#x3E;&#x27;&#x2F;&#x26;"),
    ] {
        assert_eq!(
            to_html(&element(
                "p",
                vec![(name, hast::PropertyValue::Boolean(true))],
                vec![]
            )),
            format!("<p {}=\"\"></p>", expected),
            "should write the property name {:?} as {:?}",
            name,
            expected
        );
    }
}
