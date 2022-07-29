//! To do.

use crate::token::Token;
use crate::tokenizer::{State, Tokenizer};

/// Before a BOM.
///
/// ```text
/// > | 0xEF 0xBB 0xBF
///     ^^^^
/// ```
pub fn start(tokenizer: &mut Tokenizer) -> State {
    if tokenizer.current == Some(0xEF) {
        tokenizer.enter(Token::ByteOrderMark);
        tokenizer.consume();
        State::Fn(Box::new(cont))
    } else {
        State::Nok
    }
}

/// Second byte in BOM.
///
/// ```text
/// > | 0xEF 0xBB 0xBF
///          ^^^^
/// ```
fn cont(tokenizer: &mut Tokenizer) -> State {
    if tokenizer.current == Some(0xBB) {
        tokenizer.consume();
        State::Fn(Box::new(end))
    } else {
        State::Nok
    }
}

/// Last byte in BOM.
///
/// ```text
/// > | 0xEF 0xBB 0xBF
///               ^^^^
/// ```
fn end(tokenizer: &mut Tokenizer) -> State {
    if tokenizer.current == Some(0xBF) {
        tokenizer.consume();
        tokenizer.exit(Token::ByteOrderMark);
        State::Ok
    } else {
        State::Nok
    }
}
