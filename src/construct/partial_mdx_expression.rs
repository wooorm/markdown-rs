//! To do.

use crate::construct::partial_space_or_tab::space_or_tab_min_max;
use crate::event::Name;
use crate::state::{Name as StateName, State};
use crate::tokenizer::Tokenizer;
use alloc::format;

/// Start of MDX: expression.
///
/// ```markdown
/// > | a {Math.PI} c
///       ^
/// ```
pub fn start(tokenizer: &mut Tokenizer) -> State {
    debug_assert_eq!(tokenizer.current, Some(b'{'));
    tokenizer.enter(tokenizer.tokenize_state.token_1.clone());
    tokenizer.enter(Name::MdxExpressionMarker);
    tokenizer.consume();
    tokenizer.exit(Name::MdxExpressionMarker);
    State::Next(StateName::MdxExpressionBefore)
}

pub fn before(tokenizer: &mut Tokenizer) -> State {
    match tokenizer.current {
        None => {
            State::Error(format!(
                "{}:{}: Unexpected end of file in expression, expected a corresponding closing brace for `{{`",
                tokenizer.point.line, tokenizer.point.column
            ))
        }
        Some(b'\n') => {
            tokenizer.enter(Name::LineEnding);
            tokenizer.consume();
            tokenizer.exit(Name::LineEnding);
            State::Next(StateName::MdxExpressionEolAfter)
        },
        Some(b'}') if tokenizer.tokenize_state.size == 0 => {
            tokenizer.enter(Name::MdxExpressionMarker);
            tokenizer.consume();
            tokenizer.exit(Name::MdxExpressionMarker);
            tokenizer.exit(tokenizer.tokenize_state.token_1.clone());
            State::Ok
        },
        Some(_) => {
            tokenizer.enter(Name::MdxExpressionData);
            State::Retry(StateName::MdxExpressionInside)
        }
    }
}

pub fn inside(tokenizer: &mut Tokenizer) -> State {
    if matches!(tokenizer.current, None | Some(b'\n'))
        || (tokenizer.current == Some(b'}') && tokenizer.tokenize_state.size == 0)
    {
        tokenizer.exit(Name::MdxExpressionData);
        State::Retry(StateName::MdxExpressionBefore)
    } else {
        // To do: only count if agnostic.
        if tokenizer.current == Some(b'{') {
            tokenizer.tokenize_state.size += 1;
        }

        tokenizer.consume();
        State::Next(StateName::MdxExpressionInside)
    }
}

pub fn eol_after(tokenizer: &mut Tokenizer) -> State {
    // Lazy continuation in a flow expression is a syntax error.
    if tokenizer.tokenize_state.token_1 == Name::MdxFlowExpression && tokenizer.lazy {
        State::Error(format!(
            "{}:{}: Unexpected lazy line in expression in container, expected line to be prefixed with `>` when in a block quote, whitespace when in a list, etc",
            tokenizer.point.line, tokenizer.point.column
        ))
    } else if matches!(tokenizer.current, Some(b'\t' | b' ')) {
        tokenizer.attempt(State::Next(StateName::MdxExpressionBefore), State::Nok);
        // To do: use `start_column` + constants.tabSize for max space to eat.
        State::Next(space_or_tab_min_max(tokenizer, 0, usize::MAX))
    } else {
        State::Retry(StateName::MdxExpressionBefore)
    }
}
