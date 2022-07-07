//! To do.

use crate::constant::TAB_SIZE;
use crate::construct::partial_space_or_tab::space_or_tab_min_max;
use crate::tokenizer::{Code, State, StateFnResult, TokenType, Tokenizer};

pub fn start(tokenizer: &mut Tokenizer, code: Code) -> StateFnResult {
    // To do: allow arbitrary when code (indented) is turned off.
    tokenizer.go(space_or_tab_min_max(0, TAB_SIZE - 1), before)(tokenizer, code)
}

fn before(tokenizer: &mut Tokenizer, code: Code) -> StateFnResult {
    match code {
        Code::Char('>') => {
            tokenizer.enter(TokenType::BlockQuote);
            cont_before(tokenizer, code)
        }
        _ => cont_before(tokenizer, code),
    }
}

pub fn cont(tokenizer: &mut Tokenizer, code: Code) -> StateFnResult {
    // To do: allow arbitrary when code (indented) is turned off.
    tokenizer.go(space_or_tab_min_max(0, TAB_SIZE - 1), cont_before)(tokenizer, code)
}

fn cont_before(tokenizer: &mut Tokenizer, code: Code) -> StateFnResult {
    match code {
        Code::Char('>') => {
            tokenizer.enter(TokenType::BlockQuotePrefix);
            tokenizer.enter(TokenType::BlockQuoteMarker);
            tokenizer.consume(code);
            tokenizer.exit(TokenType::BlockQuoteMarker);
            (State::Fn(Box::new(cont_after)), None)
        }
        _ => (State::Nok, None),
    }
}

fn cont_after(tokenizer: &mut Tokenizer, code: Code) -> StateFnResult {
    match code {
        Code::VirtualSpace | Code::Char('\t' | ' ') => {
            tokenizer.enter(TokenType::BlockQuotePrefixWhitespace);
            tokenizer.consume(code);
            tokenizer.exit(TokenType::BlockQuotePrefixWhitespace);
            tokenizer.exit(TokenType::BlockQuotePrefix);
            (State::Ok, None)
        }
        _ => {
            tokenizer.exit(TokenType::BlockQuotePrefix);
            (State::Ok, Some(vec![code]))
        }
    }
}

pub fn end() -> Vec<TokenType> {
    vec![TokenType::BlockQuote]
}
