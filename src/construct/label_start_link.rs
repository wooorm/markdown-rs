//! To do

use super::label_end::resolve_media;
use crate::tokenizer::{Code, LabelStart, State, StateFnResult, TokenType, Tokenizer};

/// Start of label (link) start.
///
/// ```markdown
/// a |[ b
/// ```
pub fn start(tokenizer: &mut Tokenizer, code: Code) -> StateFnResult {
    match code {
        Code::Char('[') => {
            let start = tokenizer.events.len();
            tokenizer.enter(TokenType::LabelLink);
            tokenizer.enter(TokenType::LabelMarker);
            tokenizer.consume(code);
            tokenizer.exit(TokenType::LabelMarker);
            tokenizer.exit(TokenType::LabelLink);
            tokenizer.label_start_stack.push(LabelStart {
                start: (start, tokenizer.events.len() - 1),
                balanced: false,
                inactive: false,
            });
            tokenizer.register_resolver("media".to_string(), Box::new(resolve_media));
            (State::Ok, None)
        }
        _ => (State::Nok, None),
    }
}
