//! Trailing whitespace occurs in [string][] and [text][].
//!
//! It occurs around line endings, and, in the case of text content it also
//! occurs at the start or end of the whole.
//!
//! They’re formed with the following BNF:
//!
//! ```bnf
//! ; Restriction: the start and end here count as an eol in the case of `text`.
//! whitespace ::= 0.*space_or_tab eol 0.*space_or_tab
//! ```
//!
//! Normally this whitespace is ignored.
//! In the case of text content, whitespace before a line ending that
//! consistents solely of spaces, at least 2, forms a hard break (trailing).
//!
//! The minimum number of the spaces is defined in
//! [`HARD_BREAK_PREFIX_SIZE_MIN`][hard_break_prefix_size_min].
//!
//! Hard breaks in markdown relate to the HTML element `<br>`.
//! See [*§ 4.5.27 The `br` element* in the HTML spec][html] for more info.
//!
//! It is also possible to create a hard break with a similar construct: a
//! [hard break (escape)][hard_break_escape] is a backslash followed
//! by a line ending.
//! That construct is recommended because it is similar to a
//! [character escape][character_escape] and similar to how line endings can be
//! “escaped” in other languages.
//! Trailing spaces are typically invisible in editors, or even automatically
//! removed, making hard break (trailing) hard to use.
//! ## Tokens
//!
//! *   [`HardBreakTrailing`][Token::HardBreakTrailing]
//! *   [`SpaceOrTab`][Token::SpaceOrTab]
//!
//! ## References
//!
//! *   [`initialize/text.js` in `micromark`](https://github.com/micromark/micromark/blob/main/packages/micromark/dev/lib/initialize/text.js)
//! *   [*§ 6.7 Hard line breaks* in `CommonMark`](https://spec.commonmark.org/0.30/#hard-line-breaks)
//!
//! [string]: crate::content::string
//! [text]: crate::content::text
//! [hard_break_escape]: crate::construct::hard_break_escape
//! [character_escape]: crate::construct::character_escape
//! [hard_break_prefix_size_min]: crate::constant::HARD_BREAK_PREFIX_SIZE_MIN
//! [html]: https://html.spec.whatwg.org/multipage/text-level-semantics.html#the-br-element

use crate::constant::HARD_BREAK_PREFIX_SIZE_MIN;
use crate::token::Token;
use crate::tokenizer::{Code, Event, EventType, Tokenizer};
use crate::util::span;

/// To do.
pub fn create_resolve_whitespace(hard_break: bool, trim_whole: bool) -> impl Fn(&mut Tokenizer) {
    move |t| resolve_whitespace(t, hard_break, trim_whole)
}

/// To do.
pub fn resolve_whitespace(tokenizer: &mut Tokenizer, hard_break: bool, trim_whole: bool) {
    let mut index = 0;

    while index < tokenizer.events.len() {
        let event = &tokenizer.events[index];

        if event.event_type == EventType::Exit && event.token_type == Token::Data {
            let trim_start = (trim_whole && index == 1)
                || (index > 1 && tokenizer.events[index - 2].token_type == Token::LineEnding);
            let trim_end = (trim_whole && index == tokenizer.events.len() - 1)
                || (index + 1 < tokenizer.events.len()
                    && tokenizer.events[index + 1].token_type == Token::LineEnding);

            trim_data(tokenizer, index, trim_start, trim_end, hard_break);
        }

        index += 1;
    }
}

/// To do.
#[allow(clippy::too_many_lines)]
fn trim_data(
    tokenizer: &mut Tokenizer,
    exit_index: usize,
    trim_start: bool,
    trim_end: bool,
    hard_break: bool,
) {
    let mut codes = span::codes(
        &tokenizer.parse_state.codes,
        &span::from_exit_event(&tokenizer.events, exit_index),
    );

    if trim_end {
        let mut index = codes.len();
        let mut vs = 0;
        let mut spaces_only = true;
        while index > 0 {
            match codes[index - 1] {
                Code::Char(' ') => {}
                Code::Char('\t') => spaces_only = false,
                Code::VirtualSpace => {
                    vs += 1;
                    spaces_only = false;
                }
                _ => break,
            }

            index -= 1;
        }

        let diff = codes.len() - index;
        let token_type = if spaces_only
            && hard_break
            && exit_index + 1 < tokenizer.events.len()
            && diff >= HARD_BREAK_PREFIX_SIZE_MIN
        {
            Token::HardBreakTrailing
        } else {
            Token::SpaceOrTab
        };

        // The whole data is whitespace.
        // We can be very fast: we only change the token types.
        if index == 0 {
            tokenizer.events[exit_index - 1].token_type = token_type.clone();
            tokenizer.events[exit_index].token_type = token_type;
            return;
        }

        if diff > 0 {
            let exit_point = tokenizer.events[exit_index].point.clone();
            let mut enter_point = exit_point.clone();
            enter_point.index -= diff;
            enter_point.column -= diff - vs;
            enter_point.offset -= diff - vs;

            tokenizer.map.add(
                exit_index + 1,
                0,
                vec![
                    Event {
                        event_type: EventType::Enter,
                        token_type: token_type.clone(),
                        point: enter_point.clone(),
                        link: None,
                    },
                    Event {
                        event_type: EventType::Exit,
                        token_type,
                        point: exit_point,
                        link: None,
                    },
                ],
            );

            tokenizer.events[exit_index].point = enter_point;
            codes = &codes[..index];
        }
    }

    if trim_start {
        let mut index = 0;
        let mut vs = 0;
        while index < codes.len() {
            match codes[index] {
                Code::Char(' ' | '\t') => {}
                Code::VirtualSpace => vs += 1,
                _ => break,
            }

            index += 1;
        }

        // The whole data is whitespace.
        // We can be very fast: we only change the token types.
        if index == codes.len() {
            tokenizer.events[exit_index - 1].token_type = Token::SpaceOrTab;
            tokenizer.events[exit_index].token_type = Token::SpaceOrTab;
            return;
        }

        if index > 0 {
            let enter_point = tokenizer.events[exit_index - 1].point.clone();
            let mut exit_point = enter_point.clone();
            exit_point.index += index;
            exit_point.column += index - vs;
            exit_point.offset += index - vs;

            tokenizer.map.add(
                exit_index - 1,
                0,
                vec![
                    Event {
                        event_type: EventType::Enter,
                        token_type: Token::SpaceOrTab,
                        point: enter_point,
                        link: None,
                    },
                    Event {
                        event_type: EventType::Exit,
                        token_type: Token::SpaceOrTab,
                        point: exit_point.clone(),
                        link: None,
                    },
                ],
            );

            tokenizer.events[exit_index - 1].point = exit_point;
        }
    }
}
