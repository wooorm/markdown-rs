//! The tokenizer glues states from the state machine together.
//!
//! It facilitates everything needed to turn codes into tokens and events with
//! a state machine.
//! It also enables logic needed for parsing markdown, such as an [`attempt`][]
//! to parse something, which can succeed or, when unsuccessful, revert the
//! attempt.
//! Similarly, a [`check`][] exists, which does the same as an `attempt` but
//! reverts even if successful.
//!
//! [`attempt`]: Tokenizer::attempt
//! [`check`]: Tokenizer::check

use crate::parser::ParseState;
use crate::token::{Token, VOID_TOKENS};
use crate::util::edit_map::EditMap;

/// Embedded content type.
#[derive(Debug, Clone, PartialEq)]
pub enum ContentType {
    /// Represents [text content][crate::content::text].
    Text,
    /// Represents [string content][crate::content::string].
    String,
}

/// Enum representing a character code.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Code {
    /// End of the input stream (called eof).
    None,
    /// Used to make parsing line endings easier as it represents both
    /// `Code::Char('\r')` and `Code::Char('\n')` combined.
    CarriageReturnLineFeed,
    /// the expansion of a tab (`Code::Char('\t')`), depending on where the tab
    /// ocurred, it’s followed by 0 to 3 (both inclusive) `Code::VirtualSpace`s.
    VirtualSpace,
    /// The most frequent variant of this enum is `Code::Char(char)`, which just
    /// represents a char, but micromark adds meaning to certain other values.
    Char(char),
}

/// A location in the document (`line`/`column`/`offset`).
///
/// The interface for the location in the document comes from unist `Point`:
/// <https://github.com/syntax-tree/unist#point>.
#[derive(Debug, Clone)]
pub struct Point {
    /// 1-indexed line number.
    pub line: usize,
    /// 1-indexed column number.
    /// This is increases up to a tab stop for tabs.
    /// Some editors count tabs as 1 character, so this position is not always
    /// the same as editors.
    pub column: usize,
    /// 0-indexed position in the document.
    pub offset: usize,
    /// Index into `codes`.
    pub index: usize,
}

/// Possible event types.
#[derive(Debug, PartialEq, Clone)]
pub enum EventType {
    /// The start of something.
    Enter,
    /// The end of something.
    Exit,
}

/// A link to another event.
#[derive(Debug, Clone)]
pub struct Link {
    pub previous: Option<usize>,
    pub next: Option<usize>,
    pub content_type: ContentType,
}

/// Something semantic happening somewhere.
#[derive(Debug, Clone)]
pub struct Event {
    pub event_type: EventType,
    pub token_type: Token,
    pub point: Point,
    pub link: Option<Link>,
}

/// The essence of the state machine are functions: `StateFn`.
/// It’s responsible for dealing with that single passed [`Code`][].
/// It yields a [`State`][].
pub type StateFn = dyn FnOnce(&mut Tokenizer) -> State;

/// Callback that can be registered and is called when the tokenizer is done.
///
/// Resolvers are supposed to change the list of events, because parsing is
/// sometimes messy, and they help expose a cleaner interface of events to
/// the compiler and other users.
pub type Resolver = dyn FnOnce(&mut Tokenizer);

/// The result of a state.
pub enum State {
    /// There is a future state: a boxed [`StateFn`][] to pass the next code to.
    Fn(Box<StateFn>),
    /// The state is successful.
    Ok,
    /// The state is not successful.
    Nok,
}

/// Loose label starts we found.
#[derive(Debug)]
pub struct LabelStart {
    /// Indices of where the label starts and ends in `events`.
    pub start: (usize, usize),
    /// A boolean used internally to figure out if a label start link can’t be
    /// used (because links in links are incorrect).
    pub inactive: bool,
    /// A boolean used internally to figure out if a label is balanced: they’re
    /// not media, it’s just balanced braces.
    pub balanced: bool,
}

/// Media we found.
#[derive(Debug)]
pub struct Media {
    /// Indices of where the media’s label start starts and ends in `events`.
    pub start: (usize, usize),
    /// Indices of where the media’s label end starts and ends in `events`.
    pub end: (usize, usize),
    /// Identifier
    pub id: String,
}

/// Supported containers.
#[derive(Debug, PartialEq)]
pub enum Container {
    BlockQuote,
    ListItem,
}

/// Info used to tokenize the current container.
///
/// This info is shared between the initial construct and its continuation.
/// It’s only used for list items.
#[derive(Debug)]
pub struct ContainerState {
    /// Kind.
    pub kind: Container,
    /// Whether the first line was blank.
    pub blank_initial: bool,
    /// The size of the initial construct.
    pub size: usize,
}

/// The internal state of a tokenizer, not to be confused with states from the
/// state machine, this instead is all the information about where we currently
/// are and what’s going on.
#[derive(Debug, Clone)]
struct InternalState {
    /// Length of `events`. We only add to events, so reverting will just pop stuff off.
    events_len: usize,
    /// Length of the stack. It’s not allowed to decrease the stack in a check or an attempt.
    stack_len: usize,
    /// Previous code.
    previous: Code,
    /// Current code.
    current: Code,
    /// Current relative and absolute position in the file.
    point: Point,
}

/// A tokenizer itself.
#[allow(clippy::struct_excessive_bools)]
pub struct Tokenizer<'a> {
    /// Jump between line endings.
    column_start: Vec<usize>,
    // First line.
    line_start: usize,
    /// Track whether a character is expected to be consumed, and whether it’s
    /// actually consumed
    ///
    /// Tracked to make sure everything’s valid.
    consumed: bool,
    /// Track whether this tokenizer is done.
    drained: bool,
    /// Current character code.
    pub current: Code,
    /// Previous character code.
    pub previous: Code,
    /// Current relative and absolute place in the file.
    pub point: Point,
    /// Semantic labels of one or more codes in `codes`.
    pub events: Vec<Event>,
    /// Hierarchy of semantic labels.
    ///
    /// Tracked to make sure everything’s valid.
    pub stack: Vec<Token>,
    /// To do.
    pub map: EditMap,
    /// List of attached resolvers, which will be called when done feeding,
    /// to clean events.
    resolvers: Vec<Box<Resolver>>,
    /// List of names associated with attached resolvers.
    resolver_ids: Vec<String>,
    /// Shared parsing state across tokenizers.
    pub parse_state: &'a ParseState<'a>,
    /// To do.
    pub index: usize,
    /// Stack of label (start) that could form images and links.
    ///
    /// Used when tokenizing [text content][crate::content::text].
    pub label_start_stack: Vec<LabelStart>,
    /// Stack of label (start) that cannot form images and links.
    ///
    /// Used when tokenizing [text content][crate::content::text].
    pub label_start_list_loose: Vec<LabelStart>,
    /// Stack of images and links.
    ///
    /// Used when tokenizing [text content][crate::content::text].
    pub media_list: Vec<Media>,
    /// Current container state.
    pub container: Option<ContainerState>,
    /// Whether we would be interrupting something.
    ///
    /// Used when tokenizing [flow content][crate::content::flow].
    pub interrupt: bool,
    /// Whether containers cannot “pierce” into the current construct.
    ///
    /// Used when tokenizing [document content][crate::content::document].
    pub concrete: bool,
    /// Whether this line is lazy.
    ///
    /// The previous line was a paragraph, and this line’s containers did not
    /// match.
    pub lazy: bool,
}

impl<'a> Tokenizer<'a> {
    /// Create a new tokenizer.
    pub fn new(point: Point, parse_state: &'a ParseState) -> Tokenizer<'a> {
        Tokenizer {
            previous: Code::None,
            current: Code::None,
            // To do: reserve size when feeding?
            column_start: vec![],
            line_start: point.line,
            consumed: true,
            drained: false,
            point,
            stack: vec![],
            events: vec![],
            parse_state,
            index: 0,
            map: EditMap::new(),
            label_start_stack: vec![],
            label_start_list_loose: vec![],
            media_list: vec![],
            container: None,
            interrupt: false,
            concrete: false,
            lazy: false,
            // Assume about 10 resolvers.
            resolvers: Vec::with_capacity(10),
            resolver_ids: Vec::with_capacity(10),
        }
    }

    /// Register a resolver.
    pub fn register_resolver(&mut self, id: String, resolver: Box<Resolver>) {
        if !self.resolver_ids.contains(&id) {
            self.resolver_ids.push(id);
            self.resolvers.push(resolver);
        }
    }

    /// Register a resolver, before others.
    pub fn register_resolver_before(&mut self, id: String, resolver: Box<Resolver>) {
        if !self.resolver_ids.contains(&id) {
            self.resolver_ids.push(id);
            self.resolvers.insert(0, resolver);
        }
    }

    /// Prepare for a next code to get consumed.
    pub fn expect(&mut self, code: Code) {
        assert!(self.consumed, "expected previous character to be consumed");
        self.consumed = false;
        self.current = code;
    }

    /// Define a jump between two places.
    pub fn define_skip(&mut self, point: &Point) {
        define_skip_impl(self, point.line, point.index);
    }

    /// Define the current place as a jump between two places.
    pub fn define_skip_current(&mut self) {
        define_skip_impl(self, self.point.line, self.point.index);
    }

    /// Increment the current positional info if we’re right after a line
    /// ending, which has a skip defined.
    fn account_for_potential_skip(&mut self) {
        let at = self.point.line - self.line_start;

        if self.point.column == 1 && at != self.column_start.len() {
            self.move_to(self.column_start[at]);
        }
    }

    /// Consume the current character.
    /// Each [`StateFn`][] is expected to call this to signal that this code is
    /// used, or call a next `StateFn`.
    pub fn consume(&mut self) {
        log::debug!("consume: `{:?}` ({:?})", self.current, self.point);
        assert!(!self.consumed, "expected code to not have been consumed: this might be because `x(code)` instead of `x` was returned");

        self.move_to(self.index + 1);

        self.previous = self.current;
        // Mark as consumed.
        self.consumed = true;
    }

    /// To do.
    pub fn move_to(&mut self, to: usize) {
        while self.index < to {
            let code = &self.parse_state.codes[self.index];
            self.point.index += 1;
            self.index += 1;

            match code {
                Code::CarriageReturnLineFeed | Code::Char('\n' | '\r') => {
                    self.point.line += 1;
                    self.point.column = 1;
                    self.point.offset += if *code == Code::CarriageReturnLineFeed {
                        2
                    } else {
                        1
                    };

                    if self.point.line - self.line_start + 1 > self.column_start.len() {
                        self.column_start.push(self.point.index);
                    }

                    self.account_for_potential_skip();
                    log::debug!("position: after eol: `{:?}`", self.point);
                }
                Code::VirtualSpace => {
                    // Empty.
                }
                _ => {
                    self.point.column += 1;
                    self.point.offset += 1;
                }
            }
        }
    }

    /// Mark the start of a semantic label.
    pub fn enter(&mut self, token_type: Token) {
        self.enter_with_link(token_type, None);
    }

    pub fn enter_with_content(&mut self, token_type: Token, content_type_opt: Option<ContentType>) {
        self.enter_with_link(
            token_type,
            content_type_opt.map(|content_type| Link {
                content_type,
                previous: None,
                next: None,
            }),
        );
    }

    pub fn enter_with_link(&mut self, token_type: Token, link: Option<Link>) {
        log::debug!("enter: `{:?}` ({:?})", token_type, self.point);
        self.events.push(Event {
            event_type: EventType::Enter,
            token_type: token_type.clone(),
            point: self.point.clone(),
            link,
        });
        self.stack.push(token_type);
    }

    /// Mark the end of a semantic label.
    pub fn exit(&mut self, token_type: Token) {
        let current_token = self.stack.pop().expect("cannot close w/o open tokens");

        assert_eq!(
            current_token, token_type,
            "expected exit token to match current token"
        );

        let previous = self.events.last().expect("cannot close w/o open event");
        let mut point = self.point.clone();

        assert!(
            current_token != previous.token_type || previous.point.index != point.index,
            "expected non-empty token"
        );

        if VOID_TOKENS.iter().any(|d| d == &token_type) {
            assert!(
                current_token == previous.token_type,
                "expected token to be void (`{:?}`), instead of including `{:?}`",
                current_token,
                previous.token_type
            );
        }

        // A bit weird, but if we exit right after a line ending, we *don’t* want to consider
        // potential skips.
        if matches!(
            self.previous,
            Code::CarriageReturnLineFeed | Code::Char('\n' | '\r')
        ) {
            point.column = 1;
            point.offset = previous.point.offset
                + if self.previous == Code::CarriageReturnLineFeed {
                    2
                } else {
                    1
                };
            point.index = previous.point.index + 1;
        }

        log::debug!("exit: `{:?}` ({:?})", token_type, point);
        self.events.push(Event {
            event_type: EventType::Exit,
            token_type,
            point,
            link: None,
        });
    }

    /// Capture the internal state.
    fn capture(&mut self) -> InternalState {
        InternalState {
            previous: self.previous,
            current: self.current,
            point: self.point.clone(),
            events_len: self.events.len(),
            stack_len: self.stack.len(),
        }
    }

    /// Apply the internal state.
    fn free(&mut self, previous: InternalState) {
        self.previous = previous.previous;
        self.current = previous.current;
        self.point = previous.point;
        assert!(
            self.events.len() >= previous.events_len,
            "expected to restore less events than before"
        );
        self.events.truncate(previous.events_len);
        assert!(
            self.stack.len() >= previous.stack_len,
            "expected to restore less stack items than before"
        );
        self.stack.truncate(previous.stack_len);
    }

    /// Parse with `state_fn` and its future states, switching to `ok` when
    /// successful, and passing [`State::Nok`][] back up if it occurs.
    ///
    /// This function does not capture the current state, in case of
    /// `State::Nok`, as it is assumed that this `go` is itself wrapped in
    /// another `attempt`.
    #[allow(clippy::unused_self)]
    pub fn go(
        &mut self,
        state_fn: impl FnOnce(&mut Tokenizer) -> State + 'static,
        after: impl FnOnce(&mut Tokenizer) -> State + 'static,
    ) -> Box<StateFn> {
        attempt_impl(
            state_fn,
            None,
            self.index,
            |end: usize, tokenizer: &mut Tokenizer, state| {
                if matches!(state, State::Ok) {
                    tokenizer.index = end;
                    tokenizer.consumed = true;
                    State::Fn(Box::new(after))
                } else {
                    state
                }
            },
        )
    }

    /// Like `go`, but this lets you *hijack* back to some other state after a
    /// certain code.
    #[allow(clippy::unused_self)]
    pub fn go_until(
        &mut self,
        state_fn: impl FnOnce(&mut Tokenizer) -> State + 'static,
        until: impl Fn(Code) -> bool + 'static,
        done: impl FnOnce(State) -> Box<StateFn> + 'static,
    ) -> Box<StateFn> {
        attempt_impl(
            state_fn,
            Some(Box::new(until)),
            self.index,
            |end: usize, tokenizer: &mut Tokenizer, state| {
                tokenizer.index = end;
                tokenizer.consumed = true;
                State::Fn(done(state))
            },
        )
    }

    /// Parse with `state_fn` and its future states, to check if it result in
    /// [`State::Ok`][] or [`State::Nok`][], revert on both cases, and then
    /// call `done` with whether it was successful or not.
    ///
    /// This captures the current state of the tokenizer, returns a wrapped
    /// state that captures all codes and feeds them to `state_fn` and its
    /// future states until it yields `State::Ok` or `State::Nok`.
    /// It then applies the captured state, calls `done`, and feeds all
    /// captured codes to its future states.
    pub fn check(
        &mut self,
        state_fn: impl FnOnce(&mut Tokenizer) -> State + 'static,
        done: impl FnOnce(bool) -> Box<StateFn> + 'static,
    ) -> Box<StateFn> {
        let previous = self.capture();
        let start = self.index;

        attempt_impl(
            state_fn,
            None,
            start,
            move |_: usize, tokenizer: &mut Tokenizer, state| {
                tokenizer.free(previous);
                tokenizer.index = start;
                tokenizer.consumed = true;
                State::Fn(done(matches!(state, State::Ok)))
            },
        )
    }

    /// Parse with `state_fn` and its future states, to check if it results in
    /// [`State::Ok`][] or [`State::Nok`][], revert on the case of
    /// `State::Nok`, and then call `done` with whether it was successful or
    /// not.
    ///
    /// This captures the current state of the tokenizer, returns a wrapped
    /// state that captures all codes and feeds them to `state_fn` and its
    /// future states until it yields `State::Ok`, at which point it calls
    /// `done` and yields its result.
    /// If instead `State::Nok` was yielded, the captured state is applied,
    /// `done` is called, and all captured codes are fed to its future states.
    pub fn attempt(
        &mut self,
        state_fn: impl FnOnce(&mut Tokenizer) -> State + 'static,
        done: impl FnOnce(bool) -> Box<StateFn> + 'static,
    ) -> Box<StateFn> {
        let previous = self.capture();

        attempt_impl(
            state_fn,
            None,
            self.index,
            |end: usize, tokenizer: &mut Tokenizer, state| {
                let ok = matches!(state, State::Ok);

                if !ok {
                    tokenizer.free(previous);
                }

                log::debug!("attempt: {:?}, at {:?}", ok, tokenizer.point);

                tokenizer.index = end;
                tokenizer.consumed = true;
                State::Fn(done(ok))
            },
        )
    }

    /// Just like [`attempt`][Tokenizer::attempt], but many.
    pub fn attempt_n(
        &mut self,
        mut state_fns: Vec<Box<StateFn>>,
        done: impl FnOnce(bool) -> Box<StateFn> + 'static,
    ) -> Box<StateFn> {
        if state_fns.is_empty() {
            done(false)
        } else {
            let state_fn = state_fns.remove(0);
            self.attempt(state_fn, move |ok| {
                if ok {
                    done(ok)
                } else {
                    Box::new(|t| t.attempt_n(state_fns, done)(t))
                }
            })
        }
    }

    /// Just like [`attempt`][Tokenizer::attempt], but for when you don’t care
    /// about `ok`.
    pub fn attempt_opt(
        &mut self,
        state_fn: impl FnOnce(&mut Tokenizer) -> State + 'static,
        after: impl FnOnce(&mut Tokenizer) -> State + 'static,
    ) -> Box<StateFn> {
        self.attempt(state_fn, |_ok| Box::new(after))
    }

    /// Feed a list of `codes` into `start`.
    ///
    /// This is set up to support repeatedly calling `feed`, and thus streaming
    /// markdown into the state machine, and normally pauses after feeding.
    pub fn push(
        &mut self,
        min: usize,
        max: usize,
        start: impl FnOnce(&mut Tokenizer) -> State + 'static,
        drain: bool,
    ) -> State {
        assert!(!self.drained, "cannot feed after drain");

        let mut result = feed_impl(self, min, max, start);

        if drain {
            let func = match result {
                State::Fn(func) => func,
                _ => unreachable!("expected next state"),
            };

            result = flush_impl(self, max, func);

            self.drained = true;

            while !self.resolvers.is_empty() {
                let resolver = self.resolvers.remove(0);
                resolver(self);
            }

            self.map.consume(&mut self.events);
        }

        result
    }

    /// Flush the tokenizer.
    pub fn flush(&mut self, start: impl FnOnce(&mut Tokenizer) -> State + 'static) -> State {
        flush_impl(self, self.index, start)
    }
}

/// Internal utility to wrap states to also capture codes.
///
/// Recurses into itself.
/// Used in [`Tokenizer::attempt`][Tokenizer::attempt] and  [`Tokenizer::check`][Tokenizer::check].
fn attempt_impl(
    state: impl FnOnce(&mut Tokenizer) -> State + 'static,
    pause: Option<Box<dyn Fn(Code) -> bool + 'static>>,
    start: usize,
    done: impl FnOnce(usize, &mut Tokenizer, State) -> State + 'static,
) -> Box<StateFn> {
    Box::new(move |tokenizer| {
        if let Some(ref func) = pause {
            if tokenizer.index > start && func(tokenizer.previous) {
                return done(tokenizer.index, tokenizer, State::Fn(Box::new(state)));
            }
        }

        let state = state(tokenizer);

        match state {
            State::Ok => {
                assert!(
                    tokenizer.index >= start,
                    "`end` must not be smaller than `start`"
                );
                done(tokenizer.index, tokenizer, state)
            }
            State::Nok => done(start, tokenizer, state),
            State::Fn(func) => State::Fn(attempt_impl(func, pause, start, done)),
        }
    })
}

/// Feed a list of `codes` into `start`.
fn feed_impl(
    tokenizer: &mut Tokenizer,
    min: usize,
    max: usize,
    start: impl FnOnce(&mut Tokenizer) -> State + 'static,
) -> State {
    let mut state = State::Fn(Box::new(start));

    tokenizer.index = min;
    tokenizer.consumed = true;

    while tokenizer.index < max {
        let code = tokenizer.parse_state.codes[tokenizer.index];

        match state {
            State::Ok | State::Nok => {
                break;
            }
            State::Fn(func) => {
                log::debug!("main: passing: `{:?}` ({:?})", code, tokenizer.index);
                tokenizer.expect(code);
                state = func(tokenizer);
            }
        }
    }

    state
}

/// Flush `start`: pass `eof`s to it until done.
fn flush_impl(
    tokenizer: &mut Tokenizer,
    max: usize,
    start: impl FnOnce(&mut Tokenizer) -> State + 'static,
) -> State {
    let mut state = State::Fn(Box::new(start));
    tokenizer.consumed = true;

    loop {
        match state {
            State::Ok | State::Nok => break,
            State::Fn(func) => {
                let code = if tokenizer.index < max {
                    tokenizer.parse_state.codes[tokenizer.index]
                } else {
                    Code::None
                };
                log::debug!("main: flushing {:?}", code);
                tokenizer.expect(code);
                state = func(tokenizer);
            }
        }
    }

    match state {
        State::Ok => {}
        _ => unreachable!("expected final state to be `State::Ok`"),
    }

    state
}

/// Define a jump between two places.
///
/// This defines how much columns, offsets, and the `index` are increased when
/// consuming a line ending.
fn define_skip_impl(tokenizer: &mut Tokenizer, line: usize, index: usize) {
    log::debug!("position: define skip: {:?} -> ({:?})", line, index);
    let at = line - tokenizer.line_start;

    if at == tokenizer.column_start.len() {
        tokenizer.column_start.push(index);
    } else {
        tokenizer.column_start[at] = index;
    }

    tokenizer.account_for_potential_skip();
}
