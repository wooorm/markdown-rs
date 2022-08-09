//! The flow content type.
//!
//! **Flow** represents the sections, such as headings and code, which are
//! parsed per line.
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
//! *   [Definition][crate::construct::definition]
//! *   [Heading (atx)][crate::construct::heading_atx]
//! *   [Heading (setext)][crate::construct::heading_setext]
//! *   [HTML (flow)][crate::construct::html_flow]
//! *   [Thematic break][crate::construct::thematic_break]

use crate::token::Token;
use crate::tokenizer::{State, StateName, Tokenizer};

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
pub fn start(tokenizer: &mut Tokenizer) -> State {
    match tokenizer.current {
        Some(b'`' | b'~') => tokenizer.attempt(
            StateName::CodeFencedStart,
            State::Fn(StateName::FlowAfter),
            State::Fn(StateName::FlowBeforeParagraph),
        ),
        Some(b'<') => tokenizer.attempt(
            StateName::HtmlFlowStart,
            State::Fn(StateName::FlowAfter),
            State::Fn(StateName::FlowBeforeParagraph),
        ),
        Some(b'#') => tokenizer.attempt(
            StateName::HeadingAtxStart,
            State::Fn(StateName::FlowAfter),
            State::Fn(StateName::FlowBeforeParagraph),
        ),
        // Note: `-` is also used in thematic breaks, so it’s not included here.
        Some(b'=') => tokenizer.attempt(
            StateName::HeadingSetextStart,
            State::Fn(StateName::FlowAfter),
            State::Fn(StateName::FlowBeforeParagraph),
        ),
        Some(b'*' | b'_') => tokenizer.attempt(
            StateName::ThematicBreakStart,
            State::Fn(StateName::FlowAfter),
            State::Fn(StateName::FlowBeforeParagraph),
        ),
        Some(b'[') => tokenizer.attempt(
            StateName::DefinitionStart,
            State::Fn(StateName::FlowAfter),
            State::Fn(StateName::FlowBeforeParagraph),
        ),
        // Actual parsing: blank line? Indented code? Indented anything?
        // Also includes `-` which can be a setext heading underline or a thematic break.
        None | Some(b'\t' | b'\n' | b' ' | b'-') => before_blank_line(tokenizer),
        Some(_) => tokenizer.attempt(
            StateName::ParagraphStart,
            State::Fn(StateName::FlowAfter),
            State::Nok,
        ),
    }
}

pub fn before_blank_line(tokenizer: &mut Tokenizer) -> State {
    tokenizer.attempt(
        StateName::BlankLineStart,
        State::Fn(StateName::FlowBlankLineAfter),
        State::Fn(StateName::FlowBeforeCodeIndented),
    )
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
/// |~~~js
/// |<div>
/// ```
pub fn before_code_indented(tokenizer: &mut Tokenizer) -> State {
    tokenizer.attempt(
        StateName::CodeIndentedStart,
        State::Fn(StateName::FlowAfter),
        State::Fn(StateName::FlowBeforeCodeFenced),
    )
}

pub fn before_code_fenced(tokenizer: &mut Tokenizer) -> State {
    tokenizer.attempt(
        StateName::CodeFencedStart,
        State::Fn(StateName::FlowAfter),
        State::Fn(StateName::FlowBeforeHtml),
    )
}

pub fn before_html(tokenizer: &mut Tokenizer) -> State {
    tokenizer.attempt(
        StateName::HtmlFlowStart,
        State::Fn(StateName::FlowAfter),
        State::Fn(StateName::FlowBeforeHeadingAtx),
    )
}

pub fn before_heading_atx(tokenizer: &mut Tokenizer) -> State {
    tokenizer.attempt(
        StateName::HeadingAtxStart,
        State::Fn(StateName::FlowAfter),
        State::Fn(StateName::FlowBeforeHeadingSetext),
    )
}

pub fn before_heading_setext(tokenizer: &mut Tokenizer) -> State {
    tokenizer.attempt(
        StateName::HeadingSetextStart,
        State::Fn(StateName::FlowAfter),
        State::Fn(StateName::FlowBeforeThematicBreak),
    )
}

pub fn before_thematic_break(tokenizer: &mut Tokenizer) -> State {
    tokenizer.attempt(
        StateName::ThematicBreakStart,
        State::Fn(StateName::FlowAfter),
        State::Fn(StateName::FlowBeforeDefinition),
    )
}

pub fn before_definition(tokenizer: &mut Tokenizer) -> State {
    tokenizer.attempt(
        StateName::DefinitionStart,
        State::Fn(StateName::FlowAfter),
        State::Fn(StateName::FlowBeforeParagraph),
    )
}

/// After a blank line.
///
/// Move to `start` afterwards.
///
/// ```markdown
/// ␠␠|
/// ```
pub fn blank_line_after(tokenizer: &mut Tokenizer) -> State {
    match tokenizer.current {
        None => State::Ok,
        Some(b'\n') => {
            tokenizer.enter(Token::BlankLineEnding);
            tokenizer.consume();
            tokenizer.exit(Token::BlankLineEnding);
            // Feel free to interrupt.
            tokenizer.interrupt = false;
            State::Fn(StateName::FlowStart)
        }
        _ => unreachable!("expected eol/eof"),
    }
}

/// After something.
///
/// ```markdown
/// ## alpha|
/// |
/// ~~~js
/// asd
/// ~~~|
/// ```
pub fn after(tokenizer: &mut Tokenizer) -> State {
    match tokenizer.current {
        None => State::Ok,
        Some(b'\n') => {
            tokenizer.enter(Token::LineEnding);
            tokenizer.consume();
            tokenizer.exit(Token::LineEnding);
            State::Fn(StateName::FlowStart)
        }
        _ => unreachable!("expected eol/eof"),
    }
}

/// Before a paragraph.
///
/// ```markdown
/// |asd
/// ```
pub fn before_paragraph(tokenizer: &mut Tokenizer) -> State {
    tokenizer.attempt(
        StateName::ParagraphStart,
        State::Fn(StateName::FlowAfter),
        State::Nok,
    )
}
