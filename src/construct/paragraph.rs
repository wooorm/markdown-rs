//! Paragraph is a construct that occurs in the [flow] content type.
//!
//! They’re formed with the following BNF:
//!
//! ```bnf
//! ; Restriction: lines cannot start other flow constructs.
//! ; Restriction: lines cannot be blank.
//! paragraph ::= 1*line *( eol 1*line )
//! ```
//!
//! Paragraphs in markdown relate to the `<p>` element in HTML.
//! See [*§ 4.4.1 The `p` element* in the HTML spec][html] for more info.
//!
//! Paragraphs can contain line endings and whitespace, but they are not
//! allowed to contain blank lines, or to be blank themselves.
//!
//! The paragraph is interpreted as the [text][] content type.
//! That means that [autolinks][autolink], [code (text)][code_text], etc are allowed.
//!
//! ## Tokens
//!
//! *   [`Paragraph`][Token::Paragraph]
//!
//! ## References
//!
//! *   [`content.js` in `micromark`](https://github.com/micromark/micromark/blob/main/packages/micromark-core-commonmark/dev/lib/content.js)
//! *   [*§ 4.8 Paragraphs* in `CommonMark`](https://spec.commonmark.org/0.30/#paragraphs)
//!
//! [flow]: crate::content::flow
//! [text]: crate::content::text
//! [autolink]: crate::construct::autolink
//! [code_text]: crate::construct::code_text
//! [html]: https://html.spec.whatwg.org/multipage/grouping-content.html#the-p-element

use crate::token::Token;
use crate::tokenizer::{Code, ContentType, EventType, State, StateFnResult, Tokenizer};
use crate::util::{edit_map::EditMap, skip::opt as skip_opt};

/// Before a paragraph.
///
/// ```markdown
/// > | abc
///     ^
/// ```
pub fn start(tokenizer: &mut Tokenizer, code: Code) -> StateFnResult {
    match code {
        Code::None | Code::CarriageReturnLineFeed | Code::Char('\n' | '\r') => {
            unreachable!("unexpected eol/eof")
        }
        _ => {
            tokenizer.enter(Token::Paragraph);
            tokenizer.enter_with_content(Token::Data, Some(ContentType::Text));
            inside(tokenizer, code)
        }
    }
}

/// In a paragraph.
///
/// ```markdown
/// > | abc
///     ^^^
/// ```
fn inside(tokenizer: &mut Tokenizer, code: Code) -> StateFnResult {
    match code {
        Code::None | Code::CarriageReturnLineFeed | Code::Char('\n' | '\r') => {
            tokenizer.exit(Token::Data);
            tokenizer.exit(Token::Paragraph);
            tokenizer.register_resolver_before("paragraph".to_string(), Box::new(resolve));
            // You’d be interrupting.
            tokenizer.interrupt = true;
            (State::Ok, Some(vec![code]))
        }
        _ => {
            tokenizer.consume(code);
            (State::Fn(Box::new(inside)), None)
        }
    }
}

/// Merge “`Paragraph`”s, which currently span a single line, into actual
/// `Paragraph`s that span multiple lines.
pub fn resolve(tokenizer: &mut Tokenizer) {
    let mut edit_map = EditMap::new();
    let len = tokenizer.events.len();
    let mut index = 0;

    while index < len {
        let event = &tokenizer.events[index];

        if event.event_type == EventType::Enter && event.token_type == Token::Paragraph {
            // Exit:Paragraph
            let mut exit_index = index + 3;
            let mut enter_next_index =
                skip_opt(&tokenizer.events, exit_index + 1, &[Token::LineEnding]);
            // Enter:Paragraph
            enter_next_index = skip_opt(
                &tokenizer.events,
                enter_next_index,
                &[Token::SpaceOrTab, Token::BlockQuotePrefix],
            );

            // Find future `Paragraphs`.
            while enter_next_index < tokenizer.events.len()
                && tokenizer.events[enter_next_index].token_type == Token::Paragraph
            {
                // Remove Exit:Paragraph, Enter:LineEnding, Exit:LineEnding, Enter:Paragraph.
                edit_map.add(exit_index, 3, vec![]);

                // Remove Enter:Paragraph.
                edit_map.add(enter_next_index, 1, vec![]);

                // Add Exit:LineEnding position info to Exit:Data.
                let line_ending_exit = &tokenizer.events[exit_index + 2];
                let line_ending_point = line_ending_exit.point.clone();
                let line_ending_index = line_ending_exit.index;
                let data_exit = &mut tokenizer.events[exit_index - 1];
                data_exit.point = line_ending_point;
                data_exit.index = line_ending_index;

                // Link Enter:Data on the previous line to Enter:Data on this line.
                let data_enter_prev = &mut tokenizer.events[exit_index - 2];
                data_enter_prev.next = Some(enter_next_index + 1);
                let data_enter_next = &mut tokenizer.events[enter_next_index + 1];
                data_enter_next.previous = Some(exit_index - 2);

                // Potential next start.
                exit_index = enter_next_index + 3;
                enter_next_index =
                    skip_opt(&tokenizer.events, exit_index + 1, &[Token::LineEnding]);
                enter_next_index = skip_opt(
                    &tokenizer.events,
                    enter_next_index,
                    &[Token::SpaceOrTab, Token::BlockQuotePrefix],
                );
            }

            // Move to `Exit:Paragraph`.
            index = exit_index;
        }

        index += 1;
    }

    edit_map.consume(&mut tokenizer.events);
}
