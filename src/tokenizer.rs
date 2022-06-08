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

/// Semantic label of a span.
// To do: figure out how to share this so extensions can add their own stuff,
// though perhaps that’s impossible and we should inline all extensions?
// To do: document each variant.
#[derive(Debug, Clone, PartialEq)]
pub enum TokenType {
    AtxHeading,
    AtxHeadingSequence,
    AtxHeadingWhitespace,
    AtxHeadingText,

    CharacterEscape,
    CharacterEscapeMarker,
    CharacterEscapeValue,

    CharacterReference,
    CharacterReferenceMarker,
    CharacterReferenceMarkerNumeric,
    CharacterReferenceMarkerHexadecimal,
    CharacterReferenceMarkerSemi,
    CharacterReferenceValue,

    CodeFenced,
    CodeFencedFence,
    CodeFencedFenceSequence,
    CodeFencedFenceWhitespace,
    CodeFencedFenceInfo,
    CodeFencedFenceMeta,

    CodeIndented,
    CodeIndentedPrefixWhitespace,

    CodeFlowChunk,

    Data,

    HtmlFlow,
    HtmlFlowData,

    ThematicBreak,
    ThematicBreakSequence,
    ThematicBreakWhitespace,

    Whitespace,
    LineEnding,
    BlankLineEnding,
    BlankLineWhitespace,

    Content,
    ContentPhrasing,
    ChunkString,
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
#[derive(Debug, Clone, PartialEq)]
pub struct Point {
    /// 1-indexed line number.
    pub line: usize,
    /// 1-indexed column number.
    /// Note that this is increases up to a tab stop for tabs.
    /// Some editors count tabs as 1 character, so this position is not always
    /// the same as editors.
    pub column: usize,
    /// 0-indexed position in the document.
    pub offset: usize,
}

/// Possible event types.
#[derive(Debug, PartialEq)]
pub enum EventType {
    /// The start of something.
    Enter,
    /// The end of something.
    Exit,
}

/// Something semantic happening somewhere.
#[derive(Debug)]
pub struct Event {
    pub event_type: EventType,
    pub token_type: TokenType,
    pub point: Point,
    pub index: usize,
}

/// The essence of the state machine are functions: `StateFn`.
/// It’s responsible for dealing with that single passed [`Code`][].
/// It yields a [`StateFnResult`][].
pub type StateFn = dyn FnOnce(&mut Tokenizer, Code) -> StateFnResult;
/// Each [`StateFn`][] yields something back: primarily the state.
/// In certain cases, it can also yield back up parsed codes that were passed down.
pub type StateFnResult = (State, Option<Vec<Code>>);

/// The result of a state.
pub enum State {
    /// There is a future state: a boxed [`StateFn`][] to pass the next code to.
    Fn(Box<StateFn>),
    /// The state is successful.
    Ok,
    /// The state is not successful.
    Nok,
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
    /// Current code.
    current: Code,
    /// `index` in codes of the current code.
    index: usize,
    /// Current relative and absolute position in the file.
    point: Point,
}

/// A tokenizer itself.
#[derive(Debug)]
pub struct Tokenizer {
    /// Track whether a character is expected to be consumed, and whether it’s
    /// actually consumed
    ///
    /// Tracked to make sure everything’s valid.
    consumed: bool,
    /// Semantic labels of one or more codes in `codes`.
    pub events: Vec<Event>,
    /// Hierarchy of semantic labels.
    ///
    /// Tracked to make sure everything’s valid.
    stack: Vec<TokenType>,
    /// Current character code.
    current: Code,
    /// `index` in codes of the current code.
    index: usize,
    /// Current relative and absolute place in the file.
    point: Point,
}

impl Tokenizer {
    /// Create a new tokenizer.
    pub fn new() -> Tokenizer {
        Tokenizer {
            current: Code::None,
            index: 0,
            consumed: true,
            point: Point {
                line: 1,
                column: 1,
                offset: 0,
            },
            stack: vec![],
            events: vec![],
        }
    }

    /// Prepare for a next code to get consumed.
    fn expect(&mut self, code: Code) {
        assert!(self.consumed, "expected previous character to be consumed");
        self.consumed = false;
        self.current = code;
    }

    /// Consume the current character.
    /// Each [`StateFn`][] is expected to call this to signal that this code is
    /// used, or call a next `StateFn`.
    pub fn consume(&mut self, code: Code) {
        assert_eq!(
            code, self.current,
            "expected given code to equal expected code"
        );
        log::debug!("consume: `{:?}` ({:?})", code, self.point);
        assert!(!self.consumed, "expected code to not have been consumed: this might be because `x(code)` instead of `x` was returned");

        match code {
            Code::CarriageReturnLineFeed | Code::Char('\n' | '\r') => {
                self.point.line += 1;
                self.point.column = 1;
                self.point.offset += if code == Code::CarriageReturnLineFeed {
                    2
                } else {
                    1
                };
                // To do: accountForPotentialSkip()
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

        self.index += 1;
        // Mark as consumed.
        self.consumed = true;
    }

    /// Mark the start of a semantic label.
    pub fn enter(&mut self, token_type: TokenType) {
        log::debug!("enter `{:?}` ({:?})", token_type, self.point);
        let event = Event {
            event_type: EventType::Enter,
            token_type: token_type.clone(),
            point: self.point.clone(),
            index: self.index,
        };

        self.events.push(event);
        self.stack.push(token_type);
    }

    /// Mark the end of a semantic label.
    pub fn exit(&mut self, token_type: TokenType) {
        let token_on_stack = self.stack.pop().expect("cannot close w/o open tokens");

        assert_eq!(
            token_on_stack, token_type,
            "expected exit TokenType to match current TokenType"
        );

        let ev = self.events.last().expect("cannot close w/o open event");

        let point = self.point.clone();

        assert!(
            token_on_stack != ev.token_type || ev.point != point,
            "expected non-empty TokenType"
        );

        log::debug!("exit `{:?}` ({:?})", token_type, self.point);
        let event = Event {
            event_type: EventType::Exit,
            token_type,
            point,
            index: self.index,
        };

        self.events.push(event);
    }

    /// Capture the internal state.
    fn capture(&mut self) -> InternalState {
        InternalState {
            index: self.index,
            current: self.current,
            point: self.point.clone(),
            events_len: self.events.len(),
            stack_len: self.stack.len(),
        }
    }

    /// Apply the internal state.
    fn free(&mut self, previous: InternalState) {
        self.index = previous.index;
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

    /// Check if `state` and its future states are successful or not.
    ///
    /// This captures the current state of the tokenizer, returns a wrapped
    /// state that captures all codes and feeds them to `state` and its future
    /// states until it yields [`State::Ok`][] or [`State::Nok`][].
    /// It then applies the captured state, calls `done`, and feeds all
    /// captured codes to its future states.
    pub fn check(
        &mut self,
        state: impl FnOnce(&mut Tokenizer, Code) -> StateFnResult + 'static,
        done: impl FnOnce(bool) -> Box<StateFn> + 'static,
    ) -> Box<StateFn> {
        let previous = self.capture();

        attempt_impl(
            state,
            vec![],
            |result: (Vec<Code>, Vec<Code>), ok, tokenizer: &mut Tokenizer| {
                let codes = result.0;
                tokenizer.free(previous);
                log::debug!(
                    "check: {:?}, codes: {:?}, at {:?}",
                    ok,
                    codes,
                    tokenizer.point
                );
                let result = done(ok);
                tokenizer.feed(codes, result, false)
            },
        )
    }

    /// Attempt to parse with `state` and its future states, reverting if
    /// unsuccessful.
    ///
    /// This captures the current state of the tokenizer, returns a wrapped
    /// state that captures all codes and feeds them to `state` and its future
    /// states until it yields [`State::Ok`][], at which point it calls `done`
    /// and yields its result.
    /// If instead [`State::Nok`][] was yielded, the captured state is applied,
    /// `done` is called, and all captured codes are fed to its future states.
    pub fn attempt(
        &mut self,
        state: impl FnOnce(&mut Tokenizer, Code) -> StateFnResult + 'static,
        done: impl FnOnce(bool) -> Box<StateFn> + 'static,
    ) -> Box<StateFn> {
        let previous = self.capture();

        attempt_impl(
            state,
            vec![],
            |result: (Vec<Code>, Vec<Code>), ok, tokenizer: &mut Tokenizer| {
                let codes = if ok {
                    result.1
                } else {
                    tokenizer.free(previous);
                    result.0
                };

                log::debug!(
                    "attempt: {:?}, codes: {:?}, at {:?}",
                    ok,
                    codes,
                    tokenizer.point
                );
                let result = done(ok);
                tokenizer.feed(codes, result, false)
            },
        )
    }

    /// Feed a list of `codes` into `start`.
    ///
    /// This is set up to support repeatedly calling `feed`, and thus streaming
    /// markdown into the state machine, and normally pauses after feeding.
    /// When `done: true` is passed, the EOF is fed.
    pub fn feed(
        &mut self,
        codes: Vec<Code>,
        start: impl FnOnce(&mut Tokenizer, Code) -> StateFnResult + 'static,
        drain: bool,
    ) -> StateFnResult {
        let mut codes = codes;
        let mut state = State::Fn(Box::new(start));
        let mut index = 0;

        self.consumed = true;

        while index < codes.len() {
            let code = codes[index];

            match state {
                State::Nok | State::Ok => {
                    break;
                }
                State::Fn(func) => {
                    log::debug!("main: passing `{:?}`", code);
                    self.expect(code);
                    let (next, remainder) = check_statefn_result(func(self, code));
                    state = next;
                    index = index + 1
                        - (if let Some(ref x) = remainder {
                            x.len()
                        } else {
                            0
                        });
                }
            }
        }

        // Yield to a higher loop if we shouldn’t feed EOFs.
        if !drain {
            return (state, Some(codes.split_off(index)));
        }

        loop {
            // Feed EOF.
            match state {
                State::Ok | State::Nok => break,
                State::Fn(func) => {
                    let code = Code::None;
                    log::debug!("main: passing eof");
                    self.expect(code);
                    let (next, remainder) = check_statefn_result(func(self, code));

                    if let Some(ref x) = remainder {
                        if !x.is_empty() {
                            // To do: handle?
                            unreachable!("drain:remainder {:?}", x);
                        }
                    }

                    state = next;
                }
            }
        }

        check_statefn_result((state, None))
    }
}

/// Internal utility to wrap states to also capture codes.
///
/// Recurses into itself.
/// Used in [`Tokenizer::attempt`][Tokenizer::attempt] and  [`Tokenizer::check`][Tokenizer::check].
fn attempt_impl(
    state: impl FnOnce(&mut Tokenizer, Code) -> StateFnResult + 'static,
    codes: Vec<Code>,
    done: impl FnOnce((Vec<Code>, Vec<Code>), bool, &mut Tokenizer) -> StateFnResult + 'static,
) -> Box<StateFn> {
    Box::new(|tokenizer, code| {
        let mut codes = codes;

        let (next, remainder) = check_statefn_result(state(tokenizer, code));

        match code {
            Code::None => {}
            _ => {
                codes.push(code);
            }
        }

        // To do: `remainder` must never be bigger than codes I guess?
        // To do: `remainder` probably has to be taken *from* `codes`, in a similar vain to the `Ok` handling below.
        match next {
            State::Ok => {
                let remaining = if let Some(x) = remainder { x } else { vec![] };
                check_statefn_result(done((codes, remaining), true, tokenizer))
            }
            State::Nok => check_statefn_result(done((codes, vec![]), false, tokenizer)),
            State::Fn(func) => {
                check_statefn_result((State::Fn(attempt_impl(func, codes, done)), None))
            }
        }
    })
}

/// Turn a string into codes.
// To do: handle BOM at start?
pub fn as_codes(value: &str) -> Vec<Code> {
    let mut codes: Vec<Code> = vec![];
    let mut at_carriage_return = false;
    let mut column = 1;

    for char in value.chars() {
        // Send a CRLF.
        if at_carriage_return && '\n' == char {
            at_carriage_return = false;
            codes.push(Code::CarriageReturnLineFeed);
        } else {
            // Send the previous CR: we’re not at a next `\n`.
            if at_carriage_return {
                at_carriage_return = false;
                codes.push(Code::Char('\r'));
            }

            match char {
                // Send a replacement character.
                '\0' => {
                    column += 1;
                    codes.push(Code::Char('�'));
                }
                // Send a tab and virtual spaces.
                '\t' => {
                    // To do: is this correct?
                    let virtual_spaces = TAB_SIZE - (column % TAB_SIZE);
                    println!("tabs, expand {:?}, {:?}", column, virtual_spaces);
                    codes.push(Code::Char(char));
                    column += 1;
                    let mut index = 0;
                    while index < virtual_spaces {
                        codes.push(Code::VirtualSpace);
                        column += 1;
                        index += 1;
                    }
                }
                // Send an LF.
                '\n' => {
                    column = 1;
                    codes.push(Code::Char(char));
                }
                // Don’t send anything yet.
                '\r' => {
                    column = 1;
                    at_carriage_return = true;
                }
                // Send the char.
                _ => {
                    column += 1;
                    codes.push(Code::Char(char));
                }
            }
        };
    }

    // To do: handle a final CR?

    codes
}

/// Check a [`StateFnResult`][], make sure its valid (that there are no bugs),
/// and clean a final eof passed back in `remainder`.
fn check_statefn_result(result: StateFnResult) -> StateFnResult {
    let (state, mut remainder) = result;

    match state {
        State::Nok | State::Fn(_) => {
            if let Some(ref x) = remainder {
                assert_eq!(
                    x.len(),
                    0,
                    "expected `None` to be passed back as remainder from `State::Nok`, `State::Fn`"
                );
            }
        }
        State::Ok => {}
    }

    // Remove an eof.
    // For convencience, feeding back an eof is allowed, but cleaned here.
    // Most states handle eof and eol in the same branch, and hence pass
    // all back.
    // This might not be needed, because if EOF is passed back, we’re at the EOF.
    // But they’re not supposed to be in codes, so here we remove them.
    if let Some(ref mut list) = remainder {
        if Some(&Code::None) == list.last() {
            list.pop();
        }
    }

    (state, remainder)
}
