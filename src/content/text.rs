//! The text content type.
//!
//! **Text** contains phrasing content such as attention (emphasis, strong),
//! media (links, images), and actual text.
//!
//! The constructs found in text are:
//!
//! *   [Autolink][crate::construct::autolink]
//! *   Attention
//! *   [HTML (text)][crate::construct::html_text]
//! *   Hard break escape
//! *   [Code (text)][crate::construct::code_text]
//! *   Line ending
//! *   Label start (image)
//! *   Label start (link)
//! *   [Character escape][crate::construct::character_escape]
//! *   [Character reference][crate::construct::character_reference]

use crate::construct::{
    autolink::start as autolink, character_escape::start as character_escape,
    character_reference::start as character_reference, code_text::start as code_text,
    html_text::start as html_text,
};
use crate::tokenizer::{Code, State, StateFnResult, TokenType, Tokenizer};

/// Before text.
///
/// First we assume character reference.
///
/// ```markdown
/// |&amp;
/// |\&
/// |qwe
/// ```
pub fn start(tokenizer: &mut Tokenizer, code: Code) -> StateFnResult {
    match code {
        Code::None => (State::Ok, None),
        _ => tokenizer.attempt_5(
            character_reference,
            character_escape,
            autolink,
            html_text,
            code_text,
            |ok| Box::new(if ok { start } else { before_data }),
        )(tokenizer, code),
    }
}

/// Before text.
///
/// We’re at data.
///
/// ```markdown
/// |qwe
/// ```
fn before_data(tokenizer: &mut Tokenizer, code: Code) -> StateFnResult {
    match code {
        Code::None => (State::Ok, None),
        Code::CarriageReturnLineFeed | Code::Char('\r' | '\n') => {
            tokenizer.enter(TokenType::LineEnding);
            tokenizer.consume(code);
            tokenizer.exit(TokenType::LineEnding);
            (State::Fn(Box::new(start)), None)
        }
        _ => {
            tokenizer.enter(TokenType::Data);
            tokenizer.consume(code);
            (State::Fn(Box::new(in_data)), None)
        }
    }
}

/// In data.
///
/// ```markdown
/// q|w|e
/// ```
fn in_data(tokenizer: &mut Tokenizer, code: Code) -> StateFnResult {
    match code {
        Code::None => {
            tokenizer.exit(TokenType::Data);
            (State::Ok, None)
        }
        // To do: somehow get these markers from constructs.
        Code::CarriageReturnLineFeed | Code::Char('\r' | '\n' | '&' | '<' | '\\' | '`') => {
            tokenizer.exit(TokenType::Data);
            start(tokenizer, code)
        }
        _ => {
            tokenizer.consume(code);
            (State::Fn(Box::new(in_data)), None)
        }
    }
}
