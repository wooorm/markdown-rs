//! Byte order mark occurs at the start of the document.
//!
//! It’s the three bytes 0xEF, 0xBB, and 0xBF.
//!
//! ## Tokens
//!
//! *   [`ByteOrderMark`][Token::ByteOrderMark]
//!
//! ## References
//!
//! *   [`micromark/lib/preprocess.js` in `micromark`](https://github.com/micromark/micromark/blob/ed23453/packages/micromark/dev/lib/preprocess.js#L54-L60)

use crate::event::Name;
use crate::state::{Name as StateName, State};
use crate::tokenizer::Tokenizer;

const BOM: [u8; 3] = [0xEF, 0xBB, 0xBF];

/// Before a BOM.
///
/// ```text
/// > | 0xEF 0xBB 0xBF
///     ^^^^
/// ```
pub fn start(tokenizer: &mut Tokenizer) -> State {
    if tokenizer.current == Some(BOM[0]) {
        tokenizer.enter(Name::ByteOrderMark);
        State::Retry(StateName::BomInside)
    } else {
        State::Nok
    }
}

/// Inside the BOM.
///
/// ```text
/// > | 0xEF 0xBB 0xBF
///     ^^^^ ^^^^ ^^^^
/// ```
pub fn inside(tokenizer: &mut Tokenizer) -> State {
    if tokenizer.current == Some(BOM[tokenizer.tokenize_state.size]) {
        tokenizer.tokenize_state.size += 1;
        tokenizer.consume();
        if tokenizer.tokenize_state.size == BOM.len() {
            tokenizer.exit(Name::ByteOrderMark);
            tokenizer.tokenize_state.size = 0;
            State::Ok
        } else {
            State::Next(StateName::BomInside)
        }
    } else {
        tokenizer.tokenize_state.size = 0;
        State::Nok
    }
}
