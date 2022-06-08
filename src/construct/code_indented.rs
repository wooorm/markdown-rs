//! Code (indented) is a construct that occurs in the flow content type.
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
//! In markdown, it is also possible to use code (text) in the text content
//! type.
//! It is also possible to create code with the [code (fenced)][code-fenced]
//! construct.
//! That construct is more explicit, more similar to code (text), and has
//! support for specifying the programming language that the code is in, so it
//! is recommended to use that instead of indented code.
//!
//! ## References
//!
//! *   [`code-indented.js` in `micromark`](https://github.com/micromark/micromark/blob/main/packages/micromark-core-commonmark/dev/lib/code-indented.js)
//! *   [*§ 4.4 Indented code blocks* in `CommonMark`](https://spec.commonmark.org/0.30/#indented-code-blocks)
//!
//! [code-fenced]: crate::construct::code_fenced
//! [html-pre]: https://html.spec.whatwg.org/multipage/grouping-content.html#the-pre-element
//! [html-code]: https://html.spec.whatwg.org/multipage/text-level-semantics.html#the-code-element
//!
//! <!-- To do: link `flow`, `code_text` -->

use crate::constant::TAB_SIZE;
use crate::tokenizer::{Code, State, StateFnResult, TokenType, Tokenizer};

/// Start of code (indented).
///
/// ```markdown
/// |    asd
/// ```
pub fn start(tokenizer: &mut Tokenizer, code: Code) -> StateFnResult {
    match code {
        Code::VirtualSpace | Code::Char(' ' | '\t') => {
            tokenizer.enter(TokenType::CodeIndented);
            tokenizer.enter(TokenType::CodeIndentedPrefixWhitespace);
            indent(tokenizer, code, 0)
        }
        _ => (State::Nok, None),
    }
}

/// Inside the initial whitespace.
///
/// ```markdown
///  |   asd
///   |  asd
///    | asd
///     |asd
/// ```
///
/// > **Parsing note**: it is not needed to check if this first line is a
/// > filled line (that it has a non-whitespace character), because blank lines
/// > are parsed already, so we never run into that.
fn indent(tokenizer: &mut Tokenizer, code: Code, size: usize) -> StateFnResult {
    match code {
        _ if size == TAB_SIZE => {
            tokenizer.exit(TokenType::CodeIndentedPrefixWhitespace);
            at_break(tokenizer, code)
        }
        Code::VirtualSpace | Code::Char(' ' | '\t') => {
            tokenizer.consume(code);
            (
                State::Fn(Box::new(move |tokenizer, code| {
                    indent(tokenizer, code, size + 1)
                })),
                None,
            )
        }
        _ => (State::Nok, None),
    }
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
        Code::VirtualSpace | Code::Char(' ' | '\t') => {
            tokenizer.enter(TokenType::CodeIndentedPrefixWhitespace);
            further_indent(tokenizer, code, 0)
        }
        _ => (State::Nok, None),
    }
}

/// Inside further whitespace.
///
/// ```markdown
///     asd
///   |  asd
/// ```
fn further_indent(tokenizer: &mut Tokenizer, code: Code, size: usize) -> StateFnResult {
    match code {
        _ if size == TAB_SIZE => {
            tokenizer.exit(TokenType::CodeIndentedPrefixWhitespace);
            (State::Ok, Some(vec![code]))
        }
        Code::VirtualSpace | Code::Char(' ' | '\t') => {
            tokenizer.consume(code);
            (
                State::Fn(Box::new(move |tokenizer, code| {
                    further_indent(tokenizer, code, size + 1)
                })),
                None,
            )
        }
        Code::CarriageReturnLineFeed | Code::Char('\n' | '\r') => {
            tokenizer.exit(TokenType::CodeIndentedPrefixWhitespace);
            further_start(tokenizer, code)
        }
        _ => (State::Nok, None),
    }
}
