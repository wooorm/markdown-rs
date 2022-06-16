//! Thematic breaks, sometimes called horizontal rules, are a construct that
//! occurs in the [flow][] content type.
//!
//! They’re formed with the following BNF:
//!
//! ```bnf
//! ; Restriction: all markers must be identical.
//! ; Restriction: at least 3 markers must be used.
//! thematic_break ::= *space_or_tab 1*(1*marker *space_or_tab)
//!
//! space_or_tab ::= ' ' | '\t'
//! marker ::= '*' | '-' | '_'
//! ```
//!
//! Thematic breaks in markdown typically relate to the HTML element `<hr>`.
//! See [*§ 4.4.2 The `hr` element* in the HTML spec][html] for more info.
//!
//! It is recommended to use exactly three asterisks without whitespace when
//! writing markdown.
//! As using more than three markers has no effect other than wasting space,
//! it is recommended to use exactly three markers.
//! Thematic breaks formed with asterisks or dashes can interfere with lists
//! in if there is whitespace between them: `* * *` and `- - -`.
//! For these reasons, it is recommend to not use spaces or tabs between the
//! markers.
//! Thematic breaks formed with dashes (without whitespace) can also form
//! [heading (setext)][heading_setext].
//! As dashes and underscores frequently occur in natural language and URLs, it
//! is recommended to use asterisks for thematic breaks to distinguish from
//! such use.
//! Because asterisks can be used to form the most markdown constructs, using
//! them has the added benefit of making it easier to gloss over markdown: you
//! can look for asterisks to find syntax while not worrying about other
//! characters.
//!
//! ## References
//!
//! *   [`thematic-break.js` in `micromark`](https://github.com/micromark/micromark/blob/main/packages/micromark-core-commonmark/dev/lib/thematic-break.js)
//! *   [*§ 4.1 Thematic breaks* in `CommonMark`](https://spec.commonmark.org/0.30/#thematic-breaks)
//!
//! [flow]: crate::content::flow
//! [heading_setext]: crate::construct::heading_setext
//! [html]: https://html.spec.whatwg.org/multipage/grouping-content.html#the-hr-element
//!
//! <!-- To do: link `lists` -->

use crate::constant::THEMATIC_BREAK_MARKER_COUNT_MIN;
use crate::tokenizer::{Code, State, StateFnResult, TokenType, Tokenizer};

/// Start of a thematic break.
///
/// ```markdown
/// |***
/// ```
pub fn start(tokenizer: &mut Tokenizer, code: Code) -> StateFnResult {
    match code {
        Code::Char(char) if char == '*' || char == '-' || char == '_' => {
            tokenizer.enter(TokenType::ThematicBreak);
            at_break(tokenizer, code, char, 0)
        }
        _ => (State::Nok, None),
    }
}

/// After something but before something else.
///
/// ```markdown
/// |***
/// *| * *
/// * |* *
/// ```
fn at_break(tokenizer: &mut Tokenizer, code: Code, marker: char, size: usize) -> StateFnResult {
    match code {
        Code::Char(char) if char == marker => {
            tokenizer.enter(TokenType::ThematicBreakSequence);
            sequence(tokenizer, code, marker, size)
        }
        Code::VirtualSpace | Code::Char('\t' | ' ') => {
            tokenizer.enter(TokenType::ThematicBreakWhitespace);
            whitespace(tokenizer, code, marker, size)
        }
        Code::None | Code::CarriageReturnLineFeed | Code::Char('\n' | '\r')
            if size >= THEMATIC_BREAK_MARKER_COUNT_MIN =>
        {
            tokenizer.exit(TokenType::ThematicBreak);
            (State::Ok, Some(vec![code]))
        }
        _ => (State::Nok, None),
    }
}

/// In a sequence of markers.
///
/// ```markdown
/// |***
/// *|**
/// **|*
/// ```
fn sequence(tokenizer: &mut Tokenizer, code: Code, marker: char, size: usize) -> StateFnResult {
    match code {
        Code::Char(char) if char == marker => {
            tokenizer.consume(code);
            (
                State::Fn(Box::new(move |tokenizer, code| {
                    sequence(tokenizer, code, marker, size + 1)
                })),
                None,
            )
        }
        _ => {
            tokenizer.exit(TokenType::ThematicBreakSequence);
            at_break(tokenizer, code, marker, size)
        }
    }
}

/// In whitespace.
///
/// ```markdown
/// * |* *
/// * | * *
/// ```
fn whitespace(tokenizer: &mut Tokenizer, code: Code, marker: char, size: usize) -> StateFnResult {
    match code {
        Code::VirtualSpace | Code::Char('\t' | ' ') => {
            tokenizer.consume(code);
            (
                State::Fn(Box::new(move |tokenizer, code| {
                    whitespace(tokenizer, code, marker, size)
                })),
                None,
            )
        }
        _ => {
            tokenizer.exit(TokenType::ThematicBreakWhitespace);
            at_break(tokenizer, code, marker, size)
        }
    }
}
