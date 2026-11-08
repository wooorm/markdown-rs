//! Recipe 3: a span with flanking rules, such as math in `$x^2$`.
//!
//! As in Pandoc, an opening `$` has no whitespace after it, and a closing `$`
//! has none before it and no digit after it, so `$20 and $30` is text.
//!
//! A `$` with whitespace before it cannot close, and the match stops there,
//! so the next start is tried at it: `$a $b$` is `$a ` and then math.
//! Whether a `$` is followed by a digit takes a look past it: an
//! [`attempt`][crate::extension::Ctx::attempt] that keeps the `$` as math
//! when a digit follows, and is undone otherwise, so the `$` closes.
//! A start inside a match that failed would see the same bytes, and fail at
//! the same place, so [`Reach::Pair`][crate::extension::Reach::Pair] keeps
//! parsing linear.
//!
//! ```
//! use markdown::{
//!     extension::{construct, Construct, Ctx, Reach, Step, Text, Token, TokenSlice, TokenType},
//!     mdast::{InlineMath, Node},
//!     to_mdast, ParseOptions,
//! };
//!
//! #[derive(Clone, Copy, Debug, Eq, PartialEq)]
//! enum MathToken {
//!     Math,
//!     Marker,
//!     Value,
//! }
//!
//! impl TokenType for MathToken {
//!     const ALL: &'static [Self] = &[Self::Math, Self::Marker, Self::Value];
//!
//!     fn name(self) -> &'static str {
//!         match self {
//!             Self::Math => "dollarMath",
//!             Self::Marker => "dollarMathMarker",
//!             Self::Value => "dollarMathValue",
//!         }
//!     }
//! }
//!
//! #[derive(Clone, Copy, Debug)]
//! enum State {
//!     Start,
//!     Open,
//!     /// In the value, after whitespace or not.
//!     Value(bool),
//!     /// In an attempt: whether a digit follows a `$`.
//!     Dollar,
//!     DollarAfter,
//!     Close,
//! }
//!
//! struct DollarMath;
//!
//! impl Construct for DollarMath {
//!     type Place = Text;
//!     type State = State;
//!     type Token = MathToken;
//!
//!     fn markers(&self) -> &[u8] {
//!         b"$"
//!     }
//!
//!     fn start(&self) -> State {
//!         State::Start
//!     }
//!
//!     fn reach(&self) -> Reach {
//!         Reach::Pair
//!     }
//!
//!     fn step(&self, state: State, t: Ctx<'_, '_, Self>) -> Step<Self> {
//!         match (state, t.current()) {
//!             (State::Start, Some(b'$')) => t
//!                 .enter(MathToken::Math)
//!                 .enter(MathToken::Marker)
//!                 .consume()
//!                 .exit()
//!                 .next(State::Open),
//!             (State::Open, Some(byte)) if !matches!(byte, b'\t' | b'\n' | b' ' | b'$') => {
//!                 t.enter(MathToken::Value).consume().next(State::Value(false))
//!             }
//!             // A `$` after whitespace cannot close: stop at it.
//!             (State::Value(false), Some(b'$')) => {
//!                 t.attempt(State::Dollar, State::Value(false), State::Close)
//!             }
//!             (State::Value(_), Some(byte @ (b'\t' | b' '))) => {
//!                 t.consume().next(State::Value(byte != b'$'))
//!             }
//!             (State::Value(_), Some(byte)) if !matches!(byte, b'\n' | b'$') => {
//!                 t.consume().next(State::Value(false))
//!             }
//!             // Keep a `$` before a digit as math; else undo, and close.
//!             (State::Dollar, Some(b'$')) => t.consume().next(State::DollarAfter),
//!             (State::DollarAfter, Some(b'0'..=b'9')) => t.ok(),
//!             (State::Close, Some(b'$')) => t
//!                 .exit()
//!                 .enter(MathToken::Marker)
//!                 .consume()
//!                 .exit()
//!                 .exit()
//!                 .ok(),
//!             _ => t.nok(),
//!         }
//!     }
//!
//!     fn to_mdast(&self, tokens: Vec<Token<'_, MathToken>>) -> Node {
//!         let value = tokens.value(MathToken::Value).unwrap_or_default().into();
//!         Node::InlineMath(InlineMath {
//!             value,
//!             position: None,
//!         })
//!     }
//! }
//!
//! let parse = ParseOptions {
//!     text_constructs: vec![construct(DollarMath)],
//!     ..ParseOptions::default()
//! };
//! let math = |input: &str| {
//!     let tree = to_mdast(input, &parse).unwrap();
//!     let mut found = vec![];
//!     for node in tree.children().unwrap()[0].children().unwrap() {
//!         if let Node::InlineMath(math) = node {
//!             found.push(math.value.clone());
//!         }
//!     }
//!     found
//! };
//!
//! assert_eq!(math("so $x^2$ is"), vec!["x^2"]);
//! assert_eq!(math("$20 and $30"), Vec::<String>::new());
//! assert_eq!(math("$a $b$"), vec!["b"]);
//! assert_eq!(math("$a$1$ b"), vec!["a$1"]);
//! assert_eq!(math("$ a$"), Vec::<String>::new());
//! ```
