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
//! *   [`CodeIndented`][TokenType::CodeIndented]
//! *   [`CodeFlowChunk`][TokenType::CodeFlowChunk]
//! *   [`LineEnding`][TokenType::LineEnding]
//! *   [`SpaceOrTab`][TokenType::SpaceOrTab]
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
use crate::tokenizer::{Code, State, StateFnResult, TokenType, Tokenizer};

/// Start of code (indented).
///
/// ```markdown
/// |    asd
/// ```
///
/// > **Parsing note**: it is not needed to check if this first line is a
/// > filled line (that it has a non-whitespace character), because blank lines
/// > are parsed already, so we never run into that.
pub fn start(tokenizer: &mut Tokenizer, code: Code) -> StateFnResult {
    tokenizer.enter(TokenType::CodeIndented);
    tokenizer.go(space_or_tab_min_max(TAB_SIZE, TAB_SIZE), at_break)(tokenizer, code)
}

/// At a break.
///
/// ```markdown
///     |asd
///     asd|
/// ```
fn at_break(tokenizer: &mut Tokenizer, code: Code) -> StateFnResult {
    match code {
        Code::None => after(tokenizer, code),
        Code::CarriageReturnLineFeed | Code::Char('\n' | '\r') => tokenizer
            .attempt(further_start, |ok| {
                Box::new(if ok { at_break } else { after })
            })(tokenizer, code),
        _ => {
            tokenizer.enter(TokenType::CodeFlowChunk);
            content(tokenizer, code)
        }
    }
}

/// Inside code content.
///
/// ```markdown
///     |ab
///     a|b
///     ab|
/// ```
fn content(tokenizer: &mut Tokenizer, code: Code) -> StateFnResult {
    match code {
        Code::None | Code::CarriageReturnLineFeed | Code::Char('\n' | '\r') => {
            tokenizer.exit(TokenType::CodeFlowChunk);
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
///     ab|
/// ```
fn after(tokenizer: &mut Tokenizer, code: Code) -> StateFnResult {
    tokenizer.exit(TokenType::CodeIndented);
    (State::Ok, Some(vec![code]))
}

/// Right at a line ending, trying to parse another indent.
///
/// ```markdown
///     ab|
///     cd
/// ```
fn further_start(tokenizer: &mut Tokenizer, code: Code) -> StateFnResult {
    // To do: `nok` if lazy line.
    match code {
        Code::CarriageReturnLineFeed | Code::Char('\n' | '\r') => {
            tokenizer.enter(TokenType::LineEnding);
            tokenizer.consume(code);
            tokenizer.exit(TokenType::LineEnding);
            (State::Fn(Box::new(further_start)), None)
        }
        _ => tokenizer.attempt(space_or_tab_min_max(TAB_SIZE, TAB_SIZE), |ok| {
            Box::new(if ok { further_end } else { further_begin })
        })(tokenizer, code),
    }
}

/// After a proper indent.
///
/// ```markdown
///     asd
///     |asd
/// ```
fn further_end(_tokenizer: &mut Tokenizer, code: Code) -> StateFnResult {
    (State::Ok, Some(vec![code]))
}

/// At the beginning of a line that is not indented enough.
///
/// > 👉 **Note**: `␠` represents a space character.
///
/// ```markdown
///     asd
/// |␠␠
///     asd
/// ```
fn further_begin(tokenizer: &mut Tokenizer, code: Code) -> StateFnResult {
    tokenizer.attempt_opt(space_or_tab(), further_after)(tokenizer, code)
}

/// After whitespace.
///
/// ```markdown
///     asd
/// ␠␠|
///     asd
/// ```
fn further_after(tokenizer: &mut Tokenizer, code: Code) -> StateFnResult {
    match code {
        Code::CarriageReturnLineFeed | Code::Char('\n' | '\r') => further_start(tokenizer, code),
        _ => (State::Nok, None),
    }
}
