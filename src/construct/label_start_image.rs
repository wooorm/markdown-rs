//! To do

use super::label_end::resolve_media;
use crate::tokenizer::{Code, LabelStart, State, StateFnResult, TokenType, Tokenizer};

/// Start of label (image) start.
///
/// ```markdown
/// a |![ b
/// ```
pub fn start(tokenizer: &mut Tokenizer, code: Code) -> StateFnResult {
    match code {
        Code::Char('!') => {
            tokenizer.enter(TokenType::LabelImage);
            tokenizer.enter(TokenType::LabelImageMarker);
            tokenizer.consume(code);
            tokenizer.exit(TokenType::LabelImageMarker);
            (State::Fn(Box::new(open)), None)
        }
        _ => (State::Nok, None),
    }
}

/// After `!`, before a `[`.
///
/// ```markdown
/// a !|[ b
/// ```
pub fn open(tokenizer: &mut Tokenizer, code: Code) -> StateFnResult {
    match code {
        Code::Char('[') => {
            tokenizer.enter(TokenType::LabelMarker);
            tokenizer.consume(code);
            tokenizer.exit(TokenType::LabelMarker);
            tokenizer.exit(TokenType::LabelImage);
            let end = tokenizer.events.len() - 1;
            tokenizer.label_start_stack.push(LabelStart {
                start: (end - 5, end),
                balanced: false,
                inactive: false,
            });
            tokenizer.register_resolver("media".to_string(), Box::new(resolve_media));
            (State::Ok, None)
        }
        _ => (State::Nok, None),
    }
}
