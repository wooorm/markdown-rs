#![cfg(feature = "derive")]

use markdown as renamed;
use markdown::{
    extension::{construct, Construct, ContentType, Ctx, Step, Text, Token, TokenType},
    mdast::{Custom, Node},
    to_mdast, ParseOptions,
};
use pretty_assertions::assert_eq;

#[derive(Clone, Copy, Debug, Eq, PartialEq, TokenType)]
enum SpanToken {
    Span,
    #[token(name = "spanMarker")]
    Marker,
    #[token(content = text)]
    SpanLabel,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, TokenType)]
enum BodyToken {
    #[token(content = document)]
    Body,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, TokenType)]
#[token(crate = "renamed")]
enum Renamed {
    A,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, TokenType)]
enum Empty {}

#[test]
fn derives_token_types() {
    assert_eq!(
        SpanToken::ALL,
        &[SpanToken::Span, SpanToken::Marker, SpanToken::SpanLabel],
        "should list every variant, in order"
    );
    assert_eq!(
        SpanToken::ALL
            .iter()
            .map(|token| token.name())
            .collect::<Vec<_>>(),
        vec!["span", "spanMarker", "spanLabel"],
        "should name variants in camel case, or as given"
    );
    assert_eq!(
        SpanToken::ALL
            .iter()
            .map(|token| token.content())
            .collect::<Vec<_>>(),
        vec![None, None, Some(ContentType::Text)],
        "should mark text content"
    );
    assert_eq!(
        BodyToken::Body.content(),
        Some(ContentType::Document),
        "should mark document content"
    );
    assert_eq!(
        Renamed::A.name(),
        "a",
        "should use the path given for the crate"
    );
    assert_eq!(
        Empty::ALL,
        &[],
        "should derive for an enum without variants"
    );
}

/// `[`, a label as text, `]`.
struct Span;

impl Construct for Span {
    type Place = Text;
    type State = u8;
    type Token = SpanToken;

    fn markers(&self) -> &[u8] {
        b"["
    }

    fn start(&self) -> u8 {
        0
    }

    fn step(&self, state: u8, t: Ctx<'_, '_, Self>) -> Step<Self> {
        match (state, t.current()) {
            (0, Some(b'[')) => t
                .enter(SpanToken::Span)
                .enter(SpanToken::Marker)
                .consume()
                .exit()
                .next(1),
            (1, Some(b'a'..=b'z' | b'*')) => t.enter(SpanToken::SpanLabel).consume().next(2),
            (2, Some(b']')) => t
                .exit()
                .enter(SpanToken::Marker)
                .consume()
                .exit()
                .exit()
                .ok(),
            (2, Some(b'a'..=b'z' | b'*')) => t.consume().next(2),
            _ => t.nok(),
        }
    }

    fn to_mdast(&self, tokens: Vec<Token<'_, SpanToken>>) -> Node {
        Node::Custom(Custom {
            name: tokens[0].name.name().into(),
            children: tokens
                .into_iter()
                .filter(|token| token.name == SpanToken::SpanLabel)
                .flat_map(|token| token.children)
                .collect(),
            ..Custom::default()
        })
    }
}

#[test]
fn parses_with_derived_tokens() {
    let options = ParseOptions {
        text_constructs: vec![construct(Span)],
        ..ParseOptions::default()
    };
    let tree = to_mdast("[*a*]", &options).unwrap();

    assert!(
        matches!(
            &tree.children().unwrap()[0].children().unwrap()[0],
            Node::Custom(custom) if custom.name == "span" && matches!(custom.children[..], [Node::Emphasis(_)])
        ),
        "should parse content of a derived content token, got {:?}",
        tree
    );
}
