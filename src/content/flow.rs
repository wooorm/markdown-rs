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

use crate::construct::{
    blank_line::start as blank_line, code_fenced::start as code_fenced,
    code_indented::start as code_indented, definition::start as definition,
    heading_atx::start as heading_atx, heading_setext::start as heading_setext,
    html_flow::start as html_flow, paragraph::start as paragraph,
    thematic_break::start as thematic_break,
};
use crate::parser::ParseState;
use crate::subtokenize::subtokenize;
use crate::tokenizer::{Code, Event, EventType, Point, State, StateFnResult, TokenType, Tokenizer};
use crate::util::{
    normalize_identifier::normalize_identifier,
    span::{from_exit_event, serialize},
};
use std::collections::HashSet;

/// Turn `codes` as the flow content type into events.
pub fn flow(parse_state: &mut ParseState, point: Point, index: usize) -> Vec<Event> {
    let mut tokenizer = Tokenizer::new(point, index, parse_state);
    tokenizer.push(&parse_state.codes, Box::new(start), true);
    let mut next_definitions: HashSet<String> = HashSet::new();

    let mut index = 0;

    while index < tokenizer.events.len() {
        let event = &tokenizer.events[index];

        if event.event_type == EventType::Exit
            && event.token_type == TokenType::DefinitionLabelString
        {
            next_definitions.insert(normalize_identifier(
                serialize(
                    &parse_state.codes,
                    &from_exit_event(&tokenizer.events, index),
                    false,
                )
                .as_str(),
            ));
        }

        index += 1;
    }

    let mut result = (tokenizer.events, false);

    parse_state.definitions = next_definitions;

    while !result.1 {
        result = subtokenize(result.0, parse_state);
    }

    result.0
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
fn initial_before(tokenizer: &mut Tokenizer, code: Code) -> StateFnResult {
    match code {
        Code::None => (State::Ok, None),
        _ => tokenizer.attempt_n(
            vec![
                Box::new(code_indented),
                Box::new(code_fenced),
                Box::new(html_flow),
                Box::new(heading_atx),
                Box::new(heading_setext),
                Box::new(thematic_break),
                Box::new(definition),
            ],
            |ok| Box::new(if ok { after } else { before_paragraph }),
        )(tokenizer, code),
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
            // Feel free to interrupt.
            tokenizer.interrupt = false;
            (State::Fn(Box::new(start)), None)
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
fn after(tokenizer: &mut Tokenizer, code: Code) -> StateFnResult {
    match code {
        Code::None => (State::Ok, None),
        Code::CarriageReturnLineFeed | Code::Char('\n' | '\r') => {
            tokenizer.enter(TokenType::LineEnding);
            tokenizer.consume(code);
            tokenizer.exit(TokenType::LineEnding);
            (State::Fn(Box::new(start)), None)
        }
        _ => unreachable!("expected eol/eof"),
    }
}

/// Before a paragraph.
///
/// ```markdown
/// |asd
/// ```
fn before_paragraph(tokenizer: &mut Tokenizer, code: Code) -> StateFnResult {
    tokenizer.go(paragraph, after)(tokenizer, code)
}
