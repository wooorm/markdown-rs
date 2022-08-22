//! Code (text) occurs in the [text][] content type.
//!
//! ## Grammar
//!
//! Code (text) forms with the following BNF
//! (<small>see [construct][crate::construct] for character groups</small>):
//!
//! ```bnf
//! ; Restriction: the number of markers in the closing sequence must be equal
//! ; to the number of markers in the opening sequence.
//! code_text ::= sequence 1*byte sequence
//!
//! ; Restriction: not preceded or followed by `` ` ``.
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
//! It is also possible to include just one grave accent (tick):
//!
//! ```markdown
//! Include just one: `` ` ``.
//! ```
//!
//! Sequences are “gready”, in that they cannot be preceded or followed by
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
//! In markdown, it is possible to create code with the
//! [code (fenced)][code_fenced] or [code (indented)][code_indented] constructs
//! in the [flow][] content type.
//!
//! ## HTML
//!
//! Code (text) relates to the `<code>` element in HTML.
//! See [*§ 4.5.15 The `code` element*][html_code] in the HTML spec for more
//! info.
//!
//! When turning markdown into HTML, each line ending is turned into a space.
//!
//! ## Tokens
//!
//! *   [`CodeText`][Name::CodeText]
//! *   [`CodeTextData`][Name::CodeTextData]
//! *   [`CodeTextSequence`][Name::CodeTextSequence]
//! *   [`LineEnding`][Name::LineEnding]
//!
//! ## References
//!
//! *   [`code-text.js` in `micromark`](https://github.com/micromark/micromark/blob/main/packages/micromark-core-commonmark/dev/lib/code-text.js)
//! *   [*§ 6.1 Code spans* in `CommonMark`](https://spec.commonmark.org/0.30/#code-spans)
//!
//! [flow]: crate::construct::flow
//! [text]: crate::construct::text
//! [code_indented]: crate::construct::code_indented
//! [code_fenced]: crate::construct::code_fenced
//! [html_code]: https://html.spec.whatwg.org/multipage/text-level-semantics.html#the-code-element

use crate::event::Name;
use crate::state::{Name as StateName, State};
use crate::tokenizer::Tokenizer;

/// Start of code (text).
///
/// ```markdown
/// > | `a`
///     ^
/// > | \`a`
///      ^
/// ```
pub fn start(tokenizer: &mut Tokenizer) -> State {
    match tokenizer.current {
        Some(b'`')
            if tokenizer.parse_state.options.constructs.code_text
                && (tokenizer.previous != Some(b'`')
                    || (!tokenizer.events.is_empty()
                        && tokenizer.events[tokenizer.events.len() - 1].name
                            == Name::CharacterEscape)) =>
        {
            tokenizer.enter(Name::CodeText);
            tokenizer.enter(Name::CodeTextSequence);
            State::Retry(StateName::CodeTextSequenceOpen)
        }
        _ => State::Nok,
    }
}

/// In opening sequence.
///
/// ```markdown
/// > | `a`
///     ^
/// ```
pub fn sequence_open(tokenizer: &mut Tokenizer) -> State {
    if let Some(b'`') = tokenizer.current {
        tokenizer.tokenize_state.size += 1;
        tokenizer.consume();
        State::Next(StateName::CodeTextSequenceOpen)
    } else {
        tokenizer.exit(Name::CodeTextSequence);
        State::Retry(StateName::CodeTextBetween)
    }
}

/// Between something and something else
///
/// ```markdown
/// > | `a`
///      ^^
/// ```
pub fn between(tokenizer: &mut Tokenizer) -> State {
    match tokenizer.current {
        None => {
            tokenizer.tokenize_state.size = 0;
            State::Nok
        }
        Some(b'\n') => {
            tokenizer.enter(Name::LineEnding);
            tokenizer.consume();
            tokenizer.exit(Name::LineEnding);
            State::Next(StateName::CodeTextBetween)
        }
        Some(b'`') => {
            tokenizer.enter(Name::CodeTextSequence);
            State::Retry(StateName::CodeTextSequenceClose)
        }
        _ => {
            tokenizer.enter(Name::CodeTextData);
            State::Retry(StateName::CodeTextData)
        }
    }
}

/// In data.
///
/// ```markdown
/// > | `a`
///      ^
/// ```
pub fn data(tokenizer: &mut Tokenizer) -> State {
    match tokenizer.current {
        None | Some(b'\n' | b'`') => {
            tokenizer.exit(Name::CodeTextData);
            State::Retry(StateName::CodeTextBetween)
        }
        _ => {
            tokenizer.consume();
            State::Next(StateName::CodeTextData)
        }
    }
}

/// In closing sequence.
///
/// ```markdown
/// > | `a`
///       ^
/// ```
pub fn sequence_close(tokenizer: &mut Tokenizer) -> State {
    match tokenizer.current {
        Some(b'`') => {
            tokenizer.tokenize_state.size_b += 1;
            tokenizer.consume();
            State::Next(StateName::CodeTextSequenceClose)
        }
        _ => {
            if tokenizer.tokenize_state.size == tokenizer.tokenize_state.size_b {
                tokenizer.exit(Name::CodeTextSequence);
                tokenizer.exit(Name::CodeText);
                tokenizer.tokenize_state.size = 0;
                tokenizer.tokenize_state.size_b = 0;
                State::Ok
            } else {
                let index = tokenizer.events.len();
                tokenizer.exit(Name::CodeTextSequence);
                // More or less accents: mark as data.
                tokenizer.events[index - 1].name = Name::CodeTextData;
                tokenizer.events[index].name = Name::CodeTextData;
                tokenizer.tokenize_state.size_b = 0;
                State::Retry(StateName::CodeTextBetween)
            }
        }
    }
}
