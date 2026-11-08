//! Recipe 11: testing a construct.
//!
//! A construct that breaks a rule of
//! [`Construct`][crate::extension::Construct] does not match, so a
//! mistake can look like input that is not the construct.
//! The `test-util` feature turns each broken rule into a panic that names
//! it, with the construct, the byte, and the state of the step, and adds
//! helpers:
//!
//! * `extension::assert_linear`, which checks that parsing stays linear on
//!   documents made of the construct's markers, and
//!   `extension::assert_linear_with`, which also repeats patterns you give
//! * `extension::trace`, every step of every construct while parsing a
//!   document: where, in which state, and what came of it
//! * `markdown_processor::assert_same_html`, behind that crate's own
//!   `test-util` feature, which checks that `to_html` and a processor write
//!   the same HTML
//!
//! Turn it on for tests only:
//!
//! ```toml
//! [dev-dependencies]
//! markdown = { version = "1", features = ["test-util"] }
//! markdown_processor = { version = "0", features = ["test-util"] }
//! ```
//!
//! Then test, as the other recipes do:
//!
//! * the construct, and what is close to it but is not it: a marker alone,
//!   at the end, doubled, without its closer, after a character escape
//! * where it is: at the start and end of a paragraph, in a heading, a
//!   block quote, and a list item, across a lazy line, and next to the
//!   built-in constructs that share its markers or can hold it, such as a
//!   code span or a link
//! * bytes that are easy to get wrong: tabs, which are one column at a
//!   time in prefixes, `\r\n`, and characters of more than one byte
//! * `to_mdast` and `to_html` both, as each has its own code
//!
//! Documents of markers can miss a construct's own worst case, such as an
//! opener without its closer, repeated, as `:x[:x[:x[`: give those to
//! `assert_linear_with`.
//! When a test fails and the reason is not clear, `trace` shows each step.
//!
//! ```
//! use markdown::{
//!     extension::{construct, Construct, Ctx, Reach, Step, Text, Token, TokenType},
//!     mdast::{Custom, Node},
//!     to_mdast, ParseOptions,
//! };
//!
//! #[derive(Clone, Copy, Debug, Eq, PartialEq)]
//! enum WikiToken {
//!     Wiki,
//!     Target,
//! }
//!
//! impl TokenType for WikiToken {
//!     const ALL: &'static [Self] = &[Self::Wiki, Self::Target];
//!
//!     fn name(self) -> &'static str {
//!         match self {
//!             Self::Wiki => "wiki",
//!             Self::Target => "wikiTarget",
//!         }
//!     }
//! }
//!
//! /// `[[target]]`, with a reach to test, and, when `broken`, a mistake.
//! struct Wiki {
//!     reach: Reach,
//!     broken: bool,
//! }
//!
//! impl Construct for Wiki {
//!     type Place = Text;
//!     type State = u8;
//!     type Token = WikiToken;
//!
//!     fn markers(&self) -> &[u8] {
//!         b"["
//!     }
//!
//!     fn start(&self) -> u8 {
//!         0
//!     }
//!
//!     fn reach(&self) -> Reach {
//!         self.reach
//!     }
//!
//!     fn step(&self, state: u8, t: Ctx<'_, '_, Self>) -> Step<Self> {
//!         match (state, t.current()) {
//!             // The mistake: the first `[` is consumed before any token.
//!             (0, Some(b'[')) if self.broken => t.consume().next(1),
//!             (0, Some(b'[')) => t.enter(WikiToken::Wiki).consume().next(1),
//!             (1, Some(b'[')) => t.consume().next(2),
//!             (2, Some(byte)) if byte != b']' => t.enter(WikiToken::Target).consume().next(3),
//!             (3, Some(b']')) => t.exit().consume().next(4),
//!             // A target can hold `[`, so a match reads past other starts.
//!             (3, Some(_)) => t.consume().next(3),
//!             (4, Some(b']')) => t.consume().exit().ok(),
//!             _ => t.nok(),
//!         }
//!     }
//!
//!     fn to_mdast(&self, tokens: Vec<Token<'_, WikiToken>>) -> Node {
//!         Node::Custom(Custom {
//!             name: "wikiLink".into(),
//!             value: tokens.get(1).map(|token| token.value.to_string()),
//!             ..Custom::default()
//!         })
//!     }
//! }
//!
//! let options = |reach, broken| ParseOptions {
//!     text_constructs: vec![construct(Wiki { reach, broken })],
//!     ..ParseOptions::default()
//! };
//! let has_wiki = |input: &str, options: &ParseOptions| {
//!     let tree = to_mdast(input, options).unwrap();
//!     matches!(tree.children().unwrap()[0].children().unwrap()[0], Node::Custom(_))
//! };
//!
//! // The construct, and what is close to it.
//! let good = options(Reach::Pair, false);
//! assert!(has_wiki("[[a]]", &good));
//! assert!(!has_wiki("[[a]", &good));
//! assert!(!has_wiki("\\[[a]]", &good));
//! assert!(!has_wiki("[[]]", &good));
//!
//! // Without `test-util`, the mistake is a construct that never matches.
//! #[cfg(not(feature = "test-util"))]
//! assert!(!has_wiki("[[a]]", &options(Reach::Pair, true)));
//!
//! #[cfg(feature = "test-util")]
//! {
//!     use markdown::extension::{assert_linear, assert_linear_with, trace, StepResult, TraceEvent};
//!     use std::panic::catch_unwind;
//!
//!     // With it, a panic names the rule.
//!     let error = catch_unwind(|| has_wiki("[[a]]", &options(Reach::Pair, true))).unwrap_err();
//!     let message = error.downcast_ref::<String>().unwrap();
//!     assert_eq!(
//!         message,
//!         "a construct broke the rule `consume-outside-token`: Wiki at byte 0, in state 0"
//!     );
//!
//!     // `[[a [[a [[a …` without `]]`: unbounded, each `[[` reads to the
//!     // end; a pair skips what a failed match read.
//!     assert_linear(&good);
//!     assert!(catch_unwind(|| assert_linear(&options(Reach::Unbounded, false))).is_err());
//!
//!     // Patterns of its own: an opener and a target, without a closer.
//!     assert_linear_with(&good, &["[[a", "[[a]"]);
//!
//!     // Where `[[a]` fails: at the end, waiting for a second `]`.
//!     let steps = trace("[[a]", &good);
//!     assert_eq!(
//!         steps.last(),
//!         Some(&TraceEvent::Step {
//!             offset: 4,
//!             state: "4".into(),
//!             result: StepResult::Nok
//!         })
//!     );
//! }
//! ```
