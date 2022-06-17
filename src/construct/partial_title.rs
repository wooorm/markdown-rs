// To do: pass token types in.

use crate::construct::partial_whitespace::start as whitespace;
use crate::tokenizer::{Code, State, StateFnResult, TokenType, Tokenizer};

/// Type of quote, if we’re in an attribure, in complete (condition 7).
#[derive(Debug, Clone, PartialEq)]
enum TitleKind {
    /// In a parenthesised (`(` and `)`) title.
    Paren,
    /// In a double quoted (`"`) title.
    Double,
    /// In a single quoted (`"`) title.
    Single,
}

fn kind_to_marker(kind: &TitleKind) -> char {
    match kind {
        TitleKind::Double => '"',
        TitleKind::Single => '\'',
        TitleKind::Paren => ')',
    }
}

pub fn start(tokenizer: &mut Tokenizer, code: Code) -> StateFnResult {
    let kind = match code {
        Code::Char('"') => Some(TitleKind::Double),
        Code::Char('\'') => Some(TitleKind::Single),
        Code::Char('(') => Some(TitleKind::Paren),
        _ => None,
    };

    if let Some(kind) = kind {
        tokenizer.enter(TokenType::DefinitionTitle);
        tokenizer.enter(TokenType::DefinitionTitleMarker);
        tokenizer.consume(code);
        tokenizer.exit(TokenType::DefinitionTitleMarker);
        (State::Fn(Box::new(|t, c| at_first_break(t, c, kind))), None)
    } else {
        (State::Nok, None)
    }
}

/// To do.
fn at_first_break(tokenizer: &mut Tokenizer, code: Code, kind: TitleKind) -> StateFnResult {
    match code {
        Code::Char(char) if char == kind_to_marker(&kind) => {
            tokenizer.enter(TokenType::DefinitionTitleMarker);
            tokenizer.consume(code);
            tokenizer.exit(TokenType::DefinitionTitleMarker);
            tokenizer.exit(TokenType::DefinitionTitle);
            (State::Ok, None)
        }
        _ => {
            tokenizer.enter(TokenType::DefinitionTitleString);
            at_break(tokenizer, code, kind)
        }
    }
}

/// To do.
fn at_break(tokenizer: &mut Tokenizer, code: Code, kind: TitleKind) -> StateFnResult {
    match code {
        Code::Char(char) if char == kind_to_marker(&kind) => {
            tokenizer.exit(TokenType::DefinitionTitleString);
            at_first_break(tokenizer, code, kind)
        }
        Code::None => (State::Nok, None),
        Code::CarriageReturnLineFeed | Code::Char('\r' | '\n') => {
            tokenizer.enter(TokenType::LineEnding);
            tokenizer.consume(code);
            tokenizer.exit(TokenType::LineEnding);
            (
                State::Fn(Box::new(|t, c| at_break_line_start(t, c, kind))),
                None,
            )
        }
        _ => {
            // To do: link.
            tokenizer.enter(TokenType::ChunkString);
            title(tokenizer, code, kind)
        }
    }
}

fn at_break_line_start(tokenizer: &mut Tokenizer, code: Code, kind: TitleKind) -> StateFnResult {
    tokenizer.attempt(
        |t, c| whitespace(t, c, TokenType::Whitespace),
        |_ok| Box::new(|t, c| at_break_line_begin(t, c, kind)),
    )(tokenizer, code)
}

fn at_break_line_begin(tokenizer: &mut Tokenizer, code: Code, kind: TitleKind) -> StateFnResult {
    match code {
        // Blank line not allowed.
        Code::CarriageReturnLineFeed | Code::Char('\r' | '\n') => (State::Nok, None),
        _ => at_break(tokenizer, code, kind),
    }
}

/// To do.
fn title(tokenizer: &mut Tokenizer, code: Code, kind: TitleKind) -> StateFnResult {
    match code {
        Code::Char(char) if char == kind_to_marker(&kind) => {
            tokenizer.exit(TokenType::ChunkString);
            at_break(tokenizer, code, kind)
        }
        Code::None | Code::CarriageReturnLineFeed | Code::Char('\r' | '\n') => {
            tokenizer.exit(TokenType::ChunkString);
            at_break(tokenizer, code, kind)
        }
        Code::Char('\\') => {
            tokenizer.consume(code);
            (State::Fn(Box::new(|t, c| escape(t, c, kind))), None)
        }
        _ => {
            tokenizer.consume(code);
            (State::Fn(Box::new(|t, c| title(t, c, kind))), None)
        }
    }
}

/// To do.
fn escape(tokenizer: &mut Tokenizer, code: Code, kind: TitleKind) -> StateFnResult {
    match code {
        Code::Char(char) if char == kind_to_marker(&kind) => {
            tokenizer.consume(code);
            (State::Fn(Box::new(move |t, c| title(t, c, kind))), None)
        }
        Code::Char('\\') => {
            tokenizer.consume(code);
            (State::Fn(Box::new(move |t, c| title(t, c, kind))), None)
        }
        _ => title(tokenizer, code, kind),
    }
}
