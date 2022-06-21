//! Heading (setext) is a construct that occurs in the [flow] content type.
//!
//! They’re formed with the following BNF:
//!
//! ```bnf
//! heading_setext ::= line *(eol line) eol whitespace_optional (1*'-' | 1*'=') whitespace_optional
//!
//! whitespace ::= 1*space_or_tab
//! whitespace_optional ::= [ whitespace ]
//! line ::= code - eol
//! eol ::= '\r' | '\r\n' | '\n'
//! ```
//!
//! Heading (setext) in markdown relates to the `<h1>` and `<h2>` elements in
//! HTML.
//! See [*§ 4.3.6 The `h1`, `h2`, `h3`, `h4`, `h5`, and `h6` elements* in the
//! HTML spec][html] for more info.
//!
//! In markdown, it is also possible to create headings with a
//! [heading (atx)][heading_atx] construct.
//! The benefit of setext headings is that their text can include line endings,
//! and by extensions also hard breaks (e.g., with
//! [hard break (escape)][hard_break_escape]).
//! However, their limit is that they cannot form `<h3>` through `<h6>`
//! headings.
//! Due to this limitation, it is recommended to use atx headings.
//!
//! [Thematic breaks][thematic_break] formed with dashes (without whitespace)
//! can also form heading (setext).
//!
//! > 🏛 **Background**: the word *setext* originates from a small markup
//! > language by Ian Feldman from 1991.
//! > See [*§ Setext* on Wikipedia][wiki-setext] for more info.
//! > The word *atx* originates from a tiny markup language by Aaron Swartz
//! > from 2002.
//! > See [*§ atx, the true structured text format* on `aaronsw.com`][atx] for
//! > more info.
//!
//! ## References
//!
//! *   [`setext-underline.js` in `micromark`](https://github.com/micromark/micromark/blob/main/packages/micromark-core-commonmark/dev/lib/setext-underline.js)
//! *   [*§ 4.3 Setext headings* in `CommonMark`](https://spec.commonmark.org/0.30/#setext-headings)
//!
//! [flow]: crate::content::flow
//! [heading_atx]: crate::construct::heading_atx
//! [thematic_break]: crate::construct::thematic_break
//! [hard_break_escape]: crate::construct::hard_break_escape
//! [html]: https://html.spec.whatwg.org/multipage/sections.html#the-h1,-h2,-h3,-h4,-h5,-and-h6-elements
//! [wiki-setext]: https://en.wikipedia.org/wiki/Setext
//! [atx]: http://www.aaronsw.com/2002/atx/

use crate::constant::TAB_SIZE;
use crate::construct::partial_space_or_tab::space_or_tab_opt;
use crate::subtokenize::link;
use crate::tokenizer::{Code, State, StateFnResult, TokenType, Tokenizer};
use crate::util::span::from_exit_event;

/// Kind of underline.
#[derive(Debug, Clone, PartialEq)]
pub enum Kind {
    /// Dash (rank 2) heading.
    ///
    /// ## Example
    ///
    /// ```markdown
    /// alpha
    /// -----
    /// ```
    Dash,

    /// Equals to (rank 1) heading.
    ///
    /// ## Example
    ///
    /// ```markdown
    /// alpha
    /// =====
    /// ```
    EqualsTo,
}

impl Kind {
    /// Turn the kind into a [char].
    fn as_char(&self) -> char {
        match self {
            Kind::Dash => '-',
            Kind::EqualsTo => '=',
        }
    }
    /// Turn a [char] into a kind.
    ///
    /// ## Panics
    ///
    /// Panics if `char` is not `-` or `=`.
    fn from_char(char: char) -> Kind {
        match char {
            '-' => Kind::Dash,
            '=' => Kind::EqualsTo,
            _ => unreachable!("invalid char"),
        }
    }
}

/// Start of a heading (setext).
///
/// ```markdown
/// |alpha
/// ==
/// ```
pub fn start(tokenizer: &mut Tokenizer, code: Code) -> StateFnResult {
    tokenizer.enter(TokenType::HeadingSetext);
    tokenizer.go(space_or_tab_opt(), before)(tokenizer, code)
}

/// Start of a heading (setext), after whitespace.
///
/// ```markdown
/// |alpha
/// ==
/// ```
pub fn before(tokenizer: &mut Tokenizer, code: Code) -> StateFnResult {
    match code {
        Code::None | Code::CarriageReturnLineFeed | Code::Char('\n' | '\r') => {
            unreachable!("expected non-eol/eof");
        }
        _ => {
            tokenizer.enter(TokenType::HeadingSetextText);
            tokenizer.enter(TokenType::ChunkText);
            text_inside(tokenizer, code)
        }
    }
}

/// Inside text.
///
/// ```markdown
/// al|pha
/// bra|vo
/// ==
/// ```
fn text_inside(tokenizer: &mut Tokenizer, code: Code) -> StateFnResult {
    match code {
        Code::None => (State::Nok, None),
        Code::CarriageReturnLineFeed | Code::Char('\n' | '\r') => {
            tokenizer.exit(TokenType::ChunkText);
            tokenizer.exit(TokenType::HeadingSetextText);
            tokenizer.attempt(underline_before, |ok| {
                Box::new(if ok { after } else { text_continue })
            })(tokenizer, code)
        }
        _ => {
            tokenizer.consume(code);
            (State::Fn(Box::new(text_inside)), None)
        }
    }
}

/// At a line ending, not at an underline.
///
/// ```markdown
/// alpha
/// |bravo
/// ==
/// ```
fn text_continue(tokenizer: &mut Tokenizer, code: Code) -> StateFnResult {
    // Needed to connect the text.
    tokenizer.enter(TokenType::HeadingSetextText);
    tokenizer.events.pop();
    tokenizer.events.pop();

    match code {
        Code::CarriageReturnLineFeed | Code::Char('\n' | '\r') => {
            tokenizer.enter(TokenType::LineEnding);
            let index = tokenizer.events.len() - 1;
            link(&mut tokenizer.events, index);
            tokenizer.consume(code);
            tokenizer.exit(TokenType::LineEnding);

            (
                State::Fn(Box::new(tokenizer.go(space_or_tab_opt(), text_line_start))),
                None,
            )
        }
        _ => unreachable!("expected eol"),
    }
}

/// At a line ending after whitespace, not at an underline.
///
/// ```markdown
/// alpha
/// |bravo
/// ==
/// ```
fn text_line_start(tokenizer: &mut Tokenizer, code: Code) -> StateFnResult {
    let index = tokenizer.events.len() - 2;

    // Link the whitespace, if it exists.
    if tokenizer.events[index].token_type == TokenType::Whitespace {
        link(&mut tokenizer.events, index);
    }

    match code {
        // Blank lines not allowed.
        Code::None | Code::CarriageReturnLineFeed | Code::Char('\n' | '\r') => (State::Nok, None),
        _ => {
            tokenizer.enter(TokenType::ChunkText);
            let index = tokenizer.events.len() - 1;
            link(&mut tokenizer.events, index);
            text_inside(tokenizer, code)
        }
    }
}

/// After a heading (setext).
///
/// ```markdown
/// alpha
/// ==|
/// ```
fn after(tokenizer: &mut Tokenizer, code: Code) -> StateFnResult {
    tokenizer.exit(TokenType::HeadingSetext);
    (State::Ok, Some(vec![code]))
}

/// At a line ending, presumably an underline.
///
/// ```markdown
/// alpha|
/// ==
/// ```
fn underline_before(tokenizer: &mut Tokenizer, code: Code) -> StateFnResult {
    match code {
        Code::CarriageReturnLineFeed | Code::Char('\n' | '\r') => {
            tokenizer.enter(TokenType::LineEnding);
            tokenizer.consume(code);
            tokenizer.exit(TokenType::LineEnding);
            (
                State::Fn(Box::new(
                    tokenizer.go(space_or_tab_opt(), underline_sequence_start),
                )),
                None,
            )
        }
        _ => unreachable!("expected eol"),
    }
}

/// After optional whitespace, presumably an underline.
///
/// ```markdown
/// alpha
/// |==
/// ```
fn underline_sequence_start(tokenizer: &mut Tokenizer, code: Code) -> StateFnResult {
    let tail = tokenizer.events.last();
    let mut prefix = 0;

    if let Some(event) = tail {
        if event.token_type == TokenType::Whitespace {
            let span = from_exit_event(&tokenizer.events, tokenizer.events.len() - 1);
            prefix = span.end_index - span.start_index;
        }
    }

    // To do: 4+ should be okay if code (indented) is turned off!
    if prefix >= TAB_SIZE {
        return (State::Nok, None);
    }

    match code {
        Code::Char(char) if char == '-' || char == '=' => {
            tokenizer.enter(TokenType::HeadingSetextUnderline);
            underline_sequence_inside(tokenizer, code, Kind::from_char(char))
        }
        _ => (State::Nok, None),
    }
}

/// In an underline sequence.
///
/// ```markdown
/// alpha
/// =|=
/// ```
fn underline_sequence_inside(tokenizer: &mut Tokenizer, code: Code, kind: Kind) -> StateFnResult {
    match code {
        Code::Char(char) if char == kind.as_char() => {
            tokenizer.consume(code);
            (
                State::Fn(Box::new(move |t, c| underline_sequence_inside(t, c, kind))),
                None,
            )
        }
        _ => tokenizer.go(space_or_tab_opt(), underline_after)(tokenizer, code),
    }
}

/// After an underline sequence, after optional whitespace.
///
/// ```markdown
/// alpha
/// ==|
/// ```
fn underline_after(tokenizer: &mut Tokenizer, code: Code) -> StateFnResult {
    match code {
        Code::None | Code::CarriageReturnLineFeed | Code::Char('\n' | '\r') => {
            tokenizer.exit(TokenType::HeadingSetextUnderline);
            (State::Ok, Some(vec![code]))
        }
        _ => (State::Nok, None),
    }
}
