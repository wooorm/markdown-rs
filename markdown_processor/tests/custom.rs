use markdown::{
    extension::{
        construct, Construct, Ctx, HtmlEvent, HtmlWriter, Step, Text, Token, TokenType, Url,
    },
    mdast,
};
use markdown_processor::{visit::visit_mut, Processor};
use pretty_assertions::assert_eq;
use std::borrow::Cow;

/// Plugin that turns emphasis into a custom `shout` node.
fn emphasis_to_shout(processor: &mut Processor) {
    processor.add_mdast_transform(|tree| {
        visit_mut(tree, &mut |node| {
            if let mdast::Node::Emphasis(emphasis) = node {
                *node = mdast::Node::Custom(mdast::Custom {
                    name: "shout".into(),
                    fields: vec![("level".into(), "2".into())].into_iter().collect(),
                    children: emphasis.children.clone(),
                    ..mdast::Custom::default()
                });
            }
        });
        Ok(())
    });
}

fn element(tag_name: impl Into<Cow<'static, str>>, children: Vec<hast::Node>) -> hast::Node {
    hast::Node::Element(hast::Element {
        tag_name: tag_name.into(),
        properties: vec![],
        children,
        position: None,
    })
}

#[test]
fn handler_turns_custom_node_into_hast() {
    let processor =
        Processor::new()
            .plugin(emphasis_to_shout)
            .plugin(|processor: &mut Processor| {
                processor.add_hast_handler("shout", |node, children| {
                    let level = node.fields.get("level").unwrap();
                    vec![element(["h", level].concat(), children)]
                });
            });

    assert_eq!(
        processor.process("a *b*").unwrap(),
        "<p>a <h2>b</h2></p>",
        "should pass the node and its converted children to the handler"
    );
}

#[test]
fn later_handler_replaces_earlier() {
    let processor =
        Processor::new()
            .plugin(emphasis_to_shout)
            .plugin(|processor: &mut Processor| {
                processor.add_hast_handler("shout", |_, children| vec![element("b", children)]);
                processor.add_hast_handler("shout", |_, children| vec![element("i", children)]);
            });

    assert_eq!(
        processor.process("*a*").unwrap(),
        "<p><i>a</i></p>",
        "should use the last handler added for a name"
    );
}

#[test]
fn unhandled_custom_parent_becomes_div() {
    assert_eq!(
        Processor::new()
            .plugin(emphasis_to_shout)
            .process("*a*")
            .unwrap(),
        "<p><div>a</div></p>",
        "should fall back to a `div` of the children, like mdast-util-to-hast"
    );
}

#[test]
fn unhandled_custom_literal_becomes_text() {
    let processor = Processor::new().plugin(|processor: &mut Processor| {
        processor.add_mdast_transform(|tree| {
            if let Some(children) = tree.children_mut() {
                children.push(mdast::Node::Custom(mdast::Custom {
                    name: "x".into(),
                    value: Some("<b>".into()),
                    ..mdast::Custom::default()
                }));
            }
            Ok(())
        });
    });

    assert_eq!(
        processor.process("").unwrap(),
        "&lt;b&gt;",
        "should fall back to text of the value"
    );
}

#[test]
fn custom_node_names_its_element() {
    let processor = Processor::new().plugin(|processor: &mut Processor| {
        processor.add_mdast_transform(|tree| {
            visit_mut(tree, &mut |node| {
                if let mdast::Node::Emphasis(emphasis) = node {
                    *node = mdast::Node::Custom(mdast::Custom {
                        name: "link".into(),
                        children: emphasis.children.clone(),
                        attributes: vec![
                            ("class".into(), "wiki".into()),
                            ("href".into(), "javascript:alert(1)".into()),
                        ]
                        .into_iter()
                        .collect(),
                        h_name: Some("a".into()),
                        ..mdast::Custom::default()
                    });
                }
            });
            Ok(())
        });
    });

    assert_eq!(
        processor.process("*a*").unwrap(),
        "<p><a class=\"wiki\" href=\"\">a</a></p>",
        "should render the element `h_name` names, checking the protocol of its URLs"
    );
}

/// Tokens of a link to a target.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum LinkTok {
    Link,
    Target,
}

impl TokenType for LinkTok {
    const ALL: &'static [Self] = &[Self::Link, Self::Target];

    fn name(self) -> &'static str {
        match self {
            Self::Link => "link",
            Self::Target => "linkTarget",
        }
    }
}

/// `[[`, a target without `]`, and `]]`: a link with a class, written by a
/// handler in `to_html`, and with `h_name` and `attributes` in mdast.
struct TargetLink;

impl Construct for TargetLink {
    type Place = Text;
    type State = u8;
    type Token = LinkTok;

    fn markers(&self) -> &[u8] {
        b"["
    }

    fn start(&self) -> u8 {
        0
    }

    fn step(&self, state: u8, t: Ctx<'_, '_, Self>) -> Step<Self> {
        match (state, t.current()) {
            (0, Some(b'[')) => t.enter(LinkTok::Link).consume().next(1),
            (1, Some(b'[')) => t.consume().next(2),
            (2, Some(b']') | None) => t.nok(),
            (2, Some(_)) => t.enter(LinkTok::Target).consume().next(3),
            (3, Some(b']')) => t.exit().consume().next(4),
            (3, Some(_)) => t.consume().next(3),
            (4, Some(b']')) => t.consume().exit().ok(),
            _ => t.nok(),
        }
    }

    fn to_mdast(&self, tokens: Vec<Token<'_, LinkTok>>) -> mdast::Node {
        let target = tokens[1].value.to_string();
        mdast::Node::Custom(mdast::Custom {
            name: "link".into(),
            h_name: Some("a".into()),
            attributes: vec![
                ("class".to_string(), "wiki".to_string()),
                ("href".to_string(), target.clone()),
            ]
            .into_iter()
            .collect(),
            value: Some(target),
            ..mdast::Custom::default()
        })
    }

    fn html(
        &self,
        event: HtmlEvent<'_, LinkTok>,
        _: &[Token<'_, LinkTok>],
        w: &mut HtmlWriter<'_>,
    ) {
        if let HtmlEvent::Text(LinkTok::Target, value) = event {
            w.tag("<a class=\"wiki\" href=\"");
            w.url(value, Url::Href);
            w.tag("\">");
            w.text(value);
            w.tag("</a>");
        }
    }
}

#[test]
fn writes_the_same_html_through_handlers_and_h_name() {
    let processor = Processor::new().plugin(|processor: &mut Processor| {
        processor.parse.text_constructs.push(construct(TargetLink));
    });
    let options = markdown::Options {
        parse: markdown::ParseOptions {
            text_constructs: vec![construct(TargetLink)],
            ..markdown::ParseOptions::default()
        },
        ..markdown::Options::default()
    };

    for input in [
        "a [[b]] c",
        "[[https://a.b?c#d]]",
        "[[javascript:alert(1)]]",
    ] {
        assert_eq!(
            processor.process(input).unwrap(),
            markdown::to_html_with_options(input, &options).unwrap(),
            "should write {:?} the same in both paths",
            input
        );
    }
}
