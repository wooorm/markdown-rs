//! A little helper to parse `space_or_tab`
//!
//! They’re formed with the following BNF:
//!
//! ```bnf
//! space_or_tab ::= 1*(' ' '\t')
//! ```
//!
//! Depending on where whitespace can occur, it can be optional (or not),
//! and present in the rendered result (or not).
//!
//! ## References
//!
//! *   [`micromark-factory-space/index.js` in `micromark`](https://github.com/micromark/micromark/blob/main/packages/micromark-factory-space/dev/index.js)
//!
//! <!-- To do: link stuff -->

use crate::tokenizer::{Code, State, StateFnResult, TokenType, Tokenizer};

// To do: should `token_type` be a `Some`, with `None` defaulting to something?
// To do: should `max: Some(usize)` be added?

/// Before whitespace.
///
/// ```markdown
/// alpha| bravo
/// ```
pub fn start(tokenizer: &mut Tokenizer, code: Code, token_type: TokenType) -> StateFnResult {
    match code {
        Code::VirtualSpace | Code::Char('\t' | ' ') => {
            // To do: lifetimes.
            let clone = token_type.clone();
            tokenizer.enter(token_type);
            tokenizer.consume(code);
            (
                State::Fn(Box::new(|tokenizer, code| inside(tokenizer, code, clone))),
                None,
            )
        }
        _ => (State::Nok, None),
    }
}

/// In whitespace.
///
/// ```markdown
/// alpha |bravo
/// alpha | bravo
/// ```
fn inside(tokenizer: &mut Tokenizer, code: Code, token_type: TokenType) -> StateFnResult {
    match code {
        Code::VirtualSpace | Code::Char('\t' | ' ') => {
            tokenizer.consume(code);
            (
                State::Fn(Box::new(|tokenizer, code| {
                    inside(tokenizer, code, token_type)
                })),
                None,
            )
        }
        _ => {
            tokenizer.exit(token_type);
            (State::Ok, Some(vec![code]))
        }
    }
}
