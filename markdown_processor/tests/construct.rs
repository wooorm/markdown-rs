use markdown::{
    extension::{Construct, Container, ContentType, Ctx, Flow, Step, Text, Token, TokenType},
    mdast,
};
use markdown_processor::Processor;
use pretty_assertions::assert_eq;
use std::borrow::Cow;

/// Tokens of the constructs here.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum Tok {
    Mention,
    MentionName,
    Caption,
    CaptionText,
    Aside,
    AsidePrefix,
    AsideContent,
}

impl TokenType for Tok {
    const ALL: &'static [Self] = &[
        Self::Mention,
        Self::MentionName,
        Self::Caption,
        Self::CaptionText,
        Self::Aside,
        Self::AsidePrefix,
        Self::AsideContent,
    ];

    fn name(self) -> &'static str {
        match self {
            Self::Mention => "mention",
            Self::MentionName => "mentionName",
            Self::Caption => "caption",
            Self::CaptionText => "captionText",
            Self::Aside => "aside",
            Self::AsidePrefix => "asidePrefix",
            Self::AsideContent => "asideContent",
        }
    }

    fn content(self) -> Option<ContentType> {
        match self {
            Self::MentionName | Self::CaptionText => Some(ContentType::Text),
            Self::AsideContent => Some(ContentType::Document),
            _ => None,
        }
    }
}

/// A custom node called `name`, with the children of the content in
/// `tokens`.
fn node(name: &str, tokens: Vec<Token<'_, Tok>>) -> mdast::Node {
    mdast::Node::Custom(mdast::Custom {
        name: name.into(),
        children: tokens
            .into_iter()
            .flat_map(|token| token.children)
            .collect(),
        ..mdast::Custom::default()
    })
}

/// `@` and a lowercase word, the word as text, as a node called `name`.
struct Mention(&'static str);

impl Construct for Mention {
    type Place = Text;
    type State = u8;
    type Token = Tok;

    fn markers(&self) -> &[u8] {
        b"@"
    }

    fn start(&self) -> u8 {
        0
    }

    fn step(&self, state: u8, t: Ctx<'_, '_, Self>) -> Step<Self> {
        match (state, t.current()) {
            (0, Some(b'@')) => t.enter(Tok::Mention).consume().next(1),
            (1, Some(b'a'..=b'z')) => t.enter(Tok::MentionName).consume().next(2),
            (2, Some(b'a'..=b'z')) => t.consume().next(2),
            (2, _) => t.exit().exit().ok(),
            _ => t.nok(),
        }
    }

    fn to_mdast(&self, tokens: Vec<Token<'_, Tok>>) -> mdast::Node {
        node(self.0, tokens)
    }
}

/// `%` and the rest of the line as text, as a node called `name`.
struct Caption(&'static str);

impl Construct for Caption {
    type Place = Flow;
    type State = u8;
    type Token = Tok;

    fn markers(&self) -> &[u8] {
        b"%"
    }

    fn start(&self) -> u8 {
        0
    }

    fn step(&self, state: u8, t: Ctx<'_, '_, Self>) -> Step<Self> {
        match (state, t.current()) {
            (0, Some(b'%')) => t.enter(Tok::Caption).consume().next(1),
            (1, Some(byte)) if byte != b'\n' => t.enter(Tok::CaptionText).consume().next(2),
            (2, None | Some(b'\n')) => t.exit().exit().ok(),
            (2, Some(_)) => t.consume().next(2),
            _ => t.nok(),
        }
    }

    fn to_mdast(&self, tokens: Vec<Token<'_, Tok>>) -> mdast::Node {
        node(self.0, tokens)
    }
}

/// `|` and an optional space before each line of a container, as a node
/// called `name`; without `continues`, it ends at its next line.
struct Aside {
    name: &'static str,
    continues: bool,
}

impl Construct for Aside {
    type Place = Container;
    type State = u8;
    type Token = Tok;

    fn markers(&self) -> &[u8] {
        b"|"
    }

    fn start(&self) -> u8 {
        0
    }

    fn step(&self, state: u8, t: Ctx<'_, '_, Self>) -> Step<Self> {
        match (state, t.current()) {
            (10, _) if !self.continues => t.nok(),
            (0 | 10, Some(b'|')) => {
                let t = if state == 0 { t.enter(Tok::Aside) } else { t };
                t.enter(Tok::AsidePrefix).consume().next(state + 1)
            }
            (1 | 11, Some(b' ')) => t.consume().exit().next(state + 1),
            (1 | 11, _) => t.exit().retry(state + 1),
            (2, _) => t.enter(Tok::AsideContent).container(10),
            (12, _) => t.container(10),
            _ => t.nok(),
        }
    }

    fn to_mdast(&self, tokens: Vec<Token<'_, Tok>>) -> mdast::Node {
        node(self.name, tokens)
    }
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
fn adds_constructs_to_text_flow_and_containers() {
    let processor = Processor::new().plugin(|processor: &mut Processor| {
        processor.add_text_construct(Mention("mention"));
        processor.add_flow_construct(Caption("caption"));
        processor.add_document_construct(Aside {
            name: "aside",
            continues: true,
        });
        processor.add_hast_handler("mention", |_, children| vec![element("b", children)]);
        processor.add_hast_handler("caption", |_, children| {
            vec![element("figcaption", children)]
        });
        processor.add_hast_handler("aside", |_, children| vec![element("aside", children)]);
    });

    assert_eq!(
        processor.process("%a @b\n\n| c\n| d").unwrap(),
        "<figcaption>a <b>b</b></figcaption>\n<aside><p>c\nd</p></aside>",
        "should parse each kind of construct and render its node"
    );
}

#[test]
fn tries_constructs_in_the_order_added() {
    let processor = Processor::new().plugin(|processor: &mut Processor| {
        processor.add_text_construct(Mention("first"));
        processor.add_text_construct(Mention("second"));
        processor.add_flow_construct(Caption("first"));
        processor.add_flow_construct(Caption("second"));
        processor.add_document_construct(Aside {
            name: "first",
            continues: false,
        });
        processor.add_document_construct(Aside {
            name: "second",
            continues: false,
        });
        processor.add_hast_handler("first", |_, children| vec![element("b", children)]);
        processor.add_hast_handler("second", |_, children| vec![element("i", children)]);
    });

    assert_eq!(
        processor.process("@a\n\n%b\n\n| c").unwrap(),
        "<p><b>a</b></p>\n<b>b</b>\n<b><p>c</p></b>",
        "should try each kind of construct in the order added"
    );
}
