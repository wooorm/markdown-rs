use hast_util_to_html::{to_html_with_options as hast_to_html, Options};
use markdown::{mdast, to_html, to_html_with_options, to_mdast, CompileOptions, ParseOptions};
use mdast_util_to_hast::to_hast;
use pretty_assertions::assert_eq;
use std::borrow::Cow;

#[test]
fn matches_to_html_for_commonmark_basics() {
    for input in [
        "# a *b*",
        "a\nb",
        "* a\n* b",
        "1. a\n\n2. b",
        "> a",
        "![a](b \"c\")",
        "[a](<b c> 'd')",
        "a  \nb",
        "`a`",
        "```js\na\n```",
        "```\n```",
        "***",
        "<div>",
        "[a](b?c=1&d=2)",
        "![a](b&c)",
        "[a](javascript:alert(1))",
        "![a](data:x)",
        "a\u{0}b",
    ] {
        let mdast = to_mdast(input, &ParseOptions::default()).unwrap();
        assert_eq!(
            hast_to_html(&to_hast(&mdast), &Options::default()),
            to_html(input),
            "should match `to_html` for {:?}",
            input
        );
    }
}

#[test]
fn matches_to_html_when_dangerous_output_is_allowed() {
    let options = markdown::Options {
        compile: CompileOptions {
            allow_dangerous_html: true,
            allow_dangerous_protocol: true,
            ..CompileOptions::default()
        },
        ..markdown::Options::default()
    };

    for input in [
        "<div>\u{0}",
        "[a](javascript:alert(1))",
        "![a](data:x)",
        "a <b>c</b>",
    ] {
        let mdast = to_mdast(input, &ParseOptions::default()).unwrap();
        assert_eq!(
            hast_to_html(
                &to_hast(&mdast),
                &Options {
                    allow_dangerous_html: true,
                    allow_dangerous_protocol: true,
                }
            ),
            to_html_with_options(input, &options).unwrap(),
            "should match `to_html_with_options` for {:?}",
            input
        );
    }
}

#[test]
fn matches_to_html_for_gfm() {
    for input in [
        "a[^b]\n\n[^b]: c",
        "a[^b] d[^b]\n\n[^b]: c",
        "| a |\n| - |\n| b |",
        "~~a~~",
        "www.a.com",
    ] {
        let mdast = to_mdast(input, &ParseOptions::gfm()).unwrap();
        assert_eq!(
            hast_to_html(&to_hast(&mdast), &Options::default()),
            to_html_with_options(input, &markdown::Options::gfm()).unwrap(),
            "should match `to_html_with_options` with GFM for {:?}",
            input
        );
    }
}

#[test]
fn skips_footnote_calls_without_a_definition() {
    let mdast = mdast::Node::Paragraph(mdast::Paragraph {
        children: vec![mdast::Node::FootnoteReference(mdast::FootnoteReference {
            position: None,
            identifier: "a".into(),
            label: None,
        })],
        position: None,
    });

    assert_eq!(
        hast_to_html(&to_hast(&mdast), &Options::default()),
        "<p><sup><a href=\"#user-content-fn-a\" id=\"user-content-fnref-a\" data-footnote-ref=\"\" aria-describedby=\"footnote-label\">1</a></sup></p>",
        "should leave out the footer when no call has a definition"
    );
    assert!(
        matches!(to_hast(&mdast), hast::Node::Element(_)),
        "should return a single node as is without a footer"
    );

    let mdast = mdast::Node::Root(mdast::Root {
        children: vec![
            mdast::Node::Paragraph(mdast::Paragraph {
                children: vec![
                    mdast::Node::FootnoteReference(mdast::FootnoteReference {
                        position: None,
                        identifier: "x".into(),
                        label: None,
                    }),
                    mdast::Node::FootnoteReference(mdast::FootnoteReference {
                        position: None,
                        identifier: "a".into(),
                        label: None,
                    }),
                ],
                position: None,
            }),
            mdast::Node::FootnoteDefinition(mdast::FootnoteDefinition {
                children: vec![mdast::Node::Paragraph(mdast::Paragraph {
                    children: vec![mdast::Node::Text(mdast::Text {
                        value: "b".into(),
                        position: None,
                    })],
                    position: None,
                })],
                identifier: "a".into(),
                label: None,
                position: None,
            }),
        ],
        position: None,
    });
    let html = hast_to_html(&to_hast(&mdast), &Options::default());

    assert!(
        html.contains("<li id=\"user-content-fn-a\">"),
        "should keep calls with a definition after one without: {}",
        html
    );
    assert!(
        !html.contains("<li id=\"user-content-fn-x\">"),
        "should leave out calls without a definition: {}",
        html
    );
}

#[test]
fn borrows_constant_strings() {
    let mdast = to_mdast("a\n\n[b](c)", &ParseOptions::default()).unwrap();
    let hast = to_hast(&mdast);
    let children = match &hast {
        hast::Node::Root(root) => &root.children,
        _ => panic!("expected root"),
    };
    let (first, between, second) = match &children[..] {
        [hast::Node::Element(first), hast::Node::Text(between), hast::Node::Element(second)] => {
            (first, between, second)
        }
        _ => panic!("expected two paragraphs with a line ending between them"),
    };
    let link = match &second.children[..] {
        [hast::Node::Element(link)] => link,
        _ => panic!("expected a link"),
    };

    assert!(
        matches!(first.tag_name, Cow::Borrowed("p")),
        "should borrow tag names"
    );
    assert!(
        matches!(link.properties[0].0, Cow::Borrowed("href")),
        "should borrow property names"
    );
    assert!(
        matches!(between.value, Cow::Borrowed("\n")),
        "should borrow line endings between blocks"
    );
}
