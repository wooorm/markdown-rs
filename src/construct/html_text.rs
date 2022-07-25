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
use crate::tokenizer::{Code, State, StateFn, Tokenizer};
use crate::util::codes::parse;

/// Start of HTML (text)
///
/// ```markdown
/// > | a <b> c
///       ^
/// ```
pub fn start(tokenizer: &mut Tokenizer) -> State {
    if Code::Char('<') == tokenizer.current && tokenizer.parse_state.constructs.html_text {
        tokenizer.enter(Token::HtmlText);
        tokenizer.enter(Token::HtmlTextData);
        tokenizer.consume();
        State::Fn(Box::new(open))
    } else {
        State::Nok
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
fn open(tokenizer: &mut Tokenizer) -> State {
    match tokenizer.current {
        Code::Char('!') => {
            tokenizer.consume();
            State::Fn(Box::new(declaration_open))
        }
        Code::Char('/') => {
            tokenizer.consume();
            State::Fn(Box::new(tag_close_start))
        }
        Code::Char('?') => {
            tokenizer.consume();
            State::Fn(Box::new(instruction))
        }
        Code::Char('A'..='Z' | 'a'..='z') => {
            tokenizer.consume();
            State::Fn(Box::new(tag_open))
        }
        _ => State::Nok,
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
fn declaration_open(tokenizer: &mut Tokenizer) -> State {
    match tokenizer.current {
        Code::Char('-') => {
            tokenizer.consume();
            State::Fn(Box::new(comment_open_inside))
        }
        Code::Char('[') => {
            tokenizer.consume();
            let buffer = parse("CDATA[");
            State::Fn(Box::new(|t| cdata_open_inside(t, buffer, 0)))
        }
        Code::Char('A'..='Z' | 'a'..='z') => {
            tokenizer.consume();
            State::Fn(Box::new(declaration))
        }
        _ => State::Nok,
    }
}

/// After `<!-`, inside a comment, before another `-`.
///
/// ```markdown
/// > | a <!--b--> c
///          ^
/// ```
fn comment_open_inside(tokenizer: &mut Tokenizer) -> State {
    match tokenizer.current {
        Code::Char('-') => {
            tokenizer.consume();
            State::Fn(Box::new(comment_start))
        }
        _ => State::Nok,
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
fn comment_start(tokenizer: &mut Tokenizer) -> State {
    match tokenizer.current {
        Code::None | Code::Char('>') => State::Nok,
        Code::Char('-') => {
            tokenizer.consume();
            State::Fn(Box::new(comment_start_dash))
        }
        _ => comment(tokenizer),
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
fn comment_start_dash(tokenizer: &mut Tokenizer) -> State {
    match tokenizer.current {
        Code::None | Code::Char('>') => State::Nok,
        _ => comment(tokenizer),
    }
}

/// In a comment.
///
/// ```markdown
/// > | a <!--b--> c
///           ^
/// ```
fn comment(tokenizer: &mut Tokenizer) -> State {
    match tokenizer.current {
        Code::None => State::Nok,
        Code::CarriageReturnLineFeed | Code::Char('\n' | '\r') => {
            at_line_ending(tokenizer, Box::new(comment))
        }
        Code::Char('-') => {
            tokenizer.consume();
            State::Fn(Box::new(comment_close))
        }
        _ => {
            tokenizer.consume();
            State::Fn(Box::new(comment))
        }
    }
}

/// In a comment, after `-`.
///
/// ```markdown
/// > | a <!--b--> c
///             ^
/// ```
fn comment_close(tokenizer: &mut Tokenizer) -> State {
    match tokenizer.current {
        Code::Char('-') => {
            tokenizer.consume();
            State::Fn(Box::new(end))
        }
        _ => comment(tokenizer),
    }
}

/// After `<![`, inside CDATA, expecting `CDATA[`.
///
/// ```markdown
/// > | a <![CDATA[>&<]]> b
///          ^^^^^^
/// ```
fn cdata_open_inside(tokenizer: &mut Tokenizer, buffer: Vec<Code>, index: usize) -> State {
    if tokenizer.current == buffer[index] {
        tokenizer.consume();

        if index + 1 == buffer.len() {
            State::Fn(Box::new(cdata))
        } else {
            State::Fn(Box::new(move |t| cdata_open_inside(t, buffer, index + 1)))
        }
    } else {
        State::Nok
    }
}

/// In CDATA.
///
/// ```markdown
/// > | a <![CDATA[>&<]]> b
///                ^^^
/// ```
fn cdata(tokenizer: &mut Tokenizer) -> State {
    match tokenizer.current {
        Code::None => State::Nok,
        Code::CarriageReturnLineFeed | Code::Char('\n' | '\r') => {
            at_line_ending(tokenizer, Box::new(cdata))
        }
        Code::Char(']') => {
            tokenizer.consume();
            State::Fn(Box::new(cdata_close))
        }
        _ => {
            tokenizer.consume();
            State::Fn(Box::new(cdata))
        }
    }
}

/// In CDATA, after `]`.
///
/// ```markdown
/// > | a <![CDATA[>&<]]> b
///                    ^
/// ```
fn cdata_close(tokenizer: &mut Tokenizer) -> State {
    match tokenizer.current {
        Code::Char(']') => {
            tokenizer.consume();
            State::Fn(Box::new(cdata_end))
        }
        _ => cdata(tokenizer),
    }
}

/// In CDATA, after `]]`.
///
/// ```markdown
/// > | a <![CDATA[>&<]]> b
///                     ^
/// ```
fn cdata_end(tokenizer: &mut Tokenizer) -> State {
    match tokenizer.current {
        Code::Char('>') => end(tokenizer),
        Code::Char(']') => cdata_close(tokenizer),
        _ => cdata(tokenizer),
    }
}

/// In a declaration.
///
/// ```markdown
/// > | a <!b> c
///          ^
/// ```
fn declaration(tokenizer: &mut Tokenizer) -> State {
    match tokenizer.current {
        Code::None | Code::Char('>') => end(tokenizer),
        Code::CarriageReturnLineFeed | Code::Char('\n' | '\r') => {
            at_line_ending(tokenizer, Box::new(declaration))
        }
        _ => {
            tokenizer.consume();
            State::Fn(Box::new(declaration))
        }
    }
}

/// In an instruction.
///
/// ```markdown
/// > | a <?b?> c
///         ^
/// ```
fn instruction(tokenizer: &mut Tokenizer) -> State {
    match tokenizer.current {
        Code::None => State::Nok,
        Code::CarriageReturnLineFeed | Code::Char('\n' | '\r') => {
            at_line_ending(tokenizer, Box::new(instruction))
        }
        Code::Char('?') => {
            tokenizer.consume();
            State::Fn(Box::new(instruction_close))
        }
        _ => {
            tokenizer.consume();
            State::Fn(Box::new(instruction))
        }
    }
}

/// In an instruction, after `?`.
///
/// ```markdown
/// > | a <?b?> c
///           ^
/// ```
fn instruction_close(tokenizer: &mut Tokenizer) -> State {
    match tokenizer.current {
        Code::Char('>') => end(tokenizer),
        _ => instruction(tokenizer),
    }
}

/// After `</`, in a closing tag, before a tag name.
///
/// ```markdown
/// > | a </b> c
///         ^
/// ```
fn tag_close_start(tokenizer: &mut Tokenizer) -> State {
    match tokenizer.current {
        Code::Char('A'..='Z' | 'a'..='z') => {
            tokenizer.consume();
            State::Fn(Box::new(tag_close))
        }
        _ => State::Nok,
    }
}

/// After `</x`, in a tag name.
///
/// ```markdown
/// > | a </b> c
///          ^
/// ```
fn tag_close(tokenizer: &mut Tokenizer) -> State {
    match tokenizer.current {
        Code::Char('-' | '0'..='9' | 'A'..='Z' | 'a'..='z') => {
            tokenizer.consume();
            State::Fn(Box::new(tag_close))
        }
        _ => tag_close_between(tokenizer),
    }
}

/// In a closing tag, after the tag name.
///
/// ```markdown
/// > | a </b> c
///          ^
/// ```
fn tag_close_between(tokenizer: &mut Tokenizer) -> State {
    match tokenizer.current {
        Code::CarriageReturnLineFeed | Code::Char('\n' | '\r') => {
            at_line_ending(tokenizer, Box::new(tag_close_between))
        }
        Code::VirtualSpace | Code::Char('\t' | ' ') => {
            tokenizer.consume();
            State::Fn(Box::new(tag_close_between))
        }
        _ => end(tokenizer),
    }
}

/// After `<x`, in an opening tag name.
///
/// ```markdown
/// > | a <b> c
///         ^
/// ```
fn tag_open(tokenizer: &mut Tokenizer) -> State {
    match tokenizer.current {
        Code::Char('-' | '0'..='9' | 'A'..='Z' | 'a'..='z') => {
            tokenizer.consume();
            State::Fn(Box::new(tag_open))
        }
        Code::CarriageReturnLineFeed
        | Code::VirtualSpace
        | Code::Char('\t' | '\n' | '\r' | ' ' | '/' | '>') => tag_open_between(tokenizer),
        _ => State::Nok,
    }
}

/// In an opening tag, after the tag name.
///
/// ```markdown
/// > | a <b> c
///         ^
/// ```
fn tag_open_between(tokenizer: &mut Tokenizer) -> State {
    match tokenizer.current {
        Code::CarriageReturnLineFeed | Code::Char('\n' | '\r') => {
            at_line_ending(tokenizer, Box::new(tag_open_between))
        }
        Code::VirtualSpace | Code::Char('\t' | ' ') => {
            tokenizer.consume();
            State::Fn(Box::new(tag_open_between))
        }
        Code::Char('/') => {
            tokenizer.consume();
            State::Fn(Box::new(end))
        }
        Code::Char(':' | 'A'..='Z' | '_' | 'a'..='z') => {
            tokenizer.consume();
            State::Fn(Box::new(tag_open_attribute_name))
        }
        _ => end(tokenizer),
    }
}

/// In an attribute name.
///
/// ```markdown
/// > | a <b c> d
///          ^
/// ```
fn tag_open_attribute_name(tokenizer: &mut Tokenizer) -> State {
    match tokenizer.current {
        Code::Char('-' | '.' | '0'..='9' | ':' | 'A'..='Z' | '_' | 'a'..='z') => {
            tokenizer.consume();
            State::Fn(Box::new(tag_open_attribute_name))
        }
        _ => tag_open_attribute_name_after(tokenizer),
    }
}

/// After an attribute name, before an attribute initializer, the end of the
/// tag, or whitespace.
///
/// ```markdown
/// > | a <b c> d
///           ^
/// ```
fn tag_open_attribute_name_after(tokenizer: &mut Tokenizer) -> State {
    match tokenizer.current {
        Code::CarriageReturnLineFeed | Code::Char('\n' | '\r') => {
            at_line_ending(tokenizer, Box::new(tag_open_attribute_name_after))
        }
        Code::VirtualSpace | Code::Char('\t' | ' ') => {
            tokenizer.consume();
            State::Fn(Box::new(tag_open_attribute_name_after))
        }
        Code::Char('=') => {
            tokenizer.consume();
            State::Fn(Box::new(tag_open_attribute_value_before))
        }
        _ => tag_open_between(tokenizer),
    }
}

/// Before an unquoted, double quoted, or single quoted attribute value,
/// allowing whitespace.
///
/// ```markdown
/// > | a <b c=d> e
///            ^
/// ```
fn tag_open_attribute_value_before(tokenizer: &mut Tokenizer) -> State {
    match tokenizer.current {
        Code::None | Code::Char('<' | '=' | '>' | '`') => State::Nok,
        Code::CarriageReturnLineFeed | Code::Char('\n' | '\r') => {
            at_line_ending(tokenizer, Box::new(tag_open_attribute_value_before))
        }
        Code::VirtualSpace | Code::Char('\t' | ' ') => {
            tokenizer.consume();
            State::Fn(Box::new(tag_open_attribute_value_before))
        }
        Code::Char(char) if char == '"' || char == '\'' => {
            tokenizer.consume();
            State::Fn(Box::new(move |t| tag_open_attribute_value_quoted(t, char)))
        }
        Code::Char(_) => {
            tokenizer.consume();
            State::Fn(Box::new(tag_open_attribute_value_unquoted))
        }
    }
}

/// In a double or single quoted attribute value.
///
/// ```markdown
/// > | a <b c="d"> e
///             ^
/// ```
fn tag_open_attribute_value_quoted(tokenizer: &mut Tokenizer, marker: char) -> State {
    match tokenizer.current {
        Code::None => State::Nok,
        Code::CarriageReturnLineFeed | Code::Char('\n' | '\r') => at_line_ending(
            tokenizer,
            Box::new(move |t| tag_open_attribute_value_quoted(t, marker)),
        ),
        Code::Char(char) if char == marker => {
            tokenizer.consume();
            State::Fn(Box::new(tag_open_attribute_value_quoted_after))
        }
        _ => {
            tokenizer.consume();
            State::Fn(Box::new(move |t| {
                tag_open_attribute_value_quoted(t, marker)
            }))
        }
    }
}

/// In an unquoted attribute value.
///
/// ```markdown
/// > | a <b c=d> e
///            ^
/// ```
fn tag_open_attribute_value_unquoted(tokenizer: &mut Tokenizer) -> State {
    match tokenizer.current {
        Code::None | Code::Char('"' | '\'' | '<' | '=' | '`') => State::Nok,
        Code::CarriageReturnLineFeed
        | Code::VirtualSpace
        | Code::Char('\t' | '\n' | '\r' | ' ' | '/' | '>') => tag_open_between(tokenizer),
        Code::Char(_) => {
            tokenizer.consume();
            State::Fn(Box::new(tag_open_attribute_value_unquoted))
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
fn tag_open_attribute_value_quoted_after(tokenizer: &mut Tokenizer) -> State {
    match tokenizer.current {
        Code::CarriageReturnLineFeed
        | Code::VirtualSpace
        | Code::Char('\t' | '\n' | '\r' | ' ' | '>' | '/') => tag_open_between(tokenizer),
        _ => State::Nok,
    }
}

/// In certain circumstances of a complete tag where only an `>` is allowed.
///
/// ```markdown
/// > | a <b c="d"> e
///               ^
/// ```
fn end(tokenizer: &mut Tokenizer) -> State {
    match tokenizer.current {
        Code::Char('>') => {
            tokenizer.consume();
            tokenizer.exit(Token::HtmlTextData);
            tokenizer.exit(Token::HtmlText);
            State::Ok
        }
        _ => State::Nok,
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
fn at_line_ending(tokenizer: &mut Tokenizer, return_state: Box<StateFn>) -> State {
    match tokenizer.current {
        Code::CarriageReturnLineFeed | Code::Char('\n' | '\r') => {
            tokenizer.exit(Token::HtmlTextData);
            tokenizer.enter(Token::LineEnding);
            tokenizer.consume();
            tokenizer.exit(Token::LineEnding);
            State::Fn(Box::new(|t| after_line_ending(t, return_state)))
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
fn after_line_ending(tokenizer: &mut Tokenizer, return_state: Box<StateFn>) -> State {
    tokenizer.attempt_opt(space_or_tab(), |t| {
        after_line_ending_prefix(t, return_state)
    })(tokenizer)
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
fn after_line_ending_prefix(tokenizer: &mut Tokenizer, return_state: Box<StateFn>) -> State {
    tokenizer.enter(Token::HtmlTextData);
    return_state(tokenizer)
}
