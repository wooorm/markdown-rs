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
//! *   [`BlockQuote`][TokenType::BlockQuote]
//! *   [`BlockQuoteMarker`][TokenType::BlockQuoteMarker]
//! *   [`BlockQuotePrefix`][TokenType::BlockQuotePrefix]
//! *   [`BlockQuoteWhitespace`][TokenType::BlockQuoteWhitespace]
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
use crate::tokenizer::{Code, State, StateFnResult, TokenType, Tokenizer};

/// Start of block quote.
///
/// ```markdown
/// | > a
/// ```
pub fn start(tokenizer: &mut Tokenizer, code: Code) -> StateFnResult {
    // To do: allow arbitrary when code (indented) is turned off.
    tokenizer.go(space_or_tab_min_max(0, TAB_SIZE - 1), before)(tokenizer, code)
}

/// Start of block quote, after whitespace, before `>`.
///
/// ```markdown
/// |> a
/// ```
fn before(tokenizer: &mut Tokenizer, code: Code) -> StateFnResult {
    match code {
        Code::Char('>') => {
            tokenizer.enter(TokenType::BlockQuote);
            cont_before(tokenizer, code)
        }
        _ => cont_before(tokenizer, code),
    }
}

/// Start of block quote continuation.
///
/// ```markdown
/// > a
/// |> b
/// ```
pub fn cont(tokenizer: &mut Tokenizer, code: Code) -> StateFnResult {
    // To do: allow arbitrary when code (indented) is turned off.
    tokenizer.go(space_or_tab_min_max(0, TAB_SIZE - 1), cont_before)(tokenizer, code)
}

/// After whitespace, before `>`.
///
/// ```markdown
/// > a
/// |> b
/// ```
fn cont_before(tokenizer: &mut Tokenizer, code: Code) -> StateFnResult {
    match code {
        Code::Char('>') => {
            tokenizer.enter(TokenType::BlockQuotePrefix);
            tokenizer.enter(TokenType::BlockQuoteMarker);
            tokenizer.consume(code);
            tokenizer.exit(TokenType::BlockQuoteMarker);
            (State::Fn(Box::new(cont_after)), None)
        }
        _ => (State::Nok, None),
    }
}

/// After `>`, before optional whitespace.
///
/// ```markdown
/// >| a
/// >|b
/// ```
fn cont_after(tokenizer: &mut Tokenizer, code: Code) -> StateFnResult {
    match code {
        Code::VirtualSpace | Code::Char('\t' | ' ') => {
            tokenizer.enter(TokenType::BlockQuoteWhitespace);
            tokenizer.consume(code);
            tokenizer.exit(TokenType::BlockQuoteWhitespace);
            tokenizer.exit(TokenType::BlockQuotePrefix);
            (State::Ok, None)
        }
        _ => {
            tokenizer.exit(TokenType::BlockQuotePrefix);
            (State::Ok, Some(vec![code]))
        }
    }
}

/// End of a block quote.
pub fn end() -> Vec<TokenType> {
    vec![TokenType::BlockQuote]
}
