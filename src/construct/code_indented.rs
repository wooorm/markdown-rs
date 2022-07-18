//! Code (indented) is a construct that occurs in the [flow][] content type.
//!
//! It forms with the following BNF:
//!
//! ```bnf
//! code_indented ::= indented_filled_line *( eol *( blank_line eol ) indented_filled_line )
//!
//! ; Restriction: at least one `code` must not be whitespace.
//! indented_filled_line ::= 4space_or_tab *code
//! blank_line ::= *space_or_tab
//! eol ::= '\r' | '\r\n' | '\n'
//! code ::= . ; any unicode code point (other than line endings).
//! space_or_tab ::= ' ' | '\t'
//! ```
//!
//! Code (indented) relates to both the `<pre>` and the `<code>` elements in
//! HTML.
//! See [*§ 4.4.3 The `pre` element*][html-pre] and the [*§ 4.5.15 The `code`
//! element*][html-code] in the HTML spec for more info.
//!
//! In markdown, it is also possible to use [code (text)][code_text] in the
//! [text][] content type.
//! It is also possible to create code with the [code (fenced)][code_fenced]
//! construct.
//! That construct is more explicit, more similar to code (text), and has
//! support for specifying the programming language that the code is in, so it
//! is recommended to use that instead of indented code.
//!
//! ## Tokens
//!
//! *   [`CodeIndented`][Token::CodeIndented]
//! *   [`CodeFlowChunk`][Token::CodeFlowChunk]
//! *   [`LineEnding`][Token::LineEnding]
//! *   [`SpaceOrTab`][Token::SpaceOrTab]
//!
//! ## References
//!
//! *   [`code-indented.js` in `micromark`](https://github.com/micromark/micromark/blob/main/packages/micromark-core-commonmark/dev/lib/code-indented.js)
//! *   [*§ 4.4 Indented code blocks* in `CommonMark`](https://spec.commonmark.org/0.30/#indented-code-blocks)
//!
//! [flow]: crate::content::flow
//! [text]: crate::content::text
//! [code_text]: crate::construct::code_text
//! [code_fenced]: crate::construct::code_fenced
//! [html-pre]: https://html.spec.whatwg.org/multipage/grouping-content.html#the-pre-element
//! [html-code]: https://html.spec.whatwg.org/multipage/text-level-semantics.html#the-code-element

use super::partial_space_or_tab::{space_or_tab, space_or_tab_min_max};
use crate::constant::TAB_SIZE;
use crate::token::Token;
use crate::tokenizer::{Code, State, StateFnResult, Tokenizer};

/// Start of code (indented).
///
/// > **Parsing note**: it is not needed to check if this first line is a
/// > filled line (that it has a non-whitespace character), because blank lines
/// > are parsed already, so we never run into that.
///
/// ```markdown
/// > |     aaa
///     ^
/// ```
pub fn start(tokenizer: &mut Tokenizer, code: Code) -> StateFnResult {
    // Do not interrupt paragraphs.
    if tokenizer.interrupt || !tokenizer.parse_state.constructs.code_indented {
        (State::Nok, None)
    } else {
        tokenizer.enter(Token::CodeIndented);
        tokenizer.go(space_or_tab_min_max(TAB_SIZE, TAB_SIZE), at_break)(tokenizer, code)
    }
}

/// At a break.
///
/// ```markdown
/// > |     aaa
///         ^  ^
/// ```
fn at_break(tokenizer: &mut Tokenizer, code: Code) -> StateFnResult {
    match code {
        Code::None => after(tokenizer, code),
        Code::CarriageReturnLineFeed | Code::Char('\n' | '\r') => tokenizer
            .attempt(further_start, |ok| {
                Box::new(if ok { at_break } else { after })
            })(tokenizer, code),
        _ => {
            tokenizer.enter(Token::CodeFlowChunk);
            content(tokenizer, code)
        }
    }
}

/// Inside code content.
///
/// ```markdown
/// > |     aaa
///         ^^^^
/// ```
fn content(tokenizer: &mut Tokenizer, code: Code) -> StateFnResult {
    match code {
        Code::None | Code::CarriageReturnLineFeed | Code::Char('\n' | '\r') => {
            tokenizer.exit(Token::CodeFlowChunk);
            at_break(tokenizer, code)
        }
        _ => {
            tokenizer.consume(code);
            (State::Fn(Box::new(content)), None)
        }
    }
}

/// After indented code.
///
/// ```markdown
/// > |     aaa
///            ^
/// ```
fn after(tokenizer: &mut Tokenizer, code: Code) -> StateFnResult {
    tokenizer.exit(Token::CodeIndented);
    // Feel free to interrupt.
    tokenizer.interrupt = false;
    (State::Ok, Some(vec![code]))
}

/// Right at a line ending, trying to parse another indent.
///
/// ```markdown
/// > |     aaa
///            ^
///   |     bbb
/// ```
fn further_start(tokenizer: &mut Tokenizer, code: Code) -> StateFnResult {
    if tokenizer.lazy {
        (State::Nok, None)
    } else {
        match code {
            Code::CarriageReturnLineFeed | Code::Char('\n' | '\r') => {
                tokenizer.enter(Token::LineEnding);
                tokenizer.consume(code);
                tokenizer.exit(Token::LineEnding);
                (State::Fn(Box::new(further_start)), None)
            }
            _ => tokenizer.attempt(space_or_tab_min_max(TAB_SIZE, TAB_SIZE), |ok| {
                Box::new(if ok { further_end } else { further_begin })
            })(tokenizer, code),
        }
    }
}

/// After a proper indent.
///
/// ```markdown
///   |     aaa
/// > |     bbb
///         ^
/// ```
fn further_end(_tokenizer: &mut Tokenizer, code: Code) -> StateFnResult {
    (State::Ok, Some(vec![code]))
}

/// At the beginning of a line that is not indented enough.
///
/// ```markdown
///   |     aaa
/// > |   bbb
///     ^
/// ```
fn further_begin(tokenizer: &mut Tokenizer, code: Code) -> StateFnResult {
    tokenizer.attempt_opt(space_or_tab(), further_after)(tokenizer, code)
}

/// After whitespace, not indented enough.
///
/// ```markdown
///   |     aaa
/// > |   bbb
///       ^
/// ```
fn further_after(tokenizer: &mut Tokenizer, code: Code) -> StateFnResult {
    match code {
        Code::CarriageReturnLineFeed | Code::Char('\n' | '\r') => further_start(tokenizer, code),
        _ => (State::Nok, None),
    }
}
