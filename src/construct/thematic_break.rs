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
//! ## Tokens
//!
//! *   [`ThematicBreak`][TokenType::ThematicBreak]
//! *   [`ThematicBreakSequence`][TokenType::ThematicBreakSequence]
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

use super::partial_space_or_tab::{space_or_tab, space_or_tab_min_max};
use crate::constant::{TAB_SIZE, THEMATIC_BREAK_MARKER_COUNT_MIN};
use crate::tokenizer::{Code, State, StateFnResult, TokenType, Tokenizer};

/// Type of thematic break.
#[derive(Debug, PartialEq)]
enum Kind {
    /// In a thematic break using asterisks (`*`).
    ///
    /// ## Example
    ///
    /// ```markdown
    /// ***
    /// ```
    Asterisk,
    /// In a thematic break using dashes (`-`).
    ///
    /// ## Example
    ///
    /// ```markdown
    /// ---
    /// ```
    Dash,
    /// In a thematic break using underscores (`_`).
    ///
    /// ## Example
    ///
    /// ```markdown
    /// ___
    /// ```
    Underscore,
}

impl Kind {
    /// Turn the kind into a [char].
    fn as_char(&self) -> char {
        match self {
            Kind::Asterisk => '*',
            Kind::Dash => '-',
            Kind::Underscore => '_',
        }
    }
    /// Turn a [char] into a kind.
    ///
    /// ## Panics
    ///
    /// Panics if `char` is not `*`, `-`, or `_`.
    fn from_char(char: char) -> Kind {
        match char {
            '*' => Kind::Asterisk,
            '-' => Kind::Dash,
            '_' => Kind::Underscore,
            _ => unreachable!("invalid char"),
        }
    }
    /// Turn [Code] into a kind.
    ///
    /// > 👉 **Note**: an opening paren must be used for `Kind::Paren`.
    ///
    /// ## Panics
    ///
    /// Panics if `code` is not `Code::Char('*' | '-' | '_')`.
    fn from_code(code: Code) -> Kind {
        match code {
            Code::Char(char) => Kind::from_char(char),
            _ => unreachable!("invalid code"),
        }
    }
}

/// State needed to parse thematic breaks.
#[derive(Debug)]
struct Info {
    /// Kind of marker.
    kind: Kind,
    /// Number of markers.
    size: usize,
}

/// Start of a thematic break.
///
/// ```markdown
/// |***
/// ```
pub fn start(tokenizer: &mut Tokenizer, code: Code) -> StateFnResult {
    tokenizer.enter(TokenType::ThematicBreak);
    // To do: allow arbitrary when code (indented) is turned off.
    tokenizer.go(space_or_tab_min_max(0, TAB_SIZE - 1), before)(tokenizer, code)
}

/// Start of a thematic break, after whitespace.
///
/// ```markdown
/// |***
/// ```
fn before(tokenizer: &mut Tokenizer, code: Code) -> StateFnResult {
    match code {
        Code::Char('*' | '-' | '_') => at_break(
            tokenizer,
            code,
            Info {
                kind: Kind::from_code(code),
                size: 0,
            },
        ),
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
fn at_break(tokenizer: &mut Tokenizer, code: Code, info: Info) -> StateFnResult {
    match code {
        Code::None | Code::CarriageReturnLineFeed | Code::Char('\n' | '\r')
            if info.size >= THEMATIC_BREAK_MARKER_COUNT_MIN =>
        {
            tokenizer.exit(TokenType::ThematicBreak);
            // Feel free to interrupt.
            tokenizer.interrupt = false;
            (State::Ok, Some(vec![code]))
        }
        Code::Char(char) if char == info.kind.as_char() => {
            tokenizer.enter(TokenType::ThematicBreakSequence);
            sequence(tokenizer, code, info)
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
fn sequence(tokenizer: &mut Tokenizer, code: Code, mut info: Info) -> StateFnResult {
    match code {
        Code::Char(char) if char == info.kind.as_char() => {
            tokenizer.consume(code);
            info.size += 1;
            (State::Fn(Box::new(|t, c| sequence(t, c, info))), None)
        }
        _ => {
            tokenizer.exit(TokenType::ThematicBreakSequence);
            tokenizer.attempt_opt(space_or_tab(), |t, c| at_break(t, c, info))(tokenizer, code)
        }
    }
}
