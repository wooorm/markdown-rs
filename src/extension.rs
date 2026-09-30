//! Syntax extensions: constructs to add to markdown.
//!
//! A construct is a state machine driven by the built-in tokenizer, like a
//! micromark construct, so it sees text without container prefixes such as
//! `> `, across lines.
//! Pass constructs in `text_constructs`, `flow_constructs`, and
//! `document_constructs` of [`ParseOptions`], each wrapped by
//! [`construct()`].
//!
//! [`to_mdast()`][crate::to_mdast()] turns each match into a node with
//! [`Construct::to_mdast`].
//! HTML has no node to render, so [`to_html()`][crate::to_html()] writes
//! the source of a match as text.
//!
//! The types check what they can: a step that consumes ends in
//! [`Consumed::next`], so it cannot forget to consume or consume twice;
//! [`Ctx::exit`] closes the innermost token, so it cannot close the wrong
//! one; a flow construct ends before its line ending, so it has no
//! [`Consumed::ok`]; a container keeps a state for its later lines, which
//! only it can give, with [`Ctx::container`]; and a construct goes in the
//! list of its [`Place`].
//! The rules that depend on the input are checked as the construct runs,
//! as [`Construct`] lists them.
//!
//! ## Examples
//!
//! ```
//! use markdown::{
//!     extension::{construct, Construct, Ctx, Step, Text, Token, TokenType},
//!     mdast::{Custom, Node},
//!     to_mdast, ParseOptions,
//! };
//!
//! /// `@` and a lowercase ASCII letter, such as `@a`.
//! struct Mention;
//!
//! /// Tokens of a mention.
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
//! /// States of a mention.
//! #[derive(Clone, Copy)]
//! enum MentionState {
//!     Start,
//!     Name,
//! }
//!
//! impl Construct for Mention {
//!     type Place = Text;
//!     type State = MentionState;
//!     type Token = MentionToken;
//!
//!     fn markers(&self) -> &[u8] {
//!         b"@"
//!     }
//!
//!     fn start(&self) -> MentionState {
//!         MentionState::Start
//!     }
//!
//!     fn step(&self, state: MentionState, t: Ctx<'_, '_, Self>) -> Step<Self> {
//!         match (state, t.current()) {
//!             (MentionState::Start, Some(b'@')) => {
//!                 t.enter(MentionToken::Mention).consume().next(MentionState::Name)
//!             }
//!             (MentionState::Name, Some(b'a'..=b'z')) => {
//!                 t.enter(MentionToken::Name).consume().exit().exit().ok()
//!             }
//!             _ => t.nok(),
//!         }
//!     }
//!
//!     fn to_mdast(&self, tokens: Vec<Token<'_, MentionToken>>) -> Node {
//!         let name = tokens.iter().find(|token| token.name == MentionToken::Name);
//!         Node::Custom(Custom {
//!             name: "mention".into(),
//!             value: name.map(|token| token.value.to_string()),
//!             ..Custom::default()
//!         })
//!     }
//! }
//!
//! let options = ParseOptions {
//!     text_constructs: vec![construct(Mention)],
//!     ..ParseOptions::default()
//! };
//!
//! // `ToString` prefers the value of a custom node.
//! assert_eq!(to_mdast("hi @a, @1", &options)?.to_string(), "hi a, @1");
//! # Ok::<(), markdown::message::Message>(())
//! ```
//!
//! ## Rules the types check
//!
//! These do not compile.
//!
//! `next` needs a consumed byte:
//!
//! ```compile_fail,E0599
//! # use markdown::extension::{Construct, Ctx, Step, Text, Token, TokenType};
//! # #[derive(Clone, Copy, Debug, Eq, PartialEq)]
//! # enum T { A }
//! # impl TokenType for T {
//! #     const ALL: &'static [Self] = &[T::A];
//! #     fn name(self) -> &'static str { "a" }
//! # }
//! # struct C;
//! # impl Construct for C {
//! #     type Place = Text;
//! #     type State = u8;
//! #     type Token = T;
//! #     fn markers(&self) -> &[u8] { b"a" }
//! #     fn start(&self) -> u8 { 0 }
//! #     fn to_mdast(&self, _: Vec<Token<'_, T>>) -> markdown::mdast::Node { unimplemented!() }
//! fn step(&self, state: u8, t: Ctx<'_, '_, Self>) -> Step<Self> {
//!     t.enter(T::A).next(1)
//! }
//! # }
//! ```
//!
//! A step consumes once:
//!
//! ```compile_fail,E0599
//! # use markdown::extension::{Construct, Ctx, Step, Text, Token, TokenType};
//! # #[derive(Clone, Copy, Debug, Eq, PartialEq)]
//! # enum T { A }
//! # impl TokenType for T {
//! #     const ALL: &'static [Self] = &[T::A];
//! #     fn name(self) -> &'static str { "a" }
//! # }
//! # struct C;
//! # impl Construct for C {
//! #     type Place = Text;
//! #     type State = u8;
//! #     type Token = T;
//! #     fn markers(&self) -> &[u8] { b"a" }
//! #     fn start(&self) -> u8 { 0 }
//! #     fn to_mdast(&self, _: Vec<Token<'_, T>>) -> markdown::mdast::Node { unimplemented!() }
//! fn step(&self, state: u8, t: Ctx<'_, '_, Self>) -> Step<Self> {
//!     t.enter(T::A).consume().consume().next(1)
//! }
//! # }
//! ```
//!
//! `retry` does not consume:
//!
//! ```compile_fail,E0599
//! # use markdown::extension::{Construct, Ctx, Step, Text, Token, TokenType};
//! # #[derive(Clone, Copy, Debug, Eq, PartialEq)]
//! # enum T { A }
//! # impl TokenType for T {
//! #     const ALL: &'static [Self] = &[T::A];
//! #     fn name(self) -> &'static str { "a" }
//! # }
//! # struct C;
//! # impl Construct for C {
//! #     type Place = Text;
//! #     type State = u8;
//! #     type Token = T;
//! #     fn markers(&self) -> &[u8] { b"a" }
//! #     fn start(&self) -> u8 { 0 }
//! #     fn to_mdast(&self, _: Vec<Token<'_, T>>) -> markdown::mdast::Node { unimplemented!() }
//! fn step(&self, state: u8, t: Ctx<'_, '_, Self>) -> Step<Self> {
//!     t.enter(T::A).consume().retry(1)
//! }
//! # }
//! ```
//!
//! `attempt` does not consume:
//!
//! ```compile_fail,E0599
//! # use markdown::extension::{Construct, Ctx, Step, Text, Token, TokenType};
//! # #[derive(Clone, Copy, Debug, Eq, PartialEq)]
//! # enum T { A }
//! # impl TokenType for T {
//! #     const ALL: &'static [Self] = &[T::A];
//! #     fn name(self) -> &'static str { "a" }
//! # }
//! # struct C;
//! # impl Construct for C {
//! #     type Place = Text;
//! #     type State = u8;
//! #     type Token = T;
//! #     fn markers(&self) -> &[u8] { b"a" }
//! #     fn start(&self) -> u8 { 0 }
//! #     fn to_mdast(&self, _: Vec<Token<'_, T>>) -> markdown::mdast::Node { unimplemented!() }
//! fn step(&self, state: u8, t: Ctx<'_, '_, Self>) -> Step<Self> {
//!     t.enter(T::A).consume().attempt(1, 2, 3)
//! }
//! # }
//! ```
//!
//! `exit` closes the innermost token, so it takes no name:
//!
//! ```compile_fail,E0061
//! # use markdown::extension::{Construct, Ctx, Step, Text, Token, TokenType};
//! # #[derive(Clone, Copy, Debug, Eq, PartialEq)]
//! # enum T { A }
//! # impl TokenType for T {
//! #     const ALL: &'static [Self] = &[T::A];
//! #     fn name(self) -> &'static str { "a" }
//! # }
//! # struct C;
//! # impl Construct for C {
//! #     type Place = Text;
//! #     type State = u8;
//! #     type Token = T;
//! #     fn markers(&self) -> &[u8] { b"a" }
//! #     fn start(&self) -> u8 { 0 }
//! #     fn to_mdast(&self, _: Vec<Token<'_, T>>) -> markdown::mdast::Node { unimplemented!() }
//! fn step(&self, state: u8, t: Ctx<'_, '_, Self>) -> Step<Self> {
//!     t.enter(T::A).consume().exit(T::A).ok()
//! }
//! # }
//! ```
//!
//! A flow construct ends before its line ending, not after a consumed byte:
//!
//! ```compile_fail,E0599
//! # use markdown::extension::{construct, Construct, Ctx, Flow, Step, Token, TokenType};
//! # #[derive(Clone, Copy, Debug, Eq, PartialEq)]
//! # enum T { A }
//! # impl TokenType for T {
//! #     const ALL: &'static [Self] = &[T::A];
//! #     fn name(self) -> &'static str { "a" }
//! # }
//! # struct C;
//! # impl Construct for C {
//! #     type Place = Flow;
//! #     type State = u8;
//! #     type Token = T;
//! #     fn markers(&self) -> &[u8] { b"a" }
//! #     fn start(&self) -> u8 { 0 }
//! #     fn to_mdast(&self, _: Vec<Token<'_, T>>) -> markdown::mdast::Node { unimplemented!() }
//! fn step(&self, state: u8, t: Ctx<'_, '_, Self>) -> Step<Self> {
//!     t.enter(T::A).consume().exit().ok()
//! }
//! # }
//! ```
//!
//! Only a container keeps a state for its next line:
//!
//! ```compile_fail,E0599
//! # use markdown::extension::{Construct, Ctx, Step, Text, Token, TokenType};
//! # #[derive(Clone, Copy, Debug, Eq, PartialEq)]
//! # enum T { A }
//! # impl TokenType for T {
//! #     const ALL: &'static [Self] = &[T::A];
//! #     fn name(self) -> &'static str { "a" }
//! # }
//! # struct C;
//! # impl Construct for C {
//! #     type Place = Text;
//! #     type State = u8;
//! #     type Token = T;
//! #     fn markers(&self) -> &[u8] { b"a" }
//! #     fn start(&self) -> u8 { 0 }
//! #     fn to_mdast(&self, _: Vec<Token<'_, T>>) -> markdown::mdast::Node { unimplemented!() }
//! fn step(&self, state: u8, t: Ctx<'_, '_, Self>) -> Step<Self> {
//!     t.enter(T::A).consume().exit().container(1)
//! }
//! # }
//! ```
//!
//! A construct goes in the list of its place:
//!
//! ```compile_fail,E0308
//! # use markdown::extension::{construct, Construct, Ctx, Flow, Step, Token, TokenType};
//! # #[derive(Clone, Copy, Debug, Eq, PartialEq)]
//! # enum T { A }
//! # impl TokenType for T {
//! #     const ALL: &'static [Self] = &[T::A];
//! #     fn name(self) -> &'static str { "a" }
//! # }
//! # struct C;
//! # impl Construct for C {
//! #     type Place = Flow;
//! #     type State = u8;
//! #     type Token = T;
//! #     fn markers(&self) -> &[u8] { b"a" }
//! #     fn start(&self) -> u8 { 0 }
//! #     fn to_mdast(&self, _: Vec<Token<'_, T>>) -> markdown::mdast::Node { unimplemented!() }
//! #     fn step(&self, state: u8, t: Ctx<'_, '_, Self>) -> Step<Self> { t.nok() }
//! # }
//! let options = markdown::ParseOptions {
//!     text_constructs: vec![construct(C)],
//!     ..markdown::ParseOptions::default()
//! };
//! ```

use crate::construct::partial_space_or_tab::space_or_tab_min_max;
use crate::event::{Content, Event, Kind, Link, Name, Point};
use crate::mdast;
use crate::state::{Name as StateName, State};
use crate::subtokenize::link_to;
use crate::tokenizer::{move_point_back, Container as TokenizerContainer, Tokenizer};
use crate::unist::Position;
use crate::util::constant::TAB_SIZE;
use crate::util::slice::{Position as SlicePosition, Slice};
use crate::ParseOptions;
use alloc::{borrow::Cow, boxed::Box, string::String, vec, vec::Vec};
use core::{any::Any, convert::TryFrom, fmt, marker::PhantomData, str};

/// Most steps a match takes for each byte from its start through the furthest
/// point it reached.
const STEP_MAX: usize = 256;

/// Most attempts a construct can be in at once.
const ATTEMPT_MAX: usize = 256;

/// Most passes over content, for documents first and then for text; content
/// that needs a later pass does not match.
const CONTENT_MAX: usize = 32;

mod sealed {
    /// Only this crate defines places.
    pub trait Sealed {}

    /// Places where a match, or an attempt, can end with a consumed byte.
    pub trait OkAfterConsume {}

    /// Containers.
    pub trait IsContainer {}
}

/// Where a construct runs.
#[doc(hidden)]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum PlaceKind {
    /// In text.
    Text,
    /// At the start of a line in flow.
    Flow,
    /// A container.
    Container,
}

/// Where a construct runs: [`Text`], [`Flow`], or [`Container`].
///
/// A construct names its place in [`Construct::Place`], and goes in the
/// list of options for that place, so it cannot run anywhere else.
pub trait Place: sealed::Sealed + 'static {
    /// Which place this is.
    #[doc(hidden)]
    const KIND: PlaceKind;
}

/// Text, such as the inside of a paragraph or a heading: constructs in
/// [`ParseOptions::text_constructs`][crate::ParseOptions].
#[derive(Clone, Copy, Debug)]
pub enum Text {}

impl sealed::Sealed for Text {}

impl Place for Text {
    const KIND: PlaceKind = PlaceKind::Text;
}

impl sealed::OkAfterConsume for Text {}

/// Flow, at the start of a line, such as where a fenced code block can
/// start: constructs in
/// [`ParseOptions::flow_constructs`][crate::ParseOptions].
///
/// A flow construct starts after up to 3 columns of indentation, ends
/// before a line ending or at the end, and decides on its first line.
#[derive(Clone, Copy, Debug)]
pub enum Flow {}

impl sealed::Sealed for Flow {}

impl Place for Flow {
    const KIND: PlaceKind = PlaceKind::Flow;
}

/// A container at the start of a line, such as a block quote: constructs
/// in [`ParseOptions::document_constructs`][crate::ParseOptions].
///
/// A container matches the prefix of its first line, after up to 3 columns
/// of indentation, and ends with [`Ctx::container`], with its first token
/// and a token of document content open, which holds the rest of the
/// container.
/// Before that, its first line can hold text content, such as the title of
/// an admonition.
/// The state it gives checks the prefix of each later line, like
/// micromark’s `continuation`: [`Ctx::container`] continues the container,
/// with a state for the line after, and `nok` ends it, though a lazy line
/// can still continue a paragraph in it.
/// A later line starts before its indentation, which the state takes
/// itself, as micromark’s block quote does, a column at a time, as
/// [`Ctx::current`] describes for tabs.
#[derive(Clone, Copy, Debug)]
pub enum Container {}

impl sealed::Sealed for Container {}

impl Place for Container {
    const KIND: PlaceKind = PlaceKind::Container;
}

// In an attempt, such as one that tries a prefix.
impl sealed::OkAfterConsume for Container {}

impl sealed::IsContainer for Container {}

/// Names of the tokens of a construct.
///
/// Usually an enum without data, such as `enum MentionToken { Mention, Name }`.
pub trait TokenType: Copy + Eq + fmt::Debug + 'static {
    /// Every token, so names can be numbered before parsing.
    ///
    /// A construct that enters a token missing here breaks the rule
    /// `token-not-in-all`.
    const ALL: &'static [Self];

    /// Name of the token, as events and debug output show it, such as
    /// `mention`.
    fn name(self) -> &'static str;

    /// Kind of markdown inside the token, if it is parsed as markdown later,
    /// like micromark’s `contentType` (default: `None`).
    ///
    /// Bytes consumed directly in a content token are content; tokens
    /// entered in it, such as a line prefix or a closing fence, are not,
    /// and are not given to [`Construct::to_mdast`].
    fn content(self) -> Option<ContentType> {
        None
    }
}

/// Kind of markdown inside a content token.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum ContentType {
    /// Phrasing, such as a label.
    Text,
    /// Blocks, such as the body of a container (flow constructs and
    /// containers only: a text construct that enters such a token breaks
    /// the rule `document-content-in-text`).
    /// A container can enter text content on its first line only, such as a
    /// title, and breaks `text-content-in-container` on a later line.
    Document,
}

/// Classify a character, such as [`Construct::previous`] or
/// [`Ctx::current_char`], as built-in constructs do, such as attention to
/// decide whether a run can open or close.
pub use crate::util::char::{classify, Kind as CharacterKind};

/// How far a match reads.
///
/// A failed construct is tried again at its next marker, so a construct
/// that reads to the end of a paragraph before it fails makes parsing
/// quadratic, as on `@a @a @a …` for a construct that looks for an end
/// that never comes.
/// A bound keeps parsing linear.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum Reach {
    /// No bound (default): parsing can be quadratic on input that makes
    /// matches fail after reading far.
    Unbounded,
    /// A match reads at most this many bytes from its start: a step at a
    /// byte after them does not match, so a match that ends before a byte
    /// counts it too.
    Bytes(usize),
}

/// A construct, such as a mention.
///
/// Constructs in [`ParseOptions::text_constructs`][crate::ParseOptions] run
/// in text, those in `flow_constructs` at the start of a line in flow, and
/// those in `document_constructs` at the start of a line as containers, such
/// as block quotes; flow constructs and containers start after up to 3
/// columns of indentation.
/// They are tried in order, before built-ins, at their markers, which are
/// never line endings, and outside text never spaces or tabs, which are
/// indentation.
///
/// A construct that breaks one of these rules does not match, so its bytes
/// stay what they would otherwise be (in an attempt, the attempt fails);
/// with the `test-util` feature, it panics instead, naming the rule:
///
/// * every token entered is in [`TokenType::ALL`] (`token-not-in-all`)
/// * every consumed byte is inside a token, and one token holds the others
///   (`consume-outside-token`, `token-after-match`)
/// * a token holds at least one byte (`empty-token`); an empty content
///   token is dropped
/// * the first byte of a match is not content (`content-at-start`)
/// * text content has no blank line between lines with other bytes, like a
///   paragraph, though it can start or end with blank lines
///   (`blank-line-in-content`)
/// * document content is in flow constructs and containers only
///   (`document-content-in-text`), and a line ending in it is content: a
///   token in document content does not consume one
///   (`line-ending-in-document`)
/// * a container enters text content on its first line only
///   (`text-content-in-container`), consumes no line ending
///   (`line-ending-in-container`), and leaves the rest of its first line to
///   its document content (`consume-in-container-content`); its tokens on
///   later lines are in its content, like `> `
/// * a container ends its first line with its token and its content token
///   open, and a later line with its tokens closed, maybe without bytes
///   (`container-tokens`), with [`Ctx::container`], not `ok`
///   (`ok-in-container`), and not in an attempt (`container-in-attempt`)
/// * inside content, a token holds no content (`content-in-content`), and
///   starts before the content of its line, like a line prefix
///   (`token-after-content`)
/// * `exit` has a token to close (`exit-without-token`); an attempt closes
///   only tokens it opened (`exit-before-attempt`)
/// * after consuming a line ending, a step returns `next`
///   (`enter-after-line-ending`, `ok-after-line-ending`); in text, it can
///   first `exit` tokens that end there (`exit-after-line-ending`)
/// * in flow, an attempt does not consume a line ending
///   (`line-ending-in-attempt`)
/// * `ok` comes after at least one byte, with every token closed
///   (`ok-without-bytes`, `ok-with-open-tokens`); in flow, at a line ending
///   or the end (`ok-before-line-end`); in an attempt, with the tokens it
///   opened closed
/// * tokens start and end between characters, not inside one; a tab is
///   one character, but a line can start inside it, after container
///   prefixes, a container prefix can take part of it, and so can a token
///   in document content, such as a line prefix, as the indent of fenced
///   code can
///   (`token-inside-character`)
///
/// A match takes at most 256 steps for each byte from its start through the
/// furthest point it reached, failed attempts included, and moving past a
/// container prefix counts a step for each of its bytes; and a match reads
/// at most as far as [`reach`][Self::reach] allows; past either, it does
/// not match.
/// Attempts nest at most 256 deep; a deeper attempt fails.
/// A flow construct decides on its first line: after it consumed a line
/// ending, it does not return `nok` outside an attempt
/// (`nok-after-first-line`).
/// If it does, or reaches a limit there, it ends before its last line
/// ending, and the next line is parsed as other markdown: parsing never
/// fails because of a construct.
/// Content is parsed in at most 32 passes over the whole document: first
/// the passes for document content, then one for each level of text content,
/// the text of built-in constructs such as paragraphs included.
/// Content that needs a later pass does not match, so deep document content
/// anywhere leaves fewer levels for text content.
pub trait Construct: 'static {
    /// Where the construct runs: [`Text`], [`Flow`], or [`Container`].
    type Place: Place;
    /// States of the construct, usually an enum; a variant can carry what
    /// the construct has seen, such as the size of an opening fence.
    type State: Clone + 'static;
    /// Names of its tokens.
    type Token: TokenType;

    /// Bytes this construct can start at.
    fn markers(&self) -> &[u8];

    /// State to start a match in.
    fn start(&self) -> Self::State;

    /// How far a match reads (default: [`Reach::Unbounded`]).
    fn reach(&self) -> Reach {
        Reach::Unbounded
    }

    /// Whether the construct can start after `previous`, the character
    /// before it, as in micromark.
    ///
    /// In text, it is `None` at the start, such as the first line of a
    /// paragraph, `'\n'` at the start of a later line, after container
    /// prefixes, and `None` after a character escape.
    /// In flow and containers, it is `None` on the first line of the
    /// document, and `'\n'` on a later line, before indentation and after
    /// the prefixes of outer containers.
    fn previous(&self, previous: Option<char>) -> bool {
        let _ = previous;
        true
    }

    /// Take one step at `state`.
    fn step(&self, state: Self::State, t: Ctx<'_, '_, Self>) -> Step<Self>;

    /// Turn the tokens of one match into a node.
    ///
    /// `tokens[0]` is the outermost token, the first one entered.
    fn to_mdast(&self, tokens: Vec<Token<'_, Self::Token>>) -> mdast::Node;
}

/// A construct ready to be passed in options, made by [`construct()`].
pub struct BoxedConstruct<P: Place> {
    inner: Box<dyn ErasedConstruct>,
    place: PhantomData<P>,
}

impl<P: Place> fmt::Debug for BoxedConstruct<P> {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("BoxedConstruct")
    }
}

/// Wrap a construct, to pass it in the options for its place:
/// [`ParseOptions::text_constructs`][crate::ParseOptions] for [`Text`],
/// `flow_constructs` for [`Flow`], and `document_constructs` for
/// [`Container`].
pub fn construct<C: Construct>(construct: C) -> BoxedConstruct<C::Place> {
    BoxedConstruct {
        inner: Box::new(Wrap(construct)),
        place: PhantomData,
    }
}

/// What to do after a step, made by [`Ctx`] and [`Consumed`].
pub struct Step<C: Construct + ?Sized> {
    kind: StepKind<C::State>,
    report: Report,
}

impl<C: Construct + ?Sized> fmt::Debug for Step<C> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Step").finish_non_exhaustive()
    }
}

/// What a step does, with its states.
enum StepKind<S> {
    Next(S),
    Retry(S),
    Attempt { state: S, ok: S, nok: S },
    Container(S),
    Ok,
    Nok,
}

/// What a step did, for the checks.
#[derive(Clone, Copy, Debug, Default)]
struct Report {
    consumed: bool,
    /// Whether a line ending was consumed in this step.
    line_ending: bool,
    /// The first rule the step broke.
    broken: Option<&'static str>,
    /// Whether the step reached a limit, such as the depth of content.
    limit: bool,
}

/// Token of a construct: a name, the text it spans, and where.
#[derive(Clone, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub struct Token<'a, T> {
    /// Name given to `enter`.
    pub name: T,
    /// Source text, without container prefixes and content (empty for a
    /// content token).
    pub value: Cow<'a, str>,
    /// Content of a content token, parsed as markdown.
    pub children: Vec<mdast::Node>,
    /// Positional info.
    pub position: Position,
}

/// Look up tokens of a match by name, in [`Construct::to_mdast`].
///
/// ```
/// # use markdown::extension::{Token, TokenSlice, TokenType};
/// # use markdown::unist::{Point, Position};
/// # #[derive(Clone, Copy, Debug, Eq, PartialEq)]
/// # enum T { Name, Label }
/// # impl TokenType for T {
/// #     const ALL: &'static [Self] = &[T::Name, T::Label];
/// #     fn name(self) -> &'static str { "t" }
/// # }
/// fn name_of<'a>(tokens: &'a [Token<'_, T>]) -> &'a str {
///     tokens.value(T::Name).unwrap_or_default()
/// }
/// ```
pub trait TokenSlice<'a, T> {
    /// The first token named `name`.
    fn token(&self, name: T) -> Option<&Token<'a, T>>;
    /// The value of the first token named `name`.
    fn value(&self, name: T) -> Option<&str>;
    /// Take the children of the first content token named `name`, leaving
    /// it empty; empty if there is none.
    fn take_children(&mut self, name: T) -> Vec<mdast::Node>;
}

impl<'a, T: TokenType> TokenSlice<'a, T> for [Token<'a, T>] {
    fn token(&self, name: T) -> Option<&Token<'a, T>> {
        self.iter().find(|token| token.name == name)
    }

    fn value(&self, name: T) -> Option<&str> {
        self.token(name).map(|token| &*token.value)
    }

    fn take_children(&mut self, name: T) -> Vec<mdast::Node> {
        self.iter_mut()
            .find(|token| token.name == name)
            .map(|token| core::mem::take(&mut token.children))
            .unwrap_or_default()
    }
}

/// A token whose name is its index in [`TokenType::ALL`].
pub(crate) type RawToken<'a> = Token<'a, u16>;

/// What an interned token name is.
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) enum TokenKind {
    /// A token of the construct.
    Token,
    /// A token whose inside is parsed as markdown.
    Content(Content),
    /// A token of the construct inside a content token, such as a line
    /// prefix: its bytes are not content, and compilers skip it.
    InContent,
}

/// Index of a construct, index of a token name in its [`TokenType::ALL`],
/// that name, and what it is, interned per parse: events of constructs
/// refer to it by index.
pub(crate) type TokenName = (u8, u16, &'static str, TokenKind);

/// What a step does, after the construct's states are stored.
#[derive(Clone, Copy, Debug)]
enum RawStep {
    Next,
    Retry,
    Attempt,
    Container,
    Ok,
    Nok,
}

/// A construct, without its types, so constructs of different types can be
/// stored together.
trait ErasedConstruct {
    fn place(&self) -> PlaceKind;
    fn markers(&self) -> &[u8];
    fn reach(&self) -> Reach;
    fn previous(&self, previous: Option<char>) -> bool;
    fn token_count(&self) -> usize;
    /// Keep the state a container gave for its next line in `slot`, as
    /// container `id`, or as a new one; return its id.
    fn store_container(&self, slot: &mut Option<Box<dyn Any>>, id: Option<usize>) -> usize;
    /// Start a later line of container `id` from the state it kept.
    fn resume_container(&self, slot: &mut Option<Box<dyn Any>>, id: usize);
    /// Put the start state in `slot`.
    fn start(&self, slot: &mut Option<Box<dyn Any>>);
    /// Take a step from the state in `slot`, and store the next one there.
    fn step(&self, slot: &mut Option<Box<dyn Any>>, inner: Inner<'_, '_>) -> (RawStep, Report);
    /// End the innermost attempt in `slot`: go on in its `ok` or `nok`
    /// state.
    fn resume(&self, slot: &mut Option<Box<dyn Any>>, ok: bool);
    fn to_mdast(&self, tokens: Vec<RawToken>) -> mdast::Node;
}

/// A construct of a known type.
struct Wrap<C>(C);

/// Where a construct keeps its state between steps.
struct Machine<S> {
    state: Option<S>,
    /// The `ok` and `nok` states of the attempts it is in.
    attempts: Vec<(S, S)>,
    /// The state a container gave for its next line.
    container: Option<S>,
    /// The states that open containers of this construct keep for their
    /// next lines, by id, so a line does not allocate.
    containers: Vec<S>,
}

impl<C: Construct> ErasedConstruct for Wrap<C> {
    fn place(&self) -> PlaceKind {
        C::Place::KIND
    }

    fn markers(&self) -> &[u8] {
        self.0.markers()
    }

    fn reach(&self) -> Reach {
        self.0.reach()
    }

    fn previous(&self, previous: Option<char>) -> bool {
        self.0.previous(previous)
    }

    fn token_count(&self) -> usize {
        // A name can be interned as a token and, inside content, as a token
        // in content.
        C::Token::ALL.len() * 2
    }

    fn store_container(&self, slot: &mut Option<Box<dyn Any>>, id: Option<usize>) -> usize {
        let machine = slot
            .as_mut()
            .and_then(|slot| slot.downcast_mut::<Machine<C::State>>())
            .expect("expected state of construct");
        let state = machine
            .container
            .take()
            .expect("expected state of container");
        if let Some(id) = id {
            machine.containers[id] = state;
            id
        } else {
            machine.containers.push(state);
            machine.containers.len() - 1
        }
    }

    fn resume_container(&self, slot: &mut Option<Box<dyn Any>>, id: usize) {
        let machine = slot
            .as_mut()
            .and_then(|slot| slot.downcast_mut::<Machine<C::State>>())
            .expect("expected state of construct");
        machine.state = Some(machine.containers[id].clone());
        machine.attempts.clear();
    }

    fn start(&self, slot: &mut Option<Box<dyn Any>>) {
        let state = Some(self.0.start());
        match slot
            .as_mut()
            .and_then(|slot| slot.downcast_mut::<Machine<C::State>>())
        {
            // Reused for every match, so a state without data allocates
            // nothing, and one with data allocates once per parse.
            Some(machine) => {
                machine.state = state;
                machine.attempts.clear();
            }
            None => {
                *slot = Some(Box::new(Machine {
                    state,
                    attempts: Vec::new(),
                    container: None,
                    containers: Vec::new(),
                }));
            }
        }
    }

    fn step(&self, slot: &mut Option<Box<dyn Any>>, inner: Inner<'_, '_>) -> (RawStep, Report) {
        let machine = slot
            .as_mut()
            .and_then(|slot| slot.downcast_mut::<Machine<C::State>>())
            .expect("expected state of construct");
        let state = machine.state.take().expect("expected state");
        let step = self.0.step(
            state,
            Ctx {
                inner,
                construct: PhantomData,
            },
        );
        let raw = match step.kind {
            StepKind::Next(state) => {
                machine.state = Some(state);
                RawStep::Next
            }
            StepKind::Retry(state) => {
                machine.state = Some(state);
                RawStep::Retry
            }
            StepKind::Attempt { state, ok, nok } => {
                machine.state = Some(state);
                machine.attempts.push((ok, nok));
                RawStep::Attempt
            }
            StepKind::Container(state) => {
                machine.container = Some(state);
                RawStep::Container
            }
            StepKind::Ok => RawStep::Ok,
            StepKind::Nok => RawStep::Nok,
        };
        (raw, step.report)
    }

    fn resume(&self, slot: &mut Option<Box<dyn Any>>, ok: bool) {
        let machine = slot
            .as_mut()
            .and_then(|slot| slot.downcast_mut::<Machine<C::State>>())
            .expect("expected state of construct");
        let next = machine.attempts.pop().expect("expected attempt");
        machine.state = Some(if ok { next.0 } else { next.1 });
    }

    fn to_mdast(&self, tokens: Vec<RawToken>) -> mdast::Node {
        self.0.to_mdast(
            tokens
                .into_iter()
                .map(|token| Token {
                    name: C::Token::ALL[usize::from(token.name)],
                    value: token.value,
                    children: token.children,
                    position: token.position,
                })
                .collect(),
        )
    }
}

/// Where content is in its lines: blank lines can come at its start or end,
/// but not between lines with other bytes.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
enum ContentLines {
    /// Before a line with a byte other than a space or tab.
    #[default]
    Start,
    /// After such a line.
    Text,
    /// After a blank line after such a line.
    AfterBlank,
}

/// State of constructs in a tokenizer, boxed, so the tokenizer stays small
/// without them.
#[derive(Debug, Default)]
pub(crate) struct ExtensionState {
    /// Where the current construct started.
    start: usize,
    /// Number of events when the current construct started.
    events: usize,
    /// Steps the current match took, where a step that moved past more than
    /// one byte, such as a container prefix, counts each byte.
    steps: usize,
    /// Where the last step of the current match was.
    at: usize,
    /// Furthest byte a step of the current match was at.
    furthest: usize,
    /// Attempts the current construct is in.
    attempts: Vec<AttemptFrame>,
    /// Construct being tried.
    index: u8,
    /// Construct to try next at the current byte.
    pub(crate) next: u8,
    /// State of each construct, by index, made when it first runs.
    slots: Vec<Option<Box<dyn Any>>>,
    /// Interned names of the open tokens of the current match.
    open: Vec<u16>,
    /// Whether to keep the initial and final whitespace of this text, which
    /// is content of a construct, like micromark’s `_contentTypeTextTrailing`.
    pub(crate) keep_whitespace: bool,
    /// Index in `open` of the current content token, if it is open.
    content_at: usize,
    /// Index in `events` of the enter of the current content token.
    content_enter: usize,
    /// For each line ending in the current content, the enter of its chunk,
    /// and the point before it.
    content_eols: Vec<(usize, Point)>,
    /// Line of the last byte of the open content, `0` if none.
    content_line: usize,
    /// Whether the current line of the open content has only spaces and tabs.
    content_blank: bool,
    /// Where the open content is in its blank lines and lines of text.
    content_lines: ContentLines,
    /// Enter of the last chunk of the current content token.
    last_chunk: Option<usize>,
    /// Where the current flow construct is in its lines.
    line: FlowLine,
    /// Where the current flow construct was before its last line ending,
    /// to end it there if it fails on a later line.
    line_frame: Option<LineFrame>,
    /// Open tokens of the current flow construct before its last line
    /// ending, kept here so a line does not allocate.
    line_open: Vec<u16>,
    /// Why the current step does not match, if it does not.
    nok: NokCause,
    /// Columns of indentation before the current flow construct.
    indent: usize,
    /// Whether the current match checks whether a container continues.
    continuation: bool,
}

/// Where a flow construct is in its lines.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
enum FlowLine {
    /// On its first line, where it can fail.
    #[default]
    First,
    /// Past a line ending, so it matches, or ends before that line.
    Committed,
    /// At a line ending before a lazy line or the end, where it ends.
    Lazy,
}

/// Where a flow construct was before its last line ending.
#[derive(Debug)]
struct LineFrame {
    /// The tokenizer.
    progress: crate::tokenizer::Progress,
    /// The content, as an attempt keeps it.
    frame: AttemptFrame,
    /// Index in `open` of the content token.
    content_at: usize,
}

/// Why a step does not match.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
enum NokCause {
    /// The construct returned `nok`.
    #[default]
    Construct,
    /// It broke a rule, which `test-util` reports already.
    Rule,
    /// It reached a limit.
    Limit,
}

/// An attempt a construct is in, and what undoing it restores.
#[derive(Debug)]
struct AttemptFrame {
    /// Open tokens of the match when the attempt started.
    open_len: usize,
    /// Start of the line when the attempt started, which the tokenizer does
    /// not restore.
    line_start: Point,
    /// Line of the last byte of content when the attempt started.
    content_line: usize,
    /// Whether the line of content was blank when the attempt started.
    content_blank: bool,
    /// Where the content was in its lines when the attempt started.
    content_lines: ContentLines,
    /// Enter of the content token when the attempt started.
    content_enter: usize,
    /// Line endings of the content when the attempt started.
    content_eols: usize,
    /// Last chunk when the attempt started.
    last_chunk: Option<usize>,
}

/// State of constructs in `tokenizer`, to read, once one started.
pub(crate) fn ext<'t>(tokenizer: &'t Tokenizer) -> &'t ExtensionState {
    tokenizer
        .tokenize_state
        .extension
        .as_deref()
        .expect("expected state of constructs")
}

/// State of constructs in `tokenizer`, created if needed.
pub(crate) fn ext_mut<'t>(tokenizer: &'t mut Tokenizer) -> &'t mut ExtensionState {
    tokenizer
        .tokenize_state
        .extension
        .get_or_insert_with(Box::default)
}

/// The tokenizer, as a step sees it, without the type of its construct.
struct Inner<'t, 'a> {
    tokenizer: &'t mut Tokenizer<'a>,
    index: u8,
    place: PlaceKind,
    report: Report,
}

impl Inner<'_, '_> {
    /// Note that the step broke `rule`; the first one counts.
    fn broke(&mut self, rule: &'static str) {
        if self.report.broken.is_none() {
            self.report.broken = Some(rule);
        }
    }
}

/// The tokenizer, as a step of construct `C` sees it, before it consumes.
///
/// Each method takes it by value, so a step reads as one chain, such as
/// `t.enter(Token::A).consume().next(State::B)`.
pub struct Ctx<'t, 'a, C: Construct + ?Sized> {
    inner: Inner<'t, 'a>,
    construct: PhantomData<fn(&C)>,
}

/// The tokenizer, as a step of construct `C` sees it, after it consumed a
/// byte: it can close and start tokens, then go to the next state.
pub struct Consumed<'t, 'a, C: Construct + ?Sized> {
    inner: Inner<'t, 'a>,
    construct: PhantomData<fn(&C)>,
}

impl<C: Construct + ?Sized> fmt::Debug for Ctx<'_, '_, C> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Ctx")
            .field("current", &self.current())
            .finish_non_exhaustive()
    }
}

impl<C: Construct + ?Sized> fmt::Debug for Consumed<'_, '_, C> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Consumed").finish_non_exhaustive()
    }
}

impl<'t, 'a, C: Construct + ?Sized> Ctx<'t, 'a, C> {
    /// Current byte, or `None` at the end.
    ///
    /// Line endings (CR, LF, CR+LF) are all `b'\n'`.
    /// A tab is `b'\t'` at its first column and `b' '` at each column after,
    /// and `consume` takes one column.
    /// In flow, a line ending before a lazy line or the end is `None`: the
    /// construct ends there.
    pub fn current(&self) -> Option<u8> {
        current(&self.inner)
    }

    /// Current character, or `None` at the end or inside a character.
    ///
    /// Line endings are `'\n'`, as with [`current`][Self::current].
    /// `consume` takes one byte, or one column of a tab: a character takes
    /// as many steps as it has bytes or columns.
    pub fn current_char(&self) -> Option<char> {
        let tokenizer = &self.inner.tokenizer;
        match self.current() {
            Some(b'\n') => Some('\n'),
            Some(_) if tokenizer.point.vs > 0 && !tokenizer.at_line_start() => None,
            Some(_) if tokenizer.point.vs > 0 => Some(' '),
            Some(0x80..=0xBF) | None => None,
            Some(_) => {
                crate::util::char::after_index(tokenizer.parse_state.bytes, tokenizer.point.index)
            }
        }
    }

    /// Parse options, such as which built-in constructs are on, like
    /// micromark’s `parser.constructs`.
    pub fn options(&self) -> &ParseOptions {
        self.inner.tokenizer.parse_state.options
    }

    /// Columns of indentation (0 to 3) before a flow construct or container,
    /// which it starts after, like micromark’s `linePrefix`; `0` in text and
    /// on later lines of a container.
    pub fn indent(&self) -> usize {
        ext(self.inner.tokenizer).indent
    }

    /// Start a token.
    #[must_use]
    pub fn enter(mut self, token: C::Token) -> Self {
        enter(&mut self.inner, token);
        self
    }

    /// End the innermost open token.
    #[must_use]
    pub fn exit(mut self) -> Self {
        exit(&mut self.inner);
        self
    }

    /// Consume the current byte, which must be inside a token.
    ///
    /// Does nothing at the end.
    #[must_use]
    pub fn consume(mut self) -> Consumed<'t, 'a, C> {
        consume(&mut self.inner);
        Consumed {
            inner: self.inner,
            construct: PhantomData,
        }
    }

    /// Go to `state` at the current byte, without consuming.
    #[must_use]
    pub fn retry(self, state: C::State) -> Step<C> {
        Step {
            kind: StepKind::Retry(state),
            report: self.inner.report,
        }
    }

    /// Try `state` at the current byte, like micromark’s `effects.attempt`:
    /// if it reaches `ok`, go to `ok` where the attempt stopped; if it
    /// reaches `nok`, undo it, and go to `nok` at the byte where it started.
    ///
    /// The states are values, so `nok` goes on with what the construct knew
    /// when the attempt started.
    #[must_use]
    pub fn attempt(self, state: C::State, ok: C::State, nok: C::State) -> Step<C> {
        Step {
            kind: StepKind::Attempt { state, ok, nok },
            report: self.inner.report,
        }
    }

    /// The construct matched.
    #[must_use]
    pub fn ok(self) -> Step<C> {
        Step {
            kind: StepKind::Ok,
            report: self.inner.report,
        }
    }

    /// The construct did not match: its events are discarded.
    #[must_use]
    pub fn nok(self) -> Step<C> {
        Step {
            kind: StepKind::Nok,
            report: self.inner.report,
        }
    }
}

impl<C: Construct + ?Sized> Consumed<'_, '_, C> {
    /// Start a token, which holds the bytes after the consumed one.
    ///
    /// Not after a line ending: the next line starts after its container
    /// prefixes, at the next step.
    #[must_use]
    pub fn enter(mut self, token: C::Token) -> Self {
        enter(&mut self.inner, token);
        self
    }

    /// End the innermost open token.
    #[must_use]
    pub fn exit(mut self) -> Self {
        exit(&mut self.inner);
        self
    }

    /// Go to `state` at the next byte.
    #[must_use]
    pub fn next(self, state: C::State) -> Step<C> {
        Step {
            kind: StepKind::Next(state),
            report: self.inner.report,
        }
    }
}

impl<C: Construct + ?Sized> Ctx<'_, '_, C>
where
    C::Place: sealed::IsContainer,
{
    /// The container matched its first line, or continues on a later line:
    /// `state` checks the prefix of the next line.
    #[must_use]
    pub fn container(self, state: C::State) -> Step<C> {
        Step {
            kind: StepKind::Container(state),
            report: self.inner.report,
        }
    }
}

impl<C: Construct + ?Sized> Consumed<'_, '_, C>
where
    C::Place: sealed::IsContainer,
{
    /// The container matched its first line, or continues on a later line,
    /// ending with the consumed byte: `state` checks the prefix of the next
    /// line.
    #[must_use]
    pub fn container(self, state: C::State) -> Step<C> {
        Step {
            kind: StepKind::Container(state),
            report: self.inner.report,
        }
    }
}

impl<C: Construct + ?Sized> Consumed<'_, '_, C>
where
    C::Place: sealed::OkAfterConsume,
{
    /// The construct, or an attempt, matched, ending with the consumed byte;
    /// not in flow, as a flow construct ends before its line ending.
    #[must_use]
    pub fn ok(mut self) -> Step<C> {
        if self.inner.report.line_ending {
            self.inner.broke("ok-after-line-ending");
        }
        Step {
            kind: StepKind::Ok,
            report: self.inner.report,
        }
    }
}

/// Current byte, as a construct sees it.
fn current(inner: &Inner) -> Option<u8> {
    let tokenizer = &inner.tokenizer;
    if inner.place == PlaceKind::Flow
        && tokenizer.current == Some(b'\n')
        && ext(tokenizer).line == FlowLine::Lazy
    {
        None
    } else {
        tokenizer.current
    }
}

/// Start a token named `token` of construct `C`.
fn enter<T: TokenType>(inner: &mut Inner, token: T) {
    if inner.report.broken.is_some() || inner.report.limit {
        return;
    }
    let tokenizer = &*inner.tokenizer;
    let state = ext(tokenizer);
    let content = token.content().map(|content| match content {
        ContentType::Text => Content::Text,
        ContentType::Document => Content::Document,
    });
    // A container continues in its content.
    let inside_content = state.continuation
        || state.open.get(state.content_at).map_or(false, |id| {
            matches!(
                tokenizer.parse_state.extension_names.borrow()[usize::from(*id)].3,
                TokenKind::Content(_)
            )
        });
    let is_content = content.is_some();
    let is_document = content == Some(Content::Document);
    let kind = match content {
        Some(content) => TokenKind::Content(content),
        None if inside_content => TokenKind::InContent,
        None => TokenKind::Token,
    };
    // The first token starts the match, and holds the others.
    let can_start = tokenizer.events.len() == state.events || !state.open.is_empty();
    let position = T::ALL.iter().position(|known| *known == token);
    let rule = if position.is_none() {
        Some("token-not-in-all")
    } else if inner.report.line_ending {
        Some("enter-after-line-ending")
    } else if is_document && inner.place == PlaceKind::Text {
        // Document content is parsed before text, so text cannot hold it.
        Some("document-content-in-text")
    } else if is_content
        && !is_document
        && inner.place == PlaceKind::Container
        && ext(tokenizer).continuation
    {
        // Tokens of a later line are prefixes in the document content.
        Some("text-content-in-container")
    } else if inside_content && is_content {
        Some("content-in-content")
    } else if !can_start {
        Some("token-after-match")
    } else if is_content && tokenizer.point.index == state.start {
        // Content smaller than its match cannot match again forever.
        Some("content-at-start")
    } else if kind == TokenKind::InContent && state.content_line == tokenizer.point.line {
        // A token in content comes before any content on its line, such as
        // a line prefix: after content, it would split that content.
        Some("token-after-content")
    } else if !at_boundary(
        tokenizer,
        inner.place,
        open_content(tokenizer) == Some(Content::Document),
    ) {
        Some("token-inside-character")
    } else {
        None
    };
    // A limit, not a rule: the content does not match. The content of a
    // container is parsed with the container.
    let is_too_deep = is_content
        && inner.place != PlaceKind::Container
        && tokenizer.parse_state.content_depth >= CONTENT_MAX;

    if let Some(rule) = rule {
        inner.broke(rule);
        return;
    }
    if is_too_deep {
        inner.report.limit = true;
        return;
    }

    let position = position.expect("expected a token in `TokenType::ALL`, as checked");
    // Parsing checks that the names of all tokens fit in a `u16`.
    let position = u16::try_from(position).expect("expected at most 65,535 tokens");
    let id = intern(inner.tokenizer, inner.index, position, token.name(), kind);
    let content_at = ext(inner.tokenizer).open.len();
    enter_token(inner.tokenizer, id);
    if is_content {
        let content_enter = inner.tokenizer.events.len() - 1;
        let state = ext_mut(inner.tokenizer);
        state.content_at = content_at;
        state.content_line = 0;
        state.content_blank = true;
        state.content_lines = ContentLines::Start;
        state.content_enter = content_enter;
        state.content_eols.clear();
        state.last_chunk = None;
    }
}

/// End the innermost open token.
fn exit(inner: &mut Inner) {
    if inner.report.broken.is_some() || inner.report.limit {
        return;
    }
    let tokenizer = &*inner.tokenizer;
    let is_content = ext(tokenizer).open.last().map(|id| {
        matches!(
            tokenizer.parse_state.extension_names.borrow()[usize::from(*id)].3,
            TokenKind::Content(_)
        )
    });
    let state = ext(tokenizer);
    // An attempt cannot close tokens opened before it.
    let is_before_attempt = state
        .attempts
        .last()
        .map_or(false, |frame| state.open.len() <= frame.open_len);
    let rule = match is_content {
        // In flow, the line ending is consumed after the step.
        _ if inner.report.line_ending && inner.place == PlaceKind::Flow => {
            Some("exit-after-line-ending")
        }
        None => Some("exit-without-token"),
        Some(_) if is_before_attempt => Some("exit-before-attempt"),
        Some(_) if !at_boundary(tokenizer, inner.place, is_in_document_content(tokenizer)) => {
            Some("token-inside-character")
        }
        _ => None,
    };
    if let Some(rule) = rule {
        inner.broke(rule);
        return;
    }
    let is_content = is_content == Some(true);
    let is_text_content = is_content && content_of_top(inner.tokenizer) == Content::Text;

    if is_chunk_open(inner.tokenizer) {
        exit_chunk(inner.tokenizer);
    }

    if is_text_content {
        let enter = ext(inner.tokenizer).content_enter;
        if let Some(start) = trim_content_end(inner.tokenizer, enter) {
            // Line endings of flow left out go after the content, which ends
            // before them, or is no content without anything before them.
            let tokenizer = &mut *inner.tokenizer;
            let tail = tokenizer.events.split_off(start);
            if tokenizer.events.len() == enter + 1 {
                tokenizer.events.pop();
                tokenizer.stack.pop();
                ext_mut(tokenizer).open.pop();
            } else {
                exit_token(tokenizer);
                let exit = tokenizer.events.last_mut().expect("expected exit");
                exit.point = tail[0].point.clone();
            }
            tokenizer.events.extend(tail);
            return;
        }
    }

    if !top_is_empty(inner.tokenizer) {
        exit_token(inner.tokenizer);
    } else if is_content {
        // Empty content is no content.
        let tokenizer = &mut *inner.tokenizer;
        tokenizer.events.pop();
        tokenizer.stack.pop();
        ext_mut(tokenizer).open.pop();
    } else {
        inner.broke("empty-token");
    }
}

/// Consume the current byte.
fn consume(inner: &mut Inner) {
    if current(inner).is_none() || inner.report.broken.is_some() || inner.report.limit {
        return;
    }

    if ext(inner.tokenizer).open.is_empty() {
        inner.broke("consume-outside-token");
        return;
    }

    let is_eol = inner.tokenizer.current == Some(b'\n');
    let in_content = is_content_top(inner.tokenizer);

    if in_content && content_of_top(inner.tokenizer) == Content::Text {
        let is_blank = matches!(inner.tokenizer.current, Some(b'\t' | b' '));
        let state = ext_mut(inner.tokenizer);
        // Blank lines at the start or end of content are fine; one between
        // other lines is found at the first byte of the next line.
        if is_eol && state.content_blank && state.content_lines == ContentLines::Text {
            state.content_lines = ContentLines::AfterBlank;
        }
        if !is_eol && !is_blank {
            if state.content_lines == ContentLines::AfterBlank {
                inner.broke("blank-line-in-content");
                return;
            }
            state.content_lines = ContentLines::Text;
        }
    }

    // The line endings of a document go to the document, and a container
    // leaves the rest of its lines to its content.
    if is_eol && !in_content && open_content(inner.tokenizer) == Some(Content::Document) {
        let rule = if inner.place == PlaceKind::Container {
            "line-ending-in-container"
        } else {
            "line-ending-in-document"
        };
        inner.broke(rule);
        return;
    }
    // Text content, such as a title, takes bytes of the first line.
    let in_document = in_content && content_of_top(inner.tokenizer) == Content::Document;
    if inner.place == PlaceKind::Container && (is_eol || in_document) {
        let rule = if is_eol {
            "line-ending-in-container"
        } else {
            "consume-in-container-content"
        };
        inner.broke(rule);
        return;
    }

    let tokenizer = &mut *inner.tokenizer;

    // In flow, the line ending is consumed after the step, once the next
    // line is known to continue the construct.
    if inner.place == PlaceKind::Flow && is_eol {
        if !ext(tokenizer).attempts.is_empty() {
            inner.broke("line-ending-in-attempt");
            return;
        }
    } else if in_content {
        // Content goes into linked chunks, one per line, parsed later.
        if !is_chunk_open(tokenizer) {
            let content = content_of_top(tokenizer);
            enter_chunk(tokenizer, Name::ExtensionChunk, content);
            tokenizer.stack.pop();
        }
        let (line, is_blank) = (
            tokenizer.point.line,
            matches!(tokenizer.current, Some(b'\t' | b' ')),
        );
        let mut before = tokenizer.point.clone();
        move_point_back(tokenizer, &mut before);
        let state = ext_mut(tokenizer);
        state.content_line = line;
        if !is_blank {
            state.content_blank = is_eol;
        }
        if is_eol {
            let chunk = state.last_chunk.expect("expected a chunk");
            state.content_eols.push((chunk, before));
        }
        tokenizer.consume();
        if is_eol {
            exit_chunk(tokenizer);
        }
    } else if is_eol {
        tokenizer.enter(Name::LineEnding);
        tokenizer.consume();
        tokenizer.exit(Name::LineEnding);
    } else {
        tokenizer.consume();
    }

    inner.report.line_ending = is_eol;
    inner.report.consumed = true;
}

/// Whether the innermost open token has no bytes yet.
fn top_is_empty(tokenizer: &Tokenizer) -> bool {
    // Like enter points: before the CR of a CR+LF.
    let mut point = tokenizer.point.clone();
    move_point_back(tokenizer, &mut point);
    tokenizer.events.last().map_or(false, |event| {
        event.kind == Kind::Enter && event.point.index == point.index && event.point.vs == point.vs
    })
}

/// Leave a final line ending of the content that starts at the event
/// `enter`, and the blank lines after it, out of the content, as the last
/// line ending of a paragraph is not its text: text never ends after a line
/// ending.
///
/// Returns where line endings of flow that are left out start, which then go
/// after the content.
fn trim_content_end(tokenizer: &mut Tokenizer, enter: usize) -> Option<usize> {
    let bytes = tokenizer.parse_state.bytes;
    let eols = core::mem::take(&mut ext_mut(tokenizer).content_eols);
    let events = &mut tokenizer.events;
    let chunks = (enter + 1..events.len())
        .filter(|index| events[*index].kind == Kind::Enter && events[*index].link.is_some())
        .collect::<Vec<_>>();
    let is_blank = |events: &[Event], index: usize| {
        bytes[events[index].point.index..events[index + 1].point.index]
            .iter()
            .all(|byte| matches!(byte, b'\t' | b'\n' | b'\r' | b' '))
    };

    // The chunk whose line ending ends the text: the last one with text, or
    // else the first.
    let keep = chunks
        .iter()
        .rposition(|index| !is_blank(events, *index))
        .unwrap_or(0);
    let is_line_ending = |events: &[Event], index: usize| {
        let slice = &bytes[events[index].point.index..events[index + 1].point.index];
        !slice.is_empty() && slice.iter().all(|byte| matches!(byte, b'\n' | b'\r'))
    };
    let drop_from = match chunks
        .get(keep)
        .and_then(|chunk| eols.iter().find(|(eol_chunk, _)| eol_chunk == chunk))
    {
        // A line ending at the end of the chunk is left out.
        Some((chunk, before)) if before.index > events[*chunk].point.index => {
            events[*chunk + 1].point = before.clone();
            keep + 1
        }
        // A chunk of only a line ending is no content, such as one that a
        // flow construct crosses lines with.
        Some(_) => keep,
        None if chunks
            .get(keep)
            .map_or(false, |chunk| is_line_ending(events, *chunk)) =>
        {
            keep
        }
        None => keep + 1,
    };
    if drop_from >= chunks.len() {
        return None;
    }

    for index in &chunks[drop_from..] {
        for event in &mut events[*index..=*index + 1] {
            // A line ending of flow stays one, for the structure around it.
            if event.name == Name::ExtensionChunk {
                event.name = Name::SpaceOrTab;
            }
            event.link = None;
        }
    }
    if drop_from > 0 {
        if let Some(link) = events[chunks[drop_from - 1]].link.as_mut() {
            link.next = None;
        }
    }

    let start = chunks[drop_from];
    if events[start..]
        .iter()
        .any(|event| event.name == Name::LineEnding)
    {
        Some(start)
    } else {
        None
    }
}

/// Whether the innermost open token is a token in document content, such as
/// a line prefix.
fn is_in_document_content(tokenizer: &Tokenizer) -> bool {
    let state = ext(tokenizer);
    open_content(tokenizer) == Some(Content::Document) && state.open.len() > state.content_at + 1
}

/// Whether the tokenizer is between characters: not in a UTF-8 sequence, and
/// not in a tab, unless a line starts there after container prefixes, in
/// the prefix of a container, or the token is in document content, such as a
/// line prefix that takes part of a tab, as the indent of fenced code does.
fn at_boundary(tokenizer: &Tokenizer, place: PlaceKind, in_document: bool) -> bool {
    let point = &tokenizer.point;
    (point.vs == 0 || place == PlaceKind::Container || in_document || tokenizer.at_line_start())
        && tokenizer
            .parse_state
            .bytes
            .get(point.index)
            .map_or(true, |byte| !(0x80..0xC0).contains(byte))
}

/// Number of a token name in this parse.
fn intern(
    tokenizer: &Tokenizer,
    construct: u8,
    token: u16,
    name: &'static str,
    kind: TokenKind,
) -> u16 {
    let mut names = tokenizer.parse_state.extension_names.borrow_mut();
    let index = names
        .iter()
        .position(|known| known.0 == construct && known.1 == token && known.3 == kind)
        .unwrap_or_else(|| {
            names.push((construct, token, name, kind));
            names.len() - 1
        });
    // Parsing checks that the names of all tokens fit in a `u16`.
    u16::try_from(index).expect("expected at most 65,535 token names")
}

/// Enter a token of a construct.
fn enter_token(tokenizer: &mut Tokenizer, id: u16) {
    tokenizer.enter(Name::Extension);
    tokenizer
        .events
        .last_mut()
        .expect("expected event")
        .extension = id;
    ext_mut(tokenizer).open.push(id);
}

/// Exit the innermost open token of a construct.
fn exit_token(tokenizer: &mut Tokenizer) {
    let id = ext_mut(tokenizer).open.pop().expect("expected open token");
    tokenizer.exit(Name::Extension);
    tokenizer
        .events
        .last_mut()
        .expect("expected event")
        .extension = id;
}

/// Whether the innermost open token is a content token (or its chunk).
fn is_content_top(tokenizer: &Tokenizer) -> bool {
    match tokenizer.stack.last() {
        Some(Name::Extension) => ext(tokenizer).open.last().map_or(false, |id| {
            matches!(
                tokenizer.parse_state.extension_names.borrow()[usize::from(*id)].3,
                TokenKind::Content(_)
            )
        }),
        _ => false,
    }
}

/// Kind of content in the innermost open content token.
fn content_of_top(tokenizer: &Tokenizer) -> Content {
    let id = *ext(tokenizer).open.last().expect("expected content token");
    match &tokenizer.parse_state.extension_names.borrow()[usize::from(id)].3 {
        TokenKind::Content(content) => content.clone(),
        _ => unreachable!("expected content token"),
    }
}

/// Enter a chunk of content, linked to the previous chunk of the same
/// content token, across tokens in it.
fn enter_chunk(tokenizer: &mut Tokenizer, name: Name, content: Content) {
    tokenizer.enter_link(
        name,
        Link {
            previous: None,
            next: None,
            content,
        },
    );
    let current = tokenizer.events.len() - 1;
    if let Some(previous) = ext_mut(tokenizer).last_chunk.replace(current) {
        link_to(&mut tokenizer.events, previous, current);
    }
}

/// Whether a chunk of content is open: its enter is the last event.
///
/// Chunks stay off the tokenizer stack, so undoing an attempt, which
/// truncates events and the stack, also restores an open chunk.
fn is_chunk_open(tokenizer: &Tokenizer) -> bool {
    tokenizer.events.last().map_or(false, |event| {
        event.kind == Kind::Enter && event.name == Name::ExtensionChunk
    })
}

/// Exit the open chunk.
fn exit_chunk(tokenizer: &mut Tokenizer) {
    tokenizer.stack.push(Name::ExtensionChunk);
    tokenizer.exit(Name::ExtensionChunk);
}

/// Kind of content of the open content token, if any, maybe with tokens in
/// it.
fn open_content(tokenizer: &Tokenizer) -> Option<Content> {
    let state = ext(tokenizer);
    let id = *state.open.get(state.content_at)?;
    match &tokenizer.parse_state.extension_names.borrow()[usize::from(id)].3 {
        TokenKind::Content(content) => Some(content.clone()),
        _ => None,
    }
}

/// Number of token names that the constructs in `options` can use, which
/// events number with a `u16`.
pub(crate) fn token_count(options: &ParseOptions) -> usize {
    let text = options.text_constructs.iter().map(|c| &c.inner);
    let flow = options.flow_constructs.iter().map(|c| &c.inner);
    let containers = options.document_constructs.iter().map(|c| &c.inner);
    text.chain(flow)
        .chain(containers)
        .map(|construct| construct.token_count())
        .sum()
}

/// Markers of the construct at `index`.
pub(crate) fn markers(options: &ParseOptions, index: u8) -> &[u8] {
    erased(options, index).markers()
}

/// Whether the construct at `index` can start after `previous`.
pub(crate) fn can_start_after(options: &ParseOptions, index: u8, previous: Option<char>) -> bool {
    erased(options, index).previous(previous)
}

/// Turn the tokens of a match of the construct at `index` into a node.
pub(crate) fn to_mdast(options: &ParseOptions, index: u8, tokens: Vec<RawToken>) -> mdast::Node {
    erased(options, index).to_mdast(tokens)
}

/// Construct at `index`: text constructs, then flow, then containers.
fn erased(options: &ParseOptions, index: u8) -> &dyn ErasedConstruct {
    let mut index = usize::from(index);
    if index < options.text_constructs.len() {
        return &*options.text_constructs[index].inner;
    }
    index -= options.text_constructs.len();
    if index < options.flow_constructs.len() {
        return &*options.flow_constructs[index].inner;
    }
    index -= options.flow_constructs.len();
    &*options.document_constructs[index].inner
}

/// Start trying construct `index`.
pub(crate) fn start(tokenizer: &mut Tokenizer, index: u8) -> State {
    let column = tokenizer.point.column;
    let options = tokenizer.parse_state.options;
    let state = ext_mut(tokenizer);
    state.index = index;
    state.open.clear();
    state.attempts.clear();
    state.last_chunk = None;
    state.line = FlowLine::First;
    state.line_frame = None;
    state.indent = column;
    state.continuation = false;

    if erased(options, index).place() != PlaceKind::Text
        && matches!(tokenizer.current, Some(b'\t' | b' '))
    {
        tokenizer.attempt(State::Next(StateName::ExtensionIndentAfter), State::Nok);
        return State::Retry(space_or_tab_min_max(tokenizer, 0, TAB_SIZE - 1));
    }

    indent_after(tokenizer)
}

/// After the indentation of a flow construct or container, at its marker.
pub(crate) fn indent_after(tokenizer: &mut Tokenizer) -> State {
    let options = tokenizer.parse_state.options;
    let (start, column) = (tokenizer.point.index, tokenizer.point.column);
    let events = tokenizer.events.len();
    let current = tokenizer.current;
    let state = ext_mut(tokenizer);
    let construct = erased(options, state.index);
    state.indent = column - state.indent;
    state.start = start;
    state.events = events;
    state.steps = 0;
    state.at = start;
    state.furthest = start;

    match current {
        // Outside text, spaces and tabs are indentation.
        Some(b'\t' | b' ') if construct.place() != PlaceKind::Text => State::Nok,
        Some(byte) if byte != b'\n' && construct.markers().contains(&byte) => {
            let slot = usize::from(state.index);
            if state.slots.len() <= slot {
                state.slots.resize_with(slot + 1, || None);
            }
            construct.start(&mut state.slots[slot]);
            State::Retry(StateName::ExtensionStep)
        }
        _ => State::Nok,
    }
}

/// At the start of a later line of a container, check whether it continues.
pub(crate) fn continuation(tokenizer: &mut Tokenizer) -> State {
    let options = tokenizer.parse_state.options;
    let document = &tokenizer.tokenize_state;
    let container = &document.document_container_stack[document.document_continued];
    let (index, id) = match container.kind {
        TokenizerContainer::Extension(index, ..) => (index, container.size),
        _ => unreachable!("expected container of a construct"),
    };
    let (start, events) = (tokenizer.point.index, tokenizer.events.len());
    let tokenize_state = ext_mut(tokenizer);
    tokenize_state.index = index;
    tokenize_state.open.clear();
    tokenize_state.attempts.clear();
    tokenize_state.indent = 0;
    tokenize_state.continuation = true;
    tokenize_state.start = start;
    tokenize_state.events = events;
    tokenize_state.steps = 0;
    tokenize_state.at = start;
    tokenize_state.furthest = start;
    let slot = usize::from(index);
    if tokenize_state.slots.len() <= slot {
        tokenize_state.slots.resize_with(slot + 1, || None);
    }
    erased(options, index).resume_container(&mut tokenize_state.slots[slot], id);
    State::Retry(StateName::ExtensionStep)
}

/// After a flow line ending that continues the construct: consume it.
///
/// Line endings in flow are real, so containers can count lines; in content,
/// they are linked chunks.
pub(crate) fn at_non_lazy(tokenizer: &mut Tokenizer) -> State {
    if is_content_top(tokenizer) {
        let content = content_of_top(tokenizer);
        if is_chunk_open(tokenizer) {
            exit_chunk(tokenizer);
        }
        enter_chunk(tokenizer, Name::LineEnding, content);
        let line = tokenizer.point.line;
        let state = ext_mut(tokenizer);
        state.content_line = line;
        state.content_blank = true;
    } else {
        tokenizer.enter(Name::LineEnding);
    }
    tokenizer.consume();
    tokenizer.exit(Name::LineEnding);
    State::Next(StateName::ExtensionStep)
}

/// At a flow line ending, in a check of whether the line after it continues
/// the construct.
pub(crate) fn line_ending(tokenizer: &mut Tokenizer) -> State {
    tokenizer.enter(Name::LineEnding);
    tokenizer.consume();
    tokenizer.exit(Name::LineEnding);
    State::Next(StateName::ExtensionLineAfter)
}

/// After a flow line ending, in a check: a lazy line or the end of what the
/// tokenizer is fed does not continue the construct.
pub(crate) fn line_after(tokenizer: &mut Tokenizer) -> State {
    if tokenizer.current.is_none() || tokenizer.lazy {
        State::Nok
    } else {
        State::Ok
    }
}

/// After a flow line ending followed by a lazy line or the end.
pub(crate) fn at_lazy(tokenizer: &mut Tokenizer) -> State {
    ext_mut(tokenizer).line = FlowLine::Lazy;
    State::Retry(StateName::ExtensionStep)
}

/// After an attempt of a construct that succeeded.
pub(crate) fn attempt_ok(tokenizer: &mut Tokenizer) -> State {
    let options = tokenizer.parse_state.options;
    let state = ext_mut(tokenizer);
    state.attempts.pop().expect("expected attempt");
    let index = state.index;
    erased(options, index).resume(&mut state.slots[usize::from(index)], true);
    State::Retry(StateName::ExtensionStep)
}

/// After an attempt of a construct that failed, and was undone.
pub(crate) fn attempt_nok(tokenizer: &mut Tokenizer) -> State {
    let options = tokenizer.parse_state.options;
    let state = ext_mut(tokenizer);
    let frame = state.attempts.pop().expect("expected attempt");
    state.open.truncate(frame.open_len);
    state.content_line = frame.content_line;
    state.content_blank = frame.content_blank;
    state.content_lines = frame.content_lines;
    state.content_enter = frame.content_enter;
    state.content_eols.truncate(frame.content_eols);
    state.last_chunk = frame.last_chunk;
    let index = state.index;
    erased(options, index).resume(&mut state.slots[usize::from(index)], false);
    tokenizer.line_start = frame.line_start;

    // The undo removed chunks the attempt opened, but not the link to them.
    if let Some(chunk) = frame.last_chunk {
        let events_len = tokenizer.events.len();
        let link = tokenizer.events[chunk]
            .link
            .as_mut()
            .expect("expected link");
        if link.next.map_or(false, |next| next >= events_len) {
            link.next = None;
        }
    }

    State::Retry(StateName::ExtensionStep)
}

/// Run the current construct, enforcing the rules of [`Construct`].
pub(crate) fn step(tokenizer: &mut Tokenizer) -> State {
    let point = tokenizer.point.index;
    let tokenize_state = ext_mut(tokenizer);
    tokenize_state.steps += point.saturating_sub(tokenize_state.at).max(1);
    tokenize_state.at = point;
    tokenize_state.furthest = tokenize_state.furthest.max(point);
    // Work is linear in the bytes a match reaches, whatever its attempts do.
    let budget = STEP_MAX.saturating_mul(tokenize_state.furthest - tokenize_state.start + 1);
    let over_budget = tokenize_state.steps > budget;

    let index = tokenize_state.index;
    let construct = erased(tokenizer.parse_state.options, index);
    let place = construct.place();
    let is_flow = place == PlaceKind::Flow;

    let within_reach = match construct.reach() {
        Reach::Unbounded => true,
        // At the end, there is nothing to read.
        Reach::Bytes(bytes) => {
            tokenizer.current.is_none() || tokenizer.point.index - ext(tokenizer).start < bytes
        }
    };
    // Limits, not rules: the construct does not match, without a panic.
    ext_mut(tokenizer).nok = NokCause::Construct;
    if over_budget || !within_reach {
        ext_mut(tokenizer).nok = NokCause::Limit;
        return finish(tokenizer, RawStep::Nok, is_flow, false);
    }

    // The slot is taken out while the step runs, as the step borrows the
    // tokenizer.
    let slot = usize::from(index);
    let mut machine = ext_mut(tokenizer).slots[slot].take();
    let (step, report) = construct.step(
        &mut machine,
        Inner {
            tokenizer,
            index,
            place,
            report: Report::default(),
        },
    );
    ext_mut(tokenizer).slots[slot] = machine;
    let tokenize_state = ext(tokenizer);

    // A limit, such as the depth of content, is no mistake of the
    // construct either.
    let limit = report.limit
        || matches!(step, RawStep::Attempt if tokenize_state.attempts.len() >= ATTEMPT_MAX);
    if limit {
        if matches!(step, RawStep::Attempt) {
            // The frame of the attempt that does not start.
            construct.resume(&mut ext_mut(tokenizer).slots[slot], false);
        }
        ext_mut(tokenizer).nok = NokCause::Limit;
        return finish(tokenizer, RawStep::Nok, is_flow, false);
    }

    let attempt = tokenize_state.attempts.last();
    let is_container = place == PlaceKind::Container;
    let broken = report.broken.or(match step {
        RawStep::Container if attempt.is_some() => Some("container-in-attempt"),
        // A container starts with its token and content token open, and
        // continues with its tokens closed.
        RawStep::Container => {
            let is_whole = if tokenize_state.continuation {
                tokenize_state.open.is_empty()
            } else {
                matches!(tokenize_state.open[..], [_, content] if matches!(
                    tokenizer.parse_state.extension_names.borrow()[usize::from(content)].3,
                    TokenKind::Content(Content::Document)
                ))
            };
            if is_whole {
                None
            } else {
                Some("container-tokens")
            }
        }
        RawStep::Ok if attempt.is_none() && is_container => Some("ok-in-container"),
        // An attempt ends with the tokens it opened closed.
        RawStep::Ok
            if attempt.map_or(false, |frame| tokenize_state.open.len() != frame.open_len) =>
        {
            Some("ok-with-open-tokens")
        }
        RawStep::Ok if attempt.is_none() && tokenizer.point.index <= tokenize_state.start => {
            Some("ok-without-bytes")
        }
        RawStep::Ok if attempt.is_none() && !tokenize_state.open.is_empty() => {
            Some("ok-with-open-tokens")
        }
        // A flow construct ends at a line ending or the end.
        RawStep::Ok
            if attempt.is_none() && is_flow && !matches!(tokenizer.current, None | Some(b'\n')) =>
        {
            Some("ok-before-line-end")
        }
        _ => None,
    });

    if let Some(rule) = broken {
        broke(rule);
        ext_mut(tokenizer).nok = NokCause::Rule;
        return finish(tokenizer, RawStep::Nok, is_flow, false);
    }

    if let RawStep::Container = step {
        let tokens = match ext(tokenizer).open[..] {
            [token, content] => Some((token, content)),
            _ => None,
        };
        // A container keeps the id of its state in `size`.
        let document = &tokenizer.tokenize_state;
        let id = if ext(tokenizer).continuation {
            Some(document.document_container_stack[document.document_continued].size)
        } else {
            None
        };
        let id = construct.store_container(&mut ext_mut(tokenizer).slots[slot], id);
        let document = &mut tokenizer.tokenize_state;
        let container = &mut document.document_container_stack[document.document_continued];
        // A container that starts leaves its token and content open.
        if let Some((token, content)) = tokens {
            container.kind = TokenizerContainer::Extension(index, token, content);
        }
        container.size = id;
        return State::Ok;
    }

    finish(tokenizer, step, is_flow, report.line_ending)
}

/// Go on after a step of the current construct, which consumed a line
/// ending if `line_ending`.
fn finish(tokenizer: &mut Tokenizer, step: RawStep, is_flow: bool, line_ending: bool) -> State {
    let in_attempt = !ext(tokenizer).attempts.is_empty();

    match step {
        // Past its first line, a flow construct matches, or ends before the
        // line it fails on, and containers must not start in the next line
        // while it is checked.
        RawStep::Next if is_flow && line_ending => {
            // The line ending is not consumed yet: keep where the construct
            // is, to end it here if it fails on the next line.
            let progress = tokenizer.snapshot();
            let line_start = tokenizer.line_start.clone();
            let state = ext_mut(tokenizer);
            state.line_frame = Some(LineFrame {
                progress,
                frame: AttemptFrame {
                    open_len: state.open.len(),
                    line_start,
                    content_line: state.content_line,
                    content_blank: state.content_blank,
                    content_lines: state.content_lines,
                    content_enter: state.content_enter,
                    content_eols: state.content_eols.len(),
                    last_chunk: state.last_chunk,
                },
                content_at: state.content_at,
            });
            let ExtensionState {
                open, line_open, ..
            } = state;
            line_open.clear();
            line_open.extend_from_slice(open);
            ext_mut(tokenizer).line = FlowLine::Committed;
            tokenizer.concrete = true;
            tokenizer.check(
                State::Next(StateName::ExtensionNonLazy),
                State::Next(StateName::ExtensionLazy),
            );
            State::Retry(StateName::ExtensionLineEnding)
        }
        RawStep::Next => State::Next(StateName::ExtensionStep),
        RawStep::Retry => State::Retry(StateName::ExtensionStep),
        // Handled in `step`.
        RawStep::Container => unreachable!("expected a container to end in `step`"),
        RawStep::Attempt => {
            let line_start = tokenizer.line_start.clone();
            let tokenize_state = ext_mut(tokenizer);
            let frame = AttemptFrame {
                open_len: tokenize_state.open.len(),
                line_start,
                content_line: tokenize_state.content_line,
                content_blank: tokenize_state.content_blank,
                content_lines: tokenize_state.content_lines,
                content_enter: tokenize_state.content_enter,
                content_eols: tokenize_state.content_eols.len(),
                last_chunk: tokenize_state.last_chunk,
            };
            tokenize_state.attempts.push(frame);
            tokenizer.attempt(
                State::Next(StateName::ExtensionAttemptOk),
                State::Next(StateName::ExtensionAttemptNok),
            );
            State::Retry(StateName::ExtensionStep)
        }
        RawStep::Ok => {
            if is_flow && !in_attempt {
                tokenizer.interrupt = false;
                tokenizer.concrete = false;
            }
            State::Ok
        }
        RawStep::Nok if !in_attempt && ext(tokenizer).line != FlowLine::First => {
            end_before_line(tokenizer)
        }
        RawStep::Nok => State::Nok,
    }
}

/// A flow construct failed after its first line, outside an attempt: end it
/// before its last line ending, which then starts a line as other markdown
/// does, rather than an error.
fn end_before_line(tokenizer: &mut Tokenizer) -> State {
    if ext(tokenizer).nok == NokCause::Construct {
        broke("nok-after-first-line");
    }

    let state = ext_mut(tokenizer);
    let LineFrame {
        progress,
        frame,
        content_at,
    } = state
        .line_frame
        .take()
        .expect("expected the line ending of a flow construct");
    state.open.clear();
    let ExtensionState {
        open, line_open, ..
    } = state;
    open.extend_from_slice(line_open);
    state.content_at = content_at;
    state.content_line = frame.content_line;
    state.content_blank = frame.content_blank;
    state.content_lines = frame.content_lines;
    state.content_enter = frame.content_enter;
    state.content_eols.truncate(frame.content_eols);
    state.last_chunk = frame.last_chunk;
    state.line = FlowLine::First;
    tokenizer.restore(progress);
    tokenizer.line_start = frame.line_start;

    // The undo removed chunks after the line ending, but not the link to
    // them.
    if let Some(chunk) = frame.last_chunk {
        let events_len = tokenizer.events.len();
        if let Some(link) = tokenizer.events[chunk].link.as_mut() {
            if link.next.map_or(false, |next| next >= events_len) {
                link.next = None;
            }
        }
    }

    // Close what is open, as the construct would.
    let index = ext(tokenizer).index;
    for _ in 0..ext(tokenizer).open.len() {
        let mut inner = Inner {
            tokenizer,
            index,
            place: PlaceKind::Flow,
            report: Report::default(),
        };
        exit(&mut inner);
    }

    tokenizer.interrupt = false;
    tokenizer.concrete = false;
    State::Ok
}

/// A construct broke `rule`: it does not match, and with `test-util`, that
/// is a panic, so the tests of a construct find the mistake.
#[cfg_attr(not(feature = "test-util"), allow(unused_variables))]
fn broke(rule: &'static str) {
    #[cfg(feature = "test-util")]
    panic!("a construct broke the rule `{}`", rule);
}

/// Whether `event` is of a token of a construct in content, such as a line
/// prefix.
pub(crate) fn is_in_content(names: &[TokenName], event: &Event) -> bool {
    event.name == Name::Extension && names[usize::from(event.extension)].3 == TokenKind::InContent
}

/// Index of the exit of each event entered, to jump over tokens in time
/// independent of what they hold, such as nested containers; empty without
/// constructs.
pub(crate) fn exits(events: &[Event], names: &[TokenName]) -> Vec<usize> {
    if names.is_empty() {
        return Vec::new();
    }

    let mut exits = vec![0; events.len()];
    let mut open = vec![];
    for (index, event) in events.iter().enumerate() {
        match event.kind {
            Kind::Enter => open.push(index),
            Kind::Exit => exits[open.pop().expect("expected an enter")] = index,
        }
    }
    exits
}

/// A match of a construct, as event indices.
pub(crate) struct Match<'a> {
    pub tokens: Vec<RawToken<'a>>,
    /// Content tokens: token, enter, and exit, and whether the content is a
    /// document.
    pub contents: Vec<(usize, usize, usize, bool)>,
    /// Events left out of values: content, and container prefixes.
    pub excluded: Vec<(usize, usize)>,
    /// Index of the last event of the match.
    pub end: usize,
}

/// Gather the tokens of the match that starts at `start`.
pub(crate) fn collect_tokens<'a>(
    events: &[Event],
    bytes: &'a [u8],
    names: &[TokenName],
    exits: &[usize],
    start: usize,
) -> Match<'a> {
    // Name, enter, exit, and whether content.
    let mut spans: Vec<(u16, usize, usize, bool)> = vec![];
    let mut contents = vec![];
    let mut excluded = vec![];
    let mut index = start;
    // The match’s own tokens in content are inside its content tokens, which
    // are skipped, so tokens in content here are another match’s.
    let is_own = |event: &Event| names[usize::from(event.extension)].3 != TokenKind::InContent;

    loop {
        let event = &events[index];
        match (&event.kind, &event.name) {
            (Kind::Enter, Name::Extension) if is_own(event) => {
                let (_, name, _, kind) = &names[usize::from(event.extension)];
                let is_content = matches!(kind, TokenKind::Content(_));
                // Exits are known, so no stack of open tokens is needed.
                spans.push((*name, index, exits[index], is_content));

                if let TokenKind::Content(content) = kind {
                    let enter = index;
                    index = exits[index];
                    contents.push((spans.len() - 1, enter, index, *content == Content::Document));
                    excluded.push((enter, index));
                    // Handle the exit.
                    continue;
                }
            }
            (Kind::Exit, Name::Extension) if is_own(event) && index == exits[start] => break,
            // Container prefixes.
            (Kind::Enter, name) if *name != Name::LineEnding => {
                let enter = index;
                index = exits[index];
                excluded.push((enter, index));
            }
            _ => {}
        }
        index += 1;
    }

    let tokens = spans
        .into_iter()
        .map(|(name, enter, exit, is_content)| Token {
            name,
            value: if is_content {
                Cow::Borrowed("")
            } else {
                own_text(events, bytes, enter, exit, &excluded)
            },
            children: vec![],
            position: Position {
                start: events[enter].point.to_unist(),
                end: events[exit].point.to_unist(),
            },
        })
        .collect();

    Match {
        tokens,
        contents,
        excluded,
        end: index,
    }
}

/// Source text from event `enter` to event `exit`, without the excluded
/// events inside.
pub(crate) fn own_text<'a>(
    events: &[Event],
    bytes: &'a [u8],
    enter: usize,
    exit: usize,
    excluded: &[(usize, usize)],
) -> Cow<'a, str> {
    // Like `to_html`, leaves out tabs partly used by prefixes.
    let text = |from: usize, to: usize| {
        let slice = Slice::from_position(
            bytes,
            &SlicePosition {
                start: &events[from].point,
                end: &events[to].point,
            },
        );
        str::from_utf8(slice.bytes).expect("expected tokens between characters")
    };
    // Excluded spans are in order, and do not overlap.
    let first = excluded.partition_point(|(excluded_enter, _)| *excluded_enter <= enter);
    let inside = || {
        excluded[first..]
            .iter()
            .take_while(move |(_, excluded_exit)| *excluded_exit < exit)
    };

    if inside().next().is_none() {
        return Cow::Borrowed(text(enter, exit));
    }

    let mut value = String::new();
    let mut from = enter;
    for (excluded_enter, excluded_exit) in inside() {
        value.push_str(text(from, *excluded_enter));
        from = *excluded_exit;
    }
    value.push_str(text(from, exit));
    Cow::Owned(value)
}

/// Text markers: the built-in ones and those of constructs, except line
/// endings; empty without constructs.
pub(crate) fn text_markers(builtin: &[u8], constructs: &[BoxedConstruct<Text>]) -> Vec<u8> {
    if constructs.is_empty() {
        return Vec::new();
    }

    let mut markers = builtin.to_vec();
    for construct in constructs {
        for byte in construct.inner.markers() {
            if !matches!(byte, b'\n' | b'\r') && !markers.contains(byte) {
                markers.push(*byte);
            }
        }
    }
    markers
}
