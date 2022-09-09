//! MDX expression (flow) occurs in the [flow][] content type.
//!
//! ## Grammar
//!
//! MDX expression (flow) forms with the following BNF
//! (<small>see [construct][crate::construct] for character groups</small>):
//!
//! ```bnf
//! mdx_expression_flow ::= mdx_expression *space_or_tab
//!
//! ; See the `partial_mdx_expression` construct for the BNF of that part.
//! ```
//!
//! As this construct occurs in flow, like all flow constructs, it must be
//! followed by an eol (line ending) or eof (end of file).
//!
//! See [`mdx_expression`][mdx_expression] for more info.
//!
//! ## Tokens
//!
//! *   [`MdxFlowExpression`][Name::MdxFlowExpression]
//! *   [`SpaceOrTab`][Name::SpaceOrTab]
//! *   see [`mdx_expression`][mdx_expression] for more
//!
//! ## Recommendation
//!
//! See [`mdx_expression`][mdx_expression] for recommendations.
//!
//! ## References
//!
//! *   [`syntax.js` in `micromark-extension-mdx-expression`](https://github.com/micromark/micromark-extension-mdx-expression/blob/main/packages/micromark-extension-mdx-expression/dev/lib/syntax.js)
//! *   [`mdxjs.com`](https://mdxjs.com)
//!
//! [flow]: crate::construct::flow
//! [mdx_expression]: crate::construct::partial_mdx_expression

use crate::construct::partial_space_or_tab::{space_or_tab, space_or_tab_min_max};
use crate::event::Name;
use crate::state::{Name as StateName, State};
use crate::tokenizer::Tokenizer;
use crate::util::constant::TAB_SIZE;

/// Start of an MDX expression (flow).
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

/// After optional whitespace, before expression.
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

/// After expression.
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

/// After expression, after optional whitespace.
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
