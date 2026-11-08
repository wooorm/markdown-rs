//! Recipe 2: a raw span between delimiters, such as `{{ user.name }}`.
//!
//! The inside is raw: it is not parsed as markdown, so it is a token of its
//! own, whose value the node gets.
//! A match stays on one line, and stops at the first `}}`, so
//! [`Reach::Pair`][crate::extension::Reach::Pair] keeps parsing linear: a
//! start inside a match that failed would fail at the same line ending, or
//! the same lone `}`.
//!
//! A character escape before it, `\{{`, makes it text: the escaped `{` is
//! text, and the construct, tried at the next `{`, needs two.
//! A code span that starts before it holds it, as built-in constructs are
//! parsed in order; one that starts inside it is part of its raw value.
//!
//! ```
//! use markdown::{
//!     extension::{construct, Construct, Ctx, HtmlEvent, HtmlWriter, Reach, Step, Text, Token, TokenSlice, TokenType},
//!     mdast::{Custom, Node},
//!     to_html_with_options, Options, ParseOptions,
//! };
//!
//! #[derive(Clone, Copy, Debug, Eq, PartialEq)]
//! enum SpanToken {
//!     Span,
//!     Marker,
//!     Path,
//! }
//!
//! impl TokenType for SpanToken {
//!     const ALL: &'static [Self] = &[Self::Span, Self::Marker, Self::Path];
//!
//!     fn name(self) -> &'static str {
//!         match self {
//!             Self::Span => "interpolation",
//!             Self::Marker => "interpolationMarker",
//!             Self::Path => "interpolationPath",
//!         }
//!     }
//! }
//!
//! #[derive(Clone, Copy, Debug)]
//! enum State {
//!     Start,
//!     Open,
//!     PathStart,
//!     Path,
//!     Close,
//! }
//!
//! struct Interpolation;
//!
//! impl Construct for Interpolation {
//!     type Place = Text;
//!     type State = State;
//!     type Token = SpanToken;
//!
//!     fn markers(&self) -> &[u8] {
//!         b"{"
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
//!             (State::Start, Some(b'{')) => t
//!                 .enter(SpanToken::Span)
//!                 .enter(SpanToken::Marker)
//!                 .consume()
//!                 .next(State::Open),
//!             (State::Open, Some(b'{')) => t.consume().exit().next(State::PathStart),
//!             (State::PathStart, Some(byte)) if !matches!(byte, b'}' | b'\n') => {
//!                 t.enter(SpanToken::Path).consume().next(State::Path)
//!             }
//!             (State::Path, Some(b'}')) => t
//!                 .exit()
//!                 .enter(SpanToken::Marker)
//!                 .consume()
//!                 .next(State::Close),
//!             (State::Path, Some(byte)) if byte != b'\n' => t.consume().next(State::Path),
//!             (State::Close, Some(b'}')) => t.consume().exit().exit().ok(),
//!             _ => t.nok(),
//!         }
//!     }
//!
//!     fn to_mdast(&self, tokens: Vec<Token<'_, SpanToken>>) -> Node {
//!         let path = tokens.value(SpanToken::Path).map(|path| path.trim().to_string());
//!         Node::Custom(Custom {
//!             name: "interpolation".into(),
//!             value: path,
//!             ..Custom::default()
//!         })
//!     }
//!
//!     fn html(&self, event: HtmlEvent<'_, SpanToken>, _: &[Token<'_, SpanToken>], w: &mut HtmlWriter<'_>) {
//!         if let HtmlEvent::Text(SpanToken::Path, path) = event {
//!             w.tag("<var>");
//!             w.text(path.trim());
//!             w.tag("</var>");
//!         }
//!     }
//! }
//!
//! let options = Options {
//!     parse: ParseOptions {
//!         text_constructs: vec![construct(Interpolation)],
//!         ..ParseOptions::default()
//!     },
//!     ..Options::default()
//! };
//! let html = |input: &str| to_html_with_options(input, &options).unwrap();
//!
//! assert_eq!(html("Hi {{ user.name }}!"), "<p>Hi <var>user.name</var>!</p>");
//! assert_eq!(html("\\{{ a }}"), "<p>{{ a }}</p>");
//! assert_eq!(html("`{{ a }}`"), "<p><code>{{ a }}</code></p>");
//! assert_eq!(html("{{ a `b }}` c"), "<p><var>a `b</var>` c</p>");
//! assert_eq!(html("{{ a\nb }}"), "<p>{{ a\nb }}</p>");
//! assert_eq!(html("{{a} b}}"), "<p>{{a} b}}</p>");
//! ```
