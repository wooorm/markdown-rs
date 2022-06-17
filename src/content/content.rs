//! The `content`, ahum, content type.
//!
//! **Content** is zero or more definitions, and then zero or one paragraph.
//! It’s a weird one, and needed to make certain edge cases around definitions
//! spec compliant.
//! Definitions are unlike other things in markdown, in that they behave like
//! **text** in that they can contain arbitrary line endings, but *have* to end
//! at a line ending.
//! If they end in something else, the whole definition instead is seen as a
//! paragraph.
//!
//! The constructs found in content are:
//!
//! *   Definition
//! *   Paragraph

use crate::tokenizer::{Code, State, StateFnResult, TokenType, Tokenizer};

/// Before a paragraph.
///
/// ```markdown
/// |asd
/// ```
pub fn start(tokenizer: &mut Tokenizer, code: Code) -> StateFnResult {
    match code {
        Code::None | Code::CarriageReturnLineFeed | Code::Char('\n' | '\r') => {
            unreachable!("expected non-eol/eof");
        }
        _ => {
            tokenizer.enter(TokenType::Paragraph);
            tokenizer.enter(TokenType::ChunkText);
            inside(tokenizer, code, tokenizer.events.len() - 1)
        }
    }
}

/// In a line in a paragraph.
///
/// ```markdown
/// |\&
/// |qwe
/// ```
fn inside(tokenizer: &mut Tokenizer, code: Code, previous_index: usize) -> StateFnResult {
    match code {
        Code::None => {
            tokenizer.exit(TokenType::ChunkText);
            tokenizer.exit(TokenType::Paragraph);
            (State::Ok, None)
        }
        Code::CarriageReturnLineFeed | Code::Char('\n' | '\r') => {
            tokenizer.consume(code);
            tokenizer.exit(TokenType::ChunkText);
            tokenizer.enter(TokenType::ChunkText);
            let next_index = tokenizer.events.len() - 1;
            tokenizer.events[previous_index].next = Some(next_index);
            tokenizer.events[next_index].previous = Some(previous_index);
            (
                State::Fn(Box::new(move |t, c| inside(t, c, next_index))),
                None,
            )
        }
        _ => {
            tokenizer.consume(code);
            (
                State::Fn(Box::new(move |t, c| inside(t, c, previous_index))),
                None,
            )
        }
    }
}
