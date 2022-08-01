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

use crate::constant::TAB_SIZE;
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

#[derive(Debug, PartialEq)]
pub enum ByteAction {
    Normal(u8),
    Insert(u8),
    Ignore,
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
    /// Some editors count tabs as 1 character, so this position is not the
    /// same as editors.
    pub column: usize,
    /// 0-indexed position in the document.
    ///
    /// Also an `index` into `bytes`.
    pub index: usize,
    /// Virtual step on the same `index`.
    pub vs: usize,
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
/// It’s responsible for dealing with the current byte.
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
    previous: Option<u8>,
    /// Current code.
    current: Option<u8>,
    /// Current relative and absolute position in the file.
    point: Point,
}

/// A tokenizer itself.
#[allow(clippy::struct_excessive_bools)]
pub struct Tokenizer<'a> {
    /// Jump between line endings.
    column_start: Vec<(usize, usize)>,
    // First line.
    first_line: usize,
    /// First point after the last line ending.
    line_start: Point,
    /// Track whether the current byte is already consumed (`true`) or expected
    /// to be consumed (`false`).
    ///
    /// Tracked to make sure everything’s valid.
    consumed: bool,
    /// Track whether this tokenizer is done.
    resolved: bool,
    /// Current byte.
    pub current: Option<u8>,
    /// Previous byte.
    pub previous: Option<u8>,
    /// Current relative and absolute place in the file.
    pub point: Point,
    /// Semantic labels of one or more codes in `codes`.
    pub events: Vec<Event>,
    /// Hierarchy of semantic labels.
    ///
    /// Tracked to make sure everything’s valid.
    pub stack: Vec<Token>,
    /// Edit map, to batch changes.
    pub map: EditMap,
    /// List of attached resolvers, which will be called when done feeding,
    /// to clean events.
    resolvers: Vec<Box<Resolver>>,
    /// List of names associated with attached resolvers.
    resolver_ids: Vec<String>,
    /// Shared parsing state across tokenizers.
    pub parse_state: &'a ParseState<'a>,
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
            previous: None,
            current: None,
            // To do: reserve size when feeding?
            column_start: vec![],
            first_line: point.line,
            line_start: point.clone(),
            consumed: true,
            resolved: false,
            point,
            stack: vec![],
            events: vec![],
            parse_state,
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

    /// Define a jump between two places.
    pub fn define_skip(&mut self, point: &Point) {
        define_skip_impl(self, point.line, (point.index, point.vs));
    }

    /// Define the current place as a jump between two places.
    pub fn define_skip_current(&mut self) {
        define_skip_impl(self, self.point.line, (self.point.index, self.point.vs));
    }

    /// Increment the current positional info if we’re right after a line
    /// ending, which has a skip defined.
    fn account_for_potential_skip(&mut self) {
        let at = self.point.line - self.first_line;

        if self.point.column == 1 && at != self.column_start.len() {
            self.move_to(self.column_start[at]);
        }
    }

    /// Prepare for a next code to get consumed.
    pub fn expect(&mut self, byte: Option<u8>) {
        debug_assert!(self.consumed, "expected previous byte to be consumed");
        self.consumed = false;
        self.current = byte;
    }

    /// Consume the current byte.
    /// Each [`StateFn`][] is expected to call this to signal that this code is
    /// used, or call a next `StateFn`.
    pub fn consume(&mut self) {
        log::debug!("consume: `{:?}` ({:?})", self.current, self.point);
        debug_assert!(!self.consumed, "expected code to not have been consumed: this might be because `x(code)` instead of `x` was returned");

        self.move_one();

        self.previous = self.current;
        // While we’re not at the eof, it is at least better to not have the
        // same current code as `previous` *and* `current`.
        self.current = None;
        // Mark as consumed.
        self.consumed = true;
    }

    /// Move to the next (virtual) byte.
    pub fn move_one(&mut self) {
        match byte_action(self.parse_state.bytes, &self.point) {
            ByteAction::Ignore => {
                self.point.index += 1;
            }
            ByteAction::Insert(byte) => {
                self.previous = Some(byte);
                self.point.column += 1;
                self.point.vs += 1;
            }
            ByteAction::Normal(byte) => {
                self.previous = Some(byte);
                self.point.vs = 0;
                self.point.index += 1;

                if byte == b'\n' {
                    self.point.line += 1;
                    self.point.column = 1;

                    if self.point.line - self.first_line + 1 > self.column_start.len() {
                        self.column_start.push((self.point.index, self.point.vs));
                    }

                    self.line_start = self.point.clone();

                    self.account_for_potential_skip();
                    log::debug!("position: after eol: `{:?}`", self.point);
                } else {
                    self.point.column += 1;
                }
            }
        }
    }

    /// Move (virtual) bytes.
    pub fn move_to(&mut self, to: (usize, usize)) {
        let (to_index, to_vs) = to;
        while self.point.index < to_index || self.point.index == to_index && self.point.vs < to_vs {
            self.move_one();
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
        let mut point = self.point.clone();

        // Move back past ignored bytes.
        while point.index > 0 {
            point.index -= 1;
            let action = byte_action(self.parse_state.bytes, &point);
            if !matches!(action, ByteAction::Ignore) {
                point.index += 1;
                break;
            }
        }

        log::debug!("enter: `{:?}` ({:?})", token_type, point);
        self.events.push(Event {
            event_type: EventType::Enter,
            token_type: token_type.clone(),
            point,
            link,
        });
        self.stack.push(token_type);
    }

    /// Mark the end of a semantic label.
    pub fn exit(&mut self, token_type: Token) {
        let current_token = self.stack.pop().expect("cannot close w/o open tokens");

        debug_assert_eq!(
            current_token, token_type,
            "expected exit token to match current token"
        );

        let previous = self.events.last().expect("cannot close w/o open event");
        let mut point = self.point.clone();

        debug_assert!(
            current_token != previous.token_type
                || previous.point.index != point.index
                || previous.point.vs != point.vs,
            "expected non-empty token"
        );

        if VOID_TOKENS.iter().any(|d| d == &token_type) {
            debug_assert!(
                current_token == previous.token_type,
                "expected token to be void (`{:?}`), instead of including `{:?}`",
                current_token,
                previous.token_type
            );
        }

        // A bit weird, but if we exit right after a line ending, we *don’t* want to consider
        // potential skips.
        if matches!(self.previous, Some(b'\n')) {
            point = self.line_start.clone();
        } else {
            // Move back past ignored bytes.
            while point.index > 0 {
                point.index -= 1;
                let action = byte_action(self.parse_state.bytes, &point);
                if !matches!(action, ByteAction::Ignore) {
                    point.index += 1;
                    break;
                }
            }
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
        debug_assert!(
            self.events.len() >= previous.events_len,
            "expected to restore less events than before"
        );
        self.events.truncate(previous.events_len);
        debug_assert!(
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
            self.point.index,
            |tokenizer: &mut Tokenizer, state| {
                if matches!(state, State::Ok) {
                    tokenizer.consumed = true;
                    State::Fn(Box::new(after))
                } else {
                    // Must be `Nok`.
                    // We don’t capture/free state because it is assumed that
                    // `go` itself is wrapped in another attempt that does that
                    // if it can occur.
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
        until: impl Fn(Option<u8>) -> bool + 'static,
        done: impl FnOnce(State) -> Box<StateFn> + 'static,
    ) -> Box<StateFn> {
        attempt_impl(
            state_fn,
            Some(Box::new(until)),
            self.point.index,
            |tokenizer: &mut Tokenizer, state| {
                tokenizer.consumed = true;
                // We don’t capture/free state because it is assumed that
                // `go_until` itself is wrapped in another attempt that does
                // that if it can occur.
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

        attempt_impl(
            state_fn,
            None,
            self.point.index,
            |tokenizer: &mut Tokenizer, state| {
                tokenizer.free(previous);
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
            self.point.index,
            |tokenizer: &mut Tokenizer, state| {
                let ok = matches!(state, State::Ok);

                if !ok {
                    tokenizer.free(previous);
                }

                log::debug!("attempt: {:?}, at {:?}", ok, tokenizer.point);

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
    // Note: if needed: accept `vs`?
    pub fn push(
        &mut self,
        min: usize,
        max: usize,
        start: impl FnOnce(&mut Tokenizer) -> State + 'static,
    ) -> State {
        debug_assert!(!self.resolved, "cannot feed after drain");
        debug_assert!(min >= self.point.index, "cannot move backwards");
        self.move_to((min, 0));

        let mut state = State::Fn(Box::new(start));

        while self.point.index < max {
            match state {
                State::Ok | State::Nok => break,
                State::Fn(func) => match byte_action(self.parse_state.bytes, &self.point) {
                    ByteAction::Ignore => {
                        state = State::Fn(Box::new(func));
                        self.move_one();
                    }
                    ByteAction::Insert(byte) | ByteAction::Normal(byte) => {
                        log::debug!("main: passing: `{:?}` ({:?})", byte, self.point);
                        self.expect(Some(byte));
                        state = func(self);
                    }
                },
            }
        }

        state
    }

    /// Flush the tokenizer.
    pub fn flush(&mut self, mut state: State, resolve: bool) {
        let max = self.point.index;

        self.consumed = true;

        loop {
            match state {
                State::Ok | State::Nok => break,
                State::Fn(func) => {
                    // We sometimes move back when flushing, so then we use those codes.
                    let action = if self.point.index == max {
                        None
                    } else {
                        Some(byte_action(self.parse_state.bytes, &self.point))
                    };

                    if let Some(ByteAction::Ignore) = action {
                        state = State::Fn(Box::new(func));
                        self.move_one();
                    } else {
                        let byte =
                            if let Some(ByteAction::Insert(byte) | ByteAction::Normal(byte)) =
                                action
                            {
                                Some(byte)
                            } else {
                                None
                            };

                        log::debug!("main: flushing: `{:?}` ({:?})", byte, self.point);
                        self.expect(byte);
                        state = func(self);
                    }
                }
            }
        }

        debug_assert!(matches!(state, State::Ok), "must be ok");

        if resolve {
            self.resolved = true;

            while !self.resolvers.is_empty() {
                let resolver = self.resolvers.remove(0);
                resolver(self);
            }

            self.map.consume(&mut self.events);
        }
    }
}

fn byte_action(bytes: &[u8], point: &Point) -> ByteAction {
    if point.index < bytes.len() {
        let byte = bytes[point.index];

        if byte == b'\r' {
            // CRLF.
            if point.index < bytes.len() - 1 && bytes[point.index + 1] == b'\n' {
                ByteAction::Ignore
            }
            // CR.
            else {
                ByteAction::Normal(b'\n')
            }
        } else if byte == b'\t' {
            let remainder = point.column % TAB_SIZE;
            let vs = if remainder == 0 {
                0
            } else {
                TAB_SIZE - remainder
            };

            // On the tab itself, first send it.
            if point.vs == 0 {
                if vs == 0 {
                    ByteAction::Normal(byte)
                } else {
                    ByteAction::Insert(byte)
                }
            } else if vs == 0 {
                ByteAction::Normal(b' ')
            } else {
                ByteAction::Insert(b' ')
            }
        } else {
            ByteAction::Normal(byte)
        }
    } else {
        unreachable!("out of bounds")
    }
}

/// Internal utility to wrap states to also capture codes.
///
/// Recurses into itself.
/// Used in [`Tokenizer::attempt`][Tokenizer::attempt] and  [`Tokenizer::check`][Tokenizer::check].
fn attempt_impl(
    state: impl FnOnce(&mut Tokenizer) -> State + 'static,
    pause: Option<Box<dyn Fn(Option<u8>) -> bool + 'static>>,
    start: usize,
    done: impl FnOnce(&mut Tokenizer, State) -> State + 'static,
) -> Box<StateFn> {
    Box::new(move |tokenizer| {
        if let Some(ref func) = pause {
            if tokenizer.point.index > start && func(tokenizer.previous) {
                return done(tokenizer, State::Fn(Box::new(state)));
            }
        }

        let state = state(tokenizer);

        match state {
            State::Ok | State::Nok => done(tokenizer, state),
            State::Fn(func) => State::Fn(attempt_impl(func, pause, start, done)),
        }
    })
}

/// Flush `start`: pass `eof`s to it until done.
/// Define a jump between two places.
///
/// This defines to which future index we move after a line ending.
fn define_skip_impl(tokenizer: &mut Tokenizer, line: usize, info: (usize, usize)) {
    log::debug!("position: define skip: {:?} -> ({:?})", line, info);
    let at = line - tokenizer.first_line;

    if at >= tokenizer.column_start.len() {
        tokenizer.column_start.push(info);
    } else {
        tokenizer.column_start[at] = info;
    }

    tokenizer.account_for_potential_skip();
}
