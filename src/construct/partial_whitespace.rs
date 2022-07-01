//! Trailing whitespace occurs in [string][] and [text][].
//!
//! It occurs at the start or end of the whole, or around line endings.
//! This whitespace is ignored
//!
//! They’re formed with the following BNF:
//!
//! ```bnf
//! ; Restriction: the start and end here count as an eol.
//! whitespace ::= 0.*space_or_tab eol 0.*space_or_tab
//! ```
//!
//! This is similar to [`space_or_tab_eol`][space_or_tab_eol], with the main
//! difference that that *does not* require a line ending and parses any
//! `space_or_tab` with one line ending.
//! This instead *requires* the line ending (or eol).
//!
//! ## References
//!
//! *   [`initialize/text.js` in `micromark`](https://github.com/micromark/micromark/blob/main/packages/micromark/dev/lib/initialize/text.js)
//!
//! [string]: crate::content::string
//! [text]: crate::content::text
//! [space_or_tab_eol]: crate::construct::partial_space_or_tab::space_or_tab_eol

use super::partial_space_or_tab::space_or_tab;
use crate::tokenizer::{Code, State, StateFnResult, Tokenizer};

/// Parse initial or final whitespace.
pub fn whitespace(tokenizer: &mut Tokenizer, code: Code) -> StateFnResult {
    tokenizer.go(
        // Nothing if there’s no whitespace.
        space_or_tab(),
        if matches!(
            tokenizer.previous,
            Code::None | Code::CarriageReturnLineFeed | Code::Char('\r' | '\n')
        ) {
            // If there’s whitespace, and we were at an eol/eof, `ok`
            ok
        } else {
            // If there’s whitespace, and we were not at an eol/eof, there must be one here.
            at_eol
        },
    )(tokenizer, code)
}

/// After whitespace, at an eol/eof.
fn at_eol(tokenizer: &mut Tokenizer, code: Code) -> StateFnResult {
    if matches!(
        code,
        Code::None | Code::CarriageReturnLineFeed | Code::Char('\r' | '\n')
    ) {
        ok(tokenizer, code)
    } else {
        (State::Nok, None)
    }
}

/// Fine.
fn ok(_tokenizer: &mut Tokenizer, code: Code) -> StateFnResult {
    (State::Ok, Some(vec![code]))
}
