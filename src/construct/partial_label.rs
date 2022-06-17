// To do: pass token types in.

use crate::constant::LINK_REFERENCE_SIZE_MAX;
use crate::tokenizer::{Code, State, StateFnResult, TokenType, Tokenizer};

/// To do.
pub fn start(tokenizer: &mut Tokenizer, code: Code) -> StateFnResult {
    match code {
        Code::Char('[') => {
            tokenizer.enter(TokenType::DefinitionLabel);
            tokenizer.enter(TokenType::DefinitionLabelMarker);
            tokenizer.consume(code);
            tokenizer.exit(TokenType::DefinitionLabelMarker);
            tokenizer.enter(TokenType::DefinitionLabelData);
            (State::Fn(Box::new(|t, c| at_break(t, c, false, 0))), None)
        }
        // To do: allow?
        _ => unreachable!("expected `[` at start of label"),
    }
}

/// To do.
fn at_break(tokenizer: &mut Tokenizer, code: Code, data: bool, size: usize) -> StateFnResult {
    match code {
        Code::None | Code::Char('[') => (State::Nok, None),
        Code::Char(']') if !data => (State::Nok, None),
        _ if size > LINK_REFERENCE_SIZE_MAX => (State::Nok, None),
        Code::Char(']') => {
            tokenizer.exit(TokenType::DefinitionLabelData);
            tokenizer.enter(TokenType::DefinitionLabelMarker);
            tokenizer.consume(code);
            tokenizer.exit(TokenType::DefinitionLabelMarker);
            tokenizer.exit(TokenType::DefinitionLabel);
            (State::Ok, None)
        }
        Code::CarriageReturnLineFeed | Code::Char('\r' | '\n') => {
            tokenizer.enter(TokenType::LineEnding);
            tokenizer.consume(code);
            tokenizer.exit(TokenType::LineEnding);
            (
                State::Fn(Box::new(move |t, c| at_break(t, c, data, size))),
                None,
            )
        }
        _ => {
            tokenizer.enter(TokenType::ChunkString);
            // To do: link.
            label(tokenizer, code, data, size)
        }
    }
}

/// To do.
fn label(tokenizer: &mut Tokenizer, code: Code, data: bool, size: usize) -> StateFnResult {
    match code {
        Code::None | Code::CarriageReturnLineFeed | Code::Char('\r' | '\n' | '[' | ']') => {
            tokenizer.exit(TokenType::ChunkString);
            at_break(tokenizer, code, data, size)
        }
        _ if size > LINK_REFERENCE_SIZE_MAX => {
            tokenizer.exit(TokenType::ChunkString);
            at_break(tokenizer, code, data, size)
        }
        Code::VirtualSpace | Code::Char('\t' | ' ') => {
            tokenizer.consume(code);
            (
                State::Fn(Box::new(move |t, c| label(t, c, data, size + 1))),
                None,
            )
        }
        Code::Char('/') => {
            tokenizer.consume(code);
            (
                State::Fn(Box::new(move |t, c| escape(t, c, true, size + 1))),
                None,
            )
        }
        Code::Char(_) => {
            tokenizer.consume(code);
            (
                State::Fn(Box::new(move |t, c| label(t, c, true, size + 1))),
                None,
            )
        }
    }
}

/// To do.
fn escape(tokenizer: &mut Tokenizer, code: Code, data: bool, size: usize) -> StateFnResult {
    match code {
        Code::Char('[' | '\\' | ']') => {
            tokenizer.consume(code);
            (
                State::Fn(Box::new(move |t, c| label(t, c, true, size + 1))),
                None,
            )
        }
        _ => label(tokenizer, code, data, size),
    }
}
