//! Hard break (trailing) is a construct that occurs in the  [text][] content
//! type.
//!
//! They’re formed with the following BNF:
//!
//! ```bnf
//! ; Restriction: followed by a line ending  (that is part of the construct
//! ; instead of ending it).
//! hard_break_trailing ::= 2*' '
//! ```
//!
//! The minimum number of the spaces is defined in
//! [`HARD_BREAK_PREFIX_SIZE_MIN`][hard_break_prefix_size_min].
//!
//! Hard breaks in markdown relate to the HTML element `<br>`.
//! See [*§ 4.5.27 The `br` element* in the HTML spec][html] for more info.
//!
//! It is also possible to create a hard break with a similar construct: a
//! [hard break (escape)][hard_break_escape] is a backslash followed
//! by a line ending.
//! That construct is recommended because it is similar to a
//! [character escape][character_escape] and similar to how line endings can be
//! “escaped” in other languages.
//! Trailing spaces are typically invisible in editors, or even automatically
//! removed, making hard break (trailing) hard to use.
//!
//! ## Tokens
//!
//! *   [`HardBreakTrailing`][Token::HardBreakTrailing]
//! *   [`HardBreakTrailingSpace`][Token::HardBreakTrailingSpace]
//!
//! ## References
//!
//! *   [`lib/initialize/text.js` in `micromark`](https://github.com/micromark/micromark/blob/main/packages/micromark/dev/lib/initialize/text.js)
//! *   [*§ 6.7 Hard line breaks* in `CommonMark`](https://spec.commonmark.org/0.30/#hard-line-breaks)
//!
//! [text]: crate::content::text
//! [hard_break_escape]: crate::construct::hard_break_escape
//! [character_escape]: crate::construct::character_escape
//! [hard_break_prefix_size_min]: crate::constant::HARD_BREAK_PREFIX_SIZE_MIN
//! [html]: https://html.spec.whatwg.org/multipage/text-level-semantics.html#the-br-element

use crate::constant::HARD_BREAK_PREFIX_SIZE_MIN;
use crate::token::Token;
use crate::tokenizer::{Code, State, Tokenizer};

/// Start of a hard break (trailing).
///
/// ```markdown
/// > | a␠␠
///      ^
///   | b
/// ```
pub fn start(tokenizer: &mut Tokenizer, code: Code) -> State {
    match code {
        Code::Char(' ') if tokenizer.parse_state.constructs.hard_break_trailing => {
            tokenizer.enter(Token::HardBreakTrailing);
            tokenizer.enter(Token::HardBreakTrailingSpace);
            tokenizer.consume(code);
            State::Fn(Box::new(|t, c| inside(t, c, 1)))
        }
        _ => State::Nok,
    }
}

/// Inside the hard break (trailing).
///
/// ```markdown
/// > | a␠␠
///      ^
///   | b
/// ```
fn inside(tokenizer: &mut Tokenizer, code: Code, size: usize) -> State {
    match code {
        Code::Char(' ') => {
            tokenizer.consume(code);
            State::Fn(Box::new(move |t, c| inside(t, c, size + 1)))
        }
        Code::CarriageReturnLineFeed | Code::Char('\n' | '\r')
            if size >= HARD_BREAK_PREFIX_SIZE_MIN =>
        {
            tokenizer.exit(Token::HardBreakTrailingSpace);
            tokenizer.exit(Token::HardBreakTrailing);
            State::Ok(0)
        }
        _ => State::Nok,
    }
}
