// To do: pass token types in.

use crate::tokenizer::{Code, State, StateFnResult, TokenType, Tokenizer};

pub fn start(tokenizer: &mut Tokenizer, code: Code) -> StateFnResult {
    match code {
        Code::Char('<') => {
            tokenizer.enter(TokenType::DefinitionDestination);
            tokenizer.enter(TokenType::DefinitionDestinationLiteral);
            tokenizer.enter(TokenType::DefinitionDestinationLiteralMarker);
            tokenizer.consume(code);
            tokenizer.exit(TokenType::DefinitionDestinationLiteralMarker);
            (State::Fn(Box::new(enclosed_before)), None)
        }
        Code::None | Code::CarriageReturnLineFeed | Code::VirtualSpace | Code::Char(')') => {
            (State::Nok, None)
        }
        Code::Char(char) if char.is_ascii_control() => (State::Nok, None),
        Code::Char(_) => {
            tokenizer.enter(TokenType::DefinitionDestination);
            tokenizer.enter(TokenType::DefinitionDestinationRaw);
            tokenizer.enter(TokenType::DefinitionDestinationString);
            // To do: link.
            tokenizer.enter(TokenType::ChunkString);
            raw(tokenizer, code, 0)
        }
    }
}

/// To do.
fn enclosed_before(tokenizer: &mut Tokenizer, code: Code) -> StateFnResult {
    if let Code::Char('>') = code {
        tokenizer.enter(TokenType::DefinitionDestinationLiteralMarker);
        tokenizer.consume(code);
        tokenizer.exit(TokenType::DefinitionDestinationLiteralMarker);
        tokenizer.exit(TokenType::DefinitionDestinationLiteral);
        tokenizer.exit(TokenType::DefinitionDestination);
        (State::Ok, None)
    } else {
        tokenizer.enter(TokenType::DefinitionDestinationString);
        // To do: link.
        tokenizer.enter(TokenType::ChunkString);
        enclosed(tokenizer, code)
    }
}

/// To do.
fn enclosed(tokenizer: &mut Tokenizer, code: Code) -> StateFnResult {
    match code {
        Code::Char('>') => {
            tokenizer.exit(TokenType::ChunkString);
            tokenizer.exit(TokenType::DefinitionDestinationString);
            enclosed_before(tokenizer, code)
        }
        Code::None | Code::CarriageReturnLineFeed | Code::Char('\r' | '\n' | '<') => {
            (State::Nok, None)
        }
        Code::Char('\\') => {
            tokenizer.consume(code);
            (State::Fn(Box::new(enclosed_escape)), None)
        }
        _ => {
            tokenizer.consume(code);
            (State::Fn(Box::new(enclosed)), None)
        }
    }
}

/// To do.
fn enclosed_escape(tokenizer: &mut Tokenizer, code: Code) -> StateFnResult {
    match code {
        Code::Char('<' | '>' | '\\') => {
            tokenizer.consume(code);
            (State::Fn(Box::new(enclosed)), None)
        }
        _ => enclosed(tokenizer, code),
    }
}

/// To do.
// To do: these arms can be improved?
fn raw(tokenizer: &mut Tokenizer, code: Code, balance: usize) -> StateFnResult {
    // To do: configurable.
    let limit = usize::MAX;

    match code {
        Code::Char('(') if balance >= limit => (State::Nok, None),
        Code::Char('(') => {
            tokenizer.consume(code);
            (
                State::Fn(Box::new(move |t, c| raw(t, c, balance + 1))),
                None,
            )
        }
        Code::Char(')') if balance == 0 => {
            tokenizer.exit(TokenType::ChunkString);
            tokenizer.exit(TokenType::DefinitionDestinationString);
            tokenizer.exit(TokenType::DefinitionDestinationRaw);
            tokenizer.exit(TokenType::DefinitionDestination);
            (State::Ok, Some(vec![code]))
        }
        Code::Char(')') => {
            tokenizer.consume(code);
            (
                State::Fn(Box::new(move |t, c| raw(t, c, balance - 1))),
                None,
            )
        }
        Code::None
        | Code::CarriageReturnLineFeed
        | Code::VirtualSpace
        | Code::Char('\t' | '\r' | '\n' | ' ')
            if balance > 0 =>
        {
            (State::Nok, None)
        }
        Code::None
        | Code::CarriageReturnLineFeed
        | Code::VirtualSpace
        | Code::Char('\t' | '\r' | '\n' | ' ') => {
            tokenizer.exit(TokenType::ChunkString);
            tokenizer.exit(TokenType::DefinitionDestinationString);
            tokenizer.exit(TokenType::DefinitionDestinationRaw);
            tokenizer.exit(TokenType::DefinitionDestination);
            (State::Ok, Some(vec![code]))
        }
        Code::Char(char) if char.is_ascii_control() => (State::Nok, None),
        Code::Char('\\') => {
            tokenizer.consume(code);
            (
                State::Fn(Box::new(move |t, c| raw_escape(t, c, balance))),
                None,
            )
        }
        Code::Char(_) => {
            tokenizer.consume(code);
            (State::Fn(Box::new(move |t, c| raw(t, c, balance))), None)
        }
    }
}

/// To do.
fn raw_escape(tokenizer: &mut Tokenizer, code: Code, balance: usize) -> StateFnResult {
    match code {
        Code::Char('(' | ')' | '\\') => {
            tokenizer.consume(code);
            (
                State::Fn(Box::new(move |t, c| raw(t, c, balance + 1))),
                None,
            )
        }
        _ => raw(tokenizer, code, balance),
    }
}
