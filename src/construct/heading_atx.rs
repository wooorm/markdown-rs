//! Heading (atx) is a construct that occurs in the [flow] content type.
//!
//! They’re formed with the following BNF:
//!
//! ```bnf
//! heading_atx ::= 1*6'#' [ 1*space_or_tab text [ 1*space_or_tab 1*'#' ] ] *space_or_tab
//!
//! text ::= code - eol
//! space_or_tab ::= ' ' | '\t'
//! ```
//!
//! Headings in markdown relate to the `<h1>` through `<h6>` elements in HTML.
//! See [*§ 4.3.6 The `h1`, `h2`, `h3`, `h4`, `h5`, and `h6` elements* in the
//! HTML spec][html] for more info.
//!
//! `CommonMark` introduced the requirement on whitespace existing after the
//! opening sequence and before text.
//! In older markdown versions, this was not required, and headings would form
//! without it.
//!
//! In markdown, it is also possible to create headings with a
//! [heading (setext)][heading_setext] construct.
//! The benefit of setext headings is that their text can include line endings,
//! and by extensions also hard breaks (e.g., with
//! [hard break (escape)][hard_break_escape]).
//! However, their limit is that they cannot form `<h3>` through `<h6>`
//! headings.
//! Due to this limitation, it is recommended to use atx headings.
//!
//! > 🏛 **Background**: the word *setext* originates from a small markup
//! > language by Ian Feldman from 1991.
//! > See [*§ Setext* on Wikipedia][wiki-setext] for more info.
//! > The word *atx* originates from a tiny markup language by Aaron Swartz
//! > from 2002.
//! > See [*§ atx, the true structured text format* on `aaronsw.com`][atx] for
//! > more info.
//!
//! ## Tokens
//!
//! *   [`HeadingAtx`][TokenType::HeadingAtx]
//! *   [`HeadingAtxSequence`][TokenType::HeadingAtxSequence]
//! *   [`HeadingAtxText`][TokenType::HeadingAtxText]
//! *   [`HeadingAtxSpaceOrTab`][TokenType::HeadingAtxSpaceOrTab]
//!
//! ## References
//!
//! *   [`heading-atx.js` in `micromark`](https://github.com/micromark/micromark/blob/main/packages/micromark-core-commonmark/dev/lib/heading-atx.js)
//! *   [*§ 4.2 ATX headings* in `CommonMark`](https://spec.commonmark.org/0.30/#atx-headings)
//!
//! [flow]: crate::content::flow
//! [heading_setext]: crate::construct::heading_setext
//! [hard_break_escape]: crate::construct::hard_break_escape
//! [html]: https://html.spec.whatwg.org/multipage/sections.html#the-h1,-h2,-h3,-h4,-h5,-and-h6-elements
//! [wiki-setext]: https://en.wikipedia.org/wiki/Setext
//! [atx]: http://www.aaronsw.com/2002/atx/

use super::partial_space_or_tab::{
    space_or_tab, space_or_tab_with_options, Options as SpaceOrTabOptions,
};
use crate::constant::HEADING_ATX_OPENING_FENCE_SIZE_MAX;
use crate::tokenizer::{Code, State, StateFnResult, TokenType, Tokenizer};

/// Start of a heading (atx).
///
/// ```markdown
/// |## alpha
/// ```
pub fn start(tokenizer: &mut Tokenizer, code: Code) -> StateFnResult {
    tokenizer.enter(TokenType::HeadingAtx);
    tokenizer.attempt_opt(space_or_tab(), before)(tokenizer, code)
}

/// Start of a heading (atx), after whitespace.
///
/// ```markdown
/// |## alpha
/// ```
fn before(tokenizer: &mut Tokenizer, code: Code) -> StateFnResult {
    if Code::Char('#') == code {
        tokenizer.enter(TokenType::HeadingAtxSequence);
        sequence_open(tokenizer, code, 0)
    } else {
        (State::Nok, None)
    }
}

/// In the opening sequence.
///
/// ```markdown
/// #|# alpha
/// ```
fn sequence_open(tokenizer: &mut Tokenizer, code: Code, rank: usize) -> StateFnResult {
    match code {
        Code::None | Code::CarriageReturnLineFeed | Code::Char('\n' | '\r') if rank > 0 => {
            tokenizer.exit(TokenType::HeadingAtxSequence);
            at_break(tokenizer, code)
        }
        Code::Char('#') if rank < HEADING_ATX_OPENING_FENCE_SIZE_MAX => {
            tokenizer.consume(code);
            (
                State::Fn(Box::new(move |tokenizer, code| {
                    sequence_open(tokenizer, code, rank + 1)
                })),
                None,
            )
        }
        _ if rank > 0 => {
            tokenizer.exit(TokenType::HeadingAtxSequence);
            tokenizer.go(
                space_or_tab_with_options(SpaceOrTabOptions {
                    kind: TokenType::HeadingAtxSpaceOrTab,
                    min: 1,
                    max: usize::MAX,
                }),
                at_break,
            )(tokenizer, code)
        }
        _ => (State::Nok, None),
    }
}

/// After something but before something else.
///
/// ```markdown
/// ## |alpha
/// ## alpha| bravo
/// ## alpha |bravo
/// ## alpha bravo|##
/// ## alpha bravo ##|
/// ```
fn at_break(tokenizer: &mut Tokenizer, code: Code) -> StateFnResult {
    match code {
        Code::None | Code::CarriageReturnLineFeed | Code::Char('\n' | '\r') => {
            tokenizer.exit(TokenType::HeadingAtx);
            (State::Ok, Some(vec![code]))
        }
        Code::VirtualSpace | Code::Char('\t' | ' ') => tokenizer.go(
            space_or_tab_with_options(SpaceOrTabOptions {
                kind: TokenType::HeadingAtxSpaceOrTab,
                min: 1,
                max: usize::MAX,
            }),
            at_break,
        )(tokenizer, code),
        Code::Char('#') => {
            tokenizer.enter(TokenType::HeadingAtxSequence);
            further_sequence(tokenizer, code)
        }
        Code::Char(_) => {
            tokenizer.enter(TokenType::HeadingAtxText);
            tokenizer.enter(TokenType::ChunkText);
            data(tokenizer, code)
        }
    }
}

/// In a further sequence (after whitespace).
/// Could be normal “visible” hashes in the heading or a final sequence.
///
/// ```markdown
/// ## alpha #|#
/// ```
fn further_sequence(tokenizer: &mut Tokenizer, code: Code) -> StateFnResult {
    if let Code::Char('#') = code {
        tokenizer.consume(code);
        (State::Fn(Box::new(further_sequence)), None)
    } else {
        tokenizer.exit(TokenType::HeadingAtxSequence);
        at_break(tokenizer, code)
    }
}

/// In text.
///
/// ```markdown
/// ## al|pha
/// ```
fn data(tokenizer: &mut Tokenizer, code: Code) -> StateFnResult {
    match code {
        // Note: `#` for closing sequence must be preceded by whitespace, otherwise it’s just text.
        Code::None | Code::CarriageReturnLineFeed | Code::Char('\t' | '\n' | '\r' | ' ') => {
            tokenizer.exit(TokenType::ChunkText);
            tokenizer.exit(TokenType::HeadingAtxText);
            at_break(tokenizer, code)
        }
        _ => {
            tokenizer.consume(code);
            (State::Fn(Box::new(data)), None)
        }
    }
}
