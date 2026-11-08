//! Recipe 8: a container with a prefix on each line, such as an
//! admonition.
//!
//! ```markdown
//! !!! warning "Mind the *gap*"
//!     The body is indented by four columns.
//!
//!     It can hold any markdown.
//! ```
//!
//! A [`Container`][crate::extension::Container] works like a block quote:
//! it matches its first line up to where its body starts, and then returns
//! [`container`][crate::extension::Ctx::container] with the state
//! that checks the start of the next line, its prefix, as micromark's
//! `continuation` does.
//! That state can carry what the container has seen, such as the size of
//! a marker.
//! The rest of each line is flow in the body, a token with
//! [`ContentType::Document`][crate::extension::ContentType::Document]
//! entered on the first line and closed by the parser at the end.
//! A later line returns `container` again to continue, or `nok` to end
//! the container; a line that continues a paragraph, a lazy line, does so
//! without its prefix.
//!
//! The first line can also hold text content, here a title.
//! Later lines cannot: their tokens are prefixes, such as the indent here.
//! A later line starts before its indentation, which the prefix takes: a
//! tab there is `\t` at its first column and a space at each column after,
//! so each byte consumed is a column.
//! A container never consumes a line ending.
//!
//! ```
//! use markdown::{
//!     extension::{
//!         construct, Construct, Container, ContentType, Ctx, HtmlEvent, HtmlWriter, Step,
//!         Token, TokenSlice, TokenType,
//!     },
//!     mdast::{Custom, Node},
//!     to_html_with_options, Options, ParseOptions,
//! };
//!
//! #[derive(Clone, Copy, Debug, Eq, PartialEq)]
//! enum NoteToken {
//!     Admonition,
//!     Marker,
//!     Kind,
//!     Title,
//!     TitleMarker,
//!     TitleText,
//!     Body,
//!     Indent,
//! }
//!
//! use NoteToken as T;
//!
//! impl TokenType for NoteToken {
//!     const ALL: &'static [Self] = &[
//!         T::Admonition,
//!         T::Marker,
//!         T::Kind,
//!         T::Title,
//!         T::TitleMarker,
//!         T::TitleText,
//!         T::Body,
//!         T::Indent,
//!     ];
//!
//!     fn name(self) -> &'static str {
//!         match self {
//!             T::Admonition => "admonition",
//!             T::Marker => "admonitionMarker",
//!             T::Kind => "admonitionKind",
//!             T::Title => "admonitionTitle",
//!             T::TitleMarker => "admonitionTitleMarker",
//!             T::TitleText => "admonitionTitleText",
//!             T::Body => "admonitionBody",
//!             T::Indent => "admonitionIndent",
//!         }
//!     }
//!
//!     fn content(self) -> Option<ContentType> {
//!         match self {
//!             T::TitleText => Some(ContentType::Text),
//!             T::Body => Some(ContentType::Document),
//!             _ => None,
//!         }
//!     }
//! }
//!
//! #[derive(Clone, Copy, Debug)]
//! enum State {
//!     Start,
//!     /// In `!!!`, with its size so far.
//!     Marker(u8),
//!     KindBefore,
//!     Kind,
//!     AfterKind,
//!     TitleStart,
//!     TitleText,
//!     TitleClose,
//!     AfterTitle,
//!     Body,
//!     /// The start of a later line.
//!     Continuation,
//!     /// In the indent of a later line, with its columns so far.
//!     Indent(u8),
//! }
//!
//! struct Admonition;
//!
//! impl Construct for Admonition {
//!     type Place = Container;
//!     type State = State;
//!     type Token = NoteToken;
//!
//!     fn markers(&self) -> &[u8] {
//!         b"!"
//!     }
//!
//!     fn start(&self) -> State {
//!         State::Start
//!     }
//!
//!     fn step(&self, state: State, t: Ctx<'_, '_, Self>) -> Step<Self> {
//!         match (state, t.current()) {
//!             // The first line: `!!!`, a kind, and an optional title.
//!             (State::Start, Some(b'!')) => t
//!                 .enter(T::Admonition)
//!                 .enter(T::Marker)
//!                 .consume()
//!                 .next(State::Marker(1)),
//!             (State::Marker(size), Some(b'!')) if size < 3 => {
//!                 t.consume().next(State::Marker(size + 1))
//!             }
//!             (State::Marker(3), Some(b'\t' | b' ')) => t.exit().consume().next(State::KindBefore),
//!             (State::KindBefore, Some(b'\t' | b' ')) => t.consume().next(State::KindBefore),
//!             (State::KindBefore, Some(byte)) if byte.is_ascii_alphabetic() => {
//!                 t.enter(T::Kind).consume().next(State::Kind)
//!             }
//!             (State::Kind, Some(byte)) if byte.is_ascii_alphanumeric() || byte == b'-' => {
//!                 t.consume().next(State::Kind)
//!             }
//!             (State::Kind, _) => t.exit().retry(State::AfterKind),
//!             (State::AfterKind, Some(b'\t' | b' ')) => t.consume().next(State::AfterKind),
//!             (State::AfterKind, Some(b'"')) => t
//!                 .enter(T::Title)
//!                 .enter(T::TitleMarker)
//!                 .consume()
//!                 .exit()
//!                 .next(State::TitleStart),
//!             // Whitespace at the start of the title is not content, so
//!             // `" "` is no title.
//!             (State::TitleStart, Some(b'\t' | b' ')) => t.consume().next(State::TitleStart),
//!             (State::TitleStart, Some(b'"')) => t.retry(State::TitleClose),
//!             (State::TitleStart, Some(byte)) if byte != b'\n' => {
//!                 t.enter(T::TitleText).consume().next(State::TitleText)
//!             }
//!             (State::TitleText, Some(b'"')) => t.exit().retry(State::TitleClose),
//!             (State::TitleText, Some(byte)) if byte != b'\n' => t.consume().next(State::TitleText),
//!             (State::TitleClose, Some(b'"')) => t
//!                 .enter(T::TitleMarker)
//!                 .consume()
//!                 .exit()
//!                 .exit()
//!                 .next(State::AfterTitle),
//!             (State::AfterTitle, Some(b'\t' | b' ')) => t.consume().next(State::AfterTitle),
//!             (State::AfterKind | State::AfterTitle, None | Some(b'\n')) => t.retry(State::Body),
//!             // The rest of the first line is in the body: here, nothing.
//!             (State::Body, _) => t.enter(T::Body).container(State::Continuation),
//!             // A later line: blank, or indented by 4 columns.
//!             (State::Continuation, None | Some(b'\n')) => t.container(State::Continuation),
//!             (State::Continuation, Some(b'\t' | b' ')) => {
//!                 t.enter(T::Indent).consume().next(State::Indent(1))
//!             }
//!             (State::Indent(columns), Some(b'\t' | b' ')) if columns < 4 => {
//!                 t.consume().next(State::Indent(columns + 1))
//!             }
//!             (State::Indent(columns), current)
//!                 if columns == 4 || matches!(current, None | Some(b'\n')) =>
//!             {
//!                 t.exit().container(State::Continuation)
//!             }
//!             _ => t.nok(),
//!         }
//!     }
//!
//!     fn to_mdast(&self, mut tokens: Vec<Token<'_, NoteToken>>) -> Node {
//!         let mut custom = Custom {
//!             name: "admonition".into(),
//!             ..Custom::default()
//!         };
//!         let kind = tokens.value(T::Kind).unwrap_or_default();
//!         custom.fields.insert("kind".into(), kind.into());
//!         if tokens.token(T::TitleText).is_some() {
//!             custom.fields.insert("title".into(), "true".into());
//!             custom.children.push(Node::Paragraph(markdown::mdast::Paragraph {
//!                 children: tokens.take_children(T::TitleText),
//!                 position: None,
//!             }));
//!         }
//!         custom.children.extend(tokens.take_children(T::Body));
//!         Node::Custom(custom)
//!     }
//!
//!     fn html(&self, event: HtmlEvent<'_, NoteToken>, tokens: &[Token<'_, NoteToken>], w: &mut HtmlWriter<'_>) {
//!         match event {
//!             HtmlEvent::Enter(token) if token.name == T::Admonition => {
//!                 let kind = tokens.value(T::Kind).unwrap_or_default();
//!                 w.tag("<div");
//!                 w.attr("class", &format!("admonition {}", kind));
//!                 w.tag(">\n<p class=\"admonition-title\">");
//!                 // Without a title, the kind is the title.
//!                 if tokens.token(T::TitleText).is_none() {
//!                     w.text(kind);
//!                     w.tag("</p>");
//!                 }
//!             }
//!             HtmlEvent::Exit(token) if token.name == T::TitleText => w.tag("</p>"),
//!             HtmlEvent::Exit(token) if token.name == T::Admonition => {
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
//!         document_constructs: vec![construct(Admonition)],
//!         ..ParseOptions::default()
//!     },
//!     ..Options::default()
//! };
//! let html = |input: &str| to_html_with_options(input, &options).unwrap();
//!
//! assert_eq!(
//!     html("!!! warning \"Mind the *gap*\"\n    a\n\n    - b\n\nc"),
//!     "<div class=\"admonition warning\">\n<p class=\"admonition-title\">Mind the <em>gap</em></p>\n<p>a</p>\n<ul>\n<li>b</li>\n</ul>\n</div>\n<p>c</p>"
//! );
//! // Without a title, or with a blank one.
//! assert_eq!(
//!     html("!!! note\n    a"),
//!     "<div class=\"admonition note\">\n<p class=\"admonition-title\">note</p>\n<p>a</p>\n</div>"
//! );
//! assert_eq!(html("!!! note \" \""), html("!!! note"));
//! // A tab is 4 columns, and a lazy line continues a paragraph.
//! assert_eq!(html("!!! note\n\ta\nb"), html("!!! note\n    a\n    b"));
//! // An unindented line that is not lazy ends it.
//! assert_eq!(
//!     html("!!! note\n    a\n\nb"),
//!     "<div class=\"admonition note\">\n<p class=\"admonition-title\">note</p>\n<p>a</p>\n</div>\n<p>b</p>"
//! );
//! // In a block quote.
//! assert_eq!(
//!     html("> !!! note\n>     a"),
//!     "<blockquote>\n<div class=\"admonition note\">\n<p class=\"admonition-title\">note</p>\n<p>a</p>\n</div>\n</blockquote>"
//! );
//! assert_eq!(html("!!!note"), "<p>!!!note</p>");
//! ```
