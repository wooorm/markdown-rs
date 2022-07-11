//! To do.

use crate::token::Token;
use crate::tokenizer::{Code, State, StateFnResult, Tokenizer};

/// To do.
pub fn start(tokenizer: &mut Tokenizer, code: Code) -> StateFnResult {
    match code {
        Code::CarriageReturnLineFeed | Code::Char('\n' | '\r') => {
            tokenizer.enter(Token::LineEnding);
            tokenizer.consume(code);
            tokenizer.exit(Token::LineEnding);
            (State::Fn(Box::new(non_lazy_after)), None)
        }
        _ => (State::Nok, None),
    }
}

/// To do.
fn non_lazy_after(tokenizer: &mut Tokenizer, code: Code) -> StateFnResult {
    if tokenizer.lazy {
        (State::Nok, None)
    } else {
        (State::Ok, Some(vec![code]))
    }
}
