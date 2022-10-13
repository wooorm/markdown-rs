extern crate markdown;
mod test_utils;
use markdown::mdast;
use pretty_assertions::assert_eq;
use test_utils::{hast, mdast_util_to_hast::mdast_util_to_hast};

#[test]
fn mdast_util_to_hast_test() {
    assert_eq!(
        mdast_util_to_hast(&mdast::Node::BlockQuote(mdast::BlockQuote {
            children: vec![],
            position: None,
        })),
        hast::Node::Element(hast::Element {
            tag_name: "blockquote".into(),
            properties: vec![],
            children: vec![hast::Node::Text(hast::Text {
                value: "\n".into(),
                position: None
            })],
            position: None
        }),
        "should support a `BlockQuote`",
    );

    assert_eq!(
        mdast_util_to_hast(&mdast::Node::Break(mdast::Break { position: None })),
        hast::Node::Root(hast::Root {
            children: vec![
                hast::Node::Element(hast::Element {
                    tag_name: "br".into(),
                    properties: vec![],
                    children: vec![],
                    position: None
                }),
                hast::Node::Text(hast::Text {
                    value: "\n".into(),
                    position: None
                })
            ],
            position: None
        }),
        "should support a `Break`",
    );

    assert_eq!(
        mdast_util_to_hast(&mdast::Node::Code(mdast::Code {
            lang: Some("b".into()),
            meta: None,
            value: "a".into(),
            position: None,
        })),
        hast::Node::Element(hast::Element {
            tag_name: "pre".into(),
            properties: vec![],
            children: vec![hast::Node::Element(hast::Element {
                tag_name: "code".into(),
                properties: vec![(
                    "className".into(),
                    hast::PropertyValue::SpaceSeparated(vec!["language-b".into()]),
                ),],
                children: vec![hast::Node::Text(hast::Text {
                    value: "a\n".into(),
                    position: None
                })],
                position: None
            })],
            position: None
        }),
        "should support a `Code`",
    );

    assert_eq!(
        mdast_util_to_hast(&mdast::Node::Definition(mdast::Definition {
            url: "b".into(),
            title: None,
            identifier: "a".into(),
            label: None,
            position: None
        })),
        hast::Node::Root(hast::Root {
            children: vec![],
            position: None
        }),
        "should support a `Definition`",
    );

    assert_eq!(
        mdast_util_to_hast(&mdast::Node::Delete(mdast::Delete {
            children: vec![mdast::Node::Text(mdast::Text {
                value: "a".into(),
                position: None
            })],
            position: None,
        })),
        hast::Node::Element(hast::Element {
            tag_name: "del".into(),
            properties: vec![],
            children: vec![hast::Node::Text(hast::Text {
                value: "a".into(),
                position: None
            })],
            position: None
        }),
        "should support a `Delete`",
    );

    assert_eq!(
        mdast_util_to_hast(&mdast::Node::Emphasis(mdast::Emphasis {
            children: vec![mdast::Node::Text(mdast::Text {
                value: "a".into(),
                position: None
            })],
            position: None,
        })),
        hast::Node::Element(hast::Element {
            tag_name: "em".into(),
            properties: vec![],
            children: vec![hast::Node::Text(hast::Text {
                value: "a".into(),
                position: None
            })],
            position: None
        }),
        "should support an `Emphasis`",
    );

    assert_eq!(
        mdast_util_to_hast(&mdast::Node::FootnoteDefinition(
            mdast::FootnoteDefinition {
                identifier: "a".into(),
                label: None,
                children: vec![],
                position: None
            }
        )),
        hast::Node::Root(hast::Root {
            children: vec![],
            position: None
        }),
        "should support a `FootnoteDefinition`",
    );

    assert_eq!(
        mdast_util_to_hast(&mdast::Node::Root(mdast::Root {
            children: vec![
                mdast::Node::FootnoteDefinition(mdast::FootnoteDefinition {
                    children: vec![mdast::Node::Paragraph(mdast::Paragraph {
                        children: vec![mdast::Node::Text(mdast::Text {
                            value: "b".into(),
                            position: None
                        })],
                        position: None
                    }),],
                    identifier: "a".into(),
                    label: None,
                    position: None
                }),
                mdast::Node::Paragraph(mdast::Paragraph {
                    children: vec![mdast::Node::FootnoteReference(mdast::FootnoteReference {
                        identifier: "a".into(),
                        label: None,
                        position: None,
                    })],
                    position: None
                }),
            ],
            position: None,
        })),
        hast::Node::Root(hast::Root {
            children: vec![
                // Main.
                hast::Node::Element(hast::Element {
                    tag_name: "p".into(),
                    properties: vec![],
                    children: vec![hast::Node::Element(hast::Element {
                        tag_name: "sup".into(),
                        properties: vec![],
                        children: vec![hast::Node::Element(hast::Element {
                            tag_name: "a".into(),
                            properties: vec![
                                ("href".into(), hast::PropertyValue::String("#fn-a".into()),),
                                ("id".into(), hast::PropertyValue::String("fnref-a".into()),),
                                ("dataFootnoteRef".into(), hast::PropertyValue::Boolean(true),),
                                (
                                    "ariaDescribedBy".into(),
                                    hast::PropertyValue::String("footnote-label".into()),
                                )
                            ],
                            children: vec![hast::Node::Text(hast::Text {
                                value: "1".into(),
                                position: None
                            })],
                            position: None
                        }),],
                        position: None
                    }),],
                    position: None
                }),
                hast::Node::Text(hast::Text {
                    value: "\n".into(),
                    position: None
                }),
                // Footer.
                hast::Node::Element(hast::Element {
                    tag_name: "section".into(),
                    properties: vec![
                        ("dataFootnotes".into(), hast::PropertyValue::Boolean(true),),
                        (
                            "className".into(),
                            hast::PropertyValue::SpaceSeparated(vec!["footnotes".into()]),
                        ),
                    ],
                    children: vec![
                        hast::Node::Element(hast::Element {
                            tag_name: "h2".into(),
                            properties: vec![
                                (
                                    "id".into(),
                                    hast::PropertyValue::String("footnote-label".into()),
                                ),
                                (
                                    "className".into(),
                                    hast::PropertyValue::SpaceSeparated(vec!["sr-only".into(),]),
                                ),
                            ],
                            children: vec![hast::Node::Text(hast::Text {
                                value: "Footnotes".into(),
                                position: None
                            }),],
                            position: None
                        }),
                        hast::Node::Text(hast::Text {
                            value: "\n".into(),
                            position: None
                        }),
                        hast::Node::Element(hast::Element {
                            tag_name: "ol".into(),
                            properties: vec![],
                            children: vec![
                                hast::Node::Text(hast::Text {
                                    value: "\n".into(),
                                    position: None
                                }),
                                hast::Node::Element(hast::Element {
                                    tag_name: "li".into(),
                                    properties: vec![(
                                        "id".into(),
                                        hast::PropertyValue::String("#fn-a".into()),
                                    )],
                                    children: vec![
                                        hast::Node::Text(hast::Text {
                                            value: "\n".into(),
                                            position: None
                                        }),
                                        hast::Node::Element(hast::Element {
                                            tag_name: "p".into(),
                                            properties: vec![],
                                            children: vec![
                                                hast::Node::Text(hast::Text {
                                                    value: "b ".into(),
                                                    position: None
                                                }),
                                                hast::Node::Element(hast::Element {
                                                    tag_name: "a".into(),
                                                    properties: vec![
                                                        (
                                                            "href".into(),
                                                            hast::PropertyValue::String(
                                                                "#fnref-a".into()
                                                            ),
                                                        ),
                                                        (
                                                            "dataFootnoteBackref".into(),
                                                            hast::PropertyValue::Boolean(true),
                                                        ),
                                                        (
                                                            "ariaLabel".into(),
                                                            hast::PropertyValue::String(
                                                                "Back to content".into()
                                                            ),
                                                        ),
                                                        (
                                                            "className".into(),
                                                            hast::PropertyValue::SpaceSeparated(
                                                                vec!["data-footnote-backref".into()]
                                                            ),
                                                        )
                                                    ],
                                                    children: vec![hast::Node::Text(hast::Text {
                                                        value: "↩".into(),
                                                        position: None
                                                    }),],
                                                    position: None
                                                })
                                            ],
                                            position: None
                                        }),
                                        hast::Node::Text(hast::Text {
                                            value: "\n".into(),
                                            position: None
                                        }),
                                    ],
                                    position: None
                                }),
                                hast::Node::Text(hast::Text {
                                    value: "\n".into(),
                                    position: None
                                }),
                            ],
                            position: None
                        }),
                        hast::Node::Text(hast::Text {
                            value: "\n".into(),
                            position: None
                        }),
                    ],
                    position: None
                }),
                hast::Node::Text(hast::Text {
                    value: "\n".into(),
                    position: None
                }),
            ],
            position: None
        }),
        "should support an `FootnoteReference`",
    );

    assert_eq!(
        mdast_util_to_hast(&mdast::Node::Root(mdast::Root {
            children: vec![
                mdast::Node::FootnoteDefinition(mdast::FootnoteDefinition {
                    children: vec![mdast::Node::Paragraph(mdast::Paragraph {
                        children: vec![mdast::Node::Text(mdast::Text {
                            value: "b".into(),
                            position: None
                        })],
                        position: None
                    }),],
                    identifier: "a".into(),
                    label: None,
                    position: None
                }),
                mdast::Node::Paragraph(mdast::Paragraph {
                    children: vec![
                        mdast::Node::FootnoteReference(mdast::FootnoteReference {
                            identifier: "a".into(),
                            label: None,
                            position: None,
                        }),
                        mdast::Node::FootnoteReference(mdast::FootnoteReference {
                            identifier: "a".into(),
                            label: None,
                            position: None,
                        })
                    ],
                    position: None
                }),
            ],
            position: None,
        })),
        hast::Node::Root(hast::Root {
            children: vec![
                // Main.
                hast::Node::Element(hast::Element {
                    tag_name: "p".into(),
                    properties: vec![],
                    children: vec![
                        hast::Node::Element(hast::Element {
                            tag_name: "sup".into(),
                            properties: vec![],
                            children: vec![hast::Node::Element(hast::Element {
                                tag_name: "a".into(),
                                properties: vec![
                                    ("href".into(), hast::PropertyValue::String("#fn-a".into()),),
                                    ("id".into(), hast::PropertyValue::String("fnref-a".into()),),
                                    ("dataFootnoteRef".into(), hast::PropertyValue::Boolean(true),),
                                    (
                                        "ariaDescribedBy".into(),
                                        hast::PropertyValue::String("footnote-label".into()),
                                    )
                                ],
                                children: vec![hast::Node::Text(hast::Text {
                                    value: "1".into(),
                                    position: None
                                })],
                                position: None
                            }),],
                            position: None
                        }),
                        hast::Node::Element(hast::Element {
                            tag_name: "sup".into(),
                            properties: vec![],
                            children: vec![hast::Node::Element(hast::Element {
                                tag_name: "a".into(),
                                properties: vec![
                                    ("href".into(), hast::PropertyValue::String("#fn-a".into()),),
                                    ("id".into(), hast::PropertyValue::String("fnref-a-2".into()),),
                                    ("dataFootnoteRef".into(), hast::PropertyValue::Boolean(true),),
                                    (
                                        "ariaDescribedBy".into(),
                                        hast::PropertyValue::String("footnote-label".into()),
                                    )
                                ],
                                children: vec![hast::Node::Text(hast::Text {
                                    value: "1".into(),
                                    position: None
                                })],
                                position: None
                            }),],
                            position: None
                        }),
                    ],
                    position: None
                }),
                hast::Node::Text(hast::Text {
                    value: "\n".into(),
                    position: None
                }),
                // Footer.
                hast::Node::Element(hast::Element {
                    tag_name: "section".into(),
                    properties: vec![
                        ("dataFootnotes".into(), hast::PropertyValue::Boolean(true),),
                        (
                            "className".into(),
                            hast::PropertyValue::SpaceSeparated(vec!["footnotes".into()]),
                        ),
                    ],
                    children: vec![
                        hast::Node::Element(hast::Element {
                            tag_name: "h2".into(),
                            properties: vec![
                                (
                                    "id".into(),
                                    hast::PropertyValue::String("footnote-label".into()),
                                ),
                                (
                                    "className".into(),
                                    hast::PropertyValue::SpaceSeparated(vec!["sr-only".into(),]),
                                ),
                            ],
                            children: vec![hast::Node::Text(hast::Text {
                                value: "Footnotes".into(),
                                position: None
                            }),],
                            position: None
                        }),
                        hast::Node::Text(hast::Text {
                            value: "\n".into(),
                            position: None
                        }),
                        hast::Node::Element(hast::Element {
                            tag_name: "ol".into(),
                            properties: vec![],
                            children: vec![
                                hast::Node::Text(hast::Text {
                                    value: "\n".into(),
                                    position: None
                                }),
                                hast::Node::Element(hast::Element {
                                    tag_name: "li".into(),
                                    properties: vec![(
                                        "id".into(),
                                        hast::PropertyValue::String("#fn-a".into()),
                                    )],
                                    children: vec![
                                        hast::Node::Text(hast::Text {
                                            value: "\n".into(),
                                            position: None
                                        }),
                                        hast::Node::Element(hast::Element {
                                            tag_name: "p".into(),
                                            properties: vec![],
                                            children: vec![
                                                hast::Node::Text(hast::Text {
                                                    value: "b ".into(),
                                                    position: None
                                                }),
                                                hast::Node::Element(hast::Element {
                                                    tag_name: "a".into(),
                                                    properties: vec![
                                                        (
                                                            "href".into(),
                                                            hast::PropertyValue::String(
                                                                "#fnref-a".into()
                                                            ),
                                                        ),
                                                        (
                                                            "dataFootnoteBackref".into(),
                                                            hast::PropertyValue::Boolean(true),
                                                        ),
                                                        (
                                                            "ariaLabel".into(),
                                                            hast::PropertyValue::String(
                                                                "Back to content".into()
                                                            ),
                                                        ),
                                                        (
                                                            "className".into(),
                                                            hast::PropertyValue::SpaceSeparated(
                                                                vec!["data-footnote-backref".into()]
                                                            ),
                                                        )
                                                    ],
                                                    children: vec![hast::Node::Text(hast::Text {
                                                        value: "↩".into(),
                                                        position: None
                                                    }),],
                                                    position: None
                                                }),
                                                hast::Node::Text(hast::Text {
                                                    value: " ".into(),
                                                    position: None
                                                }),
                                                hast::Node::Element(hast::Element {
                                                    tag_name: "a".into(),
                                                    properties: vec![
                                                        (
                                                            "href".into(),
                                                            hast::PropertyValue::String(
                                                                "#fnref-a-2".into()
                                                            ),
                                                        ),
                                                        (
                                                            "dataFootnoteBackref".into(),
                                                            hast::PropertyValue::Boolean(true),
                                                        ),
                                                        (
                                                            "ariaLabel".into(),
                                                            hast::PropertyValue::String(
                                                                "Back to content".into()
                                                            ),
                                                        ),
                                                        (
                                                            "className".into(),
                                                            hast::PropertyValue::SpaceSeparated(
                                                                vec!["data-footnote-backref".into()]
                                                            ),
                                                        )
                                                    ],
                                                    children: vec![
                                                        hast::Node::Text(hast::Text {
                                                            value: "↩".into(),
                                                            position: None
                                                        }),
                                                        hast::Node::Element(hast::Element {
                                                            tag_name: "sup".into(),
                                                            properties: vec![],
                                                            children: vec![hast::Node::Text(
                                                                hast::Text {
                                                                    value: "2".into(),
                                                                    position: None
                                                                }
                                                            ),],
                                                            position: None
                                                        })
                                                    ],
                                                    position: None
                                                })
                                            ],
                                            position: None
                                        }),
                                        hast::Node::Text(hast::Text {
                                            value: "\n".into(),
                                            position: None
                                        }),
                                    ],
                                    position: None
                                }),
                                hast::Node::Text(hast::Text {
                                    value: "\n".into(),
                                    position: None
                                }),
                            ],
                            position: None
                        }),
                        hast::Node::Text(hast::Text {
                            value: "\n".into(),
                            position: None
                        }),
                    ],
                    position: None
                }),
                hast::Node::Text(hast::Text {
                    value: "\n".into(),
                    position: None
                }),
            ],
            position: None
        }),
        "should support an `FootnoteReference` (multiple calls to same definition)",
    );

    assert_eq!(
        mdast_util_to_hast(&mdast::Node::Heading(mdast::Heading {
            depth: 1,
            children: vec![mdast::Node::Text(mdast::Text {
                value: "a".into(),
                position: None
            })],
            position: None,
        })),
        hast::Node::Element(hast::Element {
            tag_name: "h1".into(),
            properties: vec![],
            children: vec![hast::Node::Text(hast::Text {
                value: "a".into(),
                position: None
            })],
            position: None
        }),
        "should support a `Heading`",
    );

    assert_eq!(
        mdast_util_to_hast(&mdast::Node::Html(mdast::Html {
            value: "<div>".into(),
            position: None,
        })),
        hast::Node::Root(hast::Root {
            children: vec![],
            position: None
        }),
        "should support an `Html`",
    );

    assert_eq!(
        mdast_util_to_hast(&mdast::Node::Image(mdast::Image {
            url: "a".into(),
            alt: "b".into(),
            title: None,
            position: None,
        })),
        hast::Node::Element(hast::Element {
            tag_name: "img".into(),
            properties: vec![
                ("src".into(), hast::PropertyValue::String("a".into()),),
                ("alt".into(), hast::PropertyValue::String("b".into()),)
            ],
            children: vec![],
            position: None
        }),
        "should support an `Image`",
    );

    assert_eq!(
        mdast_util_to_hast(&mdast::Node::Root(mdast::Root {
            children: vec![
                mdast::Node::Definition(mdast::Definition {
                    url: "b".into(),
                    title: None,
                    identifier: "a".into(),
                    label: None,
                    position: None
                }),
                mdast::Node::Paragraph(mdast::Paragraph {
                    children: vec![mdast::Node::ImageReference(mdast::ImageReference {
                        reference_kind: mdast::ReferenceKind::Full,
                        identifier: "a".into(),
                        alt: "c".into(),
                        label: None,
                        position: None,
                    })],
                    position: None
                }),
            ],
            position: None,
        })),
        hast::Node::Root(hast::Root {
            children: vec![hast::Node::Element(hast::Element {
                tag_name: "p".into(),
                properties: vec![],
                children: vec![hast::Node::Element(hast::Element {
                    tag_name: "img".into(),
                    properties: vec![
                        ("src".into(), hast::PropertyValue::String("b".into()),),
                        ("alt".into(), hast::PropertyValue::String("c".into()),)
                    ],
                    children: vec![],
                    position: None
                }),],
                position: None
            }),],
            position: None
        }),
        "should support an `ImageReference`",
    );

    assert_eq!(
        mdast_util_to_hast(&mdast::Node::InlineCode(mdast::InlineCode {
            value: "a\nb".into(),
            position: None,
        })),
        hast::Node::Element(hast::Element {
            tag_name: "code".into(),
            properties: vec![],
            children: vec![hast::Node::Text(hast::Text {
                value: "a b".into(),
                position: None
            })],
            position: None
        }),
        "should support an `InlineCode`",
    );

    assert_eq!(
        mdast_util_to_hast(&mdast::Node::InlineMath(mdast::InlineMath {
            value: "a\nb".into(),
            position: None,
        })),
        hast::Node::Element(hast::Element {
            tag_name: "code".into(),
            properties: vec![(
                "className".into(),
                hast::PropertyValue::SpaceSeparated(vec![
                    "language-math".into(),
                    "math-inline".into()
                ]),
            ),],
            children: vec![hast::Node::Text(hast::Text {
                value: "a b".into(),
                position: None
            })],
            position: None
        }),
        "should support an `InlineMath`",
    );

    assert_eq!(
        mdast_util_to_hast(&mdast::Node::Link(mdast::Link {
            url: "a".into(),
            title: None,
            children: vec![mdast::Node::Text(mdast::Text {
                value: "b".into(),
                position: None
            })],
            position: None,
        })),
        hast::Node::Element(hast::Element {
            tag_name: "a".into(),
            properties: vec![("href".into(), hast::PropertyValue::String("a".into()),),],
            children: vec![hast::Node::Text(hast::Text {
                value: "b".into(),
                position: None
            })],
            position: None
        }),
        "should support a `Link`",
    );

    assert_eq!(
        mdast_util_to_hast(&mdast::Node::Root(mdast::Root {
            children: vec![
                mdast::Node::Definition(mdast::Definition {
                    url: "b".into(),
                    title: None,
                    identifier: "a".into(),
                    label: None,
                    position: None
                }),
                mdast::Node::Paragraph(mdast::Paragraph {
                    children: vec![mdast::Node::LinkReference(mdast::LinkReference {
                        reference_kind: mdast::ReferenceKind::Full,
                        identifier: "a".into(),
                        label: None,
                        children: vec![mdast::Node::Text(mdast::Text {
                            value: "c".into(),
                            position: None
                        })],
                        position: None,
                    })],
                    position: None
                }),
            ],
            position: None,
        })),
        hast::Node::Root(hast::Root {
            children: vec![hast::Node::Element(hast::Element {
                tag_name: "p".into(),
                properties: vec![],
                children: vec![hast::Node::Element(hast::Element {
                    tag_name: "a".into(),
                    properties: vec![("href".into(), hast::PropertyValue::String("b".into()),),],
                    children: vec![hast::Node::Text(hast::Text {
                        value: "c".into(),
                        position: None
                    })],
                    position: None
                }),],
                position: None
            }),],
            position: None
        }),
        "should support a `LinkReference`",
    );

    assert_eq!(
        mdast_util_to_hast(&mdast::Node::Root(mdast::Root {
            children: vec![mdast::Node::ListItem(mdast::ListItem {
                spread: false,
                checked: None,
                children: vec![mdast::Node::Paragraph(mdast::Paragraph {
                    children: vec![mdast::Node::Text(mdast::Text {
                        value: "a".into(),
                        position: None
                    })],
                    position: None
                }),],
                position: None
            }),],
            position: None,
        })),
        hast::Node::Root(hast::Root {
            children: vec![hast::Node::Element(hast::Element {
                tag_name: "li".into(),
                properties: vec![],
                children: vec![hast::Node::Text(hast::Text {
                    value: "a".into(),
                    position: None
                }),],
                position: None
            }),],
            position: None
        }),
        "should support a `ListItem`",
    );

    assert_eq!(
        mdast_util_to_hast(&mdast::Node::Root(mdast::Root {
            children: vec![mdast::Node::ListItem(mdast::ListItem {
                spread: true,
                checked: None,
                children: vec![mdast::Node::Paragraph(mdast::Paragraph {
                    children: vec![mdast::Node::Text(mdast::Text {
                        value: "a".into(),
                        position: None
                    })],
                    position: None
                }),],
                position: None
            }),],
            position: None,
        })),
        hast::Node::Root(hast::Root {
            children: vec![hast::Node::Element(hast::Element {
                tag_name: "li".into(),
                properties: vec![],
                children: vec![
                    hast::Node::Text(hast::Text {
                        value: "\n".into(),
                        position: None
                    }),
                    hast::Node::Element(hast::Element {
                        tag_name: "p".into(),
                        properties: vec![],
                        children: vec![hast::Node::Text(hast::Text {
                            value: "a".into(),
                            position: None
                        }),],
                        position: None
                    }),
                    hast::Node::Text(hast::Text {
                        value: "\n".into(),
                        position: None
                    }),
                ],
                position: None
            }),],
            position: None
        }),
        "should support a `ListItem` (spread: true)",
    );

    assert_eq!(
        mdast_util_to_hast(&mdast::Node::Root(mdast::Root {
            children: vec![mdast::Node::ListItem(mdast::ListItem {
                spread: false,
                checked: Some(true),
                children: vec![],
                position: None
            }),],
            position: None,
        })),
        hast::Node::Root(hast::Root {
            children: vec![hast::Node::Element(hast::Element {
                tag_name: "li".into(),
                properties: vec![(
                    "className".into(),
                    hast::PropertyValue::SpaceSeparated(vec!["task-list-item".into()])
                )],
                children: vec![hast::Node::Element(hast::Element {
                    tag_name: "input".into(),
                    properties: vec![
                        (
                            "type".into(),
                            hast::PropertyValue::String("checkbox".into()),
                        ),
                        ("checked".into(), hast::PropertyValue::Boolean(true)),
                        ("disabled".into(), hast::PropertyValue::Boolean(true)),
                    ],
                    children: vec![],
                    position: None
                }),],
                position: None
            }),],
            position: None
        }),
        "should support a `ListItem` (checked, w/o paragraph)",
    );

    assert_eq!(
        mdast_util_to_hast(&mdast::Node::Root(mdast::Root {
            children: vec![mdast::Node::ListItem(mdast::ListItem {
                spread: false,
                checked: Some(false),
                children: vec![mdast::Node::Paragraph(mdast::Paragraph {
                    children: vec![mdast::Node::Text(mdast::Text {
                        value: "a".into(),
                        position: None
                    })],
                    position: None
                }),],
                position: None
            }),],
            position: None,
        })),
        hast::Node::Root(hast::Root {
            children: vec![hast::Node::Element(hast::Element {
                tag_name: "li".into(),
                properties: vec![(
                    "className".into(),
                    hast::PropertyValue::SpaceSeparated(vec!["task-list-item".into()])
                )],
                children: vec![
                    hast::Node::Element(hast::Element {
                        tag_name: "input".into(),
                        properties: vec![
                            (
                                "type".into(),
                                hast::PropertyValue::String("checkbox".into()),
                            ),
                            ("checked".into(), hast::PropertyValue::Boolean(false)),
                            ("disabled".into(), hast::PropertyValue::Boolean(true)),
                        ],
                        children: vec![],
                        position: None
                    }),
                    hast::Node::Text(hast::Text {
                        value: " ".into(),
                        position: None
                    }),
                    hast::Node::Text(hast::Text {
                        value: "a".into(),
                        position: None
                    }),
                ],
                position: None
            }),],
            position: None
        }),
        "should support a `ListItem` (unchecked, w/ paragraph)",
    );

    assert_eq!(
        mdast_util_to_hast(&mdast::Node::List(mdast::List {
            ordered: true,
            start: Some(1),
            spread: false,
            children: vec![mdast::Node::ListItem(mdast::ListItem {
                spread: false,
                checked: None,
                children: vec![mdast::Node::Paragraph(mdast::Paragraph {
                    children: vec![mdast::Node::Text(mdast::Text {
                        value: "a".into(),
                        position: None
                    })],
                    position: None
                }),],
                position: None
            }),],
            position: None,
        })),
        hast::Node::Element(hast::Element {
            tag_name: "ol".into(),
            properties: vec![],
            children: vec![
                hast::Node::Text(hast::Text {
                    value: "\n".into(),
                    position: None
                }),
                hast::Node::Element(hast::Element {
                    tag_name: "li".into(),
                    properties: vec![],
                    children: vec![hast::Node::Text(hast::Text {
                        value: "a".into(),
                        position: None
                    }),],
                    position: None
                }),
                hast::Node::Text(hast::Text {
                    value: "\n".into(),
                    position: None
                }),
            ],
            position: None
        }),
        "should support a `List` (ordered, start: 1)",
    );

    assert_eq!(
        mdast_util_to_hast(&mdast::Node::List(mdast::List {
            ordered: true,
            start: Some(123),
            spread: false,
            children: vec![],
            position: None,
        })),
        hast::Node::Element(hast::Element {
            tag_name: "ol".into(),
            properties: vec![("start".into(), hast::PropertyValue::String("123".into()),),],
            children: vec![hast::Node::Text(hast::Text {
                value: "\n".into(),
                position: None
            })],
            position: None
        }),
        "should support a `List` (ordered, start: 123)",
    );

    assert_eq!(
        mdast_util_to_hast(&mdast::Node::List(mdast::List {
            ordered: false,
            start: None,
            spread: false,
            children: vec![],
            position: None,
        })),
        hast::Node::Element(hast::Element {
            tag_name: "ul".into(),
            properties: vec![],
            children: vec![hast::Node::Text(hast::Text {
                value: "\n".into(),
                position: None
            })],
            position: None
        }),
        "should support a `List` (unordered)",
    );

    assert_eq!(
        mdast_util_to_hast(&mdast::Node::List(mdast::List {
            ordered: false,
            start: None,
            spread: false,
            children: vec![mdast::Node::ListItem(mdast::ListItem {
                spread: false,
                checked: Some(true),
                children: vec![],
                position: None
            }),],
            position: None,
        })),
        hast::Node::Element(hast::Element {
            tag_name: "ul".into(),
            properties: vec![(
                "className".into(),
                hast::PropertyValue::SpaceSeparated(vec!["contains-task-list".into()])
            )],
            children: vec![
                hast::Node::Text(hast::Text {
                    value: "\n".into(),
                    position: None
                }),
                hast::Node::Element(hast::Element {
                    tag_name: "li".into(),
                    properties: vec![(
                        "className".into(),
                        hast::PropertyValue::SpaceSeparated(vec!["task-list-item".into()])
                    )],
                    children: vec![hast::Node::Element(hast::Element {
                        tag_name: "input".into(),
                        properties: vec![
                            (
                                "type".into(),
                                hast::PropertyValue::String("checkbox".into()),
                            ),
                            ("checked".into(), hast::PropertyValue::Boolean(true)),
                            ("disabled".into(), hast::PropertyValue::Boolean(true)),
                        ],
                        children: vec![],
                        position: None
                    }),],
                    position: None
                }),
                hast::Node::Text(hast::Text {
                    value: "\n".into(),
                    position: None
                }),
            ],
            position: None
        }),
        "should support a `List` (w/ checked item)",
    );

    assert_eq!(
        mdast_util_to_hast(&mdast::Node::Math(mdast::Math {
            meta: None,
            value: "a".into(),
            position: None,
        })),
        hast::Node::Element(hast::Element {
            tag_name: "pre".into(),
            properties: vec![],
            children: vec![hast::Node::Element(hast::Element {
                tag_name: "code".into(),
                properties: vec![(
                    "className".into(),
                    hast::PropertyValue::SpaceSeparated(vec![
                        "language-math".into(),
                        "math-display".into()
                    ]),
                ),],
                children: vec![hast::Node::Text(hast::Text {
                    value: "a\n".into(),
                    position: None
                })],
                position: None
            })],
            position: None
        }),
        "should support a `Math`",
    );

    assert_eq!(
        mdast_util_to_hast(&mdast::Node::MdxFlowExpression(mdast::MdxFlowExpression {
            value: "a".into(),
            position: None,
            stops: vec![]
        })),
        hast::Node::MdxExpression(hast::MdxExpression {
            value: "a".into(),
            position: None,
            stops: vec![]
        }),
        "should support an `MdxFlowExpression`",
    );

    assert_eq!(
        mdast_util_to_hast(&mdast::Node::MdxTextExpression(mdast::MdxTextExpression {
            value: "a".into(),
            position: None,
            stops: vec![]
        })),
        hast::Node::MdxExpression(hast::MdxExpression {
            value: "a".into(),
            position: None,
            stops: vec![]
        }),
        "should support an `MdxTextExpression`",
    );

    assert_eq!(
        mdast_util_to_hast(&mdast::Node::MdxJsxFlowElement(mdast::MdxJsxFlowElement {
            name: None,
            attributes: vec![],
            children: vec![],
            position: None,
        })),
        hast::Node::MdxJsxElement(hast::MdxJsxElement {
            name: None,
            attributes: vec![],
            children: vec![],
            position: None,
        }),
        "should support an `MdxJsxFlowElement`",
    );

    assert_eq!(
        mdast_util_to_hast(&mdast::Node::MdxJsxTextElement(mdast::MdxJsxTextElement {
            name: None,
            attributes: vec![],
            children: vec![],
            position: None,
        })),
        hast::Node::MdxJsxElement(hast::MdxJsxElement {
            name: None,
            attributes: vec![],
            children: vec![],
            position: None,
        }),
        "should support an `MdxJsxTextElement`",
    );

    assert_eq!(
        mdast_util_to_hast(&mdast::Node::MdxjsEsm(mdast::MdxjsEsm {
            value: "a".into(),
            position: None,
            stops: vec![]
        })),
        hast::Node::MdxjsEsm(hast::MdxjsEsm {
            value: "a".into(),
            position: None,
            stops: vec![]
        }),
        "should support an `MdxjsEsm`",
    );

    assert_eq!(
        mdast_util_to_hast(&mdast::Node::Paragraph(mdast::Paragraph {
            children: vec![mdast::Node::Text(mdast::Text {
                value: "a".into(),
                position: None
            })],
            position: None,
        })),
        hast::Node::Element(hast::Element {
            tag_name: "p".into(),
            properties: vec![],
            children: vec![hast::Node::Text(hast::Text {
                value: "a".into(),
                position: None
            })],
            position: None
        }),
        "should support a `Paragraph`",
    );

    assert_eq!(
        mdast_util_to_hast(&mdast::Node::Root(mdast::Root {
            children: vec![],
            position: None,
        })),
        hast::Node::Root(hast::Root {
            children: vec![],
            position: None
        }),
        "should support a `Root`",
    );

    assert_eq!(
        mdast_util_to_hast(&mdast::Node::Strong(mdast::Strong {
            children: vec![mdast::Node::Text(mdast::Text {
                value: "a".into(),
                position: None
            })],
            position: None,
        })),
        hast::Node::Element(hast::Element {
            tag_name: "strong".into(),
            properties: vec![],
            children: vec![hast::Node::Text(hast::Text {
                value: "a".into(),
                position: None
            })],
            position: None
        }),
        "should support a `Strong`",
    );

    assert_eq!(
        mdast_util_to_hast(&mdast::Node::TableCell(mdast::TableCell {
            children: vec![mdast::Node::Text(mdast::Text {
                value: "a".into(),
                position: None
            })],
            position: None,
        })),
        hast::Node::Element(hast::Element {
            tag_name: "td".into(),
            properties: vec![],
            children: vec![hast::Node::Text(hast::Text {
                value: "a".into(),
                position: None
            })],
            position: None
        }),
        "should support a `TableCell`",
    );

    assert_eq!(
        mdast_util_to_hast(&mdast::Node::TableRow(mdast::TableRow {
            children: vec![
                mdast::Node::TableCell(mdast::TableCell {
                    children: vec![mdast::Node::Text(mdast::Text {
                        value: "a".into(),
                        position: None
                    })],
                    position: None,
                }),
                mdast::Node::TableCell(mdast::TableCell {
                    children: vec![mdast::Node::Text(mdast::Text {
                        value: "b".into(),
                        position: None
                    })],
                    position: None,
                })
            ],
            position: None,
        })),
        hast::Node::Element(hast::Element {
            tag_name: "tr".into(),
            properties: vec![],
            children: vec![
                hast::Node::Text(hast::Text {
                    value: "\n".into(),
                    position: None
                }),
                hast::Node::Element(hast::Element {
                    tag_name: "td".into(),
                    properties: vec![],
                    children: vec![hast::Node::Text(hast::Text {
                        value: "a".into(),
                        position: None
                    })],
                    position: None
                }),
                hast::Node::Text(hast::Text {
                    value: "\n".into(),
                    position: None
                }),
                hast::Node::Element(hast::Element {
                    tag_name: "td".into(),
                    properties: vec![],
                    children: vec![hast::Node::Text(hast::Text {
                        value: "b".into(),
                        position: None
                    })],
                    position: None
                }),
                hast::Node::Text(hast::Text {
                    value: "\n".into(),
                    position: None
                }),
            ],
            position: None
        }),
        "should support a `TableRow`",
    );

    assert_eq!(
        mdast_util_to_hast(&mdast::Node::Table(mdast::Table {
            align: vec![mdast::AlignKind::Left, mdast::AlignKind::None],
            children: vec![
                mdast::Node::TableRow(mdast::TableRow {
                    children: vec![
                        mdast::Node::TableCell(mdast::TableCell {
                            children: vec![mdast::Node::Text(mdast::Text {
                                value: "a".into(),
                                position: None
                            })],
                            position: None,
                        }),
                        mdast::Node::TableCell(mdast::TableCell {
                            children: vec![mdast::Node::Text(mdast::Text {
                                value: "b".into(),
                                position: None
                            })],
                            position: None,
                        })
                    ],
                    position: None,
                }),
                mdast::Node::TableRow(mdast::TableRow {
                    children: vec![
                        mdast::Node::TableCell(mdast::TableCell {
                            children: vec![mdast::Node::Text(mdast::Text {
                                value: "c".into(),
                                position: None
                            })],
                            position: None,
                        }),
                        mdast::Node::TableCell(mdast::TableCell {
                            children: vec![mdast::Node::Text(mdast::Text {
                                value: "d".into(),
                                position: None
                            })],
                            position: None,
                        })
                    ],
                    position: None,
                })
            ],
            position: None,
        })),
        hast::Node::Element(hast::Element {
            tag_name: "table".into(),
            properties: vec![],
            children: vec![
                hast::Node::Text(hast::Text {
                    value: "\n".into(),
                    position: None
                }),
                hast::Node::Element(hast::Element {
                    tag_name: "thead".into(),
                    properties: vec![],
                    children: vec![
                        hast::Node::Text(hast::Text {
                            value: "\n".into(),
                            position: None
                        }),
                        hast::Node::Element(hast::Element {
                            tag_name: "tr".into(),
                            properties: vec![],
                            children: vec![
                                hast::Node::Text(hast::Text {
                                    value: "\n".into(),
                                    position: None
                                }),
                                hast::Node::Element(hast::Element {
                                    tag_name: "th".into(),
                                    properties: vec![(
                                        "align".into(),
                                        hast::PropertyValue::String("left".into()),
                                    ),],
                                    children: vec![hast::Node::Text(hast::Text {
                                        value: "a".into(),
                                        position: None
                                    })],
                                    position: None
                                }),
                                hast::Node::Text(hast::Text {
                                    value: "\n".into(),
                                    position: None
                                }),
                                hast::Node::Element(hast::Element {
                                    tag_name: "th".into(),
                                    properties: vec![],
                                    children: vec![hast::Node::Text(hast::Text {
                                        value: "b".into(),
                                        position: None
                                    })],
                                    position: None
                                }),
                                hast::Node::Text(hast::Text {
                                    value: "\n".into(),
                                    position: None
                                }),
                            ],
                            position: None
                        }),
                        hast::Node::Text(hast::Text {
                            value: "\n".into(),
                            position: None
                        }),
                    ],
                    position: None
                }),
                hast::Node::Text(hast::Text {
                    value: "\n".into(),
                    position: None
                }),
                hast::Node::Element(hast::Element {
                    tag_name: "tbody".into(),
                    properties: vec![],
                    children: vec![
                        hast::Node::Text(hast::Text {
                            value: "\n".into(),
                            position: None
                        }),
                        hast::Node::Element(hast::Element {
                            tag_name: "tr".into(),
                            properties: vec![],
                            children: vec![
                                hast::Node::Text(hast::Text {
                                    value: "\n".into(),
                                    position: None
                                }),
                                hast::Node::Element(hast::Element {
                                    tag_name: "td".into(),
                                    properties: vec![(
                                        "align".into(),
                                        hast::PropertyValue::String("left".into()),
                                    ),],
                                    children: vec![hast::Node::Text(hast::Text {
                                        value: "c".into(),
                                        position: None
                                    })],
                                    position: None
                                }),
                                hast::Node::Text(hast::Text {
                                    value: "\n".into(),
                                    position: None
                                }),
                                hast::Node::Element(hast::Element {
                                    tag_name: "td".into(),
                                    properties: vec![],
                                    children: vec![hast::Node::Text(hast::Text {
                                        value: "d".into(),
                                        position: None
                                    })],
                                    position: None
                                }),
                                hast::Node::Text(hast::Text {
                                    value: "\n".into(),
                                    position: None
                                }),
                            ],
                            position: None
                        }),
                        hast::Node::Text(hast::Text {
                            value: "\n".into(),
                            position: None
                        }),
                    ],
                    position: None
                }),
                hast::Node::Text(hast::Text {
                    value: "\n".into(),
                    position: None
                }),
            ],
            position: None
        }),
        "should support a `Table`",
    );

    assert_eq!(
        mdast_util_to_hast(&mdast::Node::Text(mdast::Text {
            value: "a".into(),
            position: None,
        })),
        hast::Node::Text(hast::Text {
            value: "a".into(),
            position: None
        }),
        "should support a `Text`",
    );

    assert_eq!(
        mdast_util_to_hast(&mdast::Node::ThematicBreak(mdast::ThematicBreak {
            position: None
        })),
        hast::Node::Element(hast::Element {
            tag_name: "hr".into(),
            properties: vec![],
            children: vec![],
            position: None
        }),
        "should support a `Thematicbreak`",
    );

    assert_eq!(
        mdast_util_to_hast(&mdast::Node::Yaml(mdast::Yaml {
            value: "a".into(),
            position: None
        })),
        hast::Node::Root(hast::Root {
            children: vec![],
            position: None
        }),
        "should support a `Yaml`",
    );

    assert_eq!(
        mdast_util_to_hast(&mdast::Node::Toml(mdast::Toml {
            value: "a".into(),
            position: None
        })),
        hast::Node::Root(hast::Root {
            children: vec![],
            position: None
        }),
        "should support a `Toml`",
    );
}
