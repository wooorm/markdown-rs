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
//! *   [`CodeIndented`][Name::CodeIndented]
//! *   [`CodeFlowChunk`][Name::CodeFlowChunk]
//! *   [`LineEnding`][Name::LineEnding]
//! *   [`SpaceOrTab`][Name::SpaceOrTab]
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
use crate::event::Name;
use crate::state::{Name as StateName, State};
use crate::tokenizer::Tokenizer;

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
pub fn start(tokenizer: &mut Tokenizer) -> State {
    // Do not interrupt paragraphs.
    if !tokenizer.interrupt && tokenizer.parse_state.constructs.code_indented {
        tokenizer.enter(Name::CodeIndented);
        tokenizer.attempt(State::Next(StateName::CodeIndentedAtBreak), State::Nok);
        State::Retry(space_or_tab_min_max(tokenizer, TAB_SIZE, TAB_SIZE))
    } else {
        State::Nok
    }
}

/// At a break.
///
/// ```markdown
/// > |     aaa
///         ^  ^
/// ```
pub fn at_break(tokenizer: &mut Tokenizer) -> State {
    match tokenizer.current {
        None => State::Retry(StateName::CodeIndentedAfter),
        Some(b'\n') => {
            tokenizer.attempt(
                State::Next(StateName::CodeIndentedAtBreak),
                State::Next(StateName::CodeIndentedAfter),
            );
            State::Retry(StateName::CodeIndentedFurtherStart)
        }
        _ => {
            tokenizer.enter(Name::CodeFlowChunk);
            State::Retry(StateName::CodeIndentedInside)
        }
    }
}

/// In code content.
///
/// ```markdown
/// > |     aaa
///         ^^^^
/// ```
pub fn inside(tokenizer: &mut Tokenizer) -> State {
    match tokenizer.current {
        None | Some(b'\n') => {
            tokenizer.exit(Name::CodeFlowChunk);
            State::Retry(StateName::CodeIndentedAtBreak)
        }
        _ => {
            tokenizer.consume();
            State::Next(StateName::CodeIndentedInside)
        }
    }
}

/// After indented code.
///
/// ```markdown
/// > |     aaa
///            ^
/// ```
pub fn after(tokenizer: &mut Tokenizer) -> State {
    tokenizer.exit(Name::CodeIndented);
    // Feel free to interrupt.
    tokenizer.interrupt = false;
    State::Ok
}

/// At eol, trying to parse another indent.
///
/// ```markdown
/// > |     aaa
///            ^
///   |     bbb
/// ```
pub fn further_start(tokenizer: &mut Tokenizer) -> State {
    match tokenizer.current {
        Some(b'\n') if !tokenizer.lazy => {
            tokenizer.enter(Name::LineEnding);
            tokenizer.consume();
            tokenizer.exit(Name::LineEnding);
            State::Next(StateName::CodeIndentedFurtherStart)
        }
        _ if !tokenizer.lazy => {
            tokenizer.attempt(
                State::Next(StateName::CodeIndentedFurtherEnd),
                State::Next(StateName::CodeIndentedFurtherBegin),
            );
            State::Retry(space_or_tab_min_max(tokenizer, TAB_SIZE, TAB_SIZE))
        }
        _ => State::Nok,
    }
}

/// At eol, followed by an indented line.
///
/// ```markdown
/// >  |     aaa
///             ^
///    |     bbb
/// ```
pub fn further_end(_tokenizer: &mut Tokenizer) -> State {
    State::Ok
}

/// At the beginning of a line that is not indented enough.
///
/// ```markdown
///   |     aaa
/// > |   bbb
///     ^
/// ```
pub fn further_begin(tokenizer: &mut Tokenizer) -> State {
    tokenizer.attempt(
        State::Next(StateName::CodeIndentedFurtherAfter),
        State::Next(StateName::CodeIndentedFurtherAfter),
    );
    State::Retry(space_or_tab(tokenizer))
}

/// After whitespace, not indented enough.
///
/// ```markdown
///   |     aaa
/// > |   bbb
///       ^
/// ```
pub fn further_after(tokenizer: &mut Tokenizer) -> State {
    match tokenizer.current {
        Some(b'\n') => State::Retry(StateName::CodeIndentedFurtherStart),
        _ => State::Nok,
    }
}
