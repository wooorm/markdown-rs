//! Recipe 1: a mention, such as `@octo-cat`, as GitHub has it.
//!
//! A name is letters, digits, and single hyphens, which neither start nor
//! end it, of at most 39 bytes.
//! A mention starts only after whitespace or punctuation, which
//! [`previous`][crate::extension::Construct::previous] checks with
//! [`classify`][crate::extension::classify], so `me@example.com` is no
//! mention.
//!
//! A match reads the `@`, the name, and one byte after it, at most 41 bytes,
//! so [`Reach::Bytes`][crate::extension::Reach::Bytes] keeps parsing linear.
//! The state carries the length of the name so far, and whether its last
//! byte was a hyphen, so the construct checks both as it goes.
//!
//! ```
//! use markdown::{
//!     extension::{
//!         classify, construct, CharacterKind, Construct, Ctx, HtmlEvent, HtmlWriter, Reach,
//!         Step, Text, Token, TokenSlice, TokenType,
//!     },
//!     mdast::{Custom, Node},
//!     to_html_with_options, to_mdast, Options, ParseOptions,
//! };
//!
//! #[derive(Clone, Copy, Debug, Eq, PartialEq)]
//! enum MentionToken {
//!     Mention,
//!     Marker,
//!     Name,
//! }
//!
//! impl TokenType for MentionToken {
//!     const ALL: &'static [Self] = &[Self::Mention, Self::Marker, Self::Name];
//!
//!     fn name(self) -> &'static str {
//!         match self {
//!             Self::Mention => "mention",
//!             Self::Marker => "mentionMarker",
//!             Self::Name => "mentionName",
//!         }
//!     }
//! }
//!
//! #[derive(Clone, Copy, Debug)]
//! enum State {
//!     Start,
//!     NameStart,
//!     /// In the name, with its size, and whether it ends in a hyphen.
//!     Name(usize, bool),
//! }
//!
//! const NAME_MAX: usize = 39;
//!
//! struct Mention;
//!
//! impl Construct for Mention {
//!     type Place = Text;
//!     type State = State;
//!     type Token = MentionToken;
//!
//!     fn markers(&self) -> &[u8] {
//!         b"@"
//!     }
//!
//!     fn start(&self) -> State {
//!         State::Start
//!     }
//!
//!     fn previous(&self, previous: Option<char>) -> bool {
//!         previous.map_or(true, |char| classify(char) != CharacterKind::Other)
//!     }
//!
//!     fn reach(&self) -> Reach {
//!         Reach::Bytes(NAME_MAX + 2)
//!     }
//!
//!     fn step(&self, state: State, t: Ctx<'_, '_, Self>) -> Step<Self> {
//!         match (state, t.current()) {
//!             (State::Start, Some(b'@')) => t
//!                 .enter(MentionToken::Mention)
//!                 .enter(MentionToken::Marker)
//!                 .consume()
//!                 .exit()
//!                 .next(State::NameStart),
//!             (State::NameStart, Some(byte)) if byte.is_ascii_alphanumeric() => t
//!                 .enter(MentionToken::Name)
//!                 .consume()
//!                 .next(State::Name(1, false)),
//!             (State::Name(size, _), Some(byte))
//!                 if size < NAME_MAX && byte.is_ascii_alphanumeric() =>
//!             {
//!                 t.consume().next(State::Name(size + 1, false))
//!             }
//!             (State::Name(size, false), Some(b'-')) if size < NAME_MAX => {
//!                 t.consume().next(State::Name(size + 1, true))
//!             }
//!             // Too long, or two hyphens.
//!             (State::Name(..), Some(byte)) if byte.is_ascii_alphanumeric() || byte == b'-' => {
//!                 t.nok()
//!             }
//!             // A name does not end in a hyphen.
//!             (State::Name(_, false), _) => t.exit().exit().ok(),
//!             _ => t.nok(),
//!         }
//!     }
//!
//!     fn to_mdast(&self, tokens: Vec<Token<'_, MentionToken>>) -> Node {
//!         let name = tokens.value(MentionToken::Name).map(String::from);
//!         Node::Custom(Custom {
//!             name: "mention".into(),
//!             value: name,
//!             ..Custom::default()
//!         })
//!     }
//!
//!     fn html(
//!         &self,
//!         event: HtmlEvent<'_, MentionToken>,
//!         _: &[Token<'_, MentionToken>],
//!         w: &mut HtmlWriter<'_>,
//!     ) {
//!         if let HtmlEvent::Text(MentionToken::Name, name) = event {
//!             w.tag("<a");
//!             w.attr("href", &format!("https://github.com/{}", name));
//!             w.tag(">@");
//!             w.text(name);
//!             w.tag("</a>");
//!         }
//!     }
//! }
//!
//! let parse = ParseOptions {
//!     text_constructs: vec![construct(Mention)],
//!     ..ParseOptions::default()
//! };
//! let mention = |input: &str| to_mdast(input, &parse).unwrap().to_string();
//!
//! assert_eq!(mention("hi @octo-cat."), "hi octo-cat.");
//! assert_eq!(mention("me@example.com"), "me@example.com");
//! assert_eq!(mention("@octo- @-cat @octo--cat"), "@octo- @-cat @octo--cat");
//! let long = format!("@{}", "a".repeat(40));
//! assert_eq!(mention(&long), long);
//!
//! let options = Options {
//!     parse: ParseOptions {
//!         text_constructs: vec![construct(Mention)],
//!         ..ParseOptions::default()
//!     },
//!     ..Options::default()
//! };
//! assert_eq!(
//!     to_html_with_options("cc @octo-cat", &options)?,
//!     "<p>cc <a href=\"https://github.com/octo-cat\">@octo-cat</a></p>"
//! );
//! # Ok::<(), markdown::message::Message>(())
//! ```
