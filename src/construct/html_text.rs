//! To do.

use crate::construct::partial_whitespace::start as whitespace;
use crate::tokenizer::{Code, State, StateFn, StateFnResult, TokenType, Tokenizer};

/// Start of HTML (text)
///
/// ```markdown
/// a |<x> b
/// ```
pub fn start(tokenizer: &mut Tokenizer, code: Code) -> StateFnResult {
    tokenizer.enter(TokenType::HtmlText);
    tokenizer.enter(TokenType::HtmlTextData);
    tokenizer.consume(code);
    (State::Fn(Box::new(open)), None)
}

/// To do.
pub fn open(tokenizer: &mut Tokenizer, code: Code) -> StateFnResult {
    match code {
        Code::Char('!') => {
            tokenizer.consume(code);
            (State::Fn(Box::new(declaration_open)), None)
        }
        Code::Char('/') => {
            tokenizer.consume(code);
            (State::Fn(Box::new(tag_close_start)), None)
        }
        Code::Char('?') => {
            tokenizer.consume(code);
            (State::Fn(Box::new(instruction)), None)
        }
        Code::Char(char) if char.is_ascii_alphabetic() => {
            tokenizer.consume(code);
            (State::Fn(Box::new(tag_open)), None)
        }
        _ => (State::Nok, None),
    }
}

/// To do.
pub fn declaration_open(tokenizer: &mut Tokenizer, code: Code) -> StateFnResult {
    match code {
        Code::Char('-') => {
            tokenizer.consume(code);
            (State::Fn(Box::new(comment_open)), None)
        }
        Code::Char('[') => {
            tokenizer.consume(code);
            let buffer = vec!['C', 'D', 'A', 'T', 'A', '['];
            (
                State::Fn(Box::new(|tokenizer, code| {
                    cdata_open(tokenizer, code, buffer, 0)
                })),
                None,
            )
        }
        Code::Char(char) if char.is_ascii_alphabetic() => {
            tokenizer.consume(code);
            (State::Fn(Box::new(declaration)), None)
        }
        _ => (State::Nok, None),
    }
}

/// To do.
pub fn comment_open(tokenizer: &mut Tokenizer, code: Code) -> StateFnResult {
    match code {
        Code::Char('-') => {
            tokenizer.consume(code);
            (State::Fn(Box::new(comment_start)), None)
        }
        _ => (State::Nok, None),
    }
}

/// To do.
pub fn comment_start(tokenizer: &mut Tokenizer, code: Code) -> StateFnResult {
    match code {
        Code::None | Code::Char('>') => (State::Nok, None),
        Code::Char('-') => {
            tokenizer.consume(code);
            (State::Fn(Box::new(comment_start_dash)), None)
        }
        _ => comment(tokenizer, code),
    }
}

/// To do.
pub fn comment_start_dash(tokenizer: &mut Tokenizer, code: Code) -> StateFnResult {
    match code {
        Code::None | Code::Char('>') => (State::Nok, None),
        _ => comment(tokenizer, code),
    }
}

/// To do.
pub fn comment(tokenizer: &mut Tokenizer, code: Code) -> StateFnResult {
    match code {
        Code::None => (State::Nok, None),
        Code::CarriageReturnLineFeed | Code::Char('\r' | '\n') => {
            at_line_ending(tokenizer, code, Box::new(comment))
        }
        Code::Char('-') => {
            tokenizer.consume(code);
            (State::Fn(Box::new(comment_close)), None)
        }
        _ => {
            tokenizer.consume(code);
            (State::Fn(Box::new(comment)), None)
        }
    }
}

/// To do.
pub fn comment_close(tokenizer: &mut Tokenizer, code: Code) -> StateFnResult {
    match code {
        Code::Char('-') => {
            tokenizer.consume(code);
            (State::Fn(Box::new(end)), None)
        }
        _ => comment(tokenizer, code),
    }
}

/// To do.
pub fn cdata_open(
    tokenizer: &mut Tokenizer,
    code: Code,
    buffer: Vec<char>,
    index: usize,
) -> StateFnResult {
    match code {
        Code::Char(char) if char == buffer[index] => {
            tokenizer.consume(code);

            if index + 1 == buffer.len() {
                (State::Fn(Box::new(cdata)), None)
            } else {
                (
                    State::Fn(Box::new(move |tokenizer, code| {
                        cdata_open(tokenizer, code, buffer, index + 1)
                    })),
                    None,
                )
            }
        }
        _ => (State::Nok, None),
    }
}

/// To do.
pub fn cdata(tokenizer: &mut Tokenizer, code: Code) -> StateFnResult {
    match code {
        Code::None => (State::Nok, None),
        Code::CarriageReturnLineFeed | Code::Char('\r' | '\n') => {
            at_line_ending(tokenizer, code, Box::new(cdata))
        }
        Code::Char(']') => {
            tokenizer.consume(code);
            (State::Fn(Box::new(cdata_close)), None)
        }
        _ => {
            tokenizer.consume(code);
            (State::Fn(Box::new(cdata)), None)
        }
    }
}

/// To do.
pub fn cdata_close(tokenizer: &mut Tokenizer, code: Code) -> StateFnResult {
    match code {
        Code::Char(']') => {
            tokenizer.consume(code);
            (State::Fn(Box::new(cdata_end)), None)
        }
        _ => cdata(tokenizer, code),
    }
}

/// To do.
pub fn cdata_end(tokenizer: &mut Tokenizer, code: Code) -> StateFnResult {
    match code {
        Code::Char('>') => end(tokenizer, code),
        Code::Char(']') => cdata_close(tokenizer, code),
        _ => cdata(tokenizer, code),
    }
}

/// To do.
pub fn declaration(tokenizer: &mut Tokenizer, code: Code) -> StateFnResult {
    match code {
        Code::None | Code::Char('>') => end(tokenizer, code),
        Code::CarriageReturnLineFeed | Code::Char('\r' | '\n') => {
            at_line_ending(tokenizer, code, Box::new(declaration))
        }
        _ => {
            tokenizer.consume(code);
            (State::Fn(Box::new(declaration)), None)
        }
    }
}

/// To do.
pub fn instruction(tokenizer: &mut Tokenizer, code: Code) -> StateFnResult {
    match code {
        Code::None => (State::Nok, None),
        Code::CarriageReturnLineFeed | Code::Char('\r' | '\n') => {
            at_line_ending(tokenizer, code, Box::new(instruction))
        }
        Code::Char('?') => {
            tokenizer.consume(code);
            (State::Fn(Box::new(instruction_close)), None)
        }
        _ => {
            tokenizer.consume(code);
            (State::Fn(Box::new(instruction)), None)
        }
    }
}

/// To do.
pub fn instruction_close(tokenizer: &mut Tokenizer, code: Code) -> StateFnResult {
    match code {
        Code::Char('>') => end(tokenizer, code),
        _ => instruction(tokenizer, code),
    }
}

/// To do.
pub fn tag_close_start(tokenizer: &mut Tokenizer, code: Code) -> StateFnResult {
    match code {
        Code::Char(char) if char.is_ascii_alphabetic() => {
            tokenizer.consume(code);
            (State::Fn(Box::new(tag_close)), None)
        }
        _ => (State::Nok, None),
    }
}

/// To do.
pub fn tag_close(tokenizer: &mut Tokenizer, code: Code) -> StateFnResult {
    match code {
        Code::Char(char) if char == '-' || char.is_ascii_alphanumeric() => {
            tokenizer.consume(code);
            (State::Fn(Box::new(tag_close)), None)
        }
        _ => tag_close_between(tokenizer, code),
    }
}

/// To do.
pub fn tag_close_between(tokenizer: &mut Tokenizer, code: Code) -> StateFnResult {
    match code {
        Code::CarriageReturnLineFeed | Code::Char('\r' | '\n') => {
            at_line_ending(tokenizer, code, Box::new(tag_close_between))
        }
        Code::VirtualSpace | Code::Char('\t' | ' ') => {
            tokenizer.consume(code);
            (State::Fn(Box::new(tag_close_between)), None)
        }
        _ => end(tokenizer, code),
    }
}

/// To do.
pub fn tag_open(tokenizer: &mut Tokenizer, code: Code) -> StateFnResult {
    match code {
        Code::Char(char) if char == '-' || char.is_ascii_alphanumeric() => {
            tokenizer.consume(code);
            (State::Fn(Box::new(tag_open)), None)
        }

        Code::CarriageReturnLineFeed
        | Code::VirtualSpace
        | Code::Char('\r' | '\n' | '\t' | ' ' | '/' | '>') => tag_open_between(tokenizer, code),
        _ => (State::Nok, None),
    }
}

/// To do.
pub fn tag_open_between(tokenizer: &mut Tokenizer, code: Code) -> StateFnResult {
    match code {
        Code::CarriageReturnLineFeed | Code::Char('\r' | '\n') => {
            at_line_ending(tokenizer, code, Box::new(tag_open_between))
        }
        Code::VirtualSpace | Code::Char('\t' | ' ') => {
            tokenizer.consume(code);
            (State::Fn(Box::new(tag_open_between)), None)
        }
        Code::Char('/') => {
            tokenizer.consume(code);
            (State::Fn(Box::new(end)), None)
        }
        Code::Char(char) if char == ':' || char == '_' || char.is_ascii_alphabetic() => {
            tokenizer.consume(code);
            (State::Fn(Box::new(tag_open_attribute_name)), None)
        }
        _ => end(tokenizer, code),
    }
}

/// To do.
pub fn tag_open_attribute_name(tokenizer: &mut Tokenizer, code: Code) -> StateFnResult {
    match code {
        Code::Char(char)
            if char == '-'
                || char == '.'
                || char == ':'
                || char == '_'
                || char.is_ascii_alphanumeric() =>
        {
            tokenizer.consume(code);
            (State::Fn(Box::new(tag_open_attribute_name)), None)
        }
        _ => tag_open_attribute_name_after(tokenizer, code),
    }
}

/// To do.
pub fn tag_open_attribute_name_after(tokenizer: &mut Tokenizer, code: Code) -> StateFnResult {
    match code {
        Code::CarriageReturnLineFeed | Code::Char('\r' | '\n') => {
            at_line_ending(tokenizer, code, Box::new(tag_open_attribute_name_after))
        }
        Code::VirtualSpace | Code::Char('\t' | ' ') => {
            tokenizer.consume(code);
            (State::Fn(Box::new(tag_open_attribute_name_after)), None)
        }
        Code::Char('=') => {
            tokenizer.consume(code);
            (State::Fn(Box::new(tag_open_attribute_value_before)), None)
        }
        _ => tag_open_between(tokenizer, code),
    }
}

/// To do.
pub fn tag_open_attribute_value_before(tokenizer: &mut Tokenizer, code: Code) -> StateFnResult {
    match code {
        Code::None | Code::Char('<' | '=' | '>' | '`') => (State::Nok, None),
        Code::CarriageReturnLineFeed | Code::Char('\r' | '\n') => {
            at_line_ending(tokenizer, code, Box::new(tag_open_attribute_value_before))
        }
        Code::VirtualSpace | Code::Char('\t' | ' ') => {
            tokenizer.consume(code);
            (State::Fn(Box::new(tag_open_attribute_value_before)), None)
        }
        Code::Char(char) if char == '"' || char == '\'' => {
            tokenizer.consume(code);
            (
                State::Fn(Box::new(move |tokenizer, code| {
                    tag_open_attribute_value_quoted(tokenizer, code, char)
                })),
                None,
            )
        }
        Code::Char(_) => {
            tokenizer.consume(code);
            (State::Fn(Box::new(tag_open_attribute_value_unquoted)), None)
        }
    }
}

/// To do.
pub fn tag_open_attribute_value_quoted(
    tokenizer: &mut Tokenizer,
    code: Code,
    marker: char,
) -> StateFnResult {
    match code {
        Code::None => (State::Nok, None),
        Code::CarriageReturnLineFeed | Code::Char('\r' | '\n') => at_line_ending(
            tokenizer,
            code,
            Box::new(move |tokenizer, code| {
                tag_open_attribute_value_quoted(tokenizer, code, marker)
            }),
        ),
        Code::Char(char) if char == marker => {
            tokenizer.consume(code);
            (
                State::Fn(Box::new(tag_open_attribute_value_quoted_after)),
                None,
            )
        }
        _ => {
            tokenizer.consume(code);
            (
                State::Fn(Box::new(move |tokenizer, code| {
                    tag_open_attribute_value_quoted(tokenizer, code, marker)
                })),
                None,
            )
        }
    }
}

/// To do.
pub fn tag_open_attribute_value_quoted_after(
    tokenizer: &mut Tokenizer,
    code: Code,
) -> StateFnResult {
    match code {
        Code::CarriageReturnLineFeed | Code::VirtualSpace | Code::Char('\t' | ' ' | '>' | '/') => {
            tag_open_between(tokenizer, code)
        }
        _ => (State::Nok, None),
    }
}

/// To do.
pub fn tag_open_attribute_value_unquoted(tokenizer: &mut Tokenizer, code: Code) -> StateFnResult {
    match code {
        Code::None | Code::Char('"' | '\'' | '<' | '=' | '`') => (State::Nok, None),
        Code::CarriageReturnLineFeed | Code::VirtualSpace | Code::Char('\t' | ' ' | '>') => {
            tag_open_between(tokenizer, code)
        }
        Code::Char(_) => {
            tokenizer.consume(code);
            (State::Fn(Box::new(tag_open_attribute_value_unquoted)), None)
        }
    }
}

/// To do.
// We can’t have blank lines in content, so no need to worry about empty
// tokens.
pub fn at_line_ending(
    tokenizer: &mut Tokenizer,
    code: Code,
    return_state: Box<StateFn>,
) -> StateFnResult {
    match code {
        Code::CarriageReturnLineFeed | Code::Char('\r' | '\n') => {
            tokenizer.exit(TokenType::HtmlTextData);
            tokenizer.enter(TokenType::LineEnding);
            tokenizer.consume(code);
            tokenizer.exit(TokenType::LineEnding);
            (
                State::Fn(Box::new(|t, c| after_line_ending(t, c, return_state))),
                None,
            )
        }
        _ => unreachable!("expected line ending"),
    }
}

pub fn after_line_ending(
    tokenizer: &mut Tokenizer,
    code: Code,
    return_state: Box<StateFn>,
) -> StateFnResult {
    tokenizer.attempt(
        |tokenizer, code| whitespace(tokenizer, code, TokenType::Whitespace),
        |_ok| Box::new(|t, c| after_line_ending_prefix(t, c, return_state)),
    )(tokenizer, code)
}

pub fn after_line_ending_prefix(
    tokenizer: &mut Tokenizer,
    code: Code,
    return_state: Box<StateFn>,
) -> StateFnResult {
    tokenizer.enter(TokenType::HtmlTextData);
    return_state(tokenizer, code)
}

/// To do.
pub fn end(tokenizer: &mut Tokenizer, code: Code) -> StateFnResult {
    match code {
        Code::Char('>') => {
            tokenizer.consume(code);
            tokenizer.exit(TokenType::HtmlTextData);
            tokenizer.exit(TokenType::HtmlText);
            (State::Ok, None)
        }
        _ => (State::Nok, None),
    }
}
