//! Block quote is a construct that occurs in the [document][] content type.
//!
//! It forms with the following BNF:
//!
//! ```bnf
//! block_quote_start ::= '>' [ space_or_tab ]
//! block_quote_cont ::= '>' [ space_or_tab ]
//! ```
//!
//! Further lines that are not prefixed with `block_quote_cont` cause the block
//! quote to be exited, except when those lines are lazy continuation.
//! Like so many things in markdown, block quotes too, are very complex.
//! See [*§ Phase 1: block structure*][commonmark-block] for more on parsing
//! details.
//!
//! Block quote relates to the `<blockquote>` element in HTML.
//! See [*§ 4.4.4 The `blockquote` element*][html-blockquote] in the HTML spec
//! for more info.
//!
//! ## Tokens
//!
//! *   [`BlockQuote`][Token::BlockQuote]
//! *   [`BlockQuoteMarker`][Token::BlockQuoteMarker]
//! *   [`BlockQuotePrefix`][Token::BlockQuotePrefix]
//! *   [`SpaceOrTab`][Token::SpaceOrTab]
//!
//! ## References
//!
//! *   [`block-quote.js` in `micromark`](https://github.com/micromark/micromark/blob/main/packages/micromark-core-commonmark/dev/lib/block-quote.js)
//! *   [*§ 5.1 Block quotes* in `CommonMark`](https://spec.commonmark.org/0.30/#block-quotes)
//!
//! [document]: crate::content::document
//! [html-blockquote]: https://html.spec.whatwg.org/multipage/grouping-content.html#the-blockquote-element
//! [commonmark-block]: https://spec.commonmark.org/0.30/#phase-1-block-structure

use crate::constant::TAB_SIZE;
use crate::construct::partial_space_or_tab::space_or_tab_min_max;
use crate::token::Token;
use crate::tokenizer::{Code, State, Tokenizer};

/// Start of block quote.
///
/// ```markdown
/// > | > a
///     ^
/// ```
pub fn start(tokenizer: &mut Tokenizer, code: Code) -> State {
    let max = if tokenizer.parse_state.constructs.code_indented {
        TAB_SIZE - 1
    } else {
        usize::MAX
    };
    if tokenizer.parse_state.constructs.block_quote {
        tokenizer.go(space_or_tab_min_max(0, max), before)(tokenizer, code)
    } else {
        State::Nok
    }
}

/// Start of block quote, after whitespace, before `>`.
///
/// ```markdown
/// > | > a
///     ^
/// ```
fn before(tokenizer: &mut Tokenizer, code: Code) -> State {
    match code {
        Code::Char('>') => {
            tokenizer.enter(Token::BlockQuote);
            cont_before(tokenizer, code)
        }
        _ => cont_before(tokenizer, code),
    }
}

/// Start of block quote continuation.
///
/// ```markdown
///   | > a
/// > | > b
///     ^
/// ```
pub fn cont(tokenizer: &mut Tokenizer, code: Code) -> State {
    let max = if tokenizer.parse_state.constructs.code_indented {
        TAB_SIZE - 1
    } else {
        usize::MAX
    };
    tokenizer.go(space_or_tab_min_max(0, max), cont_before)(tokenizer, code)
}

/// After whitespace, before `>`.
///
/// ```markdown
///   | > a
/// > | > b
///     ^
/// ```
fn cont_before(tokenizer: &mut Tokenizer, code: Code) -> State {
    match code {
        Code::Char('>') => {
            tokenizer.enter(Token::BlockQuotePrefix);
            tokenizer.enter(Token::BlockQuoteMarker);
            tokenizer.consume(code);
            tokenizer.exit(Token::BlockQuoteMarker);
            State::Fn(Box::new(cont_after))
        }
        _ => State::Nok,
    }
}

/// After `>`, before optional whitespace.
///
/// ```markdown
/// > | > a
///      ^
/// > | >b
///      ^
/// ```
fn cont_after(tokenizer: &mut Tokenizer, code: Code) -> State {
    match code {
        Code::VirtualSpace | Code::Char('\t' | ' ') => {
            tokenizer.enter(Token::SpaceOrTab);
            tokenizer.consume(code);
            tokenizer.exit(Token::SpaceOrTab);
            tokenizer.exit(Token::BlockQuotePrefix);
            State::Ok(0)
        }
        _ => {
            tokenizer.exit(Token::BlockQuotePrefix);
            State::Ok(if matches!(code, Code::None) { 0 } else { 1 })
        }
    }
}
