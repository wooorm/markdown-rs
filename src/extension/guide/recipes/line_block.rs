//! Recipe 6: a block of one line, such as `:: note Read *this* first`.
//!
//! A [`Flow`][crate::extension::Flow] construct is tried at the start of a
//! line, before the built-in ones, after the prefixes of block quotes and
//! list items.
//! Up to 3 columns of indentation before it are skipped for it, so it
//! starts at its marker; 4 columns are indented code.
//! It can interrupt a paragraph, and a line that would continue a
//! paragraph in a block quote without the `>`, a lazy line, is not lazy
//! when the construct matches it: the block quote ends.
//! `\::` is text, as the line then starts with `\`.
//!
//! The construct ends at the line ending, or the end, with
//! [`Ctx::ok`][crate::extension::Ctx::ok]: in flow, a step that consumed
//! cannot end a match, so the compiler rejects ending at the last byte.
//! The rest of the line is text content, so it is markdown.
//! Flow is tried once for each line, so the reach does not matter.
//!
//! ```
//! use markdown::{
//!     extension::{
//!         construct, Construct, ContentType, Ctx, Flow, HtmlEvent, HtmlWriter, Step, Token,
//!         TokenSlice,
//!         TokenType,
//!     },
//!     mdast::{Custom, Node},
//!     to_html_with_options, Options, ParseOptions,
//! };
//!
//! #[derive(Clone, Copy, Debug, Eq, PartialEq)]
//! enum LineToken {
//!     Block,
//!     Marker,
//!     Name,
//!     Rest,
//! }
//!
//! impl TokenType for LineToken {
//!     const ALL: &'static [Self] = &[Self::Block, Self::Marker, Self::Name, Self::Rest];
//!
//!     fn name(self) -> &'static str {
//!         match self {
//!             Self::Block => "lineBlock",
//!             Self::Marker => "lineBlockMarker",
//!             Self::Name => "lineBlockName",
//!             Self::Rest => "lineBlockRest",
//!         }
//!     }
//!
//!     fn content(self) -> Option<ContentType> {
//!         match self {
//!             Self::Rest => Some(ContentType::Text),
//!             _ => None,
//!         }
//!     }
//! }
//!
//! #[derive(Clone, Copy, Debug)]
//! enum State {
//!     Start,
//!     Marker,
//!     /// After `::`, with whether whitespace came.
//!     AfterMarker(bool),
//!     Name,
//!     AfterName,
//!     Rest,
//! }
//!
//! struct LineBlock;
//!
//! impl Construct for LineBlock {
//!     type Place = Flow;
//!     type State = State;
//!     type Token = LineToken;
//!
//!     fn markers(&self) -> &[u8] {
//!         b":"
//!     }
//!
//!     fn start(&self) -> State {
//!         State::Start
//!     }
//!
//!     fn step(&self, state: State, t: Ctx<'_, '_, Self>) -> Step<Self> {
//!         match (state, t.current()) {
//!             (State::Start, Some(b':')) => t
//!                 .enter(LineToken::Block)
//!                 .enter(LineToken::Marker)
//!                 .consume()
//!                 .next(State::Marker),
//!             (State::Marker, Some(b':')) => t.consume().exit().next(State::AfterMarker(false)),
//!             (State::AfterMarker(_), Some(b'\t' | b' ')) => {
//!                 t.consume().next(State::AfterMarker(true))
//!             }
//!             (State::AfterMarker(true), Some(byte)) if byte.is_ascii_alphabetic() => {
//!                 t.enter(LineToken::Name).consume().next(State::Name)
//!             }
//!             (State::Name, Some(byte)) if byte.is_ascii_alphanumeric() || byte == b'-' => {
//!                 t.consume().next(State::Name)
//!             }
//!             (State::Name, _) => t.exit().retry(State::AfterName),
//!             (State::AfterName, Some(b'\t' | b' ')) => t.consume().next(State::AfterName),
//!             (State::AfterName | State::Rest, None | Some(b'\n')) => {
//!                 let t = match state {
//!                     State::Rest => t.exit(),
//!                     _ => t,
//!                 };
//!                 t.exit().ok()
//!             }
//!             (State::AfterName, Some(_)) => t.enter(LineToken::Rest).consume().next(State::Rest),
//!             (State::Rest, Some(_)) => t.consume().next(State::Rest),
//!             _ => t.nok(),
//!         }
//!     }
//!
//!     fn to_mdast(&self, mut tokens: Vec<Token<'_, LineToken>>) -> Node {
//!         let mut custom = Custom {
//!             name: "lineBlock".into(),
//!             children: tokens.take_children(LineToken::Rest),
//!             ..Custom::default()
//!         };
//!         let name = tokens.value(LineToken::Name).unwrap_or_default();
//!         custom.fields.insert("name".into(), name.into());
//!         Node::Custom(custom)
//!     }
//!
//!     fn html(&self, event: HtmlEvent<'_, LineToken>, tokens: &[Token<'_, LineToken>], w: &mut HtmlWriter<'_>) {
//!         match event {
//!             HtmlEvent::Enter(token) if token.name == LineToken::Block => {
//!                 w.tag("<div");
//!                 w.attr("class", tokens.value(LineToken::Name).unwrap_or_default());
//!                 w.tag(">");
//!             }
//!             HtmlEvent::Exit(token) if token.name == LineToken::Block => w.tag("</div>"),
//!             _ => {}
//!         }
//!     }
//! }
//!
//! let options = Options {
//!     parse: ParseOptions {
//!         flow_constructs: vec![construct(LineBlock)],
//!         ..ParseOptions::default()
//!     },
//!     ..Options::default()
//! };
//! let html = |input: &str| to_html_with_options(input, &options).unwrap();
//!
//! assert_eq!(html(":: note Read *this*"), "<div class=\"note\">Read <em>this</em></div>");
//! assert_eq!(html(":: toc"), "<div class=\"toc\"></div>");
//! // It interrupts a paragraph, and is not a lazy line.
//! assert_eq!(html("a\n:: toc"), "<p>a</p>\n<div class=\"toc\"></div>");
//! assert_eq!(
//!     html("> a\n:: toc"),
//!     "<blockquote>\n<p>a</p>\n</blockquote>\n<div class=\"toc\"></div>"
//! );
//! // In containers, and after indentation.
//! assert_eq!(
//!     html("- :: toc"),
//!     "<ul>\n<li>\n<div class=\"toc\"></div>\n</li>\n</ul>"
//! );
//! assert_eq!(html("   :: toc"), "<div class=\"toc\"></div>");
//! assert_eq!(html("    :: toc"), "<pre><code>:: toc\n</code></pre>");
//! // Not a block.
//! assert_eq!(html("\\:: toc"), "<p>:: toc</p>");
//! assert_eq!(html("::toc"), "<p>::toc</p>");
//! ```
