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

use crate::constant::TAB_SIZE;
use crate::construct::{
    code_fenced::start as code_fenced, heading_atx::start as heading_atx,
    html_flow::start as html_flow, partial_whitespace::start as whitespace,
    thematic_break::start as thematic_break,
};
use crate::tokenizer::{Code, State, StateFnResult, TokenType, Tokenizer};
use crate::util::span::from_exit_event;

/// Before a paragraph.
///
/// ```markdown
/// |qwe
/// ```
pub fn start(tokenizer: &mut Tokenizer, code: Code) -> StateFnResult {
    match code {
        Code::None | Code::CarriageReturnLineFeed | Code::Char('\n' | '\r') => {
            unreachable!("unexpected eol/eof at start of paragraph")
        }
        _ => {
            tokenizer.enter(TokenType::Paragraph);
            tokenizer.enter(TokenType::ChunkText);
            inside(tokenizer, code)
        }
    }
}

/// In a paragraph.
///
/// ```markdown
/// al|pha
/// ```
fn inside(tokenizer: &mut Tokenizer, code: Code) -> StateFnResult {
    match code {
        Code::None => end(tokenizer, code),
        Code::CarriageReturnLineFeed | Code::Char('\n' | '\r') => tokenizer
            .check(interrupt, |ok| {
                Box::new(if ok { at_line_ending } else { end })
            })(tokenizer, code),
        _ => {
            tokenizer.consume(code);
            (State::Fn(Box::new(inside)), None)
        }
    }
}

/// At a line ending, not interrupting.
///
/// ```markdown
/// alpha|
/// bravo.
/// ```
fn at_line_ending(tokenizer: &mut Tokenizer, code: Code) -> StateFnResult {
    tokenizer.consume(code);
    tokenizer.exit(TokenType::ChunkText);
    tokenizer.enter(TokenType::ChunkText);
    let next_index = tokenizer.events.len() - 1;
    tokenizer.events[next_index - 2].next = Some(next_index);
    tokenizer.events[next_index].previous = Some(next_index - 2);
    (State::Fn(Box::new(inside)), None)
}

/// At a line ending, done.
///
/// ```markdown
/// alpha|
/// ***
/// ```
fn end(tokenizer: &mut Tokenizer, code: Code) -> StateFnResult {
    tokenizer.exit(TokenType::ChunkText);
    tokenizer.exit(TokenType::Paragraph);
    (State::Ok, Some(vec![code]))
}

/// Before a potential interruption.
///
/// ```markdown
/// alpha|
/// ***
/// ```
fn interrupt(tokenizer: &mut Tokenizer, code: Code) -> StateFnResult {
    match code {
        Code::CarriageReturnLineFeed | Code::Char('\n' | '\r') => {
            tokenizer.enter(TokenType::LineEnding);
            tokenizer.consume(code);
            tokenizer.exit(TokenType::LineEnding);
            (State::Fn(Box::new(interrupt_initial)), None)
        }
        _ => unreachable!("expected eol"),
    }
}

/// After a line ending.
///
/// ```markdown
/// alpha|
/// ~~~js
/// ~~~
/// ```
fn interrupt_initial(tokenizer: &mut Tokenizer, code: Code) -> StateFnResult {
    tokenizer.attempt_2(code_fenced, html_flow, |ok| {
        if ok {
            Box::new(|_tokenizer, _code| (State::Nok, None))
        } else {
            Box::new(|tokenizer, code| {
                tokenizer.attempt(
                    |tokenizer, code| whitespace(tokenizer, code, TokenType::Whitespace),
                    |_ok| Box::new(interrupt_start),
                )(tokenizer, code)
            })
        }
    })(tokenizer, code)
}

/// After a line ending, after optional whitespace.
///
/// ```markdown
/// alpha|
/// # bravo
/// ```
fn interrupt_start(tokenizer: &mut Tokenizer, code: Code) -> StateFnResult {
    let tail = tokenizer.events.last();
    let mut prefix = 0;

    if let Some(event) = tail {
        if event.token_type == TokenType::Whitespace {
            let span = from_exit_event(&tokenizer.events, tokenizer.events.len() - 1);
            prefix = span.end_index - span.start_index;
        }
    }

    match code {
        // Blank lines are not allowed in paragraph.
        Code::None | Code::CarriageReturnLineFeed | Code::Char('\n' | '\r') => (State::Nok, None),
        // To do: If code is disabled, indented lines are allowed.
        _ if prefix >= TAB_SIZE => (State::Ok, None),
        // To do: definitions, setext headings, etc?
        _ => tokenizer.attempt_2(heading_atx, thematic_break, |ok| {
            let result = if ok {
                (State::Nok, None)
            } else {
                (State::Ok, None)
            };
            Box::new(|_t, _c| result)
        })(tokenizer, code),
    }
}
