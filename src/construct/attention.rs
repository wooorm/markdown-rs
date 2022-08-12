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
//! *   [`Emphasis`][Name::Emphasis]
//! *   [`EmphasisSequence`][Name::EmphasisSequence]
//! *   [`EmphasisText`][Name::EmphasisText]
//! *   [`Strong`][Name::Strong]
//! *   [`StrongSequence`][Name::StrongSequence]
//! *   [`StrongText`][Name::StrongText]
//!
//! > 👉 **Note**: while parsing, [`AttentionSequence`][Name::AttentionSequence]
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

use crate::event::{Event, Kind, Name, Point};
use crate::resolve::Name as ResolveName;
use crate::state::{Name as StateName, State};
use crate::tokenizer::Tokenizer;
use crate::unicode::PUNCTUATION;
use crate::util::slice::Slice;

/// Character code kinds.
#[derive(Debug, PartialEq)]
enum CharacterKind {
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

/// Attentention sequence that we can take markers from.
#[derive(Debug)]
struct Sequence {
    /// Marker as a byte (`u8`) used in this sequence.
    marker: u8,
    /// The depth in events where this sequence resides.
    balance: usize,
    /// The index into events where this sequence’s `Enter` currently resides.
    index: usize,
    /// The (shifted) point where this sequence starts.
    start_point: Point,
    /// The (shifted) point where this sequence end.
    end_point: Point,
    /// The number of markers we can still use.
    size: usize,
    /// Whether this sequence can open attention.
    open: bool,
    /// Whether this sequence can close attention.
    close: bool,
}

/// At start of attention.
///
/// ```markdown
/// > | **
///     ^
/// ```
pub fn start(tokenizer: &mut Tokenizer) -> State {
    match tokenizer.current {
        Some(b'*' | b'_') if tokenizer.parse_state.constructs.attention => {
            tokenizer.tokenize_state.marker = tokenizer.current.unwrap();
            tokenizer.enter(Name::AttentionSequence);
            State::Retry(StateName::AttentionInside)
        }
        _ => State::Nok,
    }
}

/// In sequence.
///
/// ```markdown
/// > | **
///     ^^
/// ```
pub fn inside(tokenizer: &mut Tokenizer) -> State {
    match tokenizer.current {
        Some(b'*' | b'_') if tokenizer.current == Some(tokenizer.tokenize_state.marker) => {
            tokenizer.consume();
            State::Next(StateName::AttentionInside)
        }
        _ => {
            tokenizer.exit(Name::AttentionSequence);
            tokenizer.register_resolver(ResolveName::Attention);
            tokenizer.tokenize_state.marker = b'\0';
            State::Ok
        }
    }
}

/// Resolve attention sequences.
pub fn resolve(tokenizer: &mut Tokenizer) {
    let mut index = 0;
    let mut balance = 0;
    let mut sequences = vec![];

    // Find all sequences, gather info about them.
    while index < tokenizer.events.len() {
        let enter = &tokenizer.events[index];

        if enter.kind == Kind::Enter {
            balance += 1;

            if enter.name == Name::AttentionSequence {
                let end = index + 1;
                let exit = &tokenizer.events[end];

                let before_end = enter.point.index;
                let before_start = if before_end < 4 { 0 } else { before_end - 4 };
                let char_before =
                    String::from_utf8_lossy(&tokenizer.parse_state.bytes[before_start..before_end])
                        .chars()
                        .last();

                let after_start = exit.point.index;
                let after_end = if after_start + 4 > tokenizer.parse_state.bytes.len() {
                    tokenizer.parse_state.bytes.len()
                } else {
                    after_start + 4
                };
                let char_after =
                    String::from_utf8_lossy(&tokenizer.parse_state.bytes[after_start..after_end])
                        .chars()
                        .next();

                let marker = Slice::from_point(tokenizer.parse_state.bytes, &enter.point)
                    .head()
                    .unwrap();
                let before = classify_character(char_before);
                let after = classify_character(char_after);
                let open = after == CharacterKind::Other
                    || (after == CharacterKind::Punctuation && before != CharacterKind::Other);
                // To do: GFM strikethrough?
                // || char_after == '~'
                let close = before == CharacterKind::Other
                    || (before == CharacterKind::Punctuation && after != CharacterKind::Other);
                // To do: GFM strikethrough?
                // || char_before == '~'

                sequences.push(Sequence {
                    index,
                    balance,
                    start_point: enter.point.clone(),
                    end_point: exit.point.clone(),
                    size: exit.point.index - enter.point.index,
                    open: if marker == b'*' {
                        open
                    } else {
                        open && (before != CharacterKind::Other || !close)
                    },
                    close: if marker == b'*' {
                        close
                    } else {
                        close && (after != CharacterKind::Other || !open)
                    },
                    marker,
                });
            }
        } else {
            balance -= 1;
        }

        index += 1;
    }

    // Now walk through them and match them.
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

                // An opener matching our closer:
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
                    next_index = match_sequences(tokenizer, &mut sequences, open, close);

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
        tokenizer.events[sequence.index].name = Name::Data;
        tokenizer.events[sequence.index + 1].name = Name::Data;
        index += 1;
    }

    tokenizer.map.consume(&mut tokenizer.events);
}

/// Match two sequences.
fn match_sequences(
    tokenizer: &mut Tokenizer,
    sequences: &mut Vec<Sequence>,
    open: usize,
    close: usize,
) -> usize {
    // Where to move to next.
    // Stay on this closing sequence for the next iteration: it
    // might close more things.
    // It’s changed if sequences are removed.
    let mut next = close;

    // Number of markers to use from the sequence.
    let take = if sequences[open].size > 1 && sequences[close].size > 1 {
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
    // Theoretically we should mark as `close: false` too, but
    // we don’t look for closers backwards, so it’s not needed.
    let mut between = open + 1;

    while between < close {
        sequences[between].open = false;
        between += 1;
    }

    let (group_name, seq_name, text_name) = if take == 1 {
        (Name::Emphasis, Name::EmphasisSequence, Name::EmphasisText)
    } else {
        (Name::Strong, Name::StrongSequence, Name::StrongText)
    };
    let open_index = sequences[open].index;
    let close_index = sequences[close].index;
    let open_exit = sequences[open].end_point.clone();
    let close_enter = sequences[close].start_point.clone();

    // No need to worry about `VS`, because sequences are only actual characters.
    sequences[open].size -= take;
    sequences[close].size -= take;
    sequences[open].end_point.column -= take;
    sequences[open].end_point.index -= take;
    sequences[close].start_point.column += take;
    sequences[close].start_point.index += take;

    // Opening.
    tokenizer.map.add_before(
        // Add after the current sequence (it might remain).
        open_index + 2,
        0,
        vec![
            Event {
                kind: Kind::Enter,
                name: group_name.clone(),
                point: sequences[open].end_point.clone(),
                link: None,
            },
            Event {
                kind: Kind::Enter,
                name: seq_name.clone(),
                point: sequences[open].end_point.clone(),
                link: None,
            },
            Event {
                kind: Kind::Exit,
                name: seq_name.clone(),
                point: open_exit.clone(),
                link: None,
            },
            Event {
                kind: Kind::Enter,
                name: text_name.clone(),
                point: open_exit,
                link: None,
            },
        ],
    );
    // Closing.
    tokenizer.map.add(
        close_index,
        0,
        vec![
            Event {
                kind: Kind::Exit,
                name: text_name,
                point: close_enter.clone(),
                link: None,
            },
            Event {
                kind: Kind::Enter,
                name: seq_name.clone(),
                point: close_enter,
                link: None,
            },
            Event {
                kind: Kind::Exit,
                name: seq_name,
                point: sequences[close].start_point.clone(),
                link: None,
            },
            Event {
                kind: Kind::Exit,
                name: group_name,
                point: sequences[close].start_point.clone(),
                link: None,
            },
        ],
    );

    // Remove closing sequence if fully used.
    if sequences[close].size == 0 {
        sequences.remove(close);
        tokenizer.map.add(close_index, 2, vec![]);
    } else {
        // Shift remaining closing sequence forward.
        // Do it here because a sequence can open and close different
        // other sequences, and the remainder can be on any side or
        // somewhere in the middle.
        tokenizer.events[close_index].point = sequences[close].start_point.clone();
    }

    if sequences[open].size == 0 {
        sequences.remove(open);
        tokenizer.map.add(open_index, 2, vec![]);
        // Everything shifts one to the left, account for it in next iteration.
        next -= 1;
    } else {
        tokenizer.events[open_index + 1].point = sequences[open].end_point.clone();
    }

    next
}

/// Classify whether a character code represents whitespace, punctuation, or
/// something else.
///
/// Used for attention (emphasis, strong), whose sequences can open or close
/// based on the class of surrounding characters.
///
/// > 👉 **Note** that eof (`None`) is seen as whitespace.
///
/// ## References
///
/// *   [`micromark-util-classify-character` in `micromark`](https://github.com/micromark/micromark/blob/main/packages/micromark-util-classify-character/dev/index.js)
fn classify_character(char: Option<char>) -> CharacterKind {
    match char {
        // EOF.
        None => CharacterKind::Whitespace,
        // Unicode whitespace.
        Some(char) if char.is_whitespace() => CharacterKind::Whitespace,
        // Unicode punctuation.
        Some(char) if PUNCTUATION.contains(&char) => CharacterKind::Punctuation,
        // Everything else.
        Some(_) => CharacterKind::Other,
    }
}
