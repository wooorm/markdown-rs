//! Attention is a construct that occurs in the [text][] content type.
//!
//! How attention parses is too complex to explain in BNF.
//! Essentially, one or more of `*` or `_` form attention sequences.
//! Depending on the code before and after a sequence, it can open or close
//! attention.
//! When everything is parsed, we find each sequence that can close, and a
//! corresponding sequence that can open which uses the same marker.
//! If both sequences have two or more markers, strong is formed.
//! Otherwise emphasis is formed.
//!
//! Attention sequences do not, on their own, relate to anything in HTML.
//! When matched with another sequence, and two markers can be “taken” from
//! them, they together relate to the `<strong>` element in HTML.
//! When one marker can be taken, they relate to the `<em>` element.
//! See [*§ 4.5.2 The `em` element*][html-em] and
//! [*§ 4.5.3 The `strong` element*][html-strong] in the HTML spec for more
//! info.
//!
//! It is recommended to use asterisks for attention when writing markdown.
//!
//! There are some small differences in whether sequences can open and/or close
//! based on whether they are formed with asterisks or underscores.
//! Because underscores also frequently occur in natural language inside words,
//! while asterisks typically never do, `CommonMark` prohobits underscore
//! sequences from opening or closing when *inside* a word.
//!
//! Because asterisks can be used to form the most markdown constructs, using
//! them has the added benefit of making it easier to gloss over markdown: you
//! can look for asterisks to find syntax while not worrying about other
//! characters.
//!
//! ## Tokens
//!
//! *   [`Emphasis`][Token::Emphasis]
//! *   [`EmphasisSequence`][Token::EmphasisSequence]
//! *   [`EmphasisText`][Token::EmphasisText]
//! *   [`Strong`][Token::Strong]
//! *   [`StrongSequence`][Token::StrongSequence]
//! *   [`StrongText`][Token::StrongText]
//!
//! > 👉 **Note**: while parsing, [`AttentionSequence`][Token::AttentionSequence]
//! > is used, which is later compiled away.
//!
//! ## References
//!
//! *   [`attention.js` in `micromark`](https://github.com/micromark/micromark/blob/main/packages/micromark-core-commonmark/dev/lib/attention.js)
//! *   [*§ 6.2 Emphasis and strong emphasis* in `CommonMark`](https://spec.commonmark.org/0.30/#emphasis-and-strong-emphasis)
//!
//! [text]: crate::content::text
//! [html-em]: https://html.spec.whatwg.org/multipage/text-level-semantics.html#the-em-element
//! [html-strong]: https://html.spec.whatwg.org/multipage/text-level-semantics.html#the-strong-element

use crate::token::Token;
use crate::tokenizer::{Code, Event, EventType, Point, State, StateFnResult, Tokenizer};
use crate::unicode::PUNCTUATION;
use crate::util::edit_map::EditMap;

/// Character code kinds.
#[derive(Debug, PartialEq)]
enum GroupKind {
    /// Whitespace.
    ///
    /// ## Example
    ///
    /// ```markdown
    /// > | **a_b_ c**.
    ///    ^      ^    ^
    /// ```
    Whitespace,
    /// Punctuation.
    ///
    /// ## Example
    ///
    /// ```markdown
    /// > | **a_b_ c**.
    ///     ^^ ^ ^    ^
    /// ```
    Punctuation,
    /// Everything else.
    ///
    /// ## Example
    ///
    /// ```markdown
    /// > | **a_b_ c**.
    ///       ^ ^  ^
    /// ```
    Other,
}

/// Type of sequence.
#[derive(Debug, PartialEq)]
enum MarkerKind {
    /// In a run with asterisks.
    ///
    /// ## Example
    ///
    /// ```markdown
    /// *a*
    /// ```
    Asterisk,
    /// In a run with underscores.
    ///
    /// ## Example
    ///
    /// ```markdown
    /// _a_
    /// ```
    Underscore,
}

impl MarkerKind {
    /// Turn the kind into a [char].
    fn as_char(&self) -> char {
        match self {
            MarkerKind::Asterisk => '*',
            MarkerKind::Underscore => '_',
        }
    }
    /// Turn [char] into a kind.
    ///
    /// ## Panics
    ///
    /// Panics if `char` is not `*` or `_`.
    fn from_char(char: char) -> MarkerKind {
        match char {
            '*' => MarkerKind::Asterisk,
            '_' => MarkerKind::Underscore,
            _ => unreachable!("invalid char"),
        }
    }
    /// Turn [Code] into a kind.
    ///
    /// ## Panics
    ///
    /// Panics if `code` is not `Code::Char('*' | '_')`.
    fn from_code(code: Code) -> MarkerKind {
        match code {
            Code::Char(char) => MarkerKind::from_char(char),
            _ => unreachable!("invalid code"),
        }
    }
}

/// Attentention sequence that we can take markers from.
#[derive(Debug)]
struct Sequence {
    /// Marker used in this sequence.
    marker: MarkerKind,
    /// The depth in events where this sequence resides.
    balance: usize,
    /// The index into events where this sequence’s `Enter` currently resides.
    event_index: usize,
    /// The (shifted) point where this sequence starts.
    start_point: Point,
    /// The (shifted) index where this sequence starts.
    start_index: usize,
    /// The (shifted) point where this sequence end.
    end_point: Point,
    /// The (shifted) index where this sequence end.
    end_index: usize,
    /// The number of markers we can still use.
    size: usize,
    /// Whether this sequence can open attention.
    open: bool,
    /// Whether this sequence can close attention.
    close: bool,
}

/// Before a sequence.
///
/// ```markdown
/// |**
/// ```
pub fn start(tokenizer: &mut Tokenizer, code: Code) -> StateFnResult {
    match code {
        Code::Char('*' | '_') => {
            tokenizer.enter(Token::AttentionSequence);
            inside(tokenizer, code, MarkerKind::from_code(code))
        }
        _ => (State::Nok, None),
    }
}

/// In a sequence.
///
/// ```markdown
/// *|*
/// ```
fn inside(tokenizer: &mut Tokenizer, code: Code, marker: MarkerKind) -> StateFnResult {
    match code {
        Code::Char(char) if char == marker.as_char() => {
            tokenizer.consume(code);
            (State::Fn(Box::new(move |t, c| inside(t, c, marker))), None)
        }
        _ => {
            tokenizer.exit(Token::AttentionSequence);
            tokenizer.register_resolver("attention".to_string(), Box::new(resolve));
            (State::Ok, Some(vec![code]))
        }
    }
}

/// Resolve attention sequences.
#[allow(clippy::too_many_lines)]
fn resolve(tokenizer: &mut Tokenizer) -> Vec<Event> {
    let codes = &tokenizer.parse_state.codes;
    let mut edit_map = EditMap::new();
    let mut start = 0;
    let mut balance = 0;
    let mut sequences: Vec<Sequence> = vec![];

    // Find sequences of sequences and information about them.
    while start < tokenizer.events.len() {
        let enter = &tokenizer.events[start];

        if enter.event_type == EventType::Enter {
            balance += 1;

            if enter.token_type == Token::AttentionSequence {
                let end = start + 1;
                let exit = &tokenizer.events[end];
                let marker = MarkerKind::from_code(codes[enter.index]);
                let before = classify_character(if enter.index > 0 {
                    codes[enter.index - 1]
                } else {
                    Code::None
                });
                let after = classify_character(if exit.index < codes.len() {
                    codes[exit.index]
                } else {
                    Code::None
                });
                let open = after == GroupKind::Other
                    || (after == GroupKind::Punctuation && before != GroupKind::Other);
                // To do: GFM strikethrough?
                // || attentionMarkers.includes(code)
                let close = before == GroupKind::Other
                    || (before == GroupKind::Punctuation && after != GroupKind::Other);
                // To do: GFM strikethrough?
                // || attentionMarkers.includes(previous)

                sequences.push(Sequence {
                    event_index: start,
                    balance,
                    start_point: enter.point.clone(),
                    start_index: enter.index,
                    end_point: exit.point.clone(),
                    end_index: exit.index,
                    size: exit.index - enter.index,
                    open: if marker == MarkerKind::Asterisk {
                        open
                    } else {
                        open && (before != GroupKind::Other || !close)
                    },
                    close: if marker == MarkerKind::Asterisk {
                        close
                    } else {
                        close && (after != GroupKind::Other || !open)
                    },
                    marker,
                });
            }
        } else {
            balance -= 1;
        }

        start += 1;
    }

    // Walk through sequences and match them.
    let mut close = 0;

    while close < sequences.len() {
        let sequence_close = &sequences[close];
        let mut next_index = close + 1;

        // Find a sequence that can close.
        if sequence_close.close {
            let mut open = close;

            // Now walk back to find an opener.
            while open > 0 {
                open -= 1;

                let sequence_open = &sequences[open];

                // We found a sequence that can open the closer we found.
                if sequence_open.open
                    && sequence_close.marker == sequence_open.marker
                    && sequence_close.balance == sequence_open.balance
                {
                    // If the opening can close or the closing can open,
                    // and the close size *is not* a multiple of three,
                    // but the sum of the opening and closing size *is*
                    // multiple of three, then **don’t** match.
                    if (sequence_open.close || sequence_close.open)
                        && sequence_close.size % 3 != 0
                        && (sequence_open.size + sequence_close.size) % 3 == 0
                    {
                        continue;
                    }

                    // We’ve found a match!

                    // Number of markers to use from the sequence.
                    let take = if sequence_open.size > 1 && sequence_close.size > 1 {
                        2
                    } else {
                        1
                    };

                    // We’re *on* a closing sequence, with a matching opening
                    // sequence.
                    // Now we make sure that we can’t have misnested attention:
                    //
                    // ```html
                    // <em>a <strong>b</em> c</strong>
                    // ```
                    //
                    // Do that by marking everything between it as no longer
                    // possible to open anything.
                    // Theoretically we could mark non-closing as well, but we
                    // don’t look for closers backwards.
                    let mut between = open + 1;

                    while between < close {
                        sequences[between].open = false;
                        between += 1;
                    }

                    let sequence_close = &mut sequences[close];
                    let close_event_index = sequence_close.event_index;
                    let seq_close_enter = (
                        sequence_close.start_point.clone(),
                        sequence_close.start_index,
                    );
                    sequence_close.size -= take;
                    sequence_close.start_point.column += take;
                    sequence_close.start_point.offset += take;
                    sequence_close.start_index += take;
                    let seq_close_exit = (
                        sequence_close.start_point.clone(),
                        sequence_close.start_index,
                    );

                    // Stay on this closing sequence for the next iteration: it
                    // might close more things.
                    next_index -= 1;

                    // Remove closing sequence if fully used.
                    if sequence_close.size == 0 {
                        sequences.remove(close);
                        edit_map.add(close_event_index, 2, vec![]);
                    } else {
                        // Shift remaining closing sequence forward.
                        // Do it here because a sequence can open and close different
                        // other sequences, and the remainder can be on any side or
                        // somewhere in the middle.
                        let mut enter = &mut tokenizer.events[close_event_index];
                        enter.point = seq_close_exit.0.clone();
                        enter.index = seq_close_exit.1;
                    }

                    let sequence_open = &mut sequences[open];
                    let open_event_index = sequence_open.event_index;
                    let seq_open_exit = (sequence_open.end_point.clone(), sequence_open.end_index);
                    sequence_open.size -= take;
                    sequence_open.end_point.column -= take;
                    sequence_open.end_point.offset -= take;
                    sequence_open.end_index -= take;
                    let seq_open_enter = (sequence_open.end_point.clone(), sequence_open.end_index);

                    // Remove opening sequence if fully used.
                    if sequence_open.size == 0 {
                        sequences.remove(open);
                        edit_map.add(open_event_index, 2, vec![]);
                        next_index -= 1;
                    } else {
                        // Shift remaining opening sequence backwards.
                        // See note above for why that happens here.
                        let mut exit = &mut tokenizer.events[open_event_index + 1];
                        exit.point = seq_open_enter.0.clone();
                        exit.index = seq_open_enter.1;
                    }

                    // Opening.
                    edit_map.add_before(
                        // Add after the current sequence (it might remain).
                        open_event_index + 2,
                        0,
                        vec![
                            Event {
                                event_type: EventType::Enter,
                                token_type: if take == 1 {
                                    Token::Emphasis
                                } else {
                                    Token::Strong
                                },
                                point: seq_open_enter.0.clone(),
                                index: seq_open_enter.1,
                                previous: None,
                                next: None,
                                content_type: None,
                            },
                            Event {
                                event_type: EventType::Enter,
                                token_type: if take == 1 {
                                    Token::EmphasisSequence
                                } else {
                                    Token::StrongSequence
                                },
                                point: seq_open_enter.0.clone(),
                                index: seq_open_enter.1,
                                previous: None,
                                next: None,
                                content_type: None,
                            },
                            Event {
                                event_type: EventType::Exit,
                                token_type: if take == 1 {
                                    Token::EmphasisSequence
                                } else {
                                    Token::StrongSequence
                                },
                                point: seq_open_exit.0.clone(),
                                index: seq_open_exit.1,
                                previous: None,
                                next: None,
                                content_type: None,
                            },
                            Event {
                                event_type: EventType::Enter,
                                token_type: if take == 1 {
                                    Token::EmphasisText
                                } else {
                                    Token::StrongText
                                },
                                point: seq_open_exit.0.clone(),
                                index: seq_open_exit.1,
                                previous: None,
                                next: None,
                                content_type: None,
                            },
                        ],
                    );
                    // Closing.
                    edit_map.add(
                        close_event_index,
                        0,
                        vec![
                            Event {
                                event_type: EventType::Exit,
                                token_type: if take == 1 {
                                    Token::EmphasisText
                                } else {
                                    Token::StrongText
                                },
                                point: seq_close_enter.0.clone(),
                                index: seq_close_enter.1,
                                previous: None,
                                next: None,
                                content_type: None,
                            },
                            Event {
                                event_type: EventType::Enter,
                                token_type: if take == 1 {
                                    Token::EmphasisSequence
                                } else {
                                    Token::StrongSequence
                                },
                                point: seq_close_enter.0.clone(),
                                index: seq_close_enter.1,
                                previous: None,
                                next: None,
                                content_type: None,
                            },
                            Event {
                                event_type: EventType::Exit,
                                token_type: if take == 1 {
                                    Token::EmphasisSequence
                                } else {
                                    Token::StrongSequence
                                },
                                point: seq_close_exit.0.clone(),
                                index: seq_close_exit.1,
                                previous: None,
                                next: None,
                                content_type: None,
                            },
                            Event {
                                event_type: EventType::Exit,
                                token_type: if take == 1 {
                                    Token::Emphasis
                                } else {
                                    Token::Strong
                                },
                                point: seq_close_exit.0.clone(),
                                index: seq_close_exit.1,
                                previous: None,
                                next: None,
                                content_type: None,
                            },
                        ],
                    );

                    break;
                }
            }
        }

        close = next_index;
    }

    // Mark remaining sequences as data.
    let mut index = 0;
    while index < sequences.len() {
        let sequence = &sequences[index];
        tokenizer.events[sequence.event_index].token_type = Token::Data;
        tokenizer.events[sequence.event_index + 1].token_type = Token::Data;
        index += 1;
    }

    edit_map.consume(&mut tokenizer.events)
}

/// Classify whether a character code represents whitespace, punctuation, or
/// something else.
///
/// Used for attention (emphasis, strong), whose sequences can open or close
/// based on the class of surrounding characters.
///
/// > 👉 **Note** that eof (`Code::None`) is seen as whitespace.
///
/// ## References
///
/// *   [`micromark-util-classify-character` in `micromark`](https://github.com/micromark/micromark/blob/main/packages/micromark-util-classify-character/dev/index.js)
fn classify_character(code: Code) -> GroupKind {
    match code {
        // Custom characters.
        Code::None | Code::CarriageReturnLineFeed | Code::VirtualSpace => GroupKind::Whitespace,
        // Unicode whitespace.
        Code::Char(char) if char.is_whitespace() => GroupKind::Whitespace,
        // Unicode punctuation.
        Code::Char(char) if PUNCTUATION.contains(&char) => GroupKind::Punctuation,
        // Everything else.
        Code::Char(_) => GroupKind::Other,
    }
}