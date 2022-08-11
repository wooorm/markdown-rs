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

use crate::constant::HTML_CDATA_PREFIX;
use crate::construct::partial_space_or_tab::space_or_tab;
use crate::state::{Name, State};
use crate::token::Token;
use crate::tokenizer::Tokenizer;

/// Start of HTML (text)
///
/// ```markdown
/// > | a <b> c
///       ^
/// ```
pub fn start(tokenizer: &mut Tokenizer) -> State {
    if Some(b'<') == tokenizer.current && tokenizer.parse_state.constructs.html_text {
        tokenizer.enter(Token::HtmlText);
        tokenizer.enter(Token::HtmlTextData);
        tokenizer.consume();
        State::Next(Name::HtmlTextOpen)
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
pub fn open(tokenizer: &mut Tokenizer) -> State {
    match tokenizer.current {
        Some(b'!') => {
            tokenizer.consume();
            State::Next(Name::HtmlTextDeclarationOpen)
        }
        Some(b'/') => {
            tokenizer.consume();
            State::Next(Name::HtmlTextTagCloseStart)
        }
        Some(b'?') => {
            tokenizer.consume();
            State::Next(Name::HtmlTextInstruction)
        }
        // ASCII alphabetical.
        Some(b'A'..=b'Z' | b'a'..=b'z') => {
            tokenizer.consume();
            State::Next(Name::HtmlTextTagOpen)
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
pub fn declaration_open(tokenizer: &mut Tokenizer) -> State {
    match tokenizer.current {
        Some(b'-') => {
            tokenizer.consume();
            State::Next(Name::HtmlTextCommentOpenInside)
        }
        // ASCII alphabetical.
        Some(b'A'..=b'Z' | b'a'..=b'z') => {
            tokenizer.consume();
            State::Next(Name::HtmlTextDeclaration)
        }
        Some(b'[') => {
            tokenizer.consume();
            State::Next(Name::HtmlTextCdataOpenInside)
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
pub fn comment_open_inside(tokenizer: &mut Tokenizer) -> State {
    match tokenizer.current {
        Some(b'-') => {
            tokenizer.consume();
            State::Next(Name::HtmlTextCommentStart)
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
pub fn comment_start(tokenizer: &mut Tokenizer) -> State {
    match tokenizer.current {
        Some(b'>') => State::Nok,
        Some(b'-') => {
            tokenizer.consume();
            State::Next(Name::HtmlTextCommentStartDash)
        }
        _ => State::Retry(Name::HtmlTextComment),
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
pub fn comment_start_dash(tokenizer: &mut Tokenizer) -> State {
    match tokenizer.current {
        Some(b'>') => State::Nok,
        _ => State::Retry(Name::HtmlTextComment),
    }
}

/// In a comment.
///
/// ```markdown
/// > | a <!--b--> c
///           ^
/// ```
pub fn comment(tokenizer: &mut Tokenizer) -> State {
    match tokenizer.current {
        None => State::Nok,
        Some(b'\n') => tokenizer.attempt(
            Name::HtmlTextLineEndingBefore,
            State::Next(Name::HtmlTextComment),
            State::Nok,
        ),
        Some(b'-') => {
            tokenizer.consume();
            State::Next(Name::HtmlTextCommentClose)
        }
        _ => {
            tokenizer.consume();
            State::Next(Name::HtmlTextComment)
        }
    }
}

/// In a comment, after `-`.
///
/// ```markdown
/// > | a <!--b--> c
///             ^
/// ```
pub fn comment_close(tokenizer: &mut Tokenizer) -> State {
    match tokenizer.current {
        Some(b'-') => {
            tokenizer.consume();
            State::Next(Name::HtmlTextEnd)
        }
        _ => State::Retry(Name::HtmlTextComment),
    }
}

/// After `<![`, inside CDATA, expecting `CDATA[`.
///
/// ```markdown
/// > | a <![CDATA[>&<]]> b
///          ^^^^^^
/// ```
pub fn cdata_open_inside(tokenizer: &mut Tokenizer) -> State {
    if tokenizer.current == Some(HTML_CDATA_PREFIX[tokenizer.tokenize_state.size]) {
        tokenizer.tokenize_state.size += 1;
        tokenizer.consume();

        if tokenizer.tokenize_state.size == HTML_CDATA_PREFIX.len() {
            tokenizer.tokenize_state.size = 0;
            State::Next(Name::HtmlTextCdata)
        } else {
            State::Next(Name::HtmlTextCdataOpenInside)
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
pub fn cdata(tokenizer: &mut Tokenizer) -> State {
    match tokenizer.current {
        None => State::Nok,
        Some(b'\n') => tokenizer.attempt(
            Name::HtmlTextLineEndingBefore,
            State::Next(Name::HtmlTextCdata),
            State::Nok,
        ),
        Some(b']') => {
            tokenizer.consume();
            State::Next(Name::HtmlTextCdataClose)
        }
        _ => {
            tokenizer.consume();
            State::Next(Name::HtmlTextCdata)
        }
    }
}

/// In CDATA, after `]`.
///
/// ```markdown
/// > | a <![CDATA[>&<]]> b
///                    ^
/// ```
pub fn cdata_close(tokenizer: &mut Tokenizer) -> State {
    match tokenizer.current {
        Some(b']') => {
            tokenizer.consume();
            State::Next(Name::HtmlTextCdataEnd)
        }
        _ => State::Retry(Name::HtmlTextCdata),
    }
}

/// In CDATA, after `]]`.
///
/// ```markdown
/// > | a <![CDATA[>&<]]> b
///                     ^
/// ```
pub fn cdata_end(tokenizer: &mut Tokenizer) -> State {
    match tokenizer.current {
        Some(b'>') => State::Retry(Name::HtmlTextEnd),
        Some(b']') => State::Retry(Name::HtmlTextCdataClose),
        _ => State::Retry(Name::HtmlTextCdata),
    }
}

/// In a declaration.
///
/// ```markdown
/// > | a <!b> c
///          ^
/// ```
pub fn declaration(tokenizer: &mut Tokenizer) -> State {
    match tokenizer.current {
        None | Some(b'>') => State::Retry(Name::HtmlTextEnd),
        Some(b'\n') => tokenizer.attempt(
            Name::HtmlTextLineEndingBefore,
            State::Next(Name::HtmlTextDeclaration),
            State::Nok,
        ),
        _ => {
            tokenizer.consume();
            State::Next(Name::HtmlTextDeclaration)
        }
    }
}

/// In an instruction.
///
/// ```markdown
/// > | a <?b?> c
///         ^
/// ```
pub fn instruction(tokenizer: &mut Tokenizer) -> State {
    match tokenizer.current {
        None => State::Nok,
        Some(b'\n') => tokenizer.attempt(
            Name::HtmlTextLineEndingBefore,
            State::Next(Name::HtmlTextInstruction),
            State::Nok,
        ),
        Some(b'?') => {
            tokenizer.consume();
            State::Next(Name::HtmlTextInstructionClose)
        }
        _ => {
            tokenizer.consume();
            State::Next(Name::HtmlTextInstruction)
        }
    }
}

/// In an instruction, after `?`.
///
/// ```markdown
/// > | a <?b?> c
///           ^
/// ```
pub fn instruction_close(tokenizer: &mut Tokenizer) -> State {
    match tokenizer.current {
        Some(b'>') => State::Retry(Name::HtmlTextEnd),
        _ => State::Retry(Name::HtmlTextInstruction),
    }
}

/// After `</`, in a closing tag, before a tag name.
///
/// ```markdown
/// > | a </b> c
///         ^
/// ```
pub fn tag_close_start(tokenizer: &mut Tokenizer) -> State {
    match tokenizer.current {
        // ASCII alphabetical.
        Some(b'A'..=b'Z' | b'a'..=b'z') => {
            tokenizer.consume();
            State::Next(Name::HtmlTextTagClose)
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
pub fn tag_close(tokenizer: &mut Tokenizer) -> State {
    match tokenizer.current {
        // ASCII alphanumerical and `-`.
        Some(b'-' | b'0'..=b'9' | b'A'..=b'Z' | b'a'..=b'z') => {
            tokenizer.consume();
            State::Next(Name::HtmlTextTagClose)
        }
        _ => State::Retry(Name::HtmlTextTagCloseBetween),
    }
}

/// In a closing tag, after the tag name.
///
/// ```markdown
/// > | a </b> c
///          ^
/// ```
pub fn tag_close_between(tokenizer: &mut Tokenizer) -> State {
    match tokenizer.current {
        Some(b'\n') => tokenizer.attempt(
            Name::HtmlTextLineEndingBefore,
            State::Next(Name::HtmlTextTagCloseBetween),
            State::Nok,
        ),
        Some(b'\t' | b' ') => {
            tokenizer.consume();
            State::Next(Name::HtmlTextTagCloseBetween)
        }
        _ => State::Retry(Name::HtmlTextEnd),
    }
}

/// After `<x`, in an opening tag name.
///
/// ```markdown
/// > | a <b> c
///         ^
/// ```
pub fn tag_open(tokenizer: &mut Tokenizer) -> State {
    match tokenizer.current {
        // ASCII alphanumerical and `-`.
        Some(b'-' | b'0'..=b'9' | b'A'..=b'Z' | b'a'..=b'z') => {
            tokenizer.consume();
            State::Next(Name::HtmlTextTagOpen)
        }
        Some(b'\t' | b'\n' | b' ' | b'/' | b'>') => State::Retry(Name::HtmlTextTagOpenBetween),
        _ => State::Nok,
    }
}

/// In an opening tag, after the tag name.
///
/// ```markdown
/// > | a <b> c
///         ^
/// ```
pub fn tag_open_between(tokenizer: &mut Tokenizer) -> State {
    match tokenizer.current {
        Some(b'\n') => tokenizer.attempt(
            Name::HtmlTextLineEndingBefore,
            State::Next(Name::HtmlTextTagOpenBetween),
            State::Nok,
        ),
        Some(b'\t' | b' ') => {
            tokenizer.consume();
            State::Next(Name::HtmlTextTagOpenBetween)
        }
        Some(b'/') => {
            tokenizer.consume();
            State::Next(Name::HtmlTextEnd)
        }
        // ASCII alphabetical and `:` and `_`.
        Some(b':' | b'A'..=b'Z' | b'_' | b'a'..=b'z') => {
            tokenizer.consume();
            State::Next(Name::HtmlTextTagOpenAttributeName)
        }
        _ => State::Retry(Name::HtmlTextEnd),
    }
}

/// In an attribute name.
///
/// ```markdown
/// > | a <b c> d
///          ^
/// ```
pub fn tag_open_attribute_name(tokenizer: &mut Tokenizer) -> State {
    match tokenizer.current {
        // ASCII alphabetical and `-`, `.`, `:`, and `_`.
        Some(b'-' | b'.' | b'0'..=b'9' | b':' | b'A'..=b'Z' | b'_' | b'a'..=b'z') => {
            tokenizer.consume();
            State::Next(Name::HtmlTextTagOpenAttributeName)
        }
        _ => State::Retry(Name::HtmlTextTagOpenAttributeNameAfter),
    }
}

/// After an attribute name, before an attribute initializer, the end of the
/// tag, or whitespace.
///
/// ```markdown
/// > | a <b c> d
///           ^
/// ```
pub fn tag_open_attribute_name_after(tokenizer: &mut Tokenizer) -> State {
    match tokenizer.current {
        Some(b'\n') => tokenizer.attempt(
            Name::HtmlTextLineEndingBefore,
            State::Next(Name::HtmlTextTagOpenAttributeNameAfter),
            State::Nok,
        ),
        Some(b'\t' | b' ') => {
            tokenizer.consume();
            State::Next(Name::HtmlTextTagOpenAttributeNameAfter)
        }
        Some(b'=') => {
            tokenizer.consume();
            State::Next(Name::HtmlTextTagOpenAttributeValueBefore)
        }
        _ => State::Retry(Name::HtmlTextTagOpenBetween),
    }
}

/// Before an unquoted, double quoted, or single quoted attribute value,
/// allowing whitespace.
///
/// ```markdown
/// > | a <b c=d> e
///            ^
/// ```
pub fn tag_open_attribute_value_before(tokenizer: &mut Tokenizer) -> State {
    match tokenizer.current {
        None | Some(b'<' | b'=' | b'>' | b'`') => State::Nok,
        Some(b'\n') => tokenizer.attempt(
            Name::HtmlTextLineEndingBefore,
            State::Next(Name::HtmlTextTagOpenAttributeValueBefore),
            State::Nok,
        ),
        Some(b'\t' | b' ') => {
            tokenizer.consume();
            State::Next(Name::HtmlTextTagOpenAttributeValueBefore)
        }
        Some(b'"' | b'\'') => {
            tokenizer.tokenize_state.marker = tokenizer.current.unwrap();
            tokenizer.consume();
            State::Next(Name::HtmlTextTagOpenAttributeValueQuoted)
        }
        Some(_) => {
            tokenizer.consume();
            State::Next(Name::HtmlTextTagOpenAttributeValueUnquoted)
        }
    }
}

/// In a double or single quoted attribute value.
///
/// ```markdown
/// > | a <b c="d"> e
///             ^
/// ```
pub fn tag_open_attribute_value_quoted(tokenizer: &mut Tokenizer) -> State {
    match tokenizer.current {
        None => {
            tokenizer.tokenize_state.marker = 0;
            State::Nok
        }
        Some(b'\n') => tokenizer.attempt(
            Name::HtmlTextLineEndingBefore,
            State::Next(Name::HtmlTextTagOpenAttributeValueQuoted),
            State::Nok,
        ),
        Some(b'"' | b'\'') if tokenizer.current.unwrap() == tokenizer.tokenize_state.marker => {
            tokenizer.tokenize_state.marker = 0;
            tokenizer.consume();
            State::Next(Name::HtmlTextTagOpenAttributeValueQuotedAfter)
        }
        _ => {
            tokenizer.consume();
            State::Next(Name::HtmlTextTagOpenAttributeValueQuoted)
        }
    }
}

/// In an unquoted attribute value.
///
/// ```markdown
/// > | a <b c=d> e
///            ^
/// ```
pub fn tag_open_attribute_value_unquoted(tokenizer: &mut Tokenizer) -> State {
    match tokenizer.current {
        None | Some(b'"' | b'\'' | b'<' | b'=' | b'`') => State::Nok,
        Some(b'\t' | b'\n' | b' ' | b'/' | b'>') => State::Retry(Name::HtmlTextTagOpenBetween),
        Some(_) => {
            tokenizer.consume();
            State::Next(Name::HtmlTextTagOpenAttributeValueUnquoted)
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
pub fn tag_open_attribute_value_quoted_after(tokenizer: &mut Tokenizer) -> State {
    match tokenizer.current {
        Some(b'\t' | b'\n' | b' ' | b'>' | b'/') => State::Retry(Name::HtmlTextTagOpenBetween),
        _ => State::Nok,
    }
}

/// In certain circumstances of a complete tag where only an `>` is allowed.
///
/// ```markdown
/// > | a <b c="d"> e
///               ^
/// ```
pub fn end(tokenizer: &mut Tokenizer) -> State {
    match tokenizer.current {
        Some(b'>') => {
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
pub fn line_ending_before(tokenizer: &mut Tokenizer) -> State {
    match tokenizer.current {
        Some(b'\n') => {
            tokenizer.exit(Token::HtmlTextData);
            tokenizer.enter(Token::LineEnding);
            tokenizer.consume();
            tokenizer.exit(Token::LineEnding);
            State::Next(Name::HtmlTextLineEndingAfter)
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
pub fn line_ending_after(tokenizer: &mut Tokenizer) -> State {
    let name = space_or_tab(tokenizer);
    tokenizer.attempt(
        name,
        State::Next(Name::HtmlTextLineEndingAfterPrefix),
        State::Next(Name::HtmlTextLineEndingAfterPrefix),
    )
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
pub fn line_ending_after_prefix(tokenizer: &mut Tokenizer) -> State {
    tokenizer.enter(Token::HtmlTextData);
    State::Ok
}
