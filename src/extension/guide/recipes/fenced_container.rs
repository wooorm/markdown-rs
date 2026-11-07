//! Recipe 7: a fenced container with a markdown body, such as a `:::`
//! block.
//!
//! ```markdown
//! ::: note
//! Some *markdown*.
//! :::
//! ```
//!
//! A [`Flow`][crate::extension::Flow] construct can span lines: its body is
//! a token with [`ContentType::Document`][crate::extension::ContentType::Document],
//! parsed later as a document of its own, so it can hold lists, block
//! quotes, and this construct again.
//! As in fenced code, the closing fence has at least as many colons as the
//! opening one, so a longer outer fence lets the same construct nest.
//!
//! At the start of each line of the body, an
//! [`attempt`][crate::extension::Ctx::attempt] looks for the closing fence,
//! and, when there is none, is undone: then the line is body.
//! Tokens entered inside the body, such as the closing fence and the
//! indentation of the opening fence, which is removed from each line as in
//! fenced code, are not part of it.
//! At the end of the document, or of a block quote or list item around it,
//! [`current`][crate::extension::Ctx::current] is `None`, and the body ends
//! there, as in fenced code.
//!
//! A flow construct decides on its first line: past it, `nok` outside an
//! attempt breaks the rule `nok-after-first-line`, and the construct ends
//! before that line, which is then other markdown.
//! So a construct that spans lines checks all it needs on its first line,
//! and after that only ends.
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
//! enum FenceToken {
//!     Container,
//!     Fence,
//!     Sequence,
//!     Name,
//!     Body,
//!     LinePrefix,
//! }
//!
//! use FenceToken as T;
//!
//! impl TokenType for FenceToken {
//!     const ALL: &'static [Self] =
//!         &[T::Container, T::Fence, T::Sequence, T::Name, T::Body, T::LinePrefix];
//!
//!     fn name(self) -> &'static str {
//!         match self {
//!             T::Container => "fencedContainer",
//!             T::Fence => "fencedContainerFence",
//!             T::Sequence => "fencedContainerSequence",
//!             T::Name => "fencedContainerName",
//!             T::Body => "fencedContainerBody",
//!             T::LinePrefix => "linePrefix",
//!         }
//!     }
//!
//!     // The body is a document.
//!     fn content(self) -> Option<ContentType> {
//!         match self {
//!             T::Body => Some(ContentType::Document),
//!             _ => None,
//!         }
//!     }
//! }
//!
//! /// States, most with the size of the opening sequence.
//! #[derive(Clone, Copy, Debug)]
//! enum State {
//!     Start,
//!     Open(usize),
//!     OpenAfter(usize),
//!     Name(usize),
//!     OpenEnd(usize),
//!     OpenEol(usize),
//!     BodyStart(usize),
//!     LineStart(usize),
//!     /// In the indent of a line, with its columns so far.
//!     Prefix(usize, usize),
//!     Line(usize),
//!     /// In an attempt: the closing fence, with the columns of its indent,
//!     /// and the size of its sequence.
//!     CloseIndent(usize, usize),
//!     Close(usize, usize),
//!     CloseAfter,
//!     AfterBody,
//! }
//!
//! struct FencedContainer;
//!
//! impl Construct for FencedContainer {
//!     type Place = Flow;
//!     type State = State;
//!     type Token = FenceToken;
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
//!             // The opening fence: `:::` or more, and an optional name.
//!             (State::Start, Some(b':')) => t
//!                 .enter(T::Container)
//!                 .enter(T::Fence)
//!                 .enter(T::Sequence)
//!                 .consume()
//!                 .next(State::Open(1)),
//!             (State::Open(size), Some(b':')) => t.consume().next(State::Open(size + 1)),
//!             (State::Open(size), _) if size < 3 => t.nok(),
//!             (State::Open(size), _) => t.exit().retry(State::OpenAfter(size)),
//!             (State::OpenAfter(size), Some(b'\t' | b' ')) => {
//!                 t.consume().next(State::OpenAfter(size))
//!             }
//!             (State::OpenAfter(size), Some(byte)) if byte.is_ascii_alphabetic() => {
//!                 t.enter(T::Name).consume().next(State::Name(size))
//!             }
//!             (State::OpenAfter(size), _) => t.retry(State::OpenEnd(size)),
//!             (State::Name(size), Some(byte)) if byte.is_ascii_alphanumeric() || byte == b'-' => {
//!                 t.consume().next(State::Name(size))
//!             }
//!             (State::Name(size), _) => t.exit().retry(State::OpenEnd(size)),
//!             (State::OpenEnd(size), Some(b'\t' | b' ')) => t.consume().next(State::OpenEnd(size)),
//!             (State::OpenEnd(size), None | Some(b'\n')) => t.exit().retry(State::OpenEol(size)),
//!             (State::OpenEol(size), Some(b'\n')) => t.consume().next(State::BodyStart(size)),
//!             (State::OpenEol(_), None) => t.exit().ok(),
//!             // The body, one line at a time.
//!             (State::BodyStart(size), _) => t.enter(T::Body).retry(State::LineStart(size)),
//!             (State::LineStart(_) | State::Line(_), None) => t.retry(State::AfterBody),
//!             (State::LineStart(size), _) => t.attempt(
//!                 State::CloseIndent(size, 0),
//!                 State::AfterBody,
//!                 State::Prefix(size, 0),
//!             ),
//!             // Up to the indent of the opening fence is not body.
//!             (State::Prefix(size, 0), Some(b'\t' | b' ')) if t.indent() > 0 => {
//!                 t.enter(T::LinePrefix).consume().next(State::Prefix(size, 1))
//!             }
//!             (State::Prefix(size, columns), Some(b'\t' | b' '))
//!                 if columns > 0 && columns < t.indent() =>
//!             {
//!                 t.consume().next(State::Prefix(size, columns + 1))
//!             }
//!             (State::Prefix(size, 0), _) => t.retry(State::Line(size)),
//!             (State::Prefix(size, _), _) => t.exit().retry(State::Line(size)),
//!             (State::Line(size), Some(b'\n')) => t.consume().next(State::LineStart(size)),
//!             (State::Line(size), Some(_)) => t.consume().next(State::Line(size)),
//!             // The closing fence: up to 3 columns, `:::` or more, and
//!             // whitespace.
//!             (State::CloseIndent(size, 0), Some(b'\t' | b' ')) => {
//!                 t.enter(T::Fence).consume().next(State::CloseIndent(size, 1))
//!             }
//!             (State::CloseIndent(size, columns), Some(b'\t' | b' ')) if columns > 0 && columns < 3 => {
//!                 t.consume().next(State::CloseIndent(size, columns + 1))
//!             }
//!             (State::CloseIndent(size, columns), Some(b':')) => {
//!                 let t = if columns == 0 { t.enter(T::Fence) } else { t };
//!                 t.enter(T::Sequence).consume().next(State::Close(size, 1))
//!             }
//!             (State::Close(size, close), Some(b':')) => t.consume().next(State::Close(size, close + 1)),
//!             (State::Close(size, close), _) if close >= size => {
//!                 t.exit().retry(State::CloseAfter)
//!             }
//!             (State::CloseAfter, Some(b'\t' | b' ')) => t.consume().next(State::CloseAfter),
//!             (State::CloseAfter, None | Some(b'\n')) => t.exit().ok(),
//!             (State::AfterBody, _) => t.exit().exit().ok(),
//!             _ => t.nok(),
//!         }
//!     }
//!
//!     fn to_mdast(&self, mut tokens: Vec<Token<'_, FenceToken>>) -> Node {
//!         let mut custom = Custom {
//!             name: "fencedContainer".into(),
//!             children: tokens.take_children(T::Body),
//!             ..Custom::default()
//!         };
//!         let name = tokens.value(T::Name).unwrap_or_default();
//!         custom.fields.insert("name".into(), name.into());
//!         Node::Custom(custom)
//!     }
//!
//!     fn html(&self, event: HtmlEvent<'_, FenceToken>, tokens: &[Token<'_, FenceToken>], w: &mut HtmlWriter<'_>) {
//!         match event {
//!             HtmlEvent::Enter(token) if token.name == T::Container => {
//!                 w.tag("<div");
//!                 w.attr("class", tokens.value(T::Name).unwrap_or_default());
//!                 w.tag(">");
//!             }
//!             HtmlEvent::Exit(token) if token.name == T::Container => {
//!                 w.line_ending_if_needed();
//!                 w.tag("</div>");
//!             }
//!             _ => {}
//!         }
//!     }
//! }
//!
//! let options = Options {
//!     parse: ParseOptions {
//!         flow_constructs: vec![construct(FencedContainer)],
//!         ..ParseOptions::default()
//!     },
//!     ..Options::default()
//! };
//! let html = |input: &str| to_html_with_options(input, &options).unwrap();
//!
//! assert_eq!(
//!     html("::: note\n*a*\n\n- b\n:::\nc"),
//!     "<div class=\"note\">\n<p><em>a</em></p>\n<ul>\n<li>b</li>\n</ul>\n</div>\n<p>c</p>"
//! );
//! // Without a closing fence, it ends with the document or its container.
//! assert_eq!(html("::: note\na"), "<div class=\"note\">\n<p>a</p>\n</div>");
//! assert_eq!(
//!     html("> ::: note\n> a\n\nb"),
//!     "<blockquote>\n<div class=\"note\">\n<p>a</p>\n</div>\n</blockquote>\n<p>b</p>"
//! );
//! // A longer fence nests.
//! assert_eq!(
//!     html(":::: a\n::: b\nc\n:::\n::::"),
//!     "<div class=\"a\">\n<div class=\"b\">\n<p>c</p>\n</div>\n</div>"
//! );
//! // The indent of the opening fence is not body.
//! assert_eq!(html("  ::: note\n  a\n  :::"), "<div class=\"note\">\n<p>a</p>\n</div>");
//! assert_eq!(html(":: note"), "<p>:: note</p>");
//! ```
