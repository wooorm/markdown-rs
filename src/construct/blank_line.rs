//! Blank lines are a construct that occurs in the [flow][] content type.
//!
//! They’re formed with the following BNF:
//!
//! ```bnf
//! blank_line ::= *(' ' '\t')
//! ```
//!
//! Blank lines are sometimes needed, such as to differentiate a [paragraph][]
//! from another paragraph.
//! In several cases, blank lines are not needed between flow constructs,
//! such as between two [heading (atx)][heading-atx]s.
//! Sometimes, whether blank lines are present, changes the behavior of how
//! HTML is rendered, such as whether blank lines are present between list
//! items in a list.
//! More than one blank line is never needed in `CommonMark`.
//!
//! Because blank lines can be empty (line endings are not considered part of
//! it), and events cannot be empty, blank lines are not present as a token.
//!
//! ## References
//!
//! *   [`blank-line.js` in `micromark`](https://github.com/micromark/micromark/blob/main/packages/micromark-core-commonmark/dev/lib/blank-line.js)
//! *   [*§ 4.9 Blank lines* in `CommonMark`](https://spec.commonmark.org/0.30/#blank-lines)
//!
//! [flow]: crate::content::flow
//! [paragraph]: crate::construct::paragraph
//! [heading-atx]: crate::construct::heading_atx
//!
//! <!-- To do: link `list` -->

use crate::construct::partial_whitespace::start as whitespace;
use crate::tokenizer::{Code, State, StateFnResult, TokenType, Tokenizer};

/// Start of a blank line.
///
/// Note: `␠` represents a space character.
///
/// ```markdown
/// |␠␠
/// |
/// ```
pub fn start(tokenizer: &mut Tokenizer, code: Code) -> StateFnResult {
    tokenizer.attempt(
        |tokenizer, code| whitespace(tokenizer, code, TokenType::BlankLineWhitespace),
        |_ok| Box::new(after),
    )(tokenizer, code)
}

/// After zero or more spaces or tabs, before a line ending or EOF.
///
/// Note: `␠` represents a space character.
///
/// ```markdown
/// |␠␠
/// |
/// ```
fn after(_tokenizer: &mut Tokenizer, code: Code) -> StateFnResult {
    match code {
        Code::None | Code::CarriageReturnLineFeed | Code::Char('\n' | '\r') => {
            (State::Ok, Some(vec![code]))
        }
        _ => (State::Nok, None),
    }
}
