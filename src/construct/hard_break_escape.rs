//! Hard break escapes are a construct that occurs in the  [text][] content
//! type.
//!
//! They’re formed with the following BNF:
//!
//! ```bnf
//! ; Restriction: followed by a line ending  (that is part of the construct
//! ; instead of ending it).
//! hard_break_escape ::= '\\'
//! ```
//! It is also possible to escape punctuation characters with a similar
//! construct: a [character escape][character_escape] is a backslash followed
//! by an ASCII punctuation character.
//! Arbitrary characters can be escaped with
//! [character reference][character_reference]s.
//!
//! ## References
//!
//! *   [`hard-break-escape.js` in `micromark`](https://github.com/micromark/micromark/blob/main/packages/micromark-core-commonmark/dev/lib/hard-break-escape.js)
//! *   [*§ 6.7 Hard line breaks* in `CommonMark`](https://spec.commonmark.org/0.30/#hard-line-breaks)
//!
//! [text]: crate::content::text
//! [character_escape]: crate::construct::character_escape
//! [character_reference]: crate::construct::character_reference
//!
//! <!-- To do: link `hard_break_escape` -->

use crate::tokenizer::{Code, State, StateFnResult, TokenType, Tokenizer};

/// Start of a hard break escape.
///
/// ```markdown
/// a|\
/// ```
pub fn start(tokenizer: &mut Tokenizer, code: Code) -> StateFnResult {
    match code {
        Code::Char('\\') => {
            tokenizer.enter(TokenType::HardBreakEscape);
            tokenizer.enter(TokenType::HardBreakEscapeMarker);
            tokenizer.consume(code);
            tokenizer.exit(TokenType::HardBreakEscapeMarker);
            (State::Fn(Box::new(inside)), None)
        }
        _ => (State::Nok, None),
    }
}

/// At the end of a hard break escape, after `\`.
///
/// ```markdown
/// a\|
/// ```
fn inside(tokenizer: &mut Tokenizer, code: Code) -> StateFnResult {
    match code {
        Code::CarriageReturnLineFeed | Code::Char('\r' | '\n') => {
            tokenizer.exit(TokenType::HardBreakEscape);
            (State::Ok, Some(vec![code]))
        }
        _ => (State::Nok, None),
    }
}
