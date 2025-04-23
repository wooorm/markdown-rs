//! Bridge between `markdown-rs` and SWC.

use crate::test_utils::swc_utils::{create_span, RewritePrefixContext};
use markdown::{MdxExpressionKind, MdxSignal};
use std::rc::Rc;
use swc_core::common::{
    comments::{Comment, SingleThreadedComments, SingleThreadedCommentsMap},
    source_map::SmallPos,
    BytePos, FileName, SourceFile, Span, Spanned,
};
use swc_core::ecma::ast::{EsVersion, Expr, Module, PropOrSpread};
use swc_core::ecma::parser::{
    error::Error as SwcError, parse_file_as_expr, parse_file_as_module, EsSyntax, Syntax,
};
use swc_core::ecma::visit::VisitMutWith;

/// Lex ESM in MDX with SWC.
pub fn parse_esm(value: &str) -> MdxSignal {
    let result = parse_esm_core(value);

    match result {
        Err((span, message)) => swc_error_to_signal(span, &message, value.len()),
        Ok(_) => MdxSignal::Ok,
    }
}

/// Core to parse ESM.
fn parse_esm_core(value: &str) -> Result<Module, (Span, String)> {
    let (file, syntax, version) = create_config(value.into());
    let mut errors = vec![];
    let result = parse_file_as_module(&file, syntax, version, None, &mut errors);

    match result {
        Err(error) => Err((
            fix_span(error.span(), 1),
            format!(
                "Could not parse esm with swc: {}",
                swc_error_to_string(&error)
            ),
        )),
        Ok(module) => {
            if errors.is_empty() {
                let mut index = 0;
                while index < module.body.len() {
                    let node = &module.body[index];

                    if !node.is_module_decl() {
                        return Err((
                            fix_span(node.span(), 1),
                            "Unexpected statement in code: only import/exports are supported"
                                .into(),
                        ));
                    }

                    index += 1;
                }

                Ok(module)
            } else {
                Err((
                    fix_span(errors[0].span(), 1),
                    format!(
                        "Could not parse esm with swc: {}",
                        swc_error_to_string(&errors[0])
                    ),
                ))
            }
        }
    }
}

fn parse_expression_core(
    value: &str,
    kind: &MdxExpressionKind,
) -> Result<Option<Box<Expr>>, (Span, String)> {
    // Empty expressions are OK.
    if matches!(kind, MdxExpressionKind::Expression) && whitespace_and_comments(0, value).is_ok() {
        return Ok(None);
    }

    // For attribute expression, a spread is needed, for which we have to prefix
    // and suffix the input.
    // See `check_expression_ast` for how the AST is verified.
    let (prefix, suffix) = if matches!(kind, MdxExpressionKind::AttributeExpression) {
        ("({", "})")
    } else {
        ("", "")
    };

    let (file, syntax, version) = create_config(format!("{}{}{}", prefix, value, suffix));
    let mut errors = vec![];
    let result = parse_file_as_expr(&file, syntax, version, None, &mut errors);

    match result {
        Err(error) => Err((
            fix_span(error.span(), prefix.len() + 1),
            format!(
                "Could not parse expression with swc: {}",
                swc_error_to_string(&error)
            ),
        )),
        Ok(mut expr) => {
            if errors.is_empty() {
                let expression_end = expr.span().hi.to_usize() - 1;
                if let Err((span, reason)) = whitespace_and_comments(expression_end, value) {
                    return Err((span, reason));
                }

                expr.visit_mut_with(&mut RewritePrefixContext {
                    prefix_len: prefix.len() as u32,
                });

                if matches!(kind, MdxExpressionKind::AttributeExpression) {
                    let expr_span = expr.span();

                    if let Expr::Paren(d) = *expr {
                        if let Expr::Object(mut obj) = *d.expr {
                            if obj.props.len() > 1 {
                                return Err((obj.span, "Unexpected extra content in spread (such as `{...x,y}`): only a single spread is supported (such as `{...x}`)".into()));
                            }

                            if let Some(PropOrSpread::Spread(d)) = obj.props.pop() {
                                return Ok(Some(d.expr));
                            }
                        }
                    };

                    return Err((
                        expr_span,
                        "Unexpected prop in spread (such as `{x}`): only a spread is supported (such as `{...x}`)".into(),
                    ));
                }

                Ok(Some(expr))
            } else {
                Err((
                    fix_span(errors[0].span(), prefix.len() + 1),
                    format!(
                        "Could not parse expression with swc: {}",
                        swc_error_to_string(&errors[0])
                    ),
                ))
            }
        }
    }
}

/// Lex expressions in MDX with SWC.
pub fn parse_expression(value: &str, kind: &MdxExpressionKind) -> MdxSignal {
    let result = parse_expression_core(value, kind);

    match result {
        Err((span, message)) => swc_error_to_signal(span, &message, value.len()),
        Ok(_) => MdxSignal::Ok,
    }
}

// To do: remove this attribute, use it somewhere.
#[allow(dead_code)]
/// Turn SWC comments into a flat vec.
pub fn flat_comments(single_threaded_comments: SingleThreadedComments) -> Vec<Comment> {
    let raw_comments = single_threaded_comments.take_all();
    let take = |list: SingleThreadedCommentsMap| {
        Rc::try_unwrap(list)
            .unwrap()
            .into_inner()
            .into_values()
            .flatten()
            .collect::<Vec<_>>()
    };
    let mut list = take(raw_comments.0);
    list.append(&mut take(raw_comments.1));
    list
}

/// Turn an SWC error into an `MdxSignal`.
///
/// * If the error happens at `value_len`, yields `MdxSignal::Eof`
/// * Else, yields `MdxSignal::Error`.
fn swc_error_to_signal(span: Span, reason: &str, value_len: usize) -> MdxSignal {
    let source = Box::new("mdx".into());
    let rule_id = Box::new("swc".into());
    let error_end = span.hi.to_usize();

    if error_end >= value_len {
        MdxSignal::Eof(reason.into(), source, rule_id)
    } else {
        MdxSignal::Error(reason.into(), span.lo.to_usize(), source, rule_id)
    }
}

/// Turn an SWC error into a string.
fn swc_error_to_string(error: &SwcError) -> String {
    error.kind().msg().into()
}

/// Move past JavaScript whitespace (well, actually ASCII whitespace) and
/// comments.
///
/// This is needed because for expressions, we use an API that parses up to
/// a valid expression, but there may be more expressions after it, which we
/// don’t alow.
fn whitespace_and_comments(mut index: usize, value: &str) -> Result<(), (Span, String)> {
    let bytes = value.as_bytes();
    let len = bytes.len();
    let mut in_multiline = false;
    let mut in_line = false;

    while index < len {
        // In a multiline comment: `/* a */`.
        if in_multiline {
            if index + 1 < len && bytes[index] == b'*' && bytes[index + 1] == b'/' {
                index += 1;
                in_multiline = false;
            }
        }
        // In a line comment: `// a`.
        else if in_line {
            if bytes[index] == b'\r' || bytes[index] == b'\n' {
                in_line = false;
            }
        }
        // Not in a comment, opening a multiline comment: `/* a */`.
        else if index + 1 < len && bytes[index] == b'/' && bytes[index + 1] == b'*' {
            index += 1;
            in_multiline = true;
        }
        // Not in a comment, opening a line comment: `// a`.
        else if index + 1 < len && bytes[index] == b'/' && bytes[index + 1] == b'/' {
            index += 1;
            in_line = true;
        }
        // Outside comment, whitespace.
        else if bytes[index].is_ascii_whitespace() {
            // Fine!
        }
        // Outside comment, not whitespace.
        else {
            return Err((
                create_span(index as u32, value.len() as u32),
                "Could not parse expression with swc: Unexpected content after expression".into(),
            ));
        }

        index += 1;
    }

    if in_multiline {
        return Err((
            create_span(index as u32, value.len() as u32), "Could not parse expression with swc: Unexpected unclosed multiline comment, expected closing: `*/`".into()));
    }

    if in_line {
        // EOF instead of EOL is specifically not allowed, because that would
        // mean the closing brace is on the commented-out line
        return Err((create_span(index as u32, value.len() as u32), "Could not parse expression with swc: Unexpected unclosed line comment, expected line ending: `\\n`".into()));
    }

    Ok(())
}

/// Create configuration for SWC, shared between ESM and expressions.
///
/// This enables modern JavaScript (ES2022) + JSX.
fn create_config(source: String) -> (SourceFile, Syntax, EsVersion) {
    (
        // File.
        SourceFile::new(
            FileName::Anon.into(),
            false,
            FileName::Anon.into(),
            source,
            BytePos::from_usize(1),
        ),
        // Syntax.
        Syntax::Es(EsSyntax {
            jsx: true,
            ..EsSyntax::default()
        }),
        // Version.
        // To do: update once in a while (last checked: 2024-04-18).
        EsVersion::Es2022,
    )
}

fn fix_span(mut span: Span, offset: usize) -> Span {
    span.lo = BytePos::from_usize(span.lo.to_usize() - offset);
    span.hi = BytePos::from_usize(span.hi.to_usize() - offset);
    span
}
