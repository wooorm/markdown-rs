//! Hard break (escape) is a construct that occurs in the  [text][] content
//! type.
//!
//! They’re formed with the following BNF:
//!
//! ```bnf
//! ; Restriction: followed by a line ending  (that is part of the construct
//! ; instead of ending it).
//! hard_break_escape ::= '\\'
//! ```
//!
//! Hard breaks in markdown relate to the HTML element `<br>`.
//! See [*§ 4.5.27 The `br` element* in the HTML spec][html] for more info.
//!
//! It is also possible to create a hard break with a
//! [hard break (trailing)][hard_break_trailing].
//! That construct is not recommended because trailing spaces are typically
//! invisible in editors, or even automatically removed, making them hard to
//! use.
//!
//! It is also possible to escape punctuation characters with a similar
//! construct: a [character escape][character_escape] is a backslash followed
//! by an ASCII punctuation character.
//! Arbitrary characters can be escaped with
//! [character reference][character_reference]s.
//!
//! ## Tokens
//!
//! *   [`HardBreakEscape`][Token::HardBreakEscape]
//! *   [`HardBreakEscapeMarker`][Token::HardBreakEscapeMarker]
//!
//! ## References
//!
//! *   [`hard-break-escape.js` in `micromark`](https://github.com/micromark/micromark/blob/main/packages/micromark-core-commonmark/dev/lib/hard-break-escape.js)
//! *   [*§ 6.7 Hard line breaks* in `CommonMark`](https://spec.commonmark.org/0.30/#hard-line-breaks)
//!
//! [text]: crate::content::text
//! [character_escape]: crate::construct::character_escape
//! [character_reference]: crate::construct::character_reference
//! [hard_break_trailing]: crate::construct::hard_break_trailing
//! [html]: https://html.spec.whatwg.org/multipage/text-level-semantics.html#the-br-element

use crate::token::Token;
use crate::tokenizer::{Code, State, StateFnResult, Tokenizer};

/// Start of a hard break (escape).
///
/// ```markdown
/// > | a\
///      ^
///   | b
/// ```
pub fn start(tokenizer: &mut Tokenizer, code: Code) -> StateFnResult {
    match code {
        Code::Char('\\') => {
            tokenizer.enter(Token::HardBreakEscape);
            tokenizer.enter(Token::HardBreakEscapeMarker);
            tokenizer.consume(code);
            tokenizer.exit(Token::HardBreakEscapeMarker);
            (State::Fn(Box::new(inside)), None)
        }
        _ => (State::Nok, None),
    }
}

/// At the end of a hard break (escape), after `\`.
///
/// ```markdown
/// > | a\
///       ^
///   | b
/// ```
fn inside(tokenizer: &mut Tokenizer, code: Code) -> StateFnResult {
    match code {
        Code::CarriageReturnLineFeed | Code::Char('\n' | '\r') => {
            tokenizer.exit(Token::HardBreakEscape);
            (State::Ok, Some(vec![code]))
        }
        _ => (State::Nok, None),
    }
}
