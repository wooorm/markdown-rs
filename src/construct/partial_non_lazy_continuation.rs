//! Non-lazy continuation.
//!
//! This is a tiny helper that [flow][] constructs can use to make sure that
//! the following line is not lazy.
//! For example, [html (flow)][html_flow] and code ([fenced][code_fenced],
//! [indented][code_indented]), stop when next line is lazy.
//!
//! [flow]: crate::content::flow
//! [code_fenced]: crate::construct::code_fenced
//! [code_indented]: crate::construct::code_indented
//! [html_flow]: crate::construct::html_flow

use crate::token::Token;
use crate::tokenizer::{State, StateName, Tokenizer};

/// Start of continuation.
///
/// ```markdown
/// > | * ```js
///            ^
///   | b
/// ```
pub fn start(tokenizer: &mut Tokenizer) -> State {
    match tokenizer.current {
        Some(b'\n') => {
            tokenizer.enter(Token::LineEnding);
            tokenizer.consume();
            tokenizer.exit(Token::LineEnding);
            State::Next(StateName::NonLazyContinuationAfter)
        }
        _ => State::Nok,
    }
}

/// After line ending.
///
/// ```markdown
///   | * ```js
/// > | b
///     ^
/// ```
pub fn after(tokenizer: &mut Tokenizer) -> State {
    if tokenizer.lazy {
        State::Nok
    } else {
        State::Ok
    }
}
