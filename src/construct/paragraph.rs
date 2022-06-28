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
//! *   [`Paragraph`][TokenType::Paragraph]
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
    blank_line::start as blank_line, code_fenced::start as code_fenced,
    heading_atx::start as heading_atx, html_flow::start as html_flow,
    partial_space_or_tab::space_or_tab_min_max, thematic_break::start as thematic_break,
};
use crate::subtokenize::link;
use crate::tokenizer::{Code, ContentType, State, StateFnResult, TokenType, Tokenizer};

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
            tokenizer.enter_with_content(TokenType::Data, Some(ContentType::Text));
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
    tokenizer.exit(TokenType::Data);
    tokenizer.enter_with_content(TokenType::Data, Some(ContentType::Text));
    let index = tokenizer.events.len() - 1;
    link(&mut tokenizer.events, index);
    (State::Fn(Box::new(inside)), None)
}

/// At a line ending, done.
///
/// ```markdown
/// alpha|
/// ***
/// ```
fn end(tokenizer: &mut Tokenizer, code: Code) -> StateFnResult {
    tokenizer.exit(TokenType::Data);
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
            (State::Fn(Box::new(interrupt_start)), None)
        }
        _ => unreachable!("expected eol"),
    }
}

/// After a line ending.
///
/// ```markdown
/// alpha
/// |~~~js
/// ~~~
/// ```
fn interrupt_start(tokenizer: &mut Tokenizer, code: Code) -> StateFnResult {
    // To do: If code is disabled, indented lines are allowed to interrupt.
    tokenizer.attempt(space_or_tab_min_max(TAB_SIZE, TAB_SIZE), |ok| {
        Box::new(if ok { interrupt_indent } else { interrupt_cont })
    })(tokenizer, code)
}

/// At an indent.
///
/// ```markdown
/// alpha
///     |
/// ```
fn interrupt_indent(_tokenizer: &mut Tokenizer, code: Code) -> StateFnResult {
    (State::Ok, Some(vec![code]))
}

/// Not at an indented line.
///
/// ```markdown
/// alpha
/// |<div>
/// ```
fn interrupt_cont(tokenizer: &mut Tokenizer, code: Code) -> StateFnResult {
    tokenizer.attempt_n(
        vec![
            Box::new(blank_line),
            Box::new(code_fenced),
            Box::new(html_flow),
            Box::new(heading_atx),
            Box::new(thematic_break),
        ],
        |ok| Box::new(move |_t, code| (if ok { State::Nok } else { State::Ok }, Some(vec![code]))),
    )(tokenizer, code)
}
