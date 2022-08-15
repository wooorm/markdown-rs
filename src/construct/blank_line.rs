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
//! items in a [list][list-item].
//! More than one blank line is never needed in `CommonMark`.
//!
//! Because blank lines can be empty (line endings are not considered part of
//! it), and events cannot be empty, blank lines are not present as a token.
//!
//! ## Tokens
//!
//! *   [`SpaceOrTab`][crate::event::Name::SpaceOrTab]
//!
//! ## References
//!
//! *   [`blank-line.js` in `micromark`](https://github.com/micromark/micromark/blob/main/packages/micromark-core-commonmark/dev/lib/blank-line.js)
//! *   [*§ 4.9 Blank lines* in `CommonMark`](https://spec.commonmark.org/0.30/#blank-lines)
//!
//! [heading-atx]: crate::construct::heading_atx
//! [list-item]: crate::construct::list_item
//! [paragraph]: crate::construct::paragraph
//! [flow]: crate::content::flow

use crate::construct::partial_space_or_tab::space_or_tab;
use crate::state::{Name as StateName, State};
use crate::tokenizer::Tokenizer;

/// Start of blank line.
///
/// > 👉 **Note**: `␠` represents a space character.
///
/// ```markdown
/// > | ␠␠␊
///     ^
/// > | ␊
///     ^
/// ```
pub fn start(tokenizer: &mut Tokenizer) -> State {
    if matches!(tokenizer.current, Some(b'\t' | b' ')) {
        tokenizer.attempt(State::Next(StateName::BlankLineAfter), State::Nok);
        State::Retry(space_or_tab(tokenizer))
    } else {
        State::Retry(StateName::BlankLineAfter)
    }
}

/// At eof/eol, after optional whitespace.
///
/// ```markdown
/// > | ␠␠␊
///       ^
/// > | ␊
///     ^
/// ```
pub fn after(tokenizer: &mut Tokenizer) -> State {
    match tokenizer.current {
        None | Some(b'\n') => State::Ok,
        _ => State::Nok,
    }
}
