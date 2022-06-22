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
//! *   [`HtmlText`][TokenType::HtmlText]
//! *   [`HtmlTextData`][TokenType::HtmlTextData]
//!
//! ## References
//!
//! *   [`html-text.js` in `micromark`](https://github.com/micromark/micromark/blob/main/packages/micromark-core-commonmark/dev/lib/html-text.js)
//! *   [*§ 6.6 Raw HTML* in `CommonMark`](https://spec.commonmark.org/0.30/#raw-html)
//!
//! [text]: crate::content::text
//! [html_flow]: crate::construct::html_flow
//! [html-parsing]: https://html.spec.whatwg.org/multipage/parsing.html#parsing

use crate::construct::partial_space_or_tab::space_or_tab_opt;
use crate::tokenizer::{Code, State, StateFn, StateFnResult, TokenType, Tokenizer};

/// Start of HTML (text)
///
/// ```markdown
/// a |<x> b
/// ```
pub fn start(tokenizer: &mut Tokenizer, code: Code) -> StateFnResult {
    if Code::Char('<') == code {
        tokenizer.enter(TokenType::HtmlText);
        tokenizer.enter(TokenType::HtmlTextData);
        tokenizer.consume(code);
        (State::Fn(Box::new(open)), None)
    } else {
        (State::Nok, None)
    }
}

/// After `<`, before a tag name or other stuff.
///
/// ```markdown
/// a <|x /> b
/// a <|!doctype> b
/// a <|!--xxx--/> b
/// ```
fn open(tokenizer: &mut Tokenizer, code: Code) -> StateFnResult {
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

/// After `<!`, so inside a declaration, comment, or CDATA.
///
/// ```markdown
/// a <!|doctype> b
/// a <!|--xxx--> b
/// a <!|[CDATA[>&<]]> b
/// ```
fn declaration_open(tokenizer: &mut Tokenizer, code: Code) -> StateFnResult {
    match code {
        Code::Char('-') => {
            tokenizer.consume(code);
            (State::Fn(Box::new(comment_open_inside)), None)
        }
        Code::Char('[') => {
            tokenizer.consume(code);
            let buffer = vec!['C', 'D', 'A', 'T', 'A', '['];
            (
                State::Fn(Box::new(|t, c| cdata_open_inside(t, c, buffer, 0))),
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

/// After `<!-`, inside a comment, before another `-`.
///
/// ```markdown
/// a <!-|-xxx--> b
/// ```
fn comment_open_inside(tokenizer: &mut Tokenizer, code: Code) -> StateFnResult {
    match code {
        Code::Char('-') => {
            tokenizer.consume(code);
            (State::Fn(Box::new(comment_start)), None)
        }
        _ => (State::Nok, None),
    }
}

/// After `<!--`, inside a comment
///
/// > **Note**: [html (flow)][html_flow] does allow `<!-->` or `<!--->` as
/// > empty comments.
/// > This is prohibited in html (text).
/// > See: <https://github.com/commonmark/commonmark-spec/issues/712>.
///
/// ```markdown
/// a <!--|xxx--> b
/// ```
///
/// [html_flow]: crate::construct::html_flow
fn comment_start(tokenizer: &mut Tokenizer, code: Code) -> StateFnResult {
    match code {
        Code::None | Code::Char('>') => (State::Nok, None),
        Code::Char('-') => {
            tokenizer.consume(code);
            (State::Fn(Box::new(comment_start_dash)), None)
        }
        _ => comment(tokenizer, code),
    }
}

/// After `<!---`, inside a comment
///
/// > **Note**: [html (flow)][html_flow] does allow `<!--->` as an empty
/// > comment.
/// > This is prohibited in html (text).
/// > See: <https://github.com/commonmark/commonmark-spec/issues/712>.
///
/// ```markdown
/// a <!---|xxx--> b
/// ```
///
/// [html_flow]: crate::construct::html_flow
fn comment_start_dash(tokenizer: &mut Tokenizer, code: Code) -> StateFnResult {
    match code {
        Code::None | Code::Char('>') => (State::Nok, None),
        _ => comment(tokenizer, code),
    }
}

/// In a comment.
///
/// ```markdown
/// a <!--|xxx--> b
/// a <!--x|xx--> b
/// ```
fn comment(tokenizer: &mut Tokenizer, code: Code) -> StateFnResult {
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

/// In a comment, after `-`.
///
/// ```markdown
/// a <!--xxx-|-> b
/// a <!--xxx-|yyy--> b
/// ```
fn comment_close(tokenizer: &mut Tokenizer, code: Code) -> StateFnResult {
    match code {
        Code::Char('-') => {
            tokenizer.consume(code);
            (State::Fn(Box::new(end)), None)
        }
        _ => comment(tokenizer, code),
    }
}

/// After `<![`, inside CDATA, expecting `CDATA[`.
///
/// ```markdown
/// a <![|CDATA[>&<]]> b
/// a <![CD|ATA[>&<]]> b
/// a <![CDA|TA[>&<]]> b
/// a <![CDAT|A[>&<]]> b
/// a <![CDATA|[>&<]]> b
/// ```
fn cdata_open_inside(
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
                    State::Fn(Box::new(move |t, c| {
                        cdata_open_inside(t, c, buffer, index + 1)
                    })),
                    None,
                )
            }
        }
        _ => (State::Nok, None),
    }
}

/// In CDATA.
///
/// ```markdown
/// a <![CDATA[|>&<]]> b
/// ```
fn cdata(tokenizer: &mut Tokenizer, code: Code) -> StateFnResult {
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

/// In CDATA, after `]`.
///
/// ```markdown
/// a <![CDATA[>&<]|]> b
/// ```
fn cdata_close(tokenizer: &mut Tokenizer, code: Code) -> StateFnResult {
    match code {
        Code::Char(']') => {
            tokenizer.consume(code);
            (State::Fn(Box::new(cdata_end)), None)
        }
        _ => cdata(tokenizer, code),
    }
}

/// In CDATA, after `]]`.
///
/// ```markdown
/// a <![CDATA[>&<]]|> b
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
/// a <!a|b> b
/// ```
fn declaration(tokenizer: &mut Tokenizer, code: Code) -> StateFnResult {
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

/// In an instruction.
///
/// ```markdown
/// a <?|ab?> b
/// a <?a|b?> b
/// ```
fn instruction(tokenizer: &mut Tokenizer, code: Code) -> StateFnResult {
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

/// In an instruction, after `?`.
///
/// ```markdown
/// a <?aa?|> b
/// a <?aa?|bb?> b
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
/// a </|x> b
/// ```
fn tag_close_start(tokenizer: &mut Tokenizer, code: Code) -> StateFnResult {
    match code {
        Code::Char(char) if char.is_ascii_alphabetic() => {
            tokenizer.consume(code);
            (State::Fn(Box::new(tag_close)), None)
        }
        _ => (State::Nok, None),
    }
}

/// After `</x`, in a tag name.
///
/// ```markdown
/// a </x|> b
/// a </x|y> b
/// ```
fn tag_close(tokenizer: &mut Tokenizer, code: Code) -> StateFnResult {
    match code {
        Code::Char(char) if char == '-' || char.is_ascii_alphanumeric() => {
            tokenizer.consume(code);
            (State::Fn(Box::new(tag_close)), None)
        }
        _ => tag_close_between(tokenizer, code),
    }
}

/// In a closing tag, after the tag name.
///
/// ```markdown
/// a </x| > b
/// a </xy |> b
/// ```
fn tag_close_between(tokenizer: &mut Tokenizer, code: Code) -> StateFnResult {
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

/// After `<x`, in an opening tag name.
///
/// ```markdown
/// a <x|> b
/// ```
fn tag_open(tokenizer: &mut Tokenizer, code: Code) -> StateFnResult {
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

/// In an opening tag, after the tag name.
///
/// ```markdown
/// a <x| y> b
/// a <x |y="z"> b
/// a <x |/> b
/// ```
fn tag_open_between(tokenizer: &mut Tokenizer, code: Code) -> StateFnResult {
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

/// In an attribute name.
///
/// ```markdown
/// a <x :|> b
/// a <x _|> b
/// a <x a|> b
/// ```
fn tag_open_attribute_name(tokenizer: &mut Tokenizer, code: Code) -> StateFnResult {
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

/// After an attribute name, before an attribute initializer, the end of the
/// tag, or whitespace.
///
/// ```markdown
/// a <x a|> b
/// a <x a|=b> b
/// a <x a|="c"> b
/// ```
fn tag_open_attribute_name_after(tokenizer: &mut Tokenizer, code: Code) -> StateFnResult {
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

/// Before an unquoted, double quoted, or single quoted attribute value,
/// allowing whitespace.
///
/// ```markdown
/// a <x a=|b> b
/// a <x a=|"c"> b
/// ```
fn tag_open_attribute_value_before(tokenizer: &mut Tokenizer, code: Code) -> StateFnResult {
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
                State::Fn(Box::new(move |t, c| {
                    tag_open_attribute_value_quoted(t, c, char)
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

/// In a double or single quoted attribute value.
///
/// ```markdown
/// a <x a="|"> b
/// a <x a='|'> b
/// ```
fn tag_open_attribute_value_quoted(
    tokenizer: &mut Tokenizer,
    code: Code,
    marker: char,
) -> StateFnResult {
    match code {
        Code::None => (State::Nok, None),
        Code::CarriageReturnLineFeed | Code::Char('\r' | '\n') => at_line_ending(
            tokenizer,
            code,
            Box::new(move |t, c| tag_open_attribute_value_quoted(t, c, marker)),
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
                State::Fn(Box::new(move |t, c| {
                    tag_open_attribute_value_quoted(t, c, marker)
                })),
                None,
            )
        }
    }
}

/// In an unquoted attribute value.
///
/// ```markdown
/// a <x a=b|c> b
/// ```
fn tag_open_attribute_value_unquoted(tokenizer: &mut Tokenizer, code: Code) -> StateFnResult {
    match code {
        Code::None | Code::Char('"' | '\'' | '<' | '=' | '`') => (State::Nok, None),
        Code::CarriageReturnLineFeed
        | Code::VirtualSpace
        | Code::Char('\r' | '\n' | '\t' | ' ' | '/' | '>') => tag_open_between(tokenizer, code),
        Code::Char(_) => {
            tokenizer.consume(code);
            (State::Fn(Box::new(tag_open_attribute_value_unquoted)), None)
        }
    }
}

/// After a double or single quoted attribute value, before whitespace or the
/// end of the tag.
///
/// ```markdown
/// a <x a="b"|> b
/// ```
fn tag_open_attribute_value_quoted_after(tokenizer: &mut Tokenizer, code: Code) -> StateFnResult {
    match code {
        Code::CarriageReturnLineFeed
        | Code::VirtualSpace
        | Code::Char('\r' | '\n' | '\t' | ' ' | '>' | '/') => tag_open_between(tokenizer, code),
        _ => (State::Nok, None),
    }
}

/// In certain circumstances of a complete tag where only an `>` is allowed.
///
/// ```markdown
/// a <x a="b"|> b
/// a <!--xx--|> b
/// a <x /|> b
/// ```
fn end(tokenizer: &mut Tokenizer, code: Code) -> StateFnResult {
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

/// At an allowed line ending.
///
/// > **Note**: we can’t have blank lines in text, so no need to worry about
/// > empty tokens.
///
/// ```markdown
/// a <!--a|
/// b--> b
/// ```
fn at_line_ending(
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

/// After a line ending.
///
/// > **Note**: we can’t have blank lines in text, so no need to worry about
/// > empty tokens.
///
/// ```markdown
/// a <!--a
/// |b--> b
/// ```
fn after_line_ending(
    tokenizer: &mut Tokenizer,
    code: Code,
    return_state: Box<StateFn>,
) -> StateFnResult {
    tokenizer.go(space_or_tab_opt(), |t, c| {
        after_line_ending_prefix(t, c, return_state)
    })(tokenizer, code)
}

/// After a line ending, after indent.
///
/// > **Note**: we can’t have blank lines in text, so no need to worry about
/// > empty tokens.
///
/// ```markdown
/// a <!--a
///   |b--> b
/// ```
fn after_line_ending_prefix(
    tokenizer: &mut Tokenizer,
    code: Code,
    return_state: Box<StateFn>,
) -> StateFnResult {
    tokenizer.enter(TokenType::HtmlTextData);
    return_state(tokenizer, code)
}
