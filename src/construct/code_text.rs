//! Code (text) is a construct that occurs in the [text][] content type.
//!
//! It forms with the following BNF:
//!
//! ```bnf
//! ; Restriction: the number of markers in the closing sequence must be equal
//! ; to the number of markers in the opening sequence.
//! code_text ::= sequence 1*code sequence
//!
//! sequence ::= 1*'`'
//! ```
//!
//! The above grammar shows that it is not possible to create empty code.
//! It is possible to include grave accents (ticks) in code, by wrapping it
//! in bigger or smaller sequences:
//!
//! ```markdown
//! Include more: `a``b` or include less: ``a`b``.
//! ```
//!
//! When turning markdown into HTML, each line ending is turned into a space.
//!
//! It is also possible to include just one grave accent (tick):
//!
//! ```markdown
//! Include just one: `` ` ``.
//! ```
//!
//! Sequences are “gready”, in that they cannot be preceded or succeeded by
//! more grave accents (ticks).
//! To illustrate:
//!
//! ```markdown
//! Not code: ``x`.
//!
//! Not code: `x``.
//!
//! Escapes work, this is code: \``x`.
//!
//! Escapes work, this is code: `x`\`.
//! ```
//!
//! Yields:
//!
//! ```html
//! <p>Not code: ``x`.</p>
//! <p>Not code: `x``.</p>
//! <p>Escapes work, this is code: `<code>x</code>.</p>
//! <p>Escapes work, this is code: <code>x</code>`.</p>
//! ```
//!
//! That is because, when turning markdown into HTML, the first and last space,
//! if both exist and there is also a non-space in the code, are removed.
//! Line endings, at that stage, are considered as spaces.
//!
//! Code (text) relates to the `<code>` element in HTML.
//! See [*§ 4.5.15 The `code` element*][html-code] in the HTML spec for more
//! info.
//!
//! In markdown, it is possible to create code with the
//! [code (fenced)][code_fenced] or [code (indented)][code_indented] constructs
//! in the [flow][] content type.
//! Compared to code (indented), fenced code is more explicit and more similar
//! to code (text), and it has support for specifying the programming language
//! that the code is in, so it is recommended to use that instead of indented
//! code.
//!
//! ## References
//!
//! *   [`code-text.js` in `micromark`](https://github.com/micromark/micromark/blob/main/packages/micromark-core-commonmark/dev/lib/code-text.js)
//! *   [*§ 6.1 Code spans* in `CommonMark`](https://spec.commonmark.org/0.30/#code-spans)
//!
//! [flow]: crate::content::flow
//! [text]: crate::content::text
//! [code_indented]: crate::construct::code_indented
//! [code_fenced]: crate::construct::code_fenced
//! [html-code]: https://html.spec.whatwg.org/multipage/text-level-semantics.html#the-code-element

use crate::tokenizer::{Code, State, StateFnResult, TokenType, Tokenizer};

/// Start of code (text).
///
/// ```markdown
/// |`a`
///
/// |\``a`
///
/// |``a`
/// ```
pub fn start(tokenizer: &mut Tokenizer, code: Code) -> StateFnResult {
    let len = tokenizer.events.len();

    match code {
        Code::Char('`')
            if tokenizer.previous != Code::Char('`')
                || (len > 0
                    && tokenizer.events[len - 1].token_type == TokenType::CharacterEscape) =>
        {
            tokenizer.enter(TokenType::CodeText);
            tokenizer.enter(TokenType::CodeTextSequence);
            sequence_open(tokenizer, code, 0)
        }
        _ => (State::Nok, None),
    }
}

/// In the opening sequence.
///
/// ```markdown
/// `|`a``
/// ```
fn sequence_open(tokenizer: &mut Tokenizer, code: Code, size: usize) -> StateFnResult {
    if let Code::Char('`') = code {
        tokenizer.consume(code);
        (
            State::Fn(Box::new(move |tokenizer, code| {
                sequence_open(tokenizer, code, size + 1)
            })),
            None,
        )
    } else {
        tokenizer.exit(TokenType::CodeTextSequence);
        between(tokenizer, code, size)
    }
}

/// Between something and something else
///
/// ```markdown
/// `|a`
/// `a|`
/// ```
fn between(tokenizer: &mut Tokenizer, code: Code, size_open: usize) -> StateFnResult {
    match code {
        Code::None => (State::Nok, None),
        Code::CarriageReturnLineFeed | Code::Char('\r' | '\n') => {
            tokenizer.enter(TokenType::CodeTextLineEnding);
            tokenizer.consume(code);
            tokenizer.exit(TokenType::CodeTextLineEnding);
            (
                State::Fn(Box::new(move |tokenizer, code| {
                    between(tokenizer, code, size_open)
                })),
                None,
            )
        }
        Code::Char('`') => {
            tokenizer.enter(TokenType::CodeTextSequence);
            sequence_close(tokenizer, code, size_open, 0)
        }
        _ => {
            tokenizer.enter(TokenType::CodeTextData);
            data(tokenizer, code, size_open)
        }
    }
}

/// In data.
///
/// ```markdown
/// `a|b`
/// ```
fn data(tokenizer: &mut Tokenizer, code: Code, size_open: usize) -> StateFnResult {
    match code {
        Code::None | Code::CarriageReturnLineFeed | Code::Char('\r' | '\n' | '`') => {
            tokenizer.exit(TokenType::CodeTextData);
            between(tokenizer, code, size_open)
        }
        _ => {
            tokenizer.consume(code);
            (
                State::Fn(Box::new(move |tokenizer, code| {
                    data(tokenizer, code, size_open)
                })),
                None,
            )
        }
    }
}

/// In the closing sequence.
///
/// ```markdown
/// ``a`|`
/// ```
fn sequence_close(
    tokenizer: &mut Tokenizer,
    code: Code,
    size_open: usize,
    size: usize,
) -> StateFnResult {
    match code {
        Code::Char('`') => {
            tokenizer.consume(code);
            (
                State::Fn(Box::new(move |tokenizer, code| {
                    sequence_close(tokenizer, code, size_open, size + 1)
                })),
                None,
            )
        }
        _ if size_open == size => {
            tokenizer.exit(TokenType::CodeTextSequence);
            tokenizer.exit(TokenType::CodeText);
            (State::Ok, Some(vec![code]))
        }
        _ => {
            let tail_index = tokenizer.events.len();
            let head_index = tokenizer.events.len() - 1;
            tokenizer.exit(TokenType::CodeTextSequence);
            // Change the token type.
            tokenizer.events[head_index].token_type = TokenType::CodeTextData;
            tokenizer.events[tail_index].token_type = TokenType::CodeTextData;
            between(tokenizer, code, size_open)
        }
    }
}
