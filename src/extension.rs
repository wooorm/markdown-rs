//! Syntax extensions: constructs to add to markdown.
//!
//! A construct is a state machine driven by the built-in tokenizer, like a
//! micromark construct, so it sees text without container prefixes such as
//! `> `, across lines.
//! Pass constructs in [`ParseOptions::text_constructs`][crate::ParseOptions],
//! each wrapped by [`construct()`].
//!
//! [`to_mdast()`][crate::to_mdast()] turns each match into a node with
//! [`Construct::to_mdast`].
//! HTML has no node to render, so [`to_html()`][crate::to_html()] writes
//! the source of a match as text.
//!
//! The types check what they can: a step that consumes ends in
//! [`Consumed::next`], so it cannot forget to consume or consume twice;
//! [`Ctx::exit`] closes the innermost token, so it cannot close the wrong
//! one; and a construct goes in the list of its [`Place`].
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

use crate::event::{Event, Kind, Name};
use crate::mdast;
use crate::state::{Name as StateName, State};
use crate::tokenizer::{move_point_back, Tokenizer};
use crate::unist::Position;
use crate::util::slice::{Position as SlicePosition, Slice};
use crate::ParseOptions;
use alloc::{borrow::Cow, boxed::Box, string::String, vec, vec::Vec};
use core::{any::Any, convert::TryFrom, fmt, marker::PhantomData, str};

/// Most steps a construct can take in a row without consuming a byte.
const RETRY_MAX: u16 = 256;

mod sealed {
    /// Only this crate defines places.
    pub trait Sealed {}
}

/// Where a construct runs.
#[doc(hidden)]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum PlaceKind {
    /// In text.
    Text,
}

/// Where a construct runs: [`Text`].
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
/// in text.
/// They are tried in order, before built-ins, at their markers (never line
/// endings).
///
/// A construct that breaks one of these rules does not match, so its bytes
/// stay what they would otherwise be; with the `test-util` feature, it
/// panics instead, naming the rule:
///
/// * every token entered is in [`TokenType::ALL`] (`token-not-in-all`)
/// * every consumed byte is inside a token, and one token holds the others
///   (`consume-outside-token`, `token-after-match`)
/// * a token holds at least one byte (`empty-token`)
/// * `exit` has a token to close (`exit-without-token`)
/// * after consuming a line ending, a step returns `next`, and can first
///   `exit` tokens that end there (`enter-after-line-ending`,
///   `ok-after-line-ending`)
/// * `ok` comes after at least one byte, with every token closed
///   (`ok-without-bytes`, `ok-with-open-tokens`)
/// * tokens start and end between characters, not inside one
///   (`token-inside-character`)
///
/// A step that does not consume, which [`Ctx::retry`] makes, can run at
/// most 256 times in a row, and a match reads at most as far as
/// [`reach`][Self::reach] allows; past that, it does not match.
pub trait Construct: 'static {
    /// Where the construct runs: [`Text`].
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
    /// before it: `None` at the start of text, such as the first line of a
    /// paragraph, `'\n'` at the start of a later line, after container
    /// prefixes, and `None` after a character escape, as in micromark.
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

/// Wrap a construct, to pass it in the options for its place, such as
/// [`ParseOptions::text_constructs`][crate::ParseOptions] for [`Text`].
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
}

/// Token of a construct: a name, the text it spans, and where.
#[derive(Clone, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub struct Token<'a, T> {
    /// Name given to `enter`.
    pub name: T,
    /// Source text, without container prefixes.
    pub value: Cow<'a, str>,
    /// Positional info.
    pub position: Position,
}

/// A token whose name is its index in [`TokenType::ALL`].
pub(crate) type RawToken<'a> = Token<'a, u16>;

/// Index of a construct, index of a token name in its [`TokenType::ALL`],
/// and that name, interned per parse: events of constructs refer to it by
/// index.
pub(crate) type TokenName = (u8, u16, &'static str);

/// What a step does, after the construct's states are stored.
#[derive(Clone, Copy, Debug)]
enum RawStep {
    Next,
    Retry,
    Ok,
    Nok,
}

/// A construct, without its types, so constructs of different types can be
/// stored together.
trait ErasedConstruct {
    fn markers(&self) -> &[u8];
    fn reach(&self) -> Reach;
    fn previous(&self, previous: Option<char>) -> bool;
    fn token_count(&self) -> usize;
    /// Put the start state in `slot`.
    fn start(&self, slot: &mut Option<Box<dyn Any>>);
    /// Take a step from the state in `slot`, and store the next one there.
    fn step(&self, slot: &mut Option<Box<dyn Any>>, inner: Inner<'_, '_>) -> (RawStep, Report);
    fn to_mdast(&self, tokens: Vec<RawToken>) -> mdast::Node;
}

/// A construct of a known type.
struct Wrap<C>(C);

/// Where a construct keeps its state between steps.
struct Machine<S> {
    state: Option<S>,
}

impl<C: Construct> ErasedConstruct for Wrap<C> {
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
        C::Token::ALL.len()
    }

    fn start(&self, slot: &mut Option<Box<dyn Any>>) {
        let state = Some(self.0.start());
        match slot
            .as_mut()
            .and_then(|slot| slot.downcast_mut::<Machine<C::State>>())
        {
            // Reused for every match, so a state without data allocates
            // nothing, and one with data allocates once per parse.
            Some(machine) => machine.state = state,
            None => *slot = Some(Box::new(Machine { state })),
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
            StepKind::Ok => RawStep::Ok,
            StepKind::Nok => RawStep::Nok,
        };
        (raw, step.report)
    }

    fn to_mdast(&self, tokens: Vec<RawToken>) -> mdast::Node {
        self.0.to_mdast(
            tokens
                .into_iter()
                .map(|token| Token {
                    name: C::Token::ALL[usize::from(token.name)],
                    value: token.value,
                    position: token.position,
                })
                .collect(),
        )
    }
}

/// State of constructs in a tokenizer, boxed, so the tokenizer stays small
/// without them.
#[derive(Debug, Default)]
pub(crate) struct ExtensionState {
    /// Where the current construct started.
    start: usize,
    /// Number of events when the current construct started.
    events: usize,
    /// Steps the current construct took without consuming.
    retries: u16,
    /// Construct being tried.
    index: u8,
    /// Construct to try next at the current byte.
    pub(crate) next: u8,
    /// State of each construct, by index, made when it first runs.
    slots: Vec<Option<Box<dyn Any>>>,
    /// Interned names of the open tokens of the current match.
    open: Vec<u16>,
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
    pub fn current(&self) -> Option<u8> {
        self.inner.tokenizer.current
    }

    /// Current character, or `None` at the end or inside a character.
    ///
    /// Line endings are `'\n'`, as with [`current`][Self::current].
    /// `consume` takes one byte: a character takes as many steps as it has
    /// bytes.
    pub fn current_char(&self) -> Option<char> {
        let tokenizer = &self.inner.tokenizer;
        match self.current() {
            Some(b'\n') => Some('\n'),
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

    /// The construct matched, ending with the consumed byte.
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

/// Start a token named `token` of construct `C`.
fn enter<T: TokenType>(inner: &mut Inner, token: T) {
    let tokenizer = &*inner.tokenizer;
    let state = ext(tokenizer);
    // The first token starts the match, and holds the others.
    let can_start = tokenizer.events.len() == state.events || !state.open.is_empty();
    let position = T::ALL.iter().position(|known| *known == token);
    let rule = if position.is_none() {
        Some("token-not-in-all")
    } else if inner.report.line_ending {
        Some("enter-after-line-ending")
    } else if !can_start {
        Some("token-after-match")
    } else if !at_boundary(tokenizer) {
        Some("token-inside-character")
    } else {
        None
    };

    if let Some(rule) = rule {
        inner.broke(rule);
    }
    if inner.report.broken.is_some() {
        return;
    }

    let position = position.expect("expected a token in `TokenType::ALL`, as checked");
    // Parsing checks that the names of all tokens fit in a `u16`.
    let position = u16::try_from(position).expect("expected at most 65,535 tokens");
    let id = intern(inner.tokenizer, inner.index, position, token.name());
    enter_token(inner.tokenizer, id);
}

/// End the innermost open token.
fn exit(inner: &mut Inner) {
    let rule = if ext(inner.tokenizer).open.is_empty() {
        Some("exit-without-token")
    } else if !at_boundary(inner.tokenizer) {
        Some("token-inside-character")
    } else if top_is_empty(inner.tokenizer) {
        Some("empty-token")
    } else {
        None
    };

    if let Some(rule) = rule {
        inner.broke(rule);
    }
    if inner.report.broken.is_none() {
        exit_token(inner.tokenizer);
    }
}

/// Consume the current byte.
fn consume(inner: &mut Inner) {
    if inner.tokenizer.current.is_none() || inner.report.broken.is_some() {
        return;
    }

    if ext(inner.tokenizer).open.is_empty() {
        inner.broke("consume-outside-token");
        return;
    }

    let tokenizer = &mut *inner.tokenizer;
    let is_eol = tokenizer.current == Some(b'\n');

    if is_eol {
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

/// Whether the tokenizer is between characters (not in a UTF-8 sequence);
/// a boundary can be in a tab, between its virtual spaces.
fn at_boundary(tokenizer: &Tokenizer) -> bool {
    let point = &tokenizer.point;
    point.vs > 0
        || tokenizer
            .parse_state
            .bytes
            .get(point.index)
            .map_or(true, |byte| !(0x80..0xC0).contains(byte))
}

/// Number of a token name in this parse.
fn intern(tokenizer: &Tokenizer, construct: u8, token: u16, name: &'static str) -> u16 {
    let mut names = tokenizer.parse_state.extension_names.borrow_mut();
    let index = names
        .iter()
        .position(|known| known.0 == construct && known.1 == token)
        .unwrap_or_else(|| {
            names.push((construct, token, name));
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

/// Number of token names that the constructs in `options` can use, which
/// events number with a `u16`.
pub(crate) fn token_count(options: &ParseOptions) -> usize {
    options
        .text_constructs
        .iter()
        .map(|construct| construct.inner.token_count())
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

/// Construct at `index`.
fn erased(options: &ParseOptions, index: u8) -> &dyn ErasedConstruct {
    &*options.text_constructs[usize::from(index)].inner
}

/// Start trying construct `index`.
pub(crate) fn start(tokenizer: &mut Tokenizer, index: u8) -> State {
    let (start, events) = (tokenizer.point.index, tokenizer.events.len());
    let options = tokenizer.parse_state.options;
    let state = ext_mut(tokenizer);
    state.start = start;
    state.events = events;
    state.retries = 0;
    state.index = index;
    state.open.clear();
    let slot = usize::from(index);
    if state.slots.len() <= slot {
        state.slots.resize_with(slot + 1, || None);
    }
    erased(options, index).start(&mut state.slots[slot]);
    State::Retry(StateName::ExtensionStep)
}

/// Run the current construct, enforcing the rules of [`Construct`].
pub(crate) fn step(tokenizer: &mut Tokenizer) -> State {
    let index = ext(tokenizer).index;
    let construct = erased(tokenizer.parse_state.options, index);

    let within_reach = match construct.reach() {
        Reach::Unbounded => true,
        // At the end, there is nothing to read.
        Reach::Bytes(bytes) => {
            tokenizer.current.is_none() || tokenizer.point.index - ext(tokenizer).start < bytes
        }
    };
    if !within_reach {
        return State::Nok;
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
            report: Report::default(),
        },
    );
    ext_mut(tokenizer).slots[slot] = machine;
    let tokenize_state = ext(tokenizer);

    let broken = report.broken.or(match step {
        RawStep::Ok if tokenizer.point.index <= tokenize_state.start => Some("ok-without-bytes"),
        RawStep::Ok if !tokenize_state.open.is_empty() => Some("ok-with-open-tokens"),
        _ => None,
    });

    if let Some(rule) = broken {
        broke(rule);
        return State::Nok;
    }

    match step {
        RawStep::Next => {
            ext_mut(tokenizer).retries = 0;
            State::Next(StateName::ExtensionStep)
        }
        RawStep::Retry if tokenize_state.retries >= RETRY_MAX => State::Nok,
        RawStep::Retry => {
            ext_mut(tokenizer).retries += 1;
            State::Retry(StateName::ExtensionStep)
        }
        RawStep::Ok => State::Ok,
        RawStep::Nok => State::Nok,
    }
}

/// A construct broke `rule`: it does not match, and with `test-util`, that
/// is a panic, so the tests of a construct find the mistake.
#[cfg_attr(not(feature = "test-util"), allow(unused_variables))]
fn broke(rule: &'static str) {
    #[cfg(feature = "test-util")]
    panic!("a construct broke the rule `{}`", rule);
}

/// Index of the exit of the event entered at `index`.
fn balanced_exit(events: &[Event], mut index: usize) -> usize {
    let mut depth = 0;
    loop {
        match events[index].kind {
            Kind::Enter => depth += 1,
            Kind::Exit => depth -= 1,
        }
        if depth == 0 {
            return index;
        }
        index += 1;
    }
}

/// A match of a construct.
pub(crate) struct Match<'a> {
    pub tokens: Vec<RawToken<'a>>,
    /// Index of the last event of the match.
    pub end: usize,
}

/// Gather the tokens of the match that starts at `start`.
pub(crate) fn collect_tokens<'a>(
    events: &[Event],
    bytes: &'a [u8],
    names: &[TokenName],
    start: usize,
) -> Match<'a> {
    // Name, enter, and exit.
    let mut spans: Vec<(u16, usize, usize)> = vec![];
    let mut open = vec![];
    // Events left out of values: container prefixes.
    let mut excluded = vec![];
    let mut index = start;

    loop {
        let event = &events[index];
        match (&event.kind, &event.name) {
            (Kind::Enter, Name::Extension) => {
                spans.push((names[usize::from(event.extension)].1, index, index));
                open.push(spans.len() - 1);
            }
            (Kind::Exit, Name::Extension) => {
                let span = open.pop().expect("expected an open token");
                spans[span].2 = index;
                if open.is_empty() {
                    break;
                }
            }
            // Container prefixes.
            (Kind::Enter, name) if *name != Name::LineEnding => {
                let enter = index;
                index = balanced_exit(events, index);
                excluded.push((enter, index));
            }
            _ => {}
        }
        index += 1;
    }

    let tokens = spans
        .into_iter()
        .map(|(name, enter, exit)| Token {
            name,
            value: own_text(events, bytes, enter, exit, &excluded),
            position: Position {
                start: events[enter].point.to_unist(),
                end: events[exit].point.to_unist(),
            },
        })
        .collect();

    Match { tokens, end: index }
}

/// Source text from event `enter` to event `exit`, without the excluded
/// events inside.
fn own_text<'a>(
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
