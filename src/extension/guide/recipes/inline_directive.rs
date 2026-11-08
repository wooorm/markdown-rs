//! Recipe 4: an inline directive with a label and attributes, such as
//! `:abbr[HTML]{title="HyperText"}`.
//!
//! The label holds markdown: its token has
//! [`ContentType::Text`][crate::extension::ContentType::Text], so
//! [`Token::children`][crate::extension::Token::children] has its nodes, and
//! `to_html` writes them between the start and end of the element.
//! The attributes are optional, and are read in an
//! [`attempt`][crate::extension::Ctx::attempt]: when they do not close on
//! their line, the attempt is undone, the directive ends after its label,
//! and the `{` is text.
//!
//! Brackets in the label nest, so the first `]` does not always close it,
//! and [`Reach::Pair`][crate::extension::Reach::Pair] does not fit: in
//! `:a[ :b[x]`, the outer start fails, but the inner one matches.
//! [`Reach::Bytes`][crate::extension::Reach::Bytes] does: as in
//! `micromark-extension-directive`, a directive is at most 999 bytes, the
//! limit `CommonMark` sets for link labels.
//! The limit is also the cost: on input such as `:x[` repeated, each `:`
//! reads up to 999 bytes, so a construct that needs less should say so.
//!
//! A code span in the label binds tighter than its brackets, so `` `]` ``
//! does not close it.
//! Whether a backtick run starts a code span takes a look ahead for a run
//! of the same size: an attempt, which, when there is none, is undone, and
//! the run is text.
//! Each run that fails is of a size no later run has, so there are few of
//! them in 999 bytes, and the work stays linear.
//!
//! The name is the class of a `<span>`.
//! [`HtmlWriter::attr`][crate::extension::HtmlWriter::attr] escapes each
//! value, checks URLs, and leaves out attributes that run script, such as
//! `onclick`; this recipe writes only a few on top of that.
//!
//! ```
//! use markdown::{
//!     extension::{
//!         construct, Construct, ContentType, Ctx, HtmlEvent, HtmlWriter, Reach, Step, Text,
//!         Token, TokenSlice, TokenType,
//!     },
//!     mdast::{Custom, Node},
//!     to_html_with_options, to_mdast, Options, ParseOptions,
//! };
//! use std::collections::BTreeMap;
//!
//! #[derive(Clone, Copy, Debug, Eq, PartialEq)]
//! enum DirectiveToken {
//!     Directive,
//!     Marker,
//!     Name,
//!     Label,
//!     LabelMarker,
//!     LabelString,
//!     Attributes,
//!     AttributesMarker,
//!     AttributesValue,
//! }
//!
//! use DirectiveToken as T;
//!
//! impl TokenType for DirectiveToken {
//!     const ALL: &'static [Self] = &[
//!         T::Directive,
//!         T::Marker,
//!         T::Name,
//!         T::Label,
//!         T::LabelMarker,
//!         T::LabelString,
//!         T::Attributes,
//!         T::AttributesMarker,
//!         T::AttributesValue,
//!     ];
//!
//!     fn name(self) -> &'static str {
//!         match self {
//!             T::Directive => "directive",
//!             T::Marker => "directiveMarker",
//!             T::Name => "directiveName",
//!             T::Label => "directiveLabel",
//!             T::LabelMarker => "directiveLabelMarker",
//!             T::LabelString => "directiveLabelString",
//!             T::Attributes => "directiveAttributes",
//!             T::AttributesMarker => "directiveAttributesMarker",
//!             T::AttributesValue => "directiveAttributesValue",
//!         }
//!     }
//!
//!     // The label is markdown.
//!     fn content(self) -> Option<ContentType> {
//!         match self {
//!             T::LabelString => Some(ContentType::Text),
//!             _ => None,
//!         }
//!     }
//! }
//!
//! #[derive(Clone, Copy, Debug)]
//! enum State {
//!     Start,
//!     NameStart,
//!     Name,
//!     LabelStart,
//!     /// In the label, with the depth of open brackets.
//!     Label(usize),
//!     LabelEscape(usize),
//!     /// The rest of a backtick run that is not a code span.
//!     LabelTicks(usize),
//!     /// In an attempt: a code span, with the size of its opening run.
//!     CodeOpen(usize),
//!     Code(usize),
//!     /// In the closing run, with the size of the opening run and of this one.
//!     CodeClose(usize, usize),
//!     AfterLabel,
//!     /// In an attempt: the attributes.
//!     AttributesStart,
//!     AttributesOpen,
//!     Attributes,
//!     Done,
//! }
//!
//! struct Directive;
//!
//! impl Construct for Directive {
//!     type Place = Text;
//!     type State = State;
//!     type Token = DirectiveToken;
//!
//!     fn markers(&self) -> &[u8] {
//!         b":"
//!     }
//!
//!     fn start(&self) -> State {
//!         State::Start
//!     }
//!
//!     fn reach(&self) -> Reach {
//!         Reach::Bytes(999)
//!     }
//!
//!     fn step(&self, state: State, t: Ctx<'_, '_, Self>) -> Step<Self> {
//!         match (state, t.current()) {
//!             (State::Start, Some(b':')) => t
//!                 .enter(T::Directive)
//!                 .enter(T::Marker)
//!                 .consume()
//!                 .exit()
//!                 .next(State::NameStart),
//!             (State::NameStart, Some(byte)) if byte.is_ascii_alphabetic() => {
//!                 t.enter(T::Name).consume().next(State::Name)
//!             }
//!             (State::Name, Some(byte)) if byte.is_ascii_alphanumeric() || byte == b'-' => {
//!                 t.consume().next(State::Name)
//!             }
//!             (State::Name, Some(b'[')) => t
//!                 .exit()
//!                 .enter(T::Label)
//!                 .enter(T::LabelMarker)
//!                 .consume()
//!                 .exit()
//!                 .next(State::LabelStart),
//!             (State::LabelStart, Some(b']')) => t
//!                 .enter(T::LabelMarker)
//!                 .consume()
//!                 .exit()
//!                 .exit()
//!                 .next(State::AfterLabel),
//!             (State::LabelStart, _) => t.enter(T::LabelString).retry(State::Label(0)),
//!             (State::Label(0), Some(b']')) => t
//!                 .exit()
//!                 .enter(T::LabelMarker)
//!                 .consume()
//!                 .exit()
//!                 .exit()
//!                 .next(State::AfterLabel),
//!             (State::Label(depth), Some(b']')) => t.consume().next(State::Label(depth - 1)),
//!             (State::Label(depth), Some(b'[')) => t.consume().next(State::Label(depth + 1)),
//!             (State::Label(depth), Some(b'\\')) => t.consume().next(State::LabelEscape(depth)),
//!             (State::Label(depth), Some(b'`')) => t.attempt(
//!                 State::CodeOpen(0),
//!                 State::Label(depth),
//!                 State::LabelTicks(depth),
//!             ),
//!             (State::Label(depth), Some(_)) => t.consume().next(State::Label(depth)),
//!             (State::LabelEscape(depth), Some(b'[' | b'\\' | b']' | b'`')) => {
//!                 t.consume().next(State::Label(depth))
//!             }
//!             (State::LabelEscape(depth), _) => t.retry(State::Label(depth)),
//!             (State::LabelTicks(depth), Some(b'`')) => t.consume().next(State::LabelTicks(depth)),
//!             (State::LabelTicks(depth), _) => t.retry(State::Label(depth)),
//!             (State::CodeOpen(size), Some(b'`')) => t.consume().next(State::CodeOpen(size + 1)),
//!             (State::CodeOpen(size), _) => t.retry(State::Code(size)),
//!             (State::Code(size), Some(b'`')) => t.consume().next(State::CodeClose(size, 1)),
//!             (State::Code(size), Some(_)) => t.consume().next(State::Code(size)),
//!             (State::CodeClose(size, run), Some(b'`')) => {
//!                 t.consume().next(State::CodeClose(size, run + 1))
//!             }
//!             (State::CodeClose(size, run), _) if size == run => t.ok(),
//!             (State::CodeClose(size, _), _) => t.retry(State::Code(size)),
//!             (State::AfterLabel, Some(b'{')) => {
//!                 t.attempt(State::AttributesStart, State::Done, State::Done)
//!             }
//!             (State::AfterLabel | State::Done, _) => t.exit().ok(),
//!             (State::AttributesStart, Some(b'{')) => t
//!                 .enter(T::Attributes)
//!                 .enter(T::AttributesMarker)
//!                 .consume()
//!                 .exit()
//!                 .next(State::AttributesOpen),
//!             (State::AttributesOpen | State::Attributes, Some(b'}')) => {
//!                 let t = match state {
//!                     State::Attributes => t.exit(),
//!                     _ => t,
//!                 };
//!                 t.enter(T::AttributesMarker).consume().exit().exit().ok()
//!             }
//!             (State::AttributesOpen, Some(byte)) if byte != b'\n' => {
//!                 t.enter(T::AttributesValue).consume().next(State::Attributes)
//!             }
//!             (State::Attributes, Some(byte)) if byte != b'\n' => {
//!                 t.consume().next(State::Attributes)
//!             }
//!             _ => t.nok(),
//!         }
//!     }
//!
//!     fn to_mdast(&self, mut tokens: Vec<Token<'_, DirectiveToken>>) -> Node {
//!         let mut custom = Custom {
//!             name: "textDirective".into(),
//!             children: tokens.take_children(T::LabelString),
//!             attributes: attributes(tokens.value(T::AttributesValue).unwrap_or_default()),
//!             ..Custom::default()
//!         };
//!         let name = tokens.value(T::Name).unwrap_or_default();
//!         custom.fields.insert("name".into(), name.into());
//!         Node::Custom(custom)
//!     }
//!
//!     fn html(
//!         &self,
//!         event: HtmlEvent<'_, DirectiveToken>,
//!         tokens: &[Token<'_, DirectiveToken>],
//!         w: &mut HtmlWriter<'_>,
//!     ) {
//!         match event {
//!             HtmlEvent::Enter(token) if token.name == T::Directive => {
//!                 w.tag("<span");
//!                 w.attr("class", tokens.value(T::Name).unwrap_or_default());
//!                 let attrs = attributes(tokens.value(T::AttributesValue).unwrap_or_default());
//!                 for (key, value) in &attrs {
//!                     // A few, on top of what `attr` leaves out.
//!                     if matches!(key.as_str(), "id" | "title" | "lang") {
//!                         w.attr(key, value);
//!                     }
//!                 }
//!                 w.tag(">");
//!             }
//!             HtmlEvent::Exit(token) if token.name == T::Directive => w.tag("</span>"),
//!             _ => {}
//!         }
//!     }
//! }
//!
//! /// Attributes from `key=value key="a value"`.
//! fn attributes(value: &str) -> BTreeMap<String, String> {
//!     let mut map = BTreeMap::new();
//!     let mut rest = value.trim_start();
//!     while !rest.is_empty() {
//!         let key_end = rest.find(|c: char| c == '=' || c.is_whitespace()).unwrap_or(rest.len());
//!         let key = &rest[..key_end];
//!         rest = &rest[key_end..];
//!         let mut value = "";
//!         if let Some(after) = rest.strip_prefix('=') {
//!             let (found, after) = match after.strip_prefix('"') {
//!                 Some(quoted) => quoted.split_once('"').unwrap_or((quoted, "")),
//!                 None => after.split_once(char::is_whitespace).unwrap_or((after, "")),
//!             };
//!             value = found;
//!             rest = after;
//!         }
//!         if !key.is_empty() {
//!             map.insert(key.to_string(), value.to_string());
//!         }
//!         rest = rest.trim_start();
//!     }
//!     map
//! }
//!
//! let options = Options {
//!     parse: ParseOptions {
//!         text_constructs: vec![construct(Directive)],
//!         ..ParseOptions::default()
//!     },
//!     ..Options::default()
//! };
//! let html = |input: &str| to_html_with_options(input, &options).unwrap();
//!
//! assert_eq!(
//!     html(":abbr[HTML]{title=\"HyperText Markup Language\"}"),
//!     "<p><span class=\"abbr\" title=\"HyperText Markup Language\">HTML</span></p>"
//! );
//! // The label is markdown, and its brackets nest.
//! assert_eq!(html(":x[*a* [b]]"), "<p><span class=\"x\"><em>a</em> [b]</span></p>");
//! // A start that fails leaves the next one.
//! assert_eq!(html(":a[ :b[x]"), "<p>:a[ <span class=\"b\">x</span></p>");
//! // A code span binds tighter; a backtick run without a closer is text.
//! assert_eq!(html(":x[`]`]"), "<p><span class=\"x\"><code>]</code></span></p>");
//! assert_eq!(html(":x[a`b]"), "<p><span class=\"x\">a`b</span></p>");
//! // Attributes that do not close are text.
//! assert_eq!(html(":x[a]{id=b"), "<p><span class=\"x\">a</span>{id=b</p>");
//! // Only some attributes are written.
//! assert_eq!(
//!     html(":x[a]{onclick=alert(1) id=b}"),
//!     "<p><span class=\"x\" id=\"b\">a</span></p>"
//! );
//! // A directive is at most 999 bytes.
//! let long = format!(":x[{}]", "a".repeat(999));
//! assert_eq!(html(&long), format!("<p>{}</p>", long));
//!
//! let tree = to_mdast(":x[*a*]{id=b}", &options.parse).unwrap();
//! match &tree.children().unwrap()[0].children().unwrap()[0] {
//!     Node::Custom(custom) => {
//!         assert_eq!(custom.fields["name"], "x");
//!         assert_eq!(custom.attributes["id"], "b");
//!         assert!(matches!(custom.children[0], Node::Emphasis(_)));
//!     }
//!     node => panic!("expected a directive, not {:?}", node),
//! }
//! ```
