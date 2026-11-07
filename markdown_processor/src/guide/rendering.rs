//! Recipe 9: one construct, rendered three ways.
//!
//! [`markdown::to_html_with_options`] writes a match with
//! [`Construct::html`][markdown::extension::Construct::html].
//! A [`Processor`][crate::Processor] does not: it turns the node from
//! [`Construct::to_mdast`][markdown::extension::Construct::to_mdast] into
//! hast, in one of two ways:
//!
//! * a [`Custom`][markdown::mdast::Custom] node with `h_name` is that
//!   element, with its `attributes` as properties, and its `value` as text
//!   or else its children inside
//! * a hast handler, added with
//!   [`add_hast_handler`][crate::Processor::add_hast_handler] for the
//!   node's `name`, turns it into any hast, and wins over `h_name`
//!
//! So a construct that is used both with `to_html` and in a processor
//! writes its HTML twice, and tests that both agree, as micromark
//! extensions and mdast utilities do.
//! The mention here is short; recipe 1 in
//! [`markdown::extension::guide::recipes::mention`] is the whole one.
//!
//! ```
//! use markdown::{
//!     extension::{
//!         construct, Construct, Ctx, HtmlEvent, HtmlWriter, Reach, Step, Text, Token, TokenType,
//!     },
//!     mdast::{Custom, Node},
//!     to_html_with_options, Options, ParseOptions,
//! };
//! use markdown_processor::Processor;
//!
//! #[derive(Clone, Copy, Debug, Eq, PartialEq)]
//! enum MentionToken {
//!     Mention,
//!     Name,
//! }
//!
//! impl TokenType for MentionToken {
//!     const ALL: &'static [Self] = &[Self::Mention, Self::Name];
//!
//!     fn name(self) -> &'static str {
//!         match self {
//!             Self::Mention => "mention",
//!             Self::Name => "mentionName",
//!         }
//!     }
//! }
//!
//! /// `@` and a name of ASCII letters and digits.
//! struct Mention;
//!
//! impl Construct for Mention {
//!     type Place = Text;
//!     type State = u8;
//!     type Token = MentionToken;
//!
//!     fn markers(&self) -> &[u8] {
//!         b"@"
//!     }
//!
//!     fn start(&self) -> u8 {
//!         0
//!     }
//!
//!     fn reach(&self) -> Reach {
//!         Reach::Bytes(40)
//!     }
//!
//!     fn step(&self, state: u8, t: Ctx<'_, '_, Self>) -> Step<Self> {
//!         match (state, t.current()) {
//!             (0, Some(b'@')) => t.enter(MentionToken::Mention).consume().next(1),
//!             (1, Some(byte)) if byte.is_ascii_alphanumeric() => {
//!                 t.enter(MentionToken::Name).consume().next(2)
//!             }
//!             (2, Some(byte)) if byte.is_ascii_alphanumeric() => t.consume().next(2),
//!             (2, _) => t.exit().exit().ok(),
//!             _ => t.nok(),
//!         }
//!     }
//!
//!     // The second way: an element, `<a href="…">@name</a>`.
//!     fn to_mdast(&self, tokens: Vec<Token<'_, MentionToken>>) -> Node {
//!         let name = &tokens[1].value;
//!         Node::Custom(Custom {
//!             name: "mention".into(),
//!             value: Some(format!("@{}", name)),
//!             h_name: Some("a".into()),
//!             attributes: vec![("href".into(), format!("https://github.com/{}", name))]
//!                 .into_iter()
//!                 .collect(),
//!             ..Custom::default()
//!         })
//!     }
//!
//!     // The first way: `to_html`.
//!     fn html(&self, event: HtmlEvent<'_, MentionToken>, _: &[Token<'_, MentionToken>], w: &mut HtmlWriter<'_>) {
//!         match event {
//!             HtmlEvent::Enter(token) if token.name == MentionToken::Mention => {
//!                 let name = &token.value[1..];
//!                 w.tag("<a");
//!                 w.attr("href", &format!("https://github.com/{}", name));
//!                 w.tag(">");
//!             }
//!             HtmlEvent::Text(_, value) => w.text(value),
//!             HtmlEvent::Exit(token) if token.name == MentionToken::Mention => w.tag("</a>"),
//!             _ => {}
//!         }
//!     }
//! }
//!
//! let expected = "<p>Hi <a href=\"https://github.com/octocat\">@octocat</a></p>";
//!
//! // `to_html`.
//! let options = Options {
//!     parse: ParseOptions {
//!         text_constructs: vec![construct(Mention)],
//!         ..ParseOptions::default()
//!     },
//!     ..Options::default()
//! };
//! assert_eq!(to_html_with_options("Hi @octocat", &options).unwrap(), expected);
//!
//! // `h_name` in a processor.
//! let processor = Processor::new().plugin(|processor: &mut Processor| {
//!     processor.add_text_construct(Mention);
//! });
//! assert_eq!(processor.process("Hi @octocat").unwrap(), expected);
//!
//! // A hast handler in a processor, here with a class too.
//! let processor = processor.plugin(|processor: &mut Processor| {
//!     processor.add_hast_handler("mention", |node, _| {
//!         vec![hast::Node::Element(hast::Element {
//!             tag_name: "a".into(),
//!             properties: vec![
//!                 ("href".into(), hast::PropertyValue::String(node.attributes["href"].clone().into())),
//!                 ("className".into(), hast::PropertyValue::SpaceSeparated(vec!["mention".into()])),
//!             ],
//!             children: vec![hast::Node::Text(hast::Text {
//!                 value: node.value.clone().unwrap_or_default().into(),
//!                 position: None,
//!             })],
//!             position: None,
//!         })]
//!     });
//! });
//! assert_eq!(
//!     processor.process("Hi @octocat").unwrap(),
//!     "<p>Hi <a href=\"https://github.com/octocat\" class=\"mention\">@octocat</a></p>"
//! );
//! ```
