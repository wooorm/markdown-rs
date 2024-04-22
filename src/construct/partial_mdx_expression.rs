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
use crate::message;
use crate::state::{Name as StateName, State};
use crate::tokenizer::Tokenizer;
use crate::util::{constant::TAB_SIZE, mdx_collect::collect};
use crate::{MdxExpressionKind, MdxExpressionParse, MdxSignal};
use alloc::boxed::Box;

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
    tokenizer.tokenize_state.start = tokenizer.events.len() - 1;
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
            let problem = tokenizer.tokenize_state.mdx_last_parse_error.take()
                        .unwrap_or_else(|| ("Unexpected end of file in expression, expected a corresponding closing brace for `{`".into(), "markdown-rs".into(), "unexpected-eof".into()));

            State::Error(message::Message {
                place: Some(Box::new(message::Place::Point(tokenizer.point.to_unist()))),
                reason: problem.0,
                rule_id: Box::new(problem.2),
                source: Box::new(problem.1),
            })
        }
        Some(b'\n') => {
            tokenizer.enter(Name::LineEnding);
            tokenizer.consume();
            tokenizer.exit(Name::LineEnding);
            State::Next(StateName::MdxExpressionEolAfter)
        }
        Some(b'}') if tokenizer.tokenize_state.size == 0 => {
            let state = if let Some(ref parse) = tokenizer.parse_state.options.mdx_expression_parse
            {
                parse_expression(tokenizer, parse)
            } else {
                State::Ok
            };

            if state == State::Ok {
                tokenizer.tokenize_state.start = 0;
                tokenizer.enter(Name::MdxExpressionMarker);
                tokenizer.consume();
                tokenizer.exit(Name::MdxExpressionMarker);
                tokenizer.exit(tokenizer.tokenize_state.token_1.clone());
            }

            state
        }
        Some(_) => {
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
        // Don’t count if gnostic.
        if tokenizer.current == Some(b'{')
            && tokenizer.parse_state.options.mdx_expression_parse.is_none()
        {
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
        State::Error(
            message::Message {
                place: Some(Box::new(message::Place::Point(tokenizer.point.to_unist()))),
                reason: "Unexpected lazy line in expression in container, expected line to be prefixed with `>` when in a block quote, whitespace when in a list, etc".into(),
                source: Box::new("markdown-rs".into()),
                rule_id: Box::new("unexpected-lazy".into()),
            }
        )
    } else if matches!(tokenizer.current, Some(b'\t' | b' ')) {
        tokenizer.attempt(State::Next(StateName::MdxExpressionBefore), State::Nok);
        // Idea: investigate if we’d need to use more complex stripping.
        // Take this example:
        //
        // ```markdown
        // >  aaa <b c={`
        // >      d
        // >  `} /> eee
        // ```
        //
        // Currently, the “paragraph” starts at `> | aaa`, so for the next line
        // here we split it into `>␠|␠␠␠␠|␠d` (prefix, this indent here,
        // expression data).
        // The intention above is likely for the split to be as `>␠␠|␠␠␠␠|d`,
        // which is impossible, but we can mimick it with `>␠|␠␠␠␠␠|d`.
        //
        // To improve the situation, we could take `tokenizer.line_start` at
        // the start of the expression and move past whitespace.
        // For future lines, we’d move at most to
        // `line_start_shifted.column + 4`.
        State::Retry(space_or_tab_min_max(tokenizer, 0, TAB_SIZE))
    } else {
        State::Retry(StateName::MdxExpressionBefore)
    }
}

/// Parse an expression with a given function.
fn parse_expression(tokenizer: &mut Tokenizer, parse: &MdxExpressionParse) -> State {
    // Collect the body of the expression and positional info for each run of it.
    let result = collect(
        &tokenizer.events,
        tokenizer.parse_state.bytes,
        tokenizer.tokenize_state.start,
        &[Name::MdxExpressionData, Name::LineEnding],
        &[],
    );

    // Turn the name of the expression into a kind.
    let kind = match tokenizer.tokenize_state.token_1 {
        Name::MdxFlowExpression | Name::MdxTextExpression => MdxExpressionKind::Expression,
        Name::MdxJsxTagAttributeExpression => MdxExpressionKind::AttributeExpression,
        Name::MdxJsxTagAttributeValueExpression => MdxExpressionKind::AttributeValueExpression,
        _ => unreachable!("cannot handle unknown expression name"),
    };

    // Parse and handle what was signaled back.
    match parse(&result.value, &kind) {
        MdxSignal::Ok => State::Ok,
        MdxSignal::Error(reason, relative, source, rule_id) => {
            let point = tokenizer
                .parse_state
                .location
                .as_ref()
                .expect("expected location index if aware mdx is on")
                .relative_to_point(&result.stops, relative)
                .unwrap_or_else(|| tokenizer.point.to_unist());

            State::Error(message::Message {
                place: Some(Box::new(message::Place::Point(point))),
                reason,
                rule_id,
                source,
            })
        }
        MdxSignal::Eof(reason, source, rule_id) => {
            tokenizer.tokenize_state.mdx_last_parse_error = Some((reason, *source, *rule_id));
            tokenizer.enter(Name::MdxExpressionData);
            tokenizer.consume();
            State::Next(StateName::MdxExpressionInside)
        }
    }
}
