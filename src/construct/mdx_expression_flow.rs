//! To do.

use crate::construct::partial_space_or_tab::{space_or_tab, space_or_tab_min_max};
use crate::event::Name;
use crate::state::{Name as StateName, State};
use crate::tokenizer::Tokenizer;
use crate::util::constant::TAB_SIZE;

/// Start of MDX: expression (flow).
///
/// ```markdown
/// > | {Math.PI}
///     ^
/// ```
pub fn start(tokenizer: &mut Tokenizer) -> State {
    if tokenizer.parse_state.options.constructs.mdx_expression_flow {
        tokenizer.tokenize_state.token_1 = Name::MdxFlowExpression;
        tokenizer.concrete = true;
        if matches!(tokenizer.current, Some(b'\t' | b' ')) {
            tokenizer.attempt(State::Next(StateName::MdxExpressionFlowBefore), State::Nok);
            State::Retry(space_or_tab_min_max(
                tokenizer,
                0,
                if tokenizer.parse_state.options.constructs.code_indented {
                    TAB_SIZE - 1
                } else {
                    usize::MAX
                },
            ))
        } else {
            State::Retry(StateName::MdxExpressionFlowBefore)
        }
    } else {
        State::Nok
    }
}

/// After optional whitespace, before of MDX expression (flow).
///
/// ```markdown
/// > | {Math.PI}
///     ^
/// ```
pub fn before(tokenizer: &mut Tokenizer) -> State {
    if Some(b'{') == tokenizer.current {
        tokenizer.attempt(State::Next(StateName::MdxExpressionFlowAfter), State::Nok);
        State::Retry(StateName::MdxExpressionStart)
    } else {
        State::Nok
    }
}

/// After an MDX expression (flow).
///
/// ```markdown
/// > | {Math.PI}
///              ^
/// ```
pub fn after(tokenizer: &mut Tokenizer) -> State {
    match tokenizer.current {
        Some(b'\t' | b' ') => {
            tokenizer.attempt(State::Next(StateName::MdxExpressionFlowEnd), State::Nok);
            State::Retry(space_or_tab(tokenizer))
        }
        _ => State::Retry(StateName::MdxExpressionFlowEnd),
    }
}

/// After an MDX expression (flow), after optional whitespace.
///
/// ```markdown
/// > | {Math.PI}␠␊
///               ^
/// ```
pub fn end(tokenizer: &mut Tokenizer) -> State {
    tokenizer.concrete = false;
    tokenizer.tokenize_state.token_1 = Name::Data;

    if matches!(tokenizer.current, None | Some(b'\n')) {
        State::Ok
    } else {
        State::Nok
    }
}
