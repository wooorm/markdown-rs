//! Data occurs in [text][] and [string][].
//!
//! It can include anything (including line endings), and stops at certain
//! characters.
//!
//! [string]: crate::content::string
//! [text]: crate::content::text

// To do: pass token types in?

use crate::tokenizer::{Code, State, StateFnResult, TokenType, Tokenizer};

/// At the beginning of data.
///
/// ```markdown
/// |&qwe
/// ```
pub fn start(tokenizer: &mut Tokenizer, code: Code, stop: Vec<Code>) -> StateFnResult {
    if stop.contains(&code) {
        tokenizer.enter(TokenType::Data);
        tokenizer.consume(code);
        (State::Fn(Box::new(|t, c| data(t, c, stop))), None)
    } else {
        at_break(tokenizer, code, stop)
    }
}

/// Before something.
///
/// ```markdown
/// |qwe| |&
/// ```
fn at_break(tokenizer: &mut Tokenizer, code: Code, stop: Vec<Code>) -> StateFnResult {
    match code {
        Code::None => (State::Ok, None),
        Code::CarriageReturnLineFeed | Code::Char('\r' | '\n') => {
            tokenizer.enter(TokenType::LineEnding);
            tokenizer.consume(code);
            tokenizer.exit(TokenType::LineEnding);
            (State::Fn(Box::new(|t, c| at_break(t, c, stop))), None)
        }
        _ if stop.contains(&code) => (State::Ok, Some(vec![code])),
        _ => {
            tokenizer.enter(TokenType::Data);
            data(tokenizer, code, stop)
        }
    }
}

/// In data.
///
/// ```markdown
/// q|w|e
/// ```
fn data(tokenizer: &mut Tokenizer, code: Code, stop: Vec<Code>) -> StateFnResult {
    let done = match code {
        Code::None | Code::CarriageReturnLineFeed | Code::Char('\r' | '\n') => true,
        _ if stop.contains(&code) => true,
        _ => false,
    };

    if done {
        tokenizer.exit(TokenType::Data);
        at_break(tokenizer, code, stop)
    } else {
        tokenizer.consume(code);
        (State::Fn(Box::new(|t, c| data(t, c, stop))), None)
    }
}
