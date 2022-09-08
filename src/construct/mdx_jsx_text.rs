//! To do.

use crate::event::Name;
use crate::state::{Name as StateName, State};
use crate::tokenizer::Tokenizer;

/// Start of MDX: JSX (text).
///
/// ```markdown
/// > | a <B /> c
///       ^
/// ```
pub fn start(tokenizer: &mut Tokenizer) -> State {
    if Some(b'<') == tokenizer.current && tokenizer.parse_state.options.constructs.mdx_jsx_text {
        tokenizer.tokenize_state.token_1 = Name::MdxJsxTextTag;
        tokenizer.attempt(State::Next(StateName::MdxJsxTextAfter), State::Next(StateName::MdxJsxTextNok));
        State::Retry(StateName::MdxJsxStart)
    } else {
        State::Nok
    }
}

/// To do
pub fn after(tokenizer: &mut Tokenizer) -> State {
    tokenizer.tokenize_state.token_1 = Name::Data;
    State::Ok
}

/// To do
pub fn nok(tokenizer: &mut Tokenizer) -> State {
    tokenizer.tokenize_state.token_1 = Name::Data;
    State::Nok
}
