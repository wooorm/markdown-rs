//! A tokenizer glues states from the state machine together.
//!
//! It facilitates everything needed to turn bytes into events with a state
//! machine.
//! It also enables the logic needed for parsing markdown, such as an
//! [`attempt`][] to try and parse something, which can succeed or, when
//! unsuccessful, revert the attempt.
//!
//! [`attempt`]: Tokenizer::attempt

use crate::event::{Content, Event, Kind, Link, Name, Point, VOID_EVENTS};
use crate::message;
use crate::parser::ParseState;
use crate::resolve::{call as call_resolve, Name as ResolveName};
use crate::state::{call, State};
use crate::subtokenize::Subresult;

#[cfg(feature = "log")]
use crate::util::char::format_byte_opt;

use crate::util::{constant::TAB_SIZE, edit_map::EditMap};
use alloc::{boxed::Box, string::String, vec, vec::Vec};

/// Containers.
///
/// Containers are found when tokenizing
/// [document content][crate::construct::document].
/// They parse a portion at the start of one or more lines.
/// The rest of those lines is a different content type (specifically, flow),
/// which they “contain”.
#[derive(Debug, Eq, PartialEq)]
pub enum Container {
    /// [Block quote][crate::construct::block_quote].
    BlockQuote,
    /// [List item][crate::construct::list_item].
    ListItem,
    /// [GFM: Footnote definition][crate::construct::gfm_footnote_definition].
    GfmFootnoteDefinition,
}

/// Info used to tokenize a container.
///
/// Practically, these fields are only used for list items.
#[derive(Debug)]
pub struct ContainerState {
    /// Kind.
    pub kind: Container,
    /// Whether the first line was blank.
    pub blank_initial: bool,
    /// Size.
    pub size: usize,
}

/// How to handle a byte.
#[derive(Debug, PartialEq)]
enum ByteAction {
    /// This is a normal byte.
    ///
    /// Includes replaced bytes.
    Normal(u8),
    /// This byte must be ignored.
    Ignore,
    /// This is a new byte.
    Insert(u8),
}

/// Label start kind.
#[derive(Debug, PartialEq, Eq)]
pub enum LabelKind {
    /// Label (image) start.
    ///
    /// ```markdown
    /// > | a ![b] c
    ///       ^^
    /// ```
    ///
    /// Construct: [Label start (image)][crate::construct::label_start_image].
    Image,
    /// Label (image) link.
    ///
    /// ```markdown
    /// > | a [b] c
    ///       ^
    /// ```
    ///
    /// Construct: [Label start (link)][crate::construct::label_start_link].
    Link,
    /// GFM: Label (footnote) link.
    ///
    /// ```markdown
    /// > | a [^b] c
    ///       ^^
    /// ```
    ///
    /// Construct: [GFM: Label start (footnote)][crate::construct::gfm_label_start_footnote].
    GfmFootnote,
    /// GFM: Label (footnote) link, not matching a footnote definition, so
    /// handled as a label (link) start.
    ///
    /// ```markdown
    /// > | a [^b](c) d
    ///       ^^
    /// ```
    ///
    /// Construct: [Label end][crate::construct::label_end].
    GfmUndefinedFootnote,
}

/// Label start, looking for an end.
#[derive(Debug)]
pub struct LabelStart {
    /// Kind of start.
    pub kind: LabelKind,
    /// Indices of where the label starts and ends in `events`.
    pub start: (usize, usize),
    /// A boolean used internally to figure out if a (link) label start can’t
    /// be used anymore (because it would contain another link).
    /// That link start is still looking for a balanced closing bracket though,
    /// so we can’t remove it just yet.
    pub inactive: bool,
}

/// Valid label.
#[derive(Debug)]
pub struct Label {
    pub kind: LabelKind,
    /// Indices of label start.
    pub start: (usize, usize),
    /// Indices of label end.
    pub end: (usize, usize),
}

/// Different kinds of attempts.
#[derive(Debug, PartialEq)]
enum AttemptKind {
    /// Discard what was tokenized when unsuccessful.
    Attempt,
    /// Discard always.
    Check,
}

/// How to handle [`State::Ok`][] or [`State::Nok`][].
#[derive(Debug)]
struct Attempt {
    /// Where to go to when successful.
    ok: State,
    /// Where to go to when unsuccessful.
    nok: State,
    /// Kind of attempt.
    kind: AttemptKind,
    /// If needed, the progress to revert to.
    ///
    /// It is not needed to discard an [`AttemptKind::Attempt`] that has a
    /// `nok` of [`State::Nok`][], because that means it is used in *another*
    /// attempt, which will receive that `Nok`, and has to handle it.
    progress: Option<Progress>,
}

/// The internal state of a tokenizer.
///
/// Not to be confused with states from the state machine, this instead is all
/// the information on where we currently are and what’s going on.
#[derive(Clone, Debug)]
struct Progress {
    /// Length of `events`.
    ///
    /// It’s not allowed to remove events, so reverting will just pop stuff off.
    events_len: usize,
    /// Length of the stack.
    ///
    /// It’s not allowed to decrease the stack in an attempt.
    stack_len: usize,
    /// Previous code.
    previous: Option<u8>,
    /// Current code.
    current: Option<u8>,
    /// Current place in the file.
    point: Point,
}

/// A lot of shared fields used to tokenize things.
#[allow(clippy::struct_excessive_bools)]
#[derive(Debug)]
pub struct TokenizeState<'a> {
    // Couple complex fields used to tokenize the document.
    /// Tokenizer, used to tokenize flow in document.
    pub document_child: Option<Box<Tokenizer<'a>>>,
    /// State, used to tokenize containers.
    pub document_child_state: Option<State>,
    /// Stack of currently active containers.
    pub document_container_stack: Vec<ContainerState>,
    /// How many active containers continued.
    pub document_continued: usize,
    /// Index of last `data`.
    pub document_data_index: Option<usize>,
    /// Container exits by line number.
    pub document_exits: Vec<Option<Vec<Event>>>,
    /// Whether the previous flow was a paragraph or a definition.
    pub document_lazy_accepting_before: bool,
    /// Whether this is the first paragraph (potentially after definitions) in
    /// a list item.
    /// Used for GFM task list items.
    pub document_at_first_paragraph_of_list_item: bool,

    // Couple of very frequent settings for parsing whitespace.
    pub space_or_tab_eol_content: Option<Content>,
    pub space_or_tab_eol_connect: bool,
    pub space_or_tab_eol_ok: bool,
    pub space_or_tab_connect: bool,
    pub space_or_tab_content: Option<Content>,
    pub space_or_tab_min: usize,
    pub space_or_tab_max: usize,
    pub space_or_tab_size: usize,
    pub space_or_tab_token: Name,

    // Couple of media related fields.
    /// List of usable label starts.
    ///
    /// Used when tokenizing [text content][crate::construct::text].
    pub label_starts: Vec<LabelStart>,
    /// List of unusable label starts.
    ///
    /// Used when tokenizing [text content][crate::construct::text].
    pub label_starts_loose: Vec<LabelStart>,
    /// Stack of images and links.
    ///
    /// Used when tokenizing [text content][crate::construct::text].
    pub labels: Vec<Label>,

    /// List of defined definition identifiers.
    pub definitions: Vec<String>,
    /// List of defined GFM footnote definition identifiers.
    pub gfm_footnote_definitions: Vec<String>,

    // Last error message provided at an EOF of an expression.
    pub mdx_last_parse_error: Option<(String, String, String)>,

    /// Whether to connect events.
    pub connect: bool,
    /// Marker.
    pub marker: u8,
    /// Secondary marker.
    pub marker_b: u8,
    /// Several markers.
    pub markers: &'static [u8],
    /// Whether something was seen.
    pub seen: bool,
    /// Size.
    pub size: usize,
    /// Secondary size.
    pub size_b: usize,
    /// Tertiary size.
    pub size_c: usize,
    /// Index.
    pub start: usize,
    /// Index.
    pub end: usize,
    /// Slot for an event name.
    pub token_1: Name,
    /// Slot for an event name.
    pub token_2: Name,
    /// Slot for an event name.
    pub token_3: Name,
    /// Slot for an event name.
    pub token_4: Name,
    /// Slot for an event name.
    pub token_5: Name,
    /// Slot for an event name.
    pub token_6: Name,
}

/// A tokenizer itself.
#[allow(clippy::struct_excessive_bools)]
#[derive(Debug)]
pub struct Tokenizer<'a> {
    /// Jump between line endings.
    column_start: Vec<(usize, usize)>,
    // First line where this tokenizer starts.
    first_line: usize,
    /// Current point after the last line ending (excluding jump).
    line_start: Point,
    /// Track whether the current byte is already consumed (`true`) or expected
    /// to be consumed (`false`).
    ///
    /// Tracked to make sure everything’s valid.
    consumed: bool,
    /// Stack of how to handle attempts.
    attempts: Vec<Attempt>,
    /// Current byte.
    pub current: Option<u8>,
    /// Previous byte.
    pub previous: Option<u8>,
    /// Current relative and absolute place in the file.
    pub point: Point,
    /// Semantic labels.
    pub events: Vec<Event>,
    /// Hierarchy of semantic labels.
    ///
    /// Tracked to make sure everything’s valid.
    pub stack: Vec<Name>,
    /// Edit map, to batch changes.
    pub map: EditMap,
    /// List of resolvers.
    pub resolvers: Vec<ResolveName>,
    /// Shared parsing state across tokenizers.
    pub parse_state: &'a ParseState<'a>,
    /// A lot of shared fields used to tokenize things.
    pub tokenize_state: TokenizeState<'a>,
    /// Whether we would be interrupting something.
    ///
    /// Used when tokenizing [flow content][crate::construct::flow].
    pub interrupt: bool,
    /// Whether containers cannot “pierce” into the current construct.
    ///
    /// Used when tokenizing [document content][crate::construct::document].
    pub concrete: bool,
    /// Whether this row is piercing into the current construct with more
    /// containers.
    ///
    /// Used when tokenizing [document content][crate::construct::document].
    pub pierce: bool,
    /// Whether this line is lazy: there are less containers than before.
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
            attempts: vec![],
            point,
            stack: vec![],
            events: vec![],
            parse_state,
            tokenize_state: TokenizeState {
                connect: false,
                document_container_stack: vec![],
                document_exits: vec![],
                document_continued: 0,
                document_lazy_accepting_before: false,
                document_data_index: None,
                document_child_state: None,
                document_child: None,
                document_at_first_paragraph_of_list_item: false,
                definitions: vec![],
                gfm_footnote_definitions: vec![],
                mdx_last_parse_error: None,
                end: 0,
                label_starts: vec![],
                label_starts_loose: vec![],
                marker: 0,
                marker_b: 0,
                markers: &[],
                labels: vec![],
                seen: false,
                size: 0,
                size_b: 0,
                size_c: 0,
                space_or_tab_eol_content: None,
                space_or_tab_eol_connect: false,
                space_or_tab_eol_ok: false,
                space_or_tab_connect: false,
                space_or_tab_content: None,
                space_or_tab_min: 0,
                space_or_tab_max: 0,
                space_or_tab_size: 0,
                space_or_tab_token: Name::SpaceOrTab,
                start: 0,
                token_1: Name::Data,
                token_2: Name::Data,
                token_3: Name::Data,
                token_4: Name::Data,
                token_5: Name::Data,
                token_6: Name::Data,
            },
            map: EditMap::new(),
            interrupt: false,
            pierce: false,
            concrete: false,
            lazy: false,
            resolvers: vec![],
        }
    }

    /// Register a resolver.
    pub fn register_resolver(&mut self, name: ResolveName) {
        if !self.resolvers.contains(&name) {
            self.resolvers.push(name);
        }
    }

    /// Register a resolver, before others.
    pub fn register_resolver_before(&mut self, name: ResolveName) {
        if !self.resolvers.contains(&name) {
            self.resolvers.insert(0, name);
        }
    }

    /// Define a jump between two places.
    ///
    /// This defines to which future index we move after a line ending.
    pub fn define_skip(&mut self, mut point: Point) {
        move_point_back(self, &mut point);

        let info = (point.index, point.vs);

        #[cfg(feature = "log")]
        log::trace!("position: define skip: {:?} -> ({:?})", point.line, info);

        let at = point.line - self.first_line;

        if at >= self.column_start.len() {
            self.column_start.push(info);
        } else {
            self.column_start[at] = info;
        }

        self.account_for_potential_skip();
    }

    /// Increment the current positional info if we’re right after a line
    /// ending, which has a skip defined.
    fn account_for_potential_skip(&mut self) {
        let at = self.point.line - self.first_line;

        if self.point.column == 1 && at != self.column_start.len() {
            self.move_to(self.column_start[at]);
        }
    }

    /// Prepare for a next byte to get consumed.
    fn expect(&mut self, byte: Option<u8>) {
        debug_assert!(self.consumed, "expected previous byte to be consumed");
        self.consumed = false;
        self.current = byte;
    }

    /// Consume the current byte.
    /// Each state function is expected to call this to signal that this code is
    /// used, or call a next function.
    pub fn consume(&mut self) {
        debug_assert!(!self.consumed, "expected code to *not* have been consumed: this might be because `State::Retry(x)` instead of `State::Next(x)` was returned");
        self.move_one();

        self.previous = self.current;
        // While we’re not at eof, it is at least better to not have the
        // same current code as `previous` *and* `current`.
        self.current = None;
        // Mark as consumed.
        self.consumed = true;
    }

    /// Move to the next (virtual) byte.
    fn move_one(&mut self) {
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

                    #[cfg(feature = "log")]
                    log::trace!("position: after eol: `{:?}`", self.point);
                } else {
                    self.point.column += 1;
                }
            }
        }
    }

    /// Move (virtual) bytes.
    fn move_to(&mut self, to: (usize, usize)) {
        let (to_index, to_vs) = to;
        while self.point.index < to_index || self.point.index == to_index && self.point.vs < to_vs {
            self.move_one();
        }
    }

    /// Mark the start of a semantic label.
    pub fn enter(&mut self, name: Name) {
        enter_impl(self, name, None);
    }

    /// Enter with a link.
    pub fn enter_link(&mut self, name: Name, link: Link) {
        enter_impl(self, name, Some(link));
    }

    /// Mark the end of a semantic label.
    pub fn exit(&mut self, name: Name) {
        let current = self.stack.pop().expect("cannot close w/o open tokens");

        debug_assert_eq!(current, name, "expected exit event to match current event");

        let previous = self.events.last().expect("cannot close w/o open event");
        let mut point = self.point.clone();

        debug_assert!(
            current != previous.name
                || previous.point.index != point.index
                || previous.point.vs != point.vs,
            "expected non-empty event"
        );

        if VOID_EVENTS.iter().any(|d| d == &name) {
            debug_assert!(
                current == previous.name,
                "expected event to be void, instead of including something"
            );
        }

        // A bit weird, but if we exit right after a line ending, we *don’t* want to consider
        // potential skips.
        if matches!(self.previous, Some(b'\n')) {
            point = self.line_start.clone();
        } else {
            move_point_back(self, &mut point);
        }

        #[cfg(feature = "log")]
        log::debug!("exit:    `{:?}`", name);

        let event = Event {
            kind: Kind::Exit,
            name,
            point,
            link: None,
        };
        self.events.push(event);
    }

    /// Capture the tokenizer progress.
    fn capture(&mut self) -> Progress {
        Progress {
            previous: self.previous,
            current: self.current,
            point: self.point.clone(),
            events_len: self.events.len(),
            stack_len: self.stack.len(),
        }
    }

    /// Apply tokenizer progress.
    fn free(&mut self, previous: Progress) {
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

    /// Stack an attempt, moving to `ok` on [`State::Ok`][] and `nok` on
    /// [`State::Nok`][], reverting in both cases.
    pub fn check(&mut self, ok: State, nok: State) {
        // Always capture (and restore) when checking.
        // No need to capture (and restore) when `nok` is `State::Nok`, because the
        // parent attempt will do it.
        let progress = Some(self.capture());
        let attempt = Attempt {
            kind: AttemptKind::Check,
            progress,
            ok,
            nok,
        };
        self.attempts.push(attempt);
    }

    /// Stack an attempt, moving to `ok` on [`State::Ok`][] and `nok` on
    /// [`State::Nok`][], reverting in the latter case.
    pub fn attempt(&mut self, ok: State, nok: State) {
        // Always capture (and restore) when checking.
        // No need to capture (and restore) when `nok` is `State::Nok`, because the
        // parent attempt will do it.
        let progress = if nok == State::Nok {
            None
        } else {
            Some(self.capture())
        };

        let attempt = Attempt {
            kind: AttemptKind::Attempt,
            progress,
            ok,
            nok,
        };
        self.attempts.push(attempt);
    }

    /// Tokenize.
    pub fn push(&mut self, from: (usize, usize), to: (usize, usize), state: State) -> State {
        push_impl(self, from, to, state, false)
    }

    /// Flush.
    pub fn flush(&mut self, state: State, resolve: bool) -> Result<Subresult, message::Message> {
        let to = (self.point.index, self.point.vs);
        let state = push_impl(self, to, to, state, true);

        state.to_result()?;

        let mut value = Subresult {
            done: false,
            gfm_footnote_definitions: self.tokenize_state.gfm_footnote_definitions.split_off(0),
            definitions: self.tokenize_state.definitions.split_off(0),
        };

        if resolve {
            let resolvers = self.resolvers.split_off(0);
            let mut index = 0;
            let defs = &mut value.definitions;
            let fn_defs = &mut value.gfm_footnote_definitions;
            while index < resolvers.len() {
                if let Some(mut result) = call_resolve(self, resolvers[index])? {
                    fn_defs.append(&mut result.gfm_footnote_definitions);
                    defs.append(&mut result.definitions);
                }
                index += 1;
            }

            self.map.consume(&mut self.events);
        }

        Ok(value)
    }
}

/// Move back past ignored bytes.
fn move_point_back(tokenizer: &mut Tokenizer, point: &mut Point) {
    while point.index > 0 {
        point.index -= 1;
        let action = byte_action(tokenizer.parse_state.bytes, point);
        if !matches!(action, ByteAction::Ignore) {
            point.index += 1;
            break;
        }
    }
}

/// Enter.
fn enter_impl(tokenizer: &mut Tokenizer, name: Name, link: Option<Link>) {
    let mut point = tokenizer.point.clone();
    move_point_back(tokenizer, &mut point);

    #[cfg(feature = "log")]
    log::debug!("enter:   `{:?}`", name);

    tokenizer.stack.push(name.clone());
    tokenizer.events.push(Event {
        kind: Kind::Enter,
        name,
        point,
        link,
    });
}

/// Run the tokenizer.
fn push_impl(
    tokenizer: &mut Tokenizer,
    from: (usize, usize),
    to: (usize, usize),
    mut state: State,
    flush: bool,
) -> State {
    debug_assert!(
        from.0 > tokenizer.point.index
            || (from.0 == tokenizer.point.index && from.1 >= tokenizer.point.vs),
        "cannot move backwards"
    );

    tokenizer.move_to(from);

    loop {
        match state {
            State::Error(_) => break,
            State::Ok | State::Nok => {
                if let Some(attempt) = tokenizer.attempts.pop() {
                    if attempt.kind == AttemptKind::Check || state == State::Nok {
                        if let Some(progress) = attempt.progress {
                            tokenizer.free(progress);
                        }
                    }

                    tokenizer.consumed = true;

                    let next = if state == State::Ok {
                        attempt.ok
                    } else {
                        attempt.nok
                    };

                    #[cfg(feature = "log")]
                    log::trace!("attempt: `{:?}` -> `{:?}`", state, next);

                    state = next;
                } else {
                    break;
                }
            }
            State::Next(name) => {
                let action = if tokenizer.point.index < to.0
                    || (tokenizer.point.index == to.0 && tokenizer.point.vs < to.1)
                {
                    Some(byte_action(tokenizer.parse_state.bytes, &tokenizer.point))
                } else if flush {
                    None
                } else {
                    break;
                };

                if let Some(ByteAction::Ignore) = action {
                    tokenizer.move_one();
                } else {
                    let byte =
                        if let Some(ByteAction::Insert(byte) | ByteAction::Normal(byte)) = action {
                            Some(byte)
                        } else {
                            None
                        };

                    #[cfg(feature = "log")]
                    log::trace!("feed:    {} to {:?}", format_byte_opt(byte), name);

                    tokenizer.expect(byte);
                    state = call(tokenizer, name);
                }
            }
            State::Retry(name) => {
                #[cfg(feature = "log")]
                log::trace!("retry:   `{:?}`", name);

                state = call(tokenizer, name);
            }
        }
    }

    tokenizer.consumed = true;

    if flush {
        debug_assert!(matches!(state, State::Ok | State::Error(_)), "must be ok");
    } else {
        debug_assert!(
            matches!(state, State::Next(_) | State::Error(_)),
            "must have a next state"
        );
    }

    state
}

/// Figure out how to handle a byte.
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
