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

use crate::event::Name;
use crate::state::{Name as StateName, State};
use crate::tokenizer::Tokenizer;

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
        Some(b'`' | b'~') => {
            tokenizer.attempt(
                State::Next(StateName::FlowAfter),
                State::Next(StateName::FlowBeforeParagraph),
            );
            State::Retry(StateName::CodeFencedStart)
        }
        Some(b'<') => {
            tokenizer.attempt(
                State::Next(StateName::FlowAfter),
                State::Next(StateName::FlowBeforeParagraph),
            );
            State::Retry(StateName::HtmlFlowStart)
        }
        Some(b'#') => {
            tokenizer.attempt(
                State::Next(StateName::FlowAfter),
                State::Next(StateName::FlowBeforeParagraph),
            );
            State::Retry(StateName::HeadingAtxStart)
        }
        // Note: `-` is also used in thematic breaks, so it’s not included here.
        Some(b'=') => {
            tokenizer.attempt(
                State::Next(StateName::FlowAfter),
                State::Next(StateName::FlowBeforeParagraph),
            );
            State::Retry(StateName::HeadingSetextStart)
        }
        Some(b'*' | b'_') => {
            tokenizer.attempt(
                State::Next(StateName::FlowAfter),
                State::Next(StateName::FlowBeforeParagraph),
            );
            State::Retry(StateName::ThematicBreakStart)
        }
        Some(b'[') => {
            tokenizer.attempt(
                State::Next(StateName::FlowAfter),
                State::Next(StateName::FlowBeforeParagraph),
            );
            State::Retry(StateName::DefinitionStart)
        }
        // Actual parsing: blank line? Indented code? Indented anything?
        // Also includes `-` which can be a setext heading underline or a thematic break.
        None | Some(b'\t' | b'\n' | b' ' | b'-') => State::Retry(StateName::FlowBlankLineBefore),
        Some(_) => {
            tokenizer.attempt(State::Next(StateName::FlowAfter), State::Nok);
            State::Retry(StateName::ParagraphStart)
        }
    }
}

pub fn blank_line_before(tokenizer: &mut Tokenizer) -> State {
    tokenizer.attempt(
        State::Next(StateName::FlowBlankLineAfter),
        State::Next(StateName::FlowBeforeCodeIndented),
    );
    State::Retry(StateName::BlankLineStart)
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
        State::Next(StateName::FlowAfter),
        State::Next(StateName::FlowBeforeCodeFenced),
    );
    State::Retry(StateName::CodeIndentedStart)
}

pub fn before_code_fenced(tokenizer: &mut Tokenizer) -> State {
    tokenizer.attempt(
        State::Next(StateName::FlowAfter),
        State::Next(StateName::FlowBeforeHtml),
    );
    State::Retry(StateName::CodeFencedStart)
}

pub fn before_html(tokenizer: &mut Tokenizer) -> State {
    tokenizer.attempt(
        State::Next(StateName::FlowAfter),
        State::Next(StateName::FlowBeforeHeadingAtx),
    );
    State::Retry(StateName::HtmlFlowStart)
}

pub fn before_heading_atx(tokenizer: &mut Tokenizer) -> State {
    tokenizer.attempt(
        State::Next(StateName::FlowAfter),
        State::Next(StateName::FlowBeforeHeadingSetext),
    );
    State::Retry(StateName::HeadingAtxStart)
}

pub fn before_heading_setext(tokenizer: &mut Tokenizer) -> State {
    tokenizer.attempt(
        State::Next(StateName::FlowAfter),
        State::Next(StateName::FlowBeforeThematicBreak),
    );
    State::Retry(StateName::HeadingSetextStart)
}

pub fn before_thematic_break(tokenizer: &mut Tokenizer) -> State {
    tokenizer.attempt(
        State::Next(StateName::FlowAfter),
        State::Next(StateName::FlowBeforeDefinition),
    );
    State::Retry(StateName::ThematicBreakStart)
}

pub fn before_definition(tokenizer: &mut Tokenizer) -> State {
    tokenizer.attempt(
        State::Next(StateName::FlowAfter),
        State::Next(StateName::FlowBeforeParagraph),
    );
    State::Retry(StateName::DefinitionStart)
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
            tokenizer.enter(Name::BlankLineEnding);
            tokenizer.consume();
            tokenizer.exit(Name::BlankLineEnding);
            // Feel free to interrupt.
            tokenizer.interrupt = false;
            State::Next(StateName::FlowStart)
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
            tokenizer.enter(Name::LineEnding);
            tokenizer.consume();
            tokenizer.exit(Name::LineEnding);
            State::Next(StateName::FlowStart)
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
    tokenizer.attempt(State::Next(StateName::FlowAfter), State::Nok);
    State::Retry(StateName::ParagraphStart)
}
