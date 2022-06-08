//! The flow content type.
//!
//! **Flow** represents the sections, such as headings, code, and content, which
//! is parsed per line.
//! An example is HTML, which has a certain starting condition (such as
//! `<script>` on its own line), then continues for a while, until an end
//! condition is found (such as `</style>`).
//! If that line with an end condition is never found, that flow goes until
//! the end.
//!
//! The constructs found in flow are:
//!
//! *   [Blank line][crate::construct::blank_line]
//! *   [Code (fenced)][crate::construct::code_fenced]
//! *   [Code (indented)][crate::construct::code_indented]
//! *   [Heading (atx)][crate::construct::heading_atx]
//! *   [HTML (flow)][crate::construct::html_flow]
//! *   [Thematic break][crate::construct::thematic_break]
//!
//! <!-- To do: `setext` in content? Link to content. -->

use crate::construct::{
    blank_line::start as blank_line, code_fenced::start as code_fenced,
    code_indented::start as code_indented, heading_atx::start as heading_atx,
    html_flow::start as html_flow, partial_whitespace::start as whitespace,
    thematic_break::start as thematic_break,
};
use crate::tokenizer::{Code, Event, State, StateFnResult, TokenType, Tokenizer};

/// Turn `codes` as the flow content type into events.
// To do: remove this `allow` when all the content types are glued together.
#[allow(dead_code)]
pub fn flow(codes: Vec<Code>) -> Vec<Event> {
    let mut tokenizer = Tokenizer::new();
    let (state, remainder) = tokenizer.feed(codes, Box::new(start), true);

    if let Some(ref x) = remainder {
        if !x.is_empty() {
            unreachable!("expected no final remainder {:?}", x);
        }
    }

    match state {
        State::Ok => {}
        _ => unreachable!("expected final state to be `State::Ok`"),
    }

    tokenizer.events
}

/// Before flow.
///
/// First we assume a blank line.
//
/// ```markdown
/// |
/// |## alpha
/// |    bravo
/// |***
/// ```
fn start(tokenizer: &mut Tokenizer, code: Code) -> StateFnResult {
    match code {
        Code::None => (State::Ok, None),
        _ => tokenizer.attempt(blank_line, |ok| {
            Box::new(if ok { blank_line_after } else { initial_before })
        })(tokenizer, code),
    }
}

/// After a blank line.
///
/// Move to `start` afterwards.
///
/// ```markdown
/// ␠␠|
/// ```
fn blank_line_after(tokenizer: &mut Tokenizer, code: Code) -> StateFnResult {
    match code {
        Code::None => (State::Ok, None),
        Code::CarriageReturnLineFeed | Code::Char('\n' | '\r') => {
            tokenizer.enter(TokenType::BlankLineEnding);
            tokenizer.consume(code);
            tokenizer.exit(TokenType::BlankLineEnding);
            (State::Fn(Box::new(start)), None)
        }
        _ => unreachable!("expected eol/eof after blank line `{:?}`", code),
    }
}

/// Before flow (initial).
///
/// “Initial” flow means unprefixed flow, so right at the start of a line.
/// Interestingly, the only flow (initial) construct is indented code.
/// Move to `before` afterwards.
///
/// ```markdown
/// |qwe
/// |    asd
/// ```
fn initial_before(tokenizer: &mut Tokenizer, code: Code) -> StateFnResult {
    match code {
        Code::None => (State::Ok, None),
        _ => tokenizer.attempt(code_indented, |ok| {
            Box::new(if ok {
                after
            } else {
                initial_before_not_code_indented
            })
        })(tokenizer, code),
    }
}

/// After a flow construct.
///
/// ```markdown
/// ## alpha|
/// |
/// ~~~js
/// asd
/// ~~~|
/// ```
fn after(tokenizer: &mut Tokenizer, code: Code) -> StateFnResult {
    match code {
        Code::None => (State::Ok, None),
        Code::CarriageReturnLineFeed | Code::Char('\n' | '\r') => {
            tokenizer.enter(TokenType::LineEnding);
            tokenizer.consume(code);
            tokenizer.exit(TokenType::LineEnding);
            (State::Fn(Box::new(start)), None)
        }
        _ => unreachable!("unexpected non-eol/eof after flow `{:?}`", code),
    }
}

/// Before flow (initial), but not at code (indented).
///
/// ```markdown
/// |qwe
/// ```
fn initial_before_not_code_indented(tokenizer: &mut Tokenizer, code: Code) -> StateFnResult {
    match code {
        Code::None => (State::Ok, None),
        _ => tokenizer.attempt(code_fenced, |ok| {
            Box::new(if ok {
                after
            } else {
                initial_before_not_code_fenced
            })
        })(tokenizer, code),
    }
}

/// Before flow (initial), but not at code (fenced).
///
/// ```markdown
/// |qwe
/// ```
fn initial_before_not_code_fenced(tokenizer: &mut Tokenizer, code: Code) -> StateFnResult {
    match code {
        Code::None => (State::Ok, None),
        _ => tokenizer.attempt(html_flow, |ok| Box::new(if ok { after } else { before }))(
            tokenizer, code,
        ),
    }
}

/// Before flow, but not at code (indented) or code (fenced).
///
/// Compared to flow (initial), normal flow can be arbitrarily prefixed.
///
/// ```markdown
/// |qwe
/// ```
pub fn before(tokenizer: &mut Tokenizer, code: Code) -> StateFnResult {
    tokenizer.attempt(
        |tokenizer, code| whitespace(tokenizer, code, TokenType::Whitespace),
        |_ok| Box::new(before_after_prefix),
    )(tokenizer, code)
}

/// Before flow, after potential whitespace.
///
/// ```markdown
/// |qwe
/// ```
pub fn before_after_prefix(tokenizer: &mut Tokenizer, code: Code) -> StateFnResult {
    tokenizer.attempt(heading_atx, |ok| {
        Box::new(if ok { after } else { before_not_heading_atx })
    })(tokenizer, code)
}

/// Before flow, but not before a heading (atx)
///
/// ```markdown
/// |qwe
/// ```
pub fn before_not_heading_atx(tokenizer: &mut Tokenizer, code: Code) -> StateFnResult {
    tokenizer.attempt(thematic_break, |ok| {
        Box::new(if ok { after } else { before_not_thematic_break })
    })(tokenizer, code)
}

/// Before flow, but not before a heading (atx) or thematic break.
///
/// ```markdown
/// |qwe
/// ```
pub fn before_not_thematic_break(tokenizer: &mut Tokenizer, code: Code) -> StateFnResult {
    tokenizer.attempt(html_flow, |ok| {
        Box::new(if ok { after } else { content_before })
    })(tokenizer, code)
}

/// Before flow, but not before a heading (atx) or thematic break.
///
/// At this point, we’re at content (zero or more definitions and zero or one
/// paragraph/setext heading).
///
/// ```markdown
/// |qwe
/// ```
// To do: currently only parses a single line.
// To do:
// - Multiline
// - One or more definitions.
// - Setext heading.
fn content_before(tokenizer: &mut Tokenizer, code: Code) -> StateFnResult {
    match code {
        Code::None | Code::CarriageReturnLineFeed | Code::Char('\n' | '\r') => {
            after(tokenizer, code)
        }
        _ => {
            tokenizer.enter(TokenType::Content);
            tokenizer.enter(TokenType::ContentPhrasing);
            tokenizer.consume(code);
            (State::Fn(Box::new(content)), None)
        }
    }
}
/// In content.
///
/// ```markdown
/// al|pha
/// ```
// To do: lift limitations as documented above.
fn content(tokenizer: &mut Tokenizer, code: Code) -> StateFnResult {
    match code {
        Code::None | Code::Char('\n' | '\r') => {
            tokenizer.exit(TokenType::ContentPhrasing);
            tokenizer.exit(TokenType::Content);
            after(tokenizer, code)
        }
        _ => {
            tokenizer.consume(code);
            (State::Fn(Box::new(content)), None)
        }
    }
}
