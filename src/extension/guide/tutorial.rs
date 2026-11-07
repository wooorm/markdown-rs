//! Tutorial: a mention, `@alice`, built step by step.
//!
//! ## 1. Tokens, states, and steps
//!
//! A construct names its tokens with a type of its own, which implements
//! [`TokenType`][crate::extension::TokenType].
//! With the `derive` feature, `#[derive(TokenType)]` writes that for an enum;
//! here it is written by hand.
//! `ALL` lists every token, and `name` gives the name that events and debug
//! output show.
//!
//! Its states are another type of its own: here, before the `@`, at the
//! start of the name, and in the name.
//!
//! [`step`][crate::extension::Construct::step] gets a state and a
//! [`Ctx`][crate::extension::Ctx], and returns a
//! [`Step`][crate::extension::Step].
//! Each method of `Ctx` takes it by value, so a step reads as one chain:
//! enter tokens, consume the current byte, exit tokens, then go to the next
//! state.
//! Only [`consume`][crate::extension::Ctx::consume] gives
//! [`next`][crate::extension::Consumed::next], so a step that goes on has
//! consumed exactly one byte; one that does not consume uses
//! [`retry`][crate::extension::Ctx::retry].
//! A match ends with [`ok`][crate::extension::Ctx::ok], or fails with
//! [`nok`][crate::extension::Ctx::nok], and then its bytes are what they
//! would be without it.
//!
//! [`to_mdast`][crate::extension::Construct::to_mdast] turns the tokens of a
//! match into a node, usually a [`Custom`][crate::mdast::Custom] one.
//!
//! ```
//! use markdown::{
//!     extension::{construct, Construct, Ctx, Step, Text, Token, TokenType},
//!     mdast::{Custom, Node},
//!     to_mdast, ParseOptions,
//! };
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
//! #[derive(Clone, Copy, Debug)]
//! enum State {
//!     Start,
//!     NameStart,
//!     Name,
//! }
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
//!     fn step(&self, state: State, t: Ctx<'_, '_, Self>) -> Step<Self> {
//!         match (state, t.current()) {
//!             (State::Start, Some(b'@')) => t
//!                 .enter(MentionToken::Mention)
//!                 .consume()
//!                 .next(State::NameStart),
//!             (State::NameStart, Some(b'a'..=b'z')) => {
//!                 t.enter(MentionToken::Name).consume().next(State::Name)
//!             }
//!             (State::Name, Some(b'a'..=b'z')) => t.consume().next(State::Name),
//!             // The name ends before any other byte: close both tokens.
//!             (State::Name, _) => t.exit().exit().ok(),
//!             _ => t.nok(),
//!         }
//!     }
//!
//!     fn to_mdast(&self, tokens: Vec<Token<'_, MentionToken>>) -> Node {
//!         Node::Custom(Custom {
//!             name: "mention".into(),
//!             value: Some(tokens[1].value.to_string()),
//!             ..Custom::default()
//!         })
//!     }
//! }
//!
//! let options = ParseOptions {
//!     text_constructs: vec![construct(Mention)],
//!     ..ParseOptions::default()
//! };
//! let tree = to_mdast("hi @alice, and @1", &options)?;
//!
//! // `ToString` prefers the value of a custom node; `@1` is no mention.
//! assert_eq!(tree.to_string(), "hi alice, and @1");
//! # Ok::<(), markdown::message::Message>(())
//! ```
//!
//! The types check what they can: a step that forgets to consume, consumes
//! twice, or gives a token of another type does not compile.
//! What depends on the input, such as consuming outside a token, is a rule
//! that [`Construct`][crate::extension::Construct] lists: a construct that
//! breaks one does not match there.
//!
//! ## 2. Where a match starts, and how far it reads
//!
//! `email@example.com` is no mention: a mention starts only after whitespace
//! or punctuation, or at the start of a line.
//! [`previous`][crate::extension::Construct::previous] gets the character
//! before a marker, and [`classify`][crate::extension::classify] tells what
//! kind it is.
//!
//! A text construct that fails is tried again at its next marker, so it
//! should not read far before it fails.
//! A name has at most 39 letters, as on GitHub, so a match reads the `@`,
//! the name, and the byte after it, which ends the name: at most 41 bytes,
//! which [`Reach::Bytes`][crate::extension::Reach::Bytes] declares, and
//! parsing stays linear.
//!
//! ```
//! # use markdown::{
//! #     extension::{construct, Construct, Ctx, Step, Text, Token, TokenType},
//! #     mdast::{Custom, Node},
//! #     to_mdast, ParseOptions,
//! # };
//! use markdown::extension::{classify, CharacterKind, Reach};
//! # #[derive(Clone, Copy, Debug, Eq, PartialEq)]
//! # enum MentionToken {
//! #     Mention,
//! #     Name,
//! # }
//! # impl TokenType for MentionToken {
//! #     const ALL: &'static [Self] = &[Self::Mention, Self::Name];
//! #     fn name(self) -> &'static str {
//! #         match self {
//! #             Self::Mention => "mention",
//! #             Self::Name => "mentionName",
//! #         }
//! #     }
//! # }
//! # #[derive(Clone, Copy, Debug)]
//! # enum State {
//! #     Start,
//! #     NameStart,
//! #     Name,
//! # }
//! struct Mention;
//!
//! impl Construct for Mention {
//! #   type Place = Text;
//! #   type State = State;
//! #   type Token = MentionToken;
//! #   fn markers(&self) -> &[u8] {
//! #       b"@"
//! #   }
//! #   fn start(&self) -> State {
//! #       State::Start
//! #   }
//! #   fn step(&self, state: State, t: Ctx<'_, '_, Self>) -> Step<Self> {
//! #       match (state, t.current()) {
//! #           (State::Start, Some(b'@')) => t
//! #               .enter(MentionToken::Mention)
//! #               .consume()
//! #               .next(State::NameStart),
//! #           (State::NameStart, Some(b'a'..=b'z')) => {
//! #               t.enter(MentionToken::Name).consume().next(State::Name)
//! #           }
//! #           (State::Name, Some(b'a'..=b'z')) => t.consume().next(State::Name),
//! #           (State::Name, _) => t.exit().exit().ok(),
//! #           _ => t.nok(),
//! #       }
//! #   }
//! #   fn to_mdast(&self, tokens: Vec<Token<'_, MentionToken>>) -> Node {
//! #       Node::Custom(Custom {
//! #           name: "mention".into(),
//! #           value: Some(tokens[1].value.to_string()),
//! #           ..Custom::default()
//! #       })
//! #   }
//!     // …the rest as before, and:
//!
//!     fn previous(&self, previous: Option<char>) -> bool {
//!         previous.map_or(true, |char| classify(char) != CharacterKind::Other)
//!     }
//!
//!     fn reach(&self) -> Reach {
//!         Reach::Bytes(41)
//!     }
//! }
//!
//! let options = ParseOptions {
//!     text_constructs: vec![construct(Mention)],
//!     ..ParseOptions::default()
//! };
//!
//! assert_eq!(
//!     to_mdast("(@alice) email@example.com", &options)?.to_string(),
//!     "(alice) email@example.com"
//! );
//!
//! // A step past the reach does not match, so a name of 40 letters is text.
//! let long = format!("@{} b", "a".repeat(40));
//! assert_eq!(to_mdast(&long, &options)?.to_string(), long);
//! # Ok::<(), markdown::message::Message>(())
//! ```
//!
//! ## 3. HTML, and tests
//!
//! [`to_html()`][crate::to_html()] writes a match with
//! [`Construct::html`][crate::extension::Construct::html], which gets each
//! event of the match: the enter and exit of each token, and the bytes a
//! step consumed in a token outside the tokens in it.
//! The default writes those bytes, so a construct without a handler is
//! written as its source.
//! This one wraps the name in a link, and leaves out the `@`.
//!
//! In the tests of a construct, the `test-util` feature makes a construct
//! that breaks a rule panic, naming the rule, and adds
//! `extension::assert_linear`, which checks that parsing stays linear.
//! [Recipe 11][super::recipes::testing] shows both.
//!
//! ```
//! # use markdown::{
//! #     extension::{construct, classify, CharacterKind, Construct, Ctx, Reach, Step, Text, Token, TokenType},
//! #     mdast::{Custom, Node},
//! #     ParseOptions,
//! # };
//! use markdown::{
//!     extension::{HtmlEvent, HtmlWriter},
//!     to_html_with_options, Options,
//! };
//! # #[derive(Clone, Copy, Debug, Eq, PartialEq)]
//! # enum MentionToken {
//! #     Mention,
//! #     Name,
//! # }
//! # impl TokenType for MentionToken {
//! #     const ALL: &'static [Self] = &[Self::Mention, Self::Name];
//! #     fn name(self) -> &'static str {
//! #         match self {
//! #             Self::Mention => "mention",
//! #             Self::Name => "mentionName",
//! #         }
//! #     }
//! # }
//! # #[derive(Clone, Copy, Debug)]
//! # enum State {
//! #     Start,
//! #     NameStart,
//! #     Name,
//! # }
//! struct Mention;
//!
//! impl Construct for Mention {
//! #   type Place = Text;
//! #   type State = State;
//! #   type Token = MentionToken;
//! #   fn markers(&self) -> &[u8] {
//! #       b"@"
//! #   }
//! #   fn start(&self) -> State {
//! #       State::Start
//! #   }
//! #   fn previous(&self, previous: Option<char>) -> bool {
//! #       previous.map_or(true, |char| classify(char) != CharacterKind::Other)
//! #   }
//! #   fn reach(&self) -> Reach {
//! #       Reach::Bytes(41)
//! #   }
//! #   fn step(&self, state: State, t: Ctx<'_, '_, Self>) -> Step<Self> {
//! #       match (state, t.current()) {
//! #           (State::Start, Some(b'@')) => t
//! #               .enter(MentionToken::Mention)
//! #               .consume()
//! #               .next(State::NameStart),
//! #           (State::NameStart, Some(b'a'..=b'z')) => {
//! #               t.enter(MentionToken::Name).consume().next(State::Name)
//! #           }
//! #           (State::Name, Some(b'a'..=b'z')) => t.consume().next(State::Name),
//! #           (State::Name, _) => t.exit().exit().ok(),
//! #           _ => t.nok(),
//! #       }
//! #   }
//! #   fn to_mdast(&self, tokens: Vec<Token<'_, MentionToken>>) -> Node {
//! #       Node::Custom(Custom {
//! #           name: "mention".into(),
//! #           value: Some(tokens[1].value.to_string()),
//! #           ..Custom::default()
//! #       })
//! #   }
//!     // …the rest as before, and:
//!
//!     fn html(
//!         &self,
//!         event: HtmlEvent<'_, MentionToken>,
//!         _: &[Token<'_, MentionToken>],
//!         w: &mut HtmlWriter<'_>,
//!     ) {
//!         if let HtmlEvent::Text(MentionToken::Name, name) = event {
//!             w.tag("<a");
//!             w.attr("href", &format!("/{}", name));
//!             w.tag(">@");
//!             w.text(name);
//!             w.tag("</a>");
//!         }
//!     }
//! }
//!
//! let options = Options {
//!     parse: ParseOptions {
//!         text_constructs: vec![construct(Mention)],
//!         ..ParseOptions::default()
//!     },
//!     ..Options::default()
//! };
//!
//! assert_eq!(
//!     to_html_with_options("hi @alice", &options)?,
//!     "<p>hi <a href=\"/alice\">@alice</a></p>"
//! );
//! # Ok::<(), markdown::message::Message>(())
//! ```
//!
//! From here, the [recipes][super::recipes] show other patterns.
