//! To do.

use crate::event::Name;
use crate::state::{Name as StateName, State};
use crate::tokenizer::Tokenizer;

/// Start of MDX: expression (text).
///
/// ```markdown
/// > | a {Math.PI} c
///       ^
/// ```
pub fn start(tokenizer: &mut Tokenizer) -> State {
    if Some(b'{') == tokenizer.current
        && tokenizer.parse_state.options.constructs.mdx_expression_text
    {
        tokenizer.tokenize_state.token_1 = Name::MdxTextExpression;
        tokenizer.attempt(State::Next(StateName::MdxExpressionTextAfter), State::Nok);
        State::Retry(StateName::MdxExpressionStart)
    } else {
        State::Nok
    }
}

/// After an MDX expression (text) tag.
///
/// ```markdown
/// > | a {Math.PI} c
///                ^
/// ```
pub fn after(tokenizer: &mut Tokenizer) -> State {
    tokenizer.tokenize_state.token_1 = Name::Data;
    State::Ok
}
