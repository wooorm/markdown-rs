//! HTML (text) is a construct that occurs in the [text][] content type.
//!
//! It forms with the following BNF:
//!
//! ```bnf
//! html_text ::= comment | instruction | declaration | cdata | tag_close | tag_open
//!
//! ; Restriction: the text is not allowed to start with `>`, `->`, or to contain `--`.
//! comment ::= '<!--' *code '-->'
//! instruction ::= '<?' *code '?>'
//! declaration ::= '<!' ascii_alphabetic *code '>'
//! ; Restriction: the text is not allowed to contain `]]`.
//! cdata ::= '<![CDATA[' *code ']]>'
//! tag_close ::= '</' tag_name whitespace_optional '>'
//! opening_tag ::= '<' tag_name *( whitespace attribute ) [ whitespace_optional '/' ] whitespace_optional '>'
//!
//! tag_name ::= ascii_alphabetic *( '-' | ascii_alphanumeric )
//! attribute ::= attribute_name [ whitespace_optional '=' whitespace_optional attribute_value ]
//! attribute_name ::= ( ':' | '_' | ascii_alphabetic ) *( '-' | '.' | ':' | '_' | ascii_alphanumeric )
//! attribute_value ::= '"' *( code - '"' ) '"' | "'" *( code - "'" )  "'" | 1*( code - space_or_tab - eol - '"' - "'" - '/' - '<' - '=' - '>' - '`')
//!
//! ; Note: blank lines can never occur in `text`.
//! whitespace ::= 1*space_or_tab | [ *space_or_tab eol *space_or_tab ]
//! whitespace_optional ::= [ whitespace ]
//! eol ::= '\r' | '\r\n' | '\n'
//! space_or_tab ::= ' ' | '\t'
//! ```
//!
//! The grammar for HTML in markdown does not resemble the rules of parsing
//! HTML according to the [*§ 13.2 Parsing HTML documents* in the HTML
//! spec][html-parsing].
//! See the related flow construct [HTML (flow)][html_flow] for more info.
//!
//! Because the **tag open** and **tag close** productions in the grammar form
//! with just tags instead of complete elements, it is possible to interleave
//! (a word for switching between languages) markdown and HTML together.
//! For example:
//!
//! ```markdown
//! This is equivalent to <code>*emphasised* code</code>.
//! ```
//!
//! ## Tokens
//!
//! *   [`HtmlText`][Token::HtmlText]
//! *   [`HtmlTextData`][Token::HtmlTextData]
//!
//! ## References
//!
//! *   [`html-text.js` in `micromark`](https://github.com/micromark/micromark/blob/main/packages/micromark-core-commonmark/dev/lib/html-text.js)
//! *   [*§ 6.6 Raw HTML* in `CommonMark`](https://spec.commonmark.org/0.30/#raw-html)
//!
//! [text]: crate::content::text
//! [html_flow]: crate::construct::html_flow
//! [html-parsing]: https://html.spec.whatwg.org/multipage/parsing.html#parsing

use crate::construct::partial_space_or_tab::space_or_tab;
use crate::token::Token;
use crate::tokenizer::{Code, State, StateFn, StateFnResult, Tokenizer};
use crate::util::codes::parse;

/// Start of HTML (text)
///
/// ```markdown
/// > | a <b> c
///       ^
/// ```
pub fn start(tokenizer: &mut Tokenizer, code: Code) -> StateFnResult {
    if Code::Char('<') == code && tokenizer.parse_state.constructs.html_text {
        tokenizer.enter(Token::HtmlText);
        tokenizer.enter(Token::HtmlTextData);
        tokenizer.consume(code);
        (State::Fn(Box::new(open)), 0)
    } else {
        (State::Nok, 0)
    }
}

/// After `<`, before a tag name or other stuff.
///
/// ```markdown
/// > | a <b> c
///        ^
/// > | a <!doctype> c
///        ^
/// > | a <!--b--> c
///        ^
/// ```
fn open(tokenizer: &mut Tokenizer, code: Code) -> StateFnResult {
    match code {
        Code::Char('!') => {
            tokenizer.consume(code);
            (State::Fn(Box::new(declaration_open)), 0)
        }
        Code::Char('/') => {
            tokenizer.consume(code);
            (State::Fn(Box::new(tag_close_start)), 0)
        }
        Code::Char('?') => {
            tokenizer.consume(code);
            (State::Fn(Box::new(instruction)), 0)
        }
        Code::Char('A'..='Z' | 'a'..='z') => {
            tokenizer.consume(code);
            (State::Fn(Box::new(tag_open)), 0)
        }
        _ => (State::Nok, 0),
    }
}

/// After `<!`, so inside a declaration, comment, or CDATA.
///
/// ```markdown
/// > | a <!doctype> c
///         ^
/// > | a <!--b--> c
///         ^
/// > | a <![CDATA[>&<]]> c
///         ^
/// ```
fn declaration_open(tokenizer: &mut Tokenizer, code: Code) -> StateFnResult {
    match code {
        Code::Char('-') => {
            tokenizer.consume(code);
            (State::Fn(Box::new(comment_open_inside)), 0)
        }
        Code::Char('[') => {
            tokenizer.consume(code);
            let buffer = parse("CDATA[");
            (
                State::Fn(Box::new(|t, c| cdata_open_inside(t, c, buffer, 0))),
                0,
            )
        }
        Code::Char('A'..='Z' | 'a'..='z') => {
            tokenizer.consume(code);
            (State::Fn(Box::new(declaration)), 0)
        }
        _ => (State::Nok, 0),
    }
}

/// After `<!-`, inside a comment, before another `-`.
///
/// ```markdown
/// > | a <!--b--> c
///          ^
/// ```
fn comment_open_inside(tokenizer: &mut Tokenizer, code: Code) -> StateFnResult {
    match code {
        Code::Char('-') => {
            tokenizer.consume(code);
            (State::Fn(Box::new(comment_start)), 0)
        }
        _ => (State::Nok, 0),
    }
}

/// After `<!--`, inside a comment
///
/// > 👉 **Note**: [html (flow)][html_flow] does allow `<!-->` or `<!--->` as
/// > empty comments.
/// > This is prohibited in html (text).
/// > See: <https://github.com/commonmark/commonmark-spec/issues/712>.
///
/// ```markdown
/// > | a <!--b--> c
///           ^
/// ```
///
/// [html_flow]: crate::construct::html_flow
fn comment_start(tokenizer: &mut Tokenizer, code: Code) -> StateFnResult {
    match code {
        Code::None | Code::Char('>') => (State::Nok, 0),
        Code::Char('-') => {
            tokenizer.consume(code);
            (State::Fn(Box::new(comment_start_dash)), 0)
        }
        _ => comment(tokenizer, code),
    }
}

/// After `<!---`, inside a comment
///
/// > 👉 **Note**: [html (flow)][html_flow] does allow `<!-->` or `<!--->` as
/// > empty comments.
/// > This is prohibited in html (text).
/// > See: <https://github.com/commonmark/commonmark-spec/issues/712>.
///
/// ```markdown
/// > | a <!---b--> c
///            ^
/// ```
///
/// [html_flow]: crate::construct::html_flow
fn comment_start_dash(tokenizer: &mut Tokenizer, code: Code) -> StateFnResult {
    match code {
        Code::None | Code::Char('>') => (State::Nok, 0),
        _ => comment(tokenizer, code),
    }
}

/// In a comment.
///
/// ```markdown
/// > | a <!--b--> c
///           ^
/// ```
fn comment(tokenizer: &mut Tokenizer, code: Code) -> StateFnResult {
    match code {
        Code::None => (State::Nok, 0),
        Code::CarriageReturnLineFeed | Code::Char('\n' | '\r') => {
            at_line_ending(tokenizer, code, Box::new(comment))
        }
        Code::Char('-') => {
            tokenizer.consume(code);
            (State::Fn(Box::new(comment_close)), 0)
        }
        _ => {
            tokenizer.consume(code);
            (State::Fn(Box::new(comment)), 0)
        }
    }
}

/// In a comment, after `-`.
///
/// ```markdown
/// > | a <!--b--> c
///             ^
/// ```
fn comment_close(tokenizer: &mut Tokenizer, code: Code) -> StateFnResult {
    match code {
        Code::Char('-') => {
            tokenizer.consume(code);
            (State::Fn(Box::new(end)), 0)
        }
        _ => comment(tokenizer, code),
    }
}

/// After `<![`, inside CDATA, expecting `CDATA[`.
///
/// ```markdown
/// > | a <![CDATA[>&<]]> b
///          ^^^^^^
/// ```
fn cdata_open_inside(
    tokenizer: &mut Tokenizer,
    code: Code,
    buffer: Vec<Code>,
    index: usize,
) -> StateFnResult {
    if code == buffer[index] {
        tokenizer.consume(code);

        if index + 1 == buffer.len() {
            (State::Fn(Box::new(cdata)), 0)
        } else {
            (
                State::Fn(Box::new(move |t, c| {
                    cdata_open_inside(t, c, buffer, index + 1)
                })),
                0,
            )
        }
    } else {
        (State::Nok, 0)
    }
}

/// In CDATA.
///
/// ```markdown
/// > | a <![CDATA[>&<]]> b
///                ^^^
/// ```
fn cdata(tokenizer: &mut Tokenizer, code: Code) -> StateFnResult {
    match code {
        Code::None => (State::Nok, 0),
        Code::CarriageReturnLineFeed | Code::Char('\n' | '\r') => {
            at_line_ending(tokenizer, code, Box::new(cdata))
        }
        Code::Char(']') => {
            tokenizer.consume(code);
            (State::Fn(Box::new(cdata_close)), 0)
        }
        _ => {
            tokenizer.consume(code);
            (State::Fn(Box::new(cdata)), 0)
        }
    }
}

/// In CDATA, after `]`.
///
/// ```markdown
/// > | a <![CDATA[>&<]]> b
///                    ^
/// ```
fn cdata_close(tokenizer: &mut Tokenizer, code: Code) -> StateFnResult {
    match code {
        Code::Char(']') => {
            tokenizer.consume(code);
            (State::Fn(Box::new(cdata_end)), 0)
        }
        _ => cdata(tokenizer, code),
    }
}

/// In CDATA, after `]]`.
///
/// ```markdown
/// > | a <![CDATA[>&<]]> b
///                     ^
/// ```
fn cdata_end(tokenizer: &mut Tokenizer, code: Code) -> StateFnResult {
    match code {
        Code::Char('>') => end(tokenizer, code),
        Code::Char(']') => cdata_close(tokenizer, code),
        _ => cdata(tokenizer, code),
    }
}

/// In a declaration.
///
/// ```markdown
/// > | a <!b> c
///          ^
/// ```
fn declaration(tokenizer: &mut Tokenizer, code: Code) -> StateFnResult {
    match code {
        Code::None | Code::Char('>') => end(tokenizer, code),
        Code::CarriageReturnLineFeed | Code::Char('\n' | '\r') => {
            at_line_ending(tokenizer, code, Box::new(declaration))
        }
        _ => {
            tokenizer.consume(code);
            (State::Fn(Box::new(declaration)), 0)
        }
    }
}

/// In an instruction.
///
/// ```markdown
/// > | a <?b?> c
///         ^
/// ```
fn instruction(tokenizer: &mut Tokenizer, code: Code) -> StateFnResult {
    match code {
        Code::None => (State::Nok, 0),
        Code::CarriageReturnLineFeed | Code::Char('\n' | '\r') => {
            at_line_ending(tokenizer, code, Box::new(instruction))
        }
        Code::Char('?') => {
            tokenizer.consume(code);
            (State::Fn(Box::new(instruction_close)), 0)
        }
        _ => {
            tokenizer.consume(code);
            (State::Fn(Box::new(instruction)), 0)
        }
    }
}

/// In an instruction, after `?`.
///
/// ```markdown
/// > | a <?b?> c
///           ^
/// ```
fn instruction_close(tokenizer: &mut Tokenizer, code: Code) -> StateFnResult {
    match code {
        Code::Char('>') => end(tokenizer, code),
        _ => instruction(tokenizer, code),
    }
}

/// After `</`, in a closing tag, before a tag name.
///
/// ```markdown
/// > | a </b> c
///         ^
/// ```
fn tag_close_start(tokenizer: &mut Tokenizer, code: Code) -> StateFnResult {
    match code {
        Code::Char('A'..='Z' | 'a'..='z') => {
            tokenizer.consume(code);
            (State::Fn(Box::new(tag_close)), 0)
        }
        _ => (State::Nok, 0),
    }
}

/// After `</x`, in a tag name.
///
/// ```markdown
/// > | a </b> c
///          ^
/// ```
fn tag_close(tokenizer: &mut Tokenizer, code: Code) -> StateFnResult {
    match code {
        Code::Char('-' | '0'..='9' | 'A'..='Z' | 'a'..='z') => {
            tokenizer.consume(code);
            (State::Fn(Box::new(tag_close)), 0)
        }
        _ => tag_close_between(tokenizer, code),
    }
}

/// In a closing tag, after the tag name.
///
/// ```markdown
/// > | a </b> c
///          ^
/// ```
fn tag_close_between(tokenizer: &mut Tokenizer, code: Code) -> StateFnResult {
    match code {
        Code::CarriageReturnLineFeed | Code::Char('\n' | '\r') => {
            at_line_ending(tokenizer, code, Box::new(tag_close_between))
        }
        Code::VirtualSpace | Code::Char('\t' | ' ') => {
            tokenizer.consume(code);
            (State::Fn(Box::new(tag_close_between)), 0)
        }
        _ => end(tokenizer, code),
    }
}

/// After `<x`, in an opening tag name.
///
/// ```markdown
/// > | a <b> c
///         ^
/// ```
fn tag_open(tokenizer: &mut Tokenizer, code: Code) -> StateFnResult {
    match code {
        Code::Char('-' | '0'..='9' | 'A'..='Z' | 'a'..='z') => {
            tokenizer.consume(code);
            (State::Fn(Box::new(tag_open)), 0)
        }
        Code::CarriageReturnLineFeed
        | Code::VirtualSpace
        | Code::Char('\t' | '\n' | '\r' | ' ' | '/' | '>') => tag_open_between(tokenizer, code),
        _ => (State::Nok, 0),
    }
}

/// In an opening tag, after the tag name.
///
/// ```markdown
/// > | a <b> c
///         ^
/// ```
fn tag_open_between(tokenizer: &mut Tokenizer, code: Code) -> StateFnResult {
    match code {
        Code::CarriageReturnLineFeed | Code::Char('\n' | '\r') => {
            at_line_ending(tokenizer, code, Box::new(tag_open_between))
        }
        Code::VirtualSpace | Code::Char('\t' | ' ') => {
            tokenizer.consume(code);
            (State::Fn(Box::new(tag_open_between)), 0)
        }
        Code::Char('/') => {
            tokenizer.consume(code);
            (State::Fn(Box::new(end)), 0)
        }
        Code::Char(':' | 'A'..='Z' | '_' | 'a'..='z') => {
            tokenizer.consume(code);
            (State::Fn(Box::new(tag_open_attribute_name)), 0)
        }
        _ => end(tokenizer, code),
    }
}

/// In an attribute name.
///
/// ```markdown
/// > | a <b c> d
///          ^
/// ```
fn tag_open_attribute_name(tokenizer: &mut Tokenizer, code: Code) -> StateFnResult {
    match code {
        Code::Char('-' | '.' | '0'..='9' | ':' | 'A'..='Z' | '_' | 'a'..='z') => {
            tokenizer.consume(code);
            (State::Fn(Box::new(tag_open_attribute_name)), 0)
        }
        _ => tag_open_attribute_name_after(tokenizer, code),
    }
}

/// After an attribute name, before an attribute initializer, the end of the
/// tag, or whitespace.
///
/// ```markdown
/// > | a <b c> d
///           ^
/// ```
fn tag_open_attribute_name_after(tokenizer: &mut Tokenizer, code: Code) -> StateFnResult {
    match code {
        Code::CarriageReturnLineFeed | Code::Char('\n' | '\r') => {
            at_line_ending(tokenizer, code, Box::new(tag_open_attribute_name_after))
        }
        Code::VirtualSpace | Code::Char('\t' | ' ') => {
            tokenizer.consume(code);
            (State::Fn(Box::new(tag_open_attribute_name_after)), 0)
        }
        Code::Char('=') => {
            tokenizer.consume(code);
            (State::Fn(Box::new(tag_open_attribute_value_before)), 0)
        }
        _ => tag_open_between(tokenizer, code),
    }
}

/// Before an unquoted, double quoted, or single quoted attribute value,
/// allowing whitespace.
///
/// ```markdown
/// > | a <b c=d> e
///            ^
/// ```
fn tag_open_attribute_value_before(tokenizer: &mut Tokenizer, code: Code) -> StateFnResult {
    match code {
        Code::None | Code::Char('<' | '=' | '>' | '`') => (State::Nok, 0),
        Code::CarriageReturnLineFeed | Code::Char('\n' | '\r') => {
            at_line_ending(tokenizer, code, Box::new(tag_open_attribute_value_before))
        }
        Code::VirtualSpace | Code::Char('\t' | ' ') => {
            tokenizer.consume(code);
            (State::Fn(Box::new(tag_open_attribute_value_before)), 0)
        }
        Code::Char(char) if char == '"' || char == '\'' => {
            tokenizer.consume(code);
            (
                State::Fn(Box::new(move |t, c| {
                    tag_open_attribute_value_quoted(t, c, char)
                })),
                0,
            )
        }
        Code::Char(_) => {
            tokenizer.consume(code);
            (State::Fn(Box::new(tag_open_attribute_value_unquoted)), 0)
        }
    }
}

/// In a double or single quoted attribute value.
///
/// ```markdown
/// > | a <b c="d"> e
///             ^
/// ```
fn tag_open_attribute_value_quoted(
    tokenizer: &mut Tokenizer,
    code: Code,
    marker: char,
) -> StateFnResult {
    match code {
        Code::None => (State::Nok, 0),
        Code::CarriageReturnLineFeed | Code::Char('\n' | '\r') => at_line_ending(
            tokenizer,
            code,
            Box::new(move |t, c| tag_open_attribute_value_quoted(t, c, marker)),
        ),
        Code::Char(char) if char == marker => {
            tokenizer.consume(code);
            (
                State::Fn(Box::new(tag_open_attribute_value_quoted_after)),
                0,
            )
        }
        _ => {
            tokenizer.consume(code);
            (
                State::Fn(Box::new(move |t, c| {
                    tag_open_attribute_value_quoted(t, c, marker)
                })),
                0,
            )
        }
    }
}

/// In an unquoted attribute value.
///
/// ```markdown
/// > | a <b c=d> e
///            ^
/// ```
fn tag_open_attribute_value_unquoted(tokenizer: &mut Tokenizer, code: Code) -> StateFnResult {
    match code {
        Code::None | Code::Char('"' | '\'' | '<' | '=' | '`') => (State::Nok, 0),
        Code::CarriageReturnLineFeed
        | Code::VirtualSpace
        | Code::Char('\t' | '\n' | '\r' | ' ' | '/' | '>') => tag_open_between(tokenizer, code),
        Code::Char(_) => {
            tokenizer.consume(code);
            (State::Fn(Box::new(tag_open_attribute_value_unquoted)), 0)
        }
    }
}

/// After a double or single quoted attribute value, before whitespace or the
/// end of the tag.
///
/// ```markdown
/// > | a <b c="d"> e
///               ^
/// ```
fn tag_open_attribute_value_quoted_after(tokenizer: &mut Tokenizer, code: Code) -> StateFnResult {
    match code {
        Code::CarriageReturnLineFeed
        | Code::VirtualSpace
        | Code::Char('\t' | '\n' | '\r' | ' ' | '>' | '/') => tag_open_between(tokenizer, code),
        _ => (State::Nok, 0),
    }
}

/// In certain circumstances of a complete tag where only an `>` is allowed.
///
/// ```markdown
/// > | a <b c="d"> e
///               ^
/// ```
fn end(tokenizer: &mut Tokenizer, code: Code) -> StateFnResult {
    match code {
        Code::Char('>') => {
            tokenizer.consume(code);
            tokenizer.exit(Token::HtmlTextData);
            tokenizer.exit(Token::HtmlText);
            (State::Ok, 0)
        }
        _ => (State::Nok, 0),
    }
}

/// At an allowed line ending.
///
/// > 👉 **Note**: we can’t have blank lines in text, so no need to worry about
/// > empty tokens.
///
/// ```markdown
/// > | a <!--a
///            ^
///   | b-->
/// ```
fn at_line_ending(
    tokenizer: &mut Tokenizer,
    code: Code,
    return_state: Box<StateFn>,
) -> StateFnResult {
    match code {
        Code::CarriageReturnLineFeed | Code::Char('\n' | '\r') => {
            tokenizer.exit(Token::HtmlTextData);
            tokenizer.enter(Token::LineEnding);
            tokenizer.consume(code);
            tokenizer.exit(Token::LineEnding);
            (
                State::Fn(Box::new(|t, c| after_line_ending(t, c, return_state))),
                0,
            )
        }
        _ => unreachable!("expected eol"),
    }
}

/// After a line ending.
///
/// > 👉 **Note**: we can’t have blank lines in text, so no need to worry about
/// > empty tokens.
///
/// ```markdown
///   | a <!--a
/// > | b-->
///     ^
/// ```
fn after_line_ending(
    tokenizer: &mut Tokenizer,
    code: Code,
    return_state: Box<StateFn>,
) -> StateFnResult {
    tokenizer.attempt_opt(space_or_tab(), |t, c| {
        after_line_ending_prefix(t, c, return_state)
    })(tokenizer, code)
}

/// After a line ending, after indent.
///
/// > 👉 **Note**: we can’t have blank lines in text, so no need to worry about
/// > empty tokens.
///
/// ```markdown
///   | a <!--a
/// > | b-->
///     ^
/// ```
fn after_line_ending_prefix(
    tokenizer: &mut Tokenizer,
    code: Code,
    return_state: Box<StateFn>,
) -> StateFnResult {
    tokenizer.enter(Token::HtmlTextData);
    return_state(tokenizer, code)
}
