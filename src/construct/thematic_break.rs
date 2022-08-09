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
//! Thematic breaks formed with asterisks or dashes can interfere with
//! [list][]s if there is whitespace between them: `* * *` and `- - -`.
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
//! ## Tokens
//!
//! *   [`ThematicBreak`][Token::ThematicBreak]
//! *   [`ThematicBreakSequence`][Token::ThematicBreakSequence]
//!
//! ## References
//!
//! *   [`thematic-break.js` in `micromark`](https://github.com/micromark/micromark/blob/main/packages/micromark-core-commonmark/dev/lib/thematic-break.js)
//! *   [*§ 4.1 Thematic breaks* in `CommonMark`](https://spec.commonmark.org/0.30/#thematic-breaks)
//!
//! [flow]: crate::content::flow
//! [heading_setext]: crate::construct::heading_setext
//! [list]: crate::construct::list
//! [html]: https://html.spec.whatwg.org/multipage/grouping-content.html#the-hr-element

use super::partial_space_or_tab::{space_or_tab, space_or_tab_min_max};
use crate::constant::{TAB_SIZE, THEMATIC_BREAK_MARKER_COUNT_MIN};
use crate::token::Token;
use crate::tokenizer::{State, StateName, Tokenizer};

/// Start of a thematic break.
///
/// ```markdown
/// > | ***
///     ^
/// ```
pub fn start(tokenizer: &mut Tokenizer) -> State {
    if tokenizer.parse_state.constructs.thematic_break {
        tokenizer.enter(Token::ThematicBreak);
        let state_name = space_or_tab_min_max(
            tokenizer,
            0,
            if tokenizer.parse_state.constructs.code_indented {
                TAB_SIZE - 1
            } else {
                usize::MAX
            },
        );

        tokenizer.go(state_name, StateName::ThematicBreakBefore)
    } else {
        State::Nok
    }
}

/// Start of a thematic break, after whitespace.
///
/// ```markdown
/// > | ***
///     ^
/// ```
pub fn before(tokenizer: &mut Tokenizer) -> State {
    match tokenizer.current {
        Some(b'*' | b'-' | b'_') => {
            tokenizer.tokenize_state.marker = tokenizer.current.unwrap();
            at_break(tokenizer)
        }
        _ => State::Nok,
    }
}

/// After something but before something else.
///
/// ```markdown
/// > | ***
///     ^
/// ```
pub fn at_break(tokenizer: &mut Tokenizer) -> State {
    match tokenizer.current {
        None | Some(b'\n') if tokenizer.tokenize_state.size >= THEMATIC_BREAK_MARKER_COUNT_MIN => {
            tokenizer.tokenize_state.marker = 0;
            tokenizer.tokenize_state.size = 0;
            tokenizer.exit(Token::ThematicBreak);
            // Feel free to interrupt.
            tokenizer.interrupt = false;
            State::Ok
        }
        Some(b'*' | b'-' | b'_')
            if tokenizer.current.unwrap() == tokenizer.tokenize_state.marker =>
        {
            tokenizer.enter(Token::ThematicBreakSequence);
            sequence(tokenizer)
        }
        _ => {
            tokenizer.tokenize_state.marker = 0;
            tokenizer.tokenize_state.size = 0;
            State::Nok
        }
    }
}

/// In a sequence of markers.
///
/// ```markdown
/// > | ***
///     ^
/// ```
pub fn sequence(tokenizer: &mut Tokenizer) -> State {
    match tokenizer.current {
        Some(b'*' | b'-' | b'_')
            if tokenizer.current.unwrap() == tokenizer.tokenize_state.marker =>
        {
            tokenizer.consume();
            tokenizer.tokenize_state.size += 1;
            State::Fn(StateName::ThematicBreakSequence)
        }
        _ => {
            tokenizer.exit(Token::ThematicBreakSequence);
            let state_name = space_or_tab(tokenizer);
            tokenizer.attempt_opt(state_name, StateName::ThematicBreakAtBreak)
        }
    }
}
