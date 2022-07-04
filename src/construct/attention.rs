//! To do.

use crate::tokenizer::{Code, Event, EventType, Point, State, StateFnResult, TokenType, Tokenizer};
use crate::util::edit_map::EditMap;

/// To do
#[derive(Debug, PartialEq)]
enum GroupKind {
    Whitespace,
    Punctuation,
    Other,
}

/// To do
#[derive(Debug, PartialEq)]
enum MarkerKind {
    Asterisk,
    Underscore,
}

impl MarkerKind {
    fn from_char(char: char) -> MarkerKind {
        match char {
            '*' => MarkerKind::Asterisk,
            '_' => MarkerKind::Underscore,
            _ => unreachable!("invalid char"),
        }
    }
    fn from_code(code: Code) -> MarkerKind {
        match code {
            Code::Char(char) => MarkerKind::from_char(char),
            _ => unreachable!("invalid code"),
        }
    }
}

/// To do
#[derive(Debug)]
struct Run {
    marker: MarkerKind,
    event_index: usize,
    start_point: Point,
    start_index: usize,
    end_point: Point,
    end_index: usize,
    size: usize,
    open: bool,
    close: bool,
}

/// Before a paragraph.
///
/// ```markdown
/// |qwe
/// ```
pub fn start(tokenizer: &mut Tokenizer, code: Code) -> StateFnResult {
    match code {
        Code::Char(char) if char == '*' || char == '_' => {
            tokenizer.enter(TokenType::AttentionSequence);
            inside(tokenizer, code, char)
        }
        _ => (State::Nok, None),
    }
}

/// In a paragraph.
///
/// ```markdown
/// al|pha
/// ```
fn inside(tokenizer: &mut Tokenizer, code: Code, marker: char) -> StateFnResult {
    match code {
        Code::Char(char) if char == marker => {
            tokenizer.consume(code);
            (State::Fn(Box::new(move |t, c| inside(t, c, marker))), None)
        }
        _ => {
            tokenizer.exit(TokenType::AttentionSequence);
            tokenizer.register_resolver("attention".to_string(), Box::new(resolve));
            (State::Ok, Some(vec![code]))
        }
    }
}

/// To do.
#[allow(clippy::too_many_lines)]
pub fn resolve(tokenizer: &mut Tokenizer) -> Vec<Event> {
    let mut index = 0;
    println!("before: {:?}", tokenizer.events.len());
    while index < tokenizer.events.len() {
        let event = &tokenizer.events[index];
        println!(
            "ev: {:?} {:?} {:?} {:?} {:?} {:?}",
            index,
            event.event_type,
            event.token_type,
            event.content_type,
            event.previous,
            event.next
        );
        index += 1;
    }

    let codes = &tokenizer.parse_state.codes;
    let mut edit_map = EditMap::new();
    let mut start = 0;
    let mut runs: Vec<Run> = vec![];

    // Find runs of sequences and information about them.
    while start < tokenizer.events.len() {
        let enter = &tokenizer.events[start];

        if enter.event_type == EventType::Enter && enter.token_type == TokenType::AttentionSequence
        {
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

            runs.push(Run {
                event_index: start,
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

            start += 1;
        }

        start += 1;
    }

    // Walk through runs and match them.
    let mut close = 0;

    while close < runs.len() {
        let run_close = &runs[close];
        let mut next_index = close + 1;
        println!("walk! {:?} {:?}", close, runs.len());

        // Find a run that can close.
        if run_close.close {
            println!("close! {:?} {:?}", close, run_close);
            let mut open = close;

            // Now walk back to find an opener.
            while open > 0 {
                open -= 1;

                let run_open = &runs[open];

                // We found a run that can open the closer we found.
                if run_open.open && run_close.marker == run_open.marker {
                    println!("open! {:?} {:?}", open, run_open);
                    // If the opening can close or the closing can open,
                    // and the close size *is not* a multiple of three,
                    // but the sum of the opening and closing size *is*
                    // multiple of three, then **don’t** match.
                    if (run_open.close || run_close.open)
                        && run_close.size % 3 != 0
                        && (run_open.size + run_close.size) % 3 == 0
                    {
                        continue;
                    }

                    // We’ve found a match!

                    // Number of markers to use from the sequence.
                    let take = if run_open.size > 1 && run_close.size > 1 {
                        2
                    } else {
                        1
                    };

                    let run_close = &mut runs[close];
                    let close_event_index = run_close.event_index;
                    let seq_close_enter = (run_close.start_point.clone(), run_close.start_index);
                    run_close.size -= take;
                    run_close.start_point.column += take;
                    run_close.start_point.offset += take;
                    run_close.start_index += take;
                    let seq_close_exit = (run_close.start_point.clone(), run_close.start_index);

                    // Stay on this closing run for the next iteration: it
                    // might close more things.
                    next_index -= 1;

                    // Remove closing run if fully used.
                    if run_close.size == 0 {
                        runs.remove(close);
                        edit_map.add(close_event_index, 2, vec![]);
                        println!("remove close");
                    } else {
                        // Shift remaining closing run forward.
                        // Do it here because a run can open and close different
                        // other runs, and the remainder can be on any side or
                        // somewhere in the middle.
                        let mut enter = &mut tokenizer.events[close_event_index];
                        enter.point = seq_close_exit.0.clone();
                        enter.index = seq_close_exit.1;
                        println!("change close");
                    }

                    let run_open = &mut runs[open];
                    let open_event_index = run_open.event_index;
                    let seq_open_exit = (run_open.end_point.clone(), run_open.end_index);
                    run_open.size -= take;
                    run_open.end_point.column -= take;
                    run_open.end_point.offset -= take;
                    run_open.end_index -= take;
                    let seq_open_enter = (run_open.end_point.clone(), run_open.end_index);

                    // Remove opening run if fully used.
                    if run_open.size == 0 {
                        runs.remove(open);
                        edit_map.add(open_event_index, 2, vec![]);
                        next_index -= 1;
                        println!("remove open");
                    } else {
                        // Shift remaining opening run backwards.
                        // See note above for why that happens here.
                        let mut exit = &mut tokenizer.events[open_event_index + 1];
                        exit.point = seq_open_enter.0.clone();
                        exit.index = seq_open_enter.1;
                        println!("change open");
                    }

                    // Opening.
                    edit_map.add_before(
                        open_event_index,
                        0,
                        vec![
                            Event {
                                event_type: EventType::Enter,
                                token_type: if take == 1 {
                                    TokenType::Emphasis
                                } else {
                                    TokenType::Strong
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
                                    TokenType::EmphasisSequence
                                } else {
                                    TokenType::StrongSequence
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
                                    TokenType::EmphasisSequence
                                } else {
                                    TokenType::StrongSequence
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
                                    TokenType::EmphasisText
                                } else {
                                    TokenType::StrongText
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
                                    TokenType::EmphasisText
                                } else {
                                    TokenType::StrongText
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
                                    TokenType::EmphasisSequence
                                } else {
                                    TokenType::StrongSequence
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
                                    TokenType::EmphasisSequence
                                } else {
                                    TokenType::StrongSequence
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
                                    TokenType::Emphasis
                                } else {
                                    TokenType::Strong
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
    while index < runs.len() {
        let run = &runs[index];
        tokenizer.events[run.event_index].token_type = TokenType::Data;
        tokenizer.events[run.event_index + 1].token_type = TokenType::Data;
        index += 1;
    }

    let events = edit_map.consume(&mut tokenizer.events);
    let mut index = 0;
    println!("after: {:?}", events.len());
    while index < events.len() {
        let event = &events[index];
        println!(
            "ev: {:?} {:?} {:?} {:?} {:?} {:?}",
            index,
            event.event_type,
            event.token_type,
            event.content_type,
            event.previous,
            event.next
        );
        index += 1;
    }

    events
}

fn classify_character(code: Code) -> GroupKind {
    match code {
        // Markdown whitespace.
        Code::None
        | Code::CarriageReturnLineFeed
        | Code::VirtualSpace
        | Code::Char('\t' | '\r' | '\n' | ' ') => GroupKind::Whitespace,
        // Unicode whitespace.
        Code::Char(char) if char.is_whitespace() => GroupKind::Whitespace,
        // Unicode punctuation.
        // To do: `is_punctuation` is not in rust? Why not?
        // Perhaps we need to generate stuff just like:
        // <https://github.com/micromark/micromark/blob/main/packages/micromark-util-character/dev/lib/unicode-punctuation-regex.js>.
        Code::Char(char) if char.is_ascii_punctuation() => GroupKind::Punctuation,
        Code::Char(_) => GroupKind::Other,
    }
}
