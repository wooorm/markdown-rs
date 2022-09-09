//! MDX expression occurs in [MDX expression (flow)][mdx_expression_flow] and
//! [MDX expression (text)][mdx_expression_text].
//!
//! ## Grammar
//!
//! MDX expression forms with the following BNF
//! (<small>see [construct][crate::construct] for character groups</small>):
//!
//! ```bnf
//! mdx_expression ::= '{' *(expression_text | expression) '}'
//! expression_text ::= char - '{' - '}'
//! ```
//!
//! ## Tokens
//!
//! *   [`LineEnding`][Name::LineEnding]
//! *   [`SpaceOrTab`][Name::SpaceOrTab]
//! *   [`MdxExpressionMarker`][Name::MdxExpressionMarker]
//! *   [`MdxExpressionData`][Name::MdxExpressionData]
//!
//! ## Recommendation
//!
//! When authoring markdown with JavaScript, keep in mind that MDX is a
//! whitespace sensitive and line-based language, while JavaScript is
//! insensitive to whitespace.
//! This affects how markdown and JavaScript interleave with eachother in MDX.
//! For more info on how it works, see [§ Interleaving][interleaving] on the
//! MDX site.
//!
//! ## Errors
//!
//! ### Unexpected end of file in expression, expected a corresponding closing brace for `{`
//!
//! This error occurs if a `{` was seen without a `}`.
//! For example:
//!
//! ```markdown
//! a { b
//! ```
//!
//! ### Unexpected lazy line in expression in container, expected line to be prefixed with `>` when in a block quote, whitespace when in a list, etc
//!
//! This error occurs if a a lazy line (of a container) is found in an expression.
//! For example:
//!
//! ```markdown
//! > {a +
//! b}
//! ```
//!
//! ## References
//!
//! *   [`micromark-factory-mdx-expression`](https://github.com/micromark/micromark-extension-mdx-expression/blob/main/packages/micromark-factory-mdx-expression/dev/index.js)
//! *   [`mdxjs.com`](https://mdxjs.com)
//!
//! [mdx_expression_flow]: crate::construct::mdx_expression_flow
//! [mdx_expression_text]: crate::construct::mdx_expression_text
//! [interleaving]: https://mdxjs.com/docs/what-is-mdx/#interleaving

use crate::construct::partial_space_or_tab::space_or_tab_min_max;
use crate::event::Name;
use crate::state::{Name as StateName, State};
use crate::tokenizer::Tokenizer;
use alloc::format;

/// Start of an MDX expression.
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

/// Before data.
///
/// ```markdown
/// > | a {Math.PI} c
///        ^
/// ```
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
            if tokenizer.tokenize_state.token_1 == Name::MdxJsxTagAttributeValueExpression && !tokenizer.tokenize_state.seen {
                State::Error(format!(
                    "{}:{}: Unexpected empty expression, expected a value between braces",
                    tokenizer.point.line, tokenizer.point.column
                ))
            } else {
                tokenizer.tokenize_state.seen = false;
                tokenizer.enter(Name::MdxExpressionMarker);
                tokenizer.consume();
                tokenizer.exit(Name::MdxExpressionMarker);
                tokenizer.exit(tokenizer.tokenize_state.token_1.clone());
                State::Ok
            }
        },
        Some(_) => {
            tokenizer.tokenize_state.seen = true;
            tokenizer.enter(Name::MdxExpressionData);
            State::Retry(StateName::MdxExpressionInside)
        }
    }
}

/// In data.
///
/// ```markdown
/// > | a {Math.PI} c
///        ^
/// ```
pub fn inside(tokenizer: &mut Tokenizer) -> State {
    if matches!(tokenizer.current, None | Some(b'\n'))
        || (tokenizer.current == Some(b'}') && tokenizer.tokenize_state.size == 0)
    {
        tokenizer.exit(Name::MdxExpressionData);
        State::Retry(StateName::MdxExpressionBefore)
    } else {
        // To do: don’t count if gnostic.
        if tokenizer.current == Some(b'{') {
            tokenizer.tokenize_state.size += 1;
        } else if tokenizer.current == Some(b'}') {
            tokenizer.tokenize_state.size -= 1;
        }

        tokenizer.consume();
        State::Next(StateName::MdxExpressionInside)
    }
}

/// After eol.
///
/// ```markdown
///   | a {b +
/// > | c} d
///     ^
/// ```
pub fn eol_after(tokenizer: &mut Tokenizer) -> State {
    // Lazy continuation in a flow expression (or flow tag) is a syntax error.
    if (tokenizer.tokenize_state.token_1 == Name::MdxFlowExpression
        || tokenizer.tokenize_state.token_2 == Name::MdxJsxFlowTag)
        && tokenizer.lazy
    {
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
