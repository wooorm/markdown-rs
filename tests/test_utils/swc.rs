//! Bridge between `markdown-rs` and SWC.

use crate::test_utils::swc_utils::{bytepos_to_point, prefix_error_with_point, RewriteContext};
use markdown::{mdast::Stop, unist::Point, Location, MdxExpressionKind, MdxSignal};
use swc_common::{
    source_map::Pos, sync::Lrc, BytePos, FileName, FilePathMapping, SourceFile, SourceMap, Spanned,
};
use swc_ecma_ast::{EsVersion, Expr, Module};
use swc_ecma_codegen::{text_writer::JsWriter, Emitter};
use swc_ecma_parser::{
    error::Error as SwcError, parse_file_as_expr, parse_file_as_module, EsConfig, Syntax,
};
use swc_ecma_visit::VisitMutWith;

/// Lex ESM in MDX with SWC.
#[allow(dead_code)]
pub fn parse_esm(value: &str) -> MdxSignal {
    let (file, syntax, version) = create_config(value.into());
    let mut errors = vec![];
    let result = parse_file_as_module(&file, syntax, version, None, &mut errors);

    match result {
        Err(error) => swc_error_to_signal(&error, "esm", value.len(), 0),
        Ok(tree) => {
            if errors.is_empty() {
                check_esm_ast(&tree)
            } else {
                swc_error_to_signal(&errors[0], "esm", value.len(), 0)
            }
        }
    }
}

/// Parse ESM in MDX with SWC.
/// See `drop_span` in `swc_ecma_utils` for inspiration?
#[allow(dead_code)]
pub fn parse_esm_to_tree(
    value: &str,
    stops: &[Stop],
    location: Option<&Location>,
) -> Result<swc_ecma_ast::Module, String> {
    let (file, syntax, version) = create_config(value.into());
    let mut errors = vec![];
    let result = parse_file_as_module(&file, syntax, version, None, &mut errors);
    let mut rewrite_context = RewriteContext {
        stops,
        location,
        prefix_len: 0,
    };

    match result {
        Err(error) => Err(swc_error_to_error(&error, "esm", &rewrite_context)),
        Ok(mut module) => {
            if errors.is_empty() {
                module.visit_mut_with(&mut rewrite_context);
                Ok(module)
            } else {
                Err(swc_error_to_error(&errors[0], "esm", &rewrite_context))
            }
        }
    }
}

/// Lex expressions in MDX with SWC.
#[allow(dead_code)]
pub fn parse_expression(value: &str, kind: &MdxExpressionKind) -> MdxSignal {
    // Empty expressions are OK.
    if matches!(kind, MdxExpressionKind::Expression)
        && matches!(whitespace_and_comments(0, value), MdxSignal::Ok)
    {
        return MdxSignal::Ok;
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
        Err(error) => swc_error_to_signal(&error, "expression", value.len(), prefix.len()),
        Ok(tree) => {
            if errors.is_empty() {
                let expression_end = fix_swc_position(tree.span().hi.to_usize(), prefix.len());
                let result = check_expression_ast(&tree, kind);
                if matches!(result, MdxSignal::Ok) {
                    whitespace_and_comments(expression_end, value)
                } else {
                    result
                }
            } else {
                swc_error_to_signal(&errors[0], "expression", value.len(), prefix.len())
            }
        }
    }
}

/// Parse ESM in MDX with SWC.
/// See `drop_span` in `swc_ecma_utils` for inspiration?
#[allow(dead_code)]
pub fn parse_expression_to_tree(
    value: &str,
    kind: &MdxExpressionKind,
    stops: &[Stop],
    location: Option<&Location>,
) -> Result<Box<swc_ecma_ast::Expr>, String> {
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
    let mut rewrite_context = RewriteContext {
        stops,
        location,
        prefix_len: prefix.len(),
    };

    match result {
        Err(error) => Err(swc_error_to_error(&error, "expression", &rewrite_context)),
        Ok(mut expr) => {
            if errors.is_empty() {
                // Fix positions.
                expr.visit_mut_with(&mut rewrite_context);

                let expr_bytepos = expr.span().lo;

                if matches!(kind, MdxExpressionKind::AttributeExpression) {
                    let mut obj = None;

                    if let swc_ecma_ast::Expr::Paren(d) = *expr {
                        if let swc_ecma_ast::Expr::Object(d) = *d.expr {
                            obj = Some(d)
                        }
                    };

                    if let Some(mut obj) = obj {
                        if obj.props.len() > 1 {
                            Err(create_error_message(
                                "Unexpected extra content in spread: only a single spread is supported",
                                "expression",
                                bytepos_to_point(&obj.span.lo, location).as_ref()
                            ))
                        } else if let Some(swc_ecma_ast::PropOrSpread::Spread(d)) = obj.props.pop()
                        {
                            Ok(d.expr)
                        } else {
                            Err(create_error_message(
                                "Unexpected prop in spread: only a spread is supported",
                                "expression",
                                bytepos_to_point(&obj.span.lo, location).as_ref(),
                            ))
                        }
                    } else {
                        Err(create_error_message(
                            "Expected an object spread (`{...spread}`)",
                            "expression",
                            bytepos_to_point(&expr_bytepos, location).as_ref(),
                        ))
                    }
                } else {
                    Ok(expr)
                }
            } else {
                Err(swc_error_to_error(
                    &errors[0],
                    "expression",
                    &rewrite_context,
                ))
            }
        }
    }
}

/// Serialize an SWC module.
/// To do: support comments.
#[allow(dead_code)]
pub fn serialize(module: &Module) -> String {
    let mut buf = vec![];
    let cm = Lrc::new(SourceMap::new(FilePathMapping::empty()));
    // let comm = &program.comments as &dyn swc_common::comments::Comments;
    {
        let mut emitter = Emitter {
            cfg: swc_ecma_codegen::Config {
                ..Default::default()
            },
            cm: cm.clone(),
            // To do: figure out how to pass them.
            comments: None,
            wr: JsWriter::new(cm, "\n", &mut buf, None),
        };

        emitter.emit_module(module).unwrap();
    }

    String::from_utf8_lossy(&buf).into()
}

/// Check that the resulting AST of ESM is OK.
///
/// This checks that only module declarations (import/exports) are used, not
/// statements.
fn check_esm_ast(tree: &Module) -> MdxSignal {
    let mut index = 0;
    while index < tree.body.len() {
        let node = &tree.body[index];

        if !node.is_module_decl() {
            let relative = fix_swc_position(node.span().lo.to_usize(), 0);
            return MdxSignal::Error(
                "Unexpected statement in code: only import/exports are supported".into(),
                relative,
            );
        }

        index += 1;
    }

    MdxSignal::Ok
}

/// Check that the resulting AST of an expressions is OK.
///
/// This checks that attribute expressions are the expected spread.
fn check_expression_ast(tree: &Expr, kind: &MdxExpressionKind) -> MdxSignal {
    if matches!(kind, MdxExpressionKind::AttributeExpression)
        && tree
            .unwrap_parens()
            .as_object()
            .and_then(|object| {
                if object.props.len() == 1 {
                    object.props[0].as_spread()
                } else {
                    None
                }
            })
            .is_none()
    {
        MdxSignal::Error("Expected a single spread value, such as `...x`".into(), 0)
    } else {
        MdxSignal::Ok
    }
}

/// Turn an SWC error into an `MdxSignal`.
///
/// * If the error happens at `value_len`, yields `MdxSignal::Eof`
/// * Else, yields `MdxSignal::Error`.
fn swc_error_to_signal(
    error: &SwcError,
    name: &str,
    value_len: usize,
    prefix_len: usize,
) -> MdxSignal {
    let reason = create_error_reason(&swc_error_to_string(error), name);
    let error_end = fix_swc_position(error.span().hi.to_usize(), prefix_len);

    if error_end >= value_len {
        MdxSignal::Eof(reason)
    } else {
        MdxSignal::Error(
            reason,
            fix_swc_position(error.span().lo.to_usize(), prefix_len),
        )
    }
}

fn swc_error_to_error(error: &SwcError, name: &str, context: &RewriteContext) -> String {
    create_error_message(
        &swc_error_to_string(error),
        name,
        context
            .location
            .and_then(|location| {
                location.relative_to_point(
                    context.stops,
                    fix_swc_position(error.span().lo.to_usize(), context.prefix_len),
                )
            })
            .as_ref(),
    )
}

fn create_error_message(reason: &str, name: &str, point: Option<&Point>) -> String {
    prefix_error_with_point(create_error_reason(name, reason), point)
}

fn create_error_reason(reason: &str, name: &str) -> String {
    format!("Could not parse {} with swc: {}", name, reason)
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
fn whitespace_and_comments(mut index: usize, value: &str) -> MdxSignal {
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
            if index + 1 < len && bytes[index] == b'\r' && bytes[index + 1] == b'\n' {
                index += 1;
                in_line = false;
            } else if bytes[index] == b'\r' || bytes[index] == b'\n' {
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
            return MdxSignal::Error(
                "Could not parse expression with swc: Unexpected content after expression".into(),
                index,
            );
        }

        index += 1;
    }

    if in_multiline {
        MdxSignal::Error(
            "Could not parse expression with swc: Unexpected unclosed multiline comment, expected closing: `*/`".into(),
            index,
        )
    } else if in_line {
        // EOF instead of EOL is specifically not allowed, because that would
        // mean the closing brace is on the commented-out line
        MdxSignal::Error(
            "Could not parse expression with swc: Unexpected unclosed line comment, expected line ending: `\\n`".into(),
            index,
        )
    } else {
        MdxSignal::Ok
    }
}

/// Create configuration for SWC, shared between ESM and expressions.
///
/// This enables modern JavaScript (ES2022) + JSX.
fn create_config(source: String) -> (SourceFile, Syntax, EsVersion) {
    (
        // File.
        SourceFile::new(
            FileName::Anon,
            false,
            FileName::Anon,
            source,
            BytePos::from_usize(1),
        ),
        // Syntax.
        Syntax::Es(EsConfig {
            jsx: true,
            ..EsConfig::default()
        }),
        // Version.
        EsVersion::Es2022,
    )
}

/// Turn an SWC byte position from a resulting AST to an offset in the original
/// input string.
fn fix_swc_position(index: usize, prefix_len: usize) -> usize {
    index - 1 - prefix_len
}
