//! HTML (flow) is a construct that occurs in the [flow][] content type.
//!
//! It forms with the following BNF:
//!
//! ```bnf
//! html_flow ::= raw | comment | instruction | declaration | cdata | basic | complete
//!
//! ; Note: closing tag name need to match opening tag name.
//! raw ::= '<' raw_tag_name [ [ ( whitespace | '>' ) *line ] *( eol *line ) ] [ '</' raw_tag_name *line ]
//! comment ::= '<!--' [ *'-' '>' *line | *line *( eol *line ) [ '-->' *line ] ]
//! instruction ::= '<?' [ '>' *line | *line *( eol *line ) [ '?>' *line ] ]
//! declaration ::= '<!' ascii_alphabetic *line *( eol *line ) [ '>' *line ]
//! cdata ::= '<![CDATA[' *line *( eol *line ) [ ']]>' *line ]
//! basic ::= '< [ '/' ] basic_tag_name [ [ '/' ] '>' *line *( eol 1*line ) ]
//! complete ::= ( opening_tag | closing_tag ) ( whitespace_optional *( eol 1*line ) | whitespace_optional )
//!
//! raw_tag_name ::= 'pre' | 'script' | 'style' | 'textarea' ; Note: case-insensitive.
//! basic_tag_name ::= 'address' | 'article' | 'aside' | ... ; See `constants.rs`, and note: case-insensitive.
//! opening_tag ::= '<' tag_name *( whitespace attribute ) [ whitespace_optional '/' ] whitespace_optional '>'
//! closing_tag ::= '</' tag_name whitespace_optional '>'
//! tag_name ::= ascii_alphabetic *( '-' | ascii_alphanumeric )
//! attribute ::= attribute_name [ whitespace_optional '=' whitespace_optional attribute_value ]
//! attribute_name ::= ( ':' | '_' | ascii_alphabetic ) *( '-' | '.' | ':' | '_' | ascii_alphanumeric )
//! attribute_value ::= '"' *( line - '"' ) '"' | "'" *( line - "'" )  "'" | 1*( line - space_or_tab - '"' - "'" - '/' - '<' - '=' - '>' - '`')
//!
//! whitespace ::= 1*space_or_tab
//! whitespace_optional ::= [ whitespace ]
//! line ::= code - eol
//! eol ::= '\r' | '\r\n' | '\n'
//! space_or_tab ::= ' ' | '\t'
//! ```
//!
//! The grammar for HTML in markdown does not resemble the rules of parsing
//! HTML according to the [*§ 13.2 Parsing HTML documents* in the HTML
//! spec][html-parsing].
//! As such, HTML in markdown *resembles* HTML, but is instead a (naïve?)
//! attempt to parse an XML-like language.
//! By extension, another notable property of the grammar is that it can
//! result in invalid HTML, in that it allows things that wouldn’t work or
//! wouldn’t work well in HTML, such as mismatched tags.
//!
//! Interestingly, most of the productions above have a clear opening and
//! closing condition (raw, comment, insutrction, declaration, cdata), but the
//! closing condition does not need to be satisfied.
//! In this case, the parser never has to backtrack.
//!
//! Because the **basic** and **complete** productions in the grammar form with
//! a tag, followed by more stuff, and stop at a blank line, it is possible to
//! interleave (a word for switching between languages) markdown and HTML
//! together, by placing the opening and closing tags on their own lines,
//! with blank lines between them and markdown.
//! For example:
//!
//! ```markdown
//! <div>This is a <code>div</code> but *this* is not emphasis.</div>
//!
//! <div>
//!
//! This is a paragraph in a `div` and *this* is emphasis.
//!
//! </div>
//! ```
//!
//! The **complete** production of HTML (flow) is not allowed to interrupt
//! content.
//! That means that a blank line is needed between a [paragraph][] and it.
//! However, [HTML (text)][html_text] has a similar production, which will
//! typically kick-in instead.
//!
//! The list of tag names allowed in the **raw** production are defined in
//! [`HTML_RAW_NAMES`][html_raw_names].
//! This production exists because there are a few cases where markdown
//! *inside* some elements, and hence interleaving, does not make sense.
//!
//! The list of tag names allowed in the **basic** production are defined in
//! [`HTML_BLOCK_NAMES`][html_block_names].
//! This production exists because there are a few cases where we can decide
//! early that something is going to be a flow (block) element instead of a
//! phrasing (inline) element.
//! We *can* interrupt and don’t have to care too much about it being
//! well-formed.
//!
//! ## References
//!
//! *   [`html-flow.js` in `micromark`](https://github.com/micromark/micromark/blob/main/packages/micromark-core-commonmark/dev/lib/html-flow.js)
//! *   [*§ 4.6 HTML blocks* in `CommonMark`](https://spec.commonmark.org/0.30/#html-blocks)
//!
//! [flow]: crate::content::flow
//! [html_text]: crate::construct::html_text
//! [paragraph]: crate::construct::paragraph
//! [html_raw_names]: crate::constant::HTML_RAW_NAMES
//! [html_block_names]: crate::constant::HTML_BLOCK_NAMES
//! [html-parsing]: https://html.spec.whatwg.org/multipage/parsing.html#parsing

use crate::constant::{HTML_BLOCK_NAMES, HTML_RAW_NAMES, HTML_RAW_SIZE_MAX};
use crate::construct::{blank_line::start as blank_line, partial_whitespace::start as whitespace};
use crate::tokenizer::{Code, State, StateFnResult, TokenType, Tokenizer};

/// Kind of HTML (flow).
#[derive(Debug, Clone, PartialEq)]
enum Kind {
    /// Not yet known.
    Unknown,
    /// Symbol for `<script>` (condition 1).
    Raw,
    /// Symbol for `<!---->` (condition 2).
    Comment,
    /// Symbol for `<?php?>` (condition 3).
    Instruction,
    /// Symbol for `<!doctype>` (condition 4).
    Declaration,
    /// Symbol for `<![CDATA[]]>` (condition 5).
    Cdata,
    /// Symbol for `<div` (condition 6).
    Basic,
    /// Symbol for `<x>` (condition 7).
    Complete,
}

/// Type of quote, if we’re in an attribure, in complete (condition 7).
#[derive(Debug, Clone, PartialEq)]
enum QuoteKind {
    /// Not in a quoted attribute.
    None,
    /// In a double quoted (`"`) attribute.
    Double,
    /// In a single quoted (`"`) attribute.
    Single,
}

/// State needed to parse HTML (flow).
#[derive(Debug, Clone)]
struct Info {
    /// Kind of HTML (flow).
    kind: Kind,
    /// Whether this is a start tag (`<` not followed by `/`).
    start_tag: bool,
    /// Used depending on `kind` to either collect all parsed characters, or to
    /// store expected characters.
    buffer: Vec<char>,
    /// `index` into `buffer` when expecting certain characters.
    index: usize,
    /// Current quote, when in a double or single quoted attribute value.
    quote: QuoteKind,
}

// To do: mark as concrete (block quotes or lists can’t “pierce” into HTML).

/// Start of HTML (flow), before optional whitespace.
///
/// ```markdown
/// |<x />
/// ```
///
pub fn start(tokenizer: &mut Tokenizer, code: Code) -> StateFnResult {
    tokenizer.enter(TokenType::HtmlFlow);
    tokenizer.enter(TokenType::HtmlFlowData);
    tokenizer.attempt(
        |tokenizer, code| whitespace(tokenizer, code, TokenType::Whitespace),
        |_ok| Box::new(before),
    )(tokenizer, code)
}

/// After optional whitespace, before `<`.
///
/// ```markdown
/// |<x />
/// ```
fn before(tokenizer: &mut Tokenizer, code: Code) -> StateFnResult {
    if Code::Char('<') == code {
        tokenizer.consume(code);
        (
            State::Fn(Box::new(|tokenizer, code| {
                open(
                    tokenizer,
                    Info {
                        kind: Kind::Unknown,
                        start_tag: false,
                        buffer: vec![],
                        index: 0,
                        quote: QuoteKind::None,
                    },
                    code,
                )
            })),
            None,
        )
    } else {
        (State::Nok, None)
    }
}

/// After `<`, before a tag name or other stuff.
///
/// ```markdown
/// <|x />
/// <|!doctype>
/// <|!--xxx-->
/// ```
fn open(tokenizer: &mut Tokenizer, info: Info, code: Code) -> StateFnResult {
    match code {
        Code::Char('!') => {
            tokenizer.consume(code);
            (
                State::Fn(Box::new(|tokenizer, code| {
                    declaration_open(tokenizer, info, code)
                })),
                None,
            )
        }
        Code::Char('/') => {
            tokenizer.consume(code);
            (
                State::Fn(Box::new(|tokenizer, code| {
                    tag_close_start(tokenizer, info, code)
                })),
                None,
            )
        }
        Code::Char('?') => {
            // To do: life times.
            let mut clone = info;
            clone.kind = Kind::Instruction;
            tokenizer.consume(code);
            // While we’re in an instruction instead of a declaration, we’re on a `?`
            // right now, so we do need to search for `>`, similar to declarations.
            (
                State::Fn(Box::new(|tokenizer, code| {
                    continuation_declaration_inside(tokenizer, clone, code)
                })),
                None,
            )
        }
        Code::Char(char) if char.is_ascii_alphabetic() => {
            // To do: life times.
            let mut clone = info;
            clone.start_tag = true;
            tag_name(tokenizer, clone, code)
        }
        _ => (State::Nok, None),
    }
}

/// After `<!`, so inside a declaration, comment, or CDATA.
///
/// ```markdown
/// <!|doctype>
/// <!|--xxx-->
/// <!|[CDATA[>&<]]>
/// ```
fn declaration_open(tokenizer: &mut Tokenizer, info: Info, code: Code) -> StateFnResult {
    match code {
        Code::Char('-') => {
            tokenizer.consume(code);
            let mut clone = info;
            clone.kind = Kind::Comment;
            (
                State::Fn(Box::new(|tokenizer, code| {
                    comment_open_inside(tokenizer, clone, code)
                })),
                None,
            )
        }
        Code::Char('[') => {
            tokenizer.consume(code);
            let mut clone = info;
            clone.kind = Kind::Cdata;
            clone.buffer = vec!['C', 'D', 'A', 'T', 'A', '['];
            clone.index = 0;
            (
                State::Fn(Box::new(|tokenizer, code| {
                    cdata_open_inside(tokenizer, clone, code)
                })),
                None,
            )
        }
        Code::Char(char) if char.is_ascii_alphabetic() => {
            tokenizer.consume(code);
            // To do: life times.
            let mut clone = info;
            clone.kind = Kind::Declaration;
            (
                State::Fn(Box::new(|tokenizer, code| {
                    continuation_declaration_inside(tokenizer, clone, code)
                })),
                None,
            )
        }
        _ => (State::Nok, None),
    }
}

/// After `<!-`, inside a comment, before another `-`.
///
/// ```markdown
/// <!-|-xxx-->
/// ```
fn comment_open_inside(tokenizer: &mut Tokenizer, info: Info, code: Code) -> StateFnResult {
    match code {
        Code::Char('-') => {
            tokenizer.consume(code);
            (
                State::Fn(Box::new(|tokenizer, code| {
                    continuation_declaration_inside(tokenizer, info, code)
                })),
                None,
            )
        }
        _ => (State::Nok, None),
    }
}

/// After `<![`, inside CDATA, expecting `CDATA[`.
///
/// ```markdown
/// <![|CDATA[>&<]]>
/// <![CD|ATA[>&<]]>
/// <![CDA|TA[>&<]]>
/// <![CDAT|A[>&<]]>
/// <![CDATA|[>&<]]>
/// ```
fn cdata_open_inside(tokenizer: &mut Tokenizer, info: Info, code: Code) -> StateFnResult {
    match code {
        Code::Char(char) if char == info.buffer[info.index] => {
            let mut clone = info;
            clone.index += 1;
            tokenizer.consume(code);

            if clone.index == clone.buffer.len() {
                clone.buffer.clear();
                (
                    State::Fn(Box::new(|tokenizer, code| {
                        continuation(tokenizer, clone, code)
                    })),
                    None,
                )
            } else {
                (
                    State::Fn(Box::new(|tokenizer, code| {
                        cdata_open_inside(tokenizer, clone, code)
                    })),
                    None,
                )
            }
        }
        _ => (State::Nok, None),
    }
}

/// After `</`, in a closing tag, before a tag name.
///
/// ```markdown
/// </|x>
/// ```
fn tag_close_start(tokenizer: &mut Tokenizer, info: Info, code: Code) -> StateFnResult {
    match code {
        Code::Char(char) if char.is_ascii_alphabetic() => {
            tokenizer.consume(code);
            // To do: life times.
            let mut clone = info;
            clone.buffer.push(char);
            (
                State::Fn(Box::new(|tokenizer, code| tag_name(tokenizer, clone, code))),
                None,
            )
        }
        _ => (State::Nok, None),
    }
}

/// In a tag name.
///
/// ```markdown
/// <a|b>
/// </a|b>
/// ```
fn tag_name(tokenizer: &mut Tokenizer, info: Info, code: Code) -> StateFnResult {
    match code {
        Code::None
        | Code::CarriageReturnLineFeed
        | Code::VirtualSpace
        | Code::Char('\t' | '\n' | '\r' | ' ' | '/' | '>') => {
            let tag_name_buffer = info.buffer.iter().collect::<String>().to_lowercase();
            let name = tag_name_buffer.as_str();
            let slash = if let Code::Char(char) = code {
                char == '/'
            } else {
                false
            };

            if !slash && info.start_tag && HTML_RAW_NAMES.contains(&name) {
                // To do: life times.
                let mut clone = info;
                clone.kind = Kind::Raw;
                clone.buffer.clear();
                continuation(tokenizer, clone, code)
            } else if HTML_BLOCK_NAMES.contains(&name) {
                // To do: life times.
                let mut clone = info;
                clone.kind = Kind::Basic;
                clone.buffer.clear();

                if slash {
                    tokenizer.consume(code);
                    (
                        State::Fn(Box::new(|tokenizer, code| {
                            basic_self_closing(tokenizer, clone, code)
                        })),
                        None,
                    )
                } else {
                    continuation(tokenizer, clone, code)
                }
            } else {
                // To do: life times.
                let mut clone = info;
                clone.kind = Kind::Complete;

                // To do: do not support complete HTML when interrupting.
                if clone.start_tag {
                    complete_attribute_name_before(tokenizer, clone, code)
                } else {
                    complete_closing_tag_after(tokenizer, clone, code)
                }
            }
        }
        Code::Char(char) if char == '-' || char.is_ascii_alphanumeric() => {
            tokenizer.consume(code);
            let mut clone = info;
            clone.buffer.push(char);
            (
                State::Fn(Box::new(|tokenizer, code| tag_name(tokenizer, clone, code))),
                None,
            )
        }
        Code::Char(_) => (State::Nok, None),
    }
}

/// After a closing slash of a basic tag name.
///
/// ```markdown
/// <div/|>
/// ```
fn basic_self_closing(tokenizer: &mut Tokenizer, info: Info, code: Code) -> StateFnResult {
    match code {
        Code::Char('>') => {
            tokenizer.consume(code);
            (
                State::Fn(Box::new(|tokenizer, code| {
                    continuation(tokenizer, info, code)
                })),
                None,
            )
        }
        _ => (State::Nok, None),
    }
}

/// After a closing slash of a complete tag name.
///
/// ```markdown
/// <x/|>
/// </x/|>
/// ```
fn complete_closing_tag_after(tokenizer: &mut Tokenizer, info: Info, code: Code) -> StateFnResult {
    match code {
        Code::VirtualSpace | Code::Char('\t' | ' ') => {
            tokenizer.consume(code);
            (
                State::Fn(Box::new(|tokenizer, code| {
                    complete_closing_tag_after(tokenizer, info, code)
                })),
                None,
            )
        }
        _ => complete_end(tokenizer, info, code),
    }
}

/// At a place where an attribute name would be valid.
///
/// At first, this state is used after a complete tag name, after whitespace,
/// where it expects optional attributes or the end of the tag.
/// It is also reused after attributes, when expecting more optional
/// attributes.
///
/// ```markdown
/// <x |/>
/// <x |:asd>
/// <x |_asd>
/// <x |asd>
/// <x | >
/// <x |>
/// ```
fn complete_attribute_name_before(
    tokenizer: &mut Tokenizer,
    info: Info,
    code: Code,
) -> StateFnResult {
    match code {
        Code::Char('/') => {
            tokenizer.consume(code);
            (
                State::Fn(Box::new(|tokenizer, code| {
                    complete_end(tokenizer, info, code)
                })),
                None,
            )
        }
        Code::Char(char) if char == ':' || char == '_' || char.is_ascii_alphabetic() => {
            tokenizer.consume(code);
            (
                State::Fn(Box::new(|tokenizer, code| {
                    complete_attribute_name(tokenizer, info, code)
                })),
                None,
            )
        }
        Code::VirtualSpace | Code::Char('\t' | ' ') => {
            tokenizer.consume(code);
            (
                State::Fn(Box::new(|tokenizer, code| {
                    complete_attribute_name_before(tokenizer, info, code)
                })),
                None,
            )
        }
        _ => complete_end(tokenizer, info, code),
    }
}

/// In an attribute name.
///
/// ```markdown
/// <x :|>
/// <x _|>
/// <x a|>
/// ```
fn complete_attribute_name(tokenizer: &mut Tokenizer, info: Info, code: Code) -> StateFnResult {
    match code {
        Code::Char(char)
            if char == '-'
                || char == '.'
                || char == ':'
                || char == '_'
                || char.is_ascii_alphanumeric() =>
        {
            tokenizer.consume(code);
            (
                State::Fn(Box::new(|tokenizer, code| {
                    complete_attribute_name(tokenizer, info, code)
                })),
                None,
            )
        }
        _ => complete_attribute_name_after(tokenizer, info, code),
    }
}

/// After an attribute name, before an attribute initializer, the end of the
/// tag, or whitespace.
///
/// ```markdown
/// <x a|>
/// <x a|=b>
/// <x a|="c">
/// ```
fn complete_attribute_name_after(
    tokenizer: &mut Tokenizer,
    info: Info,
    code: Code,
) -> StateFnResult {
    match code {
        Code::Char('=') => {
            tokenizer.consume(code);
            (
                State::Fn(Box::new(|tokenizer, code| {
                    complete_attribute_value_before(tokenizer, info, code)
                })),
                None,
            )
        }
        Code::VirtualSpace | Code::Char('\t' | ' ') => {
            tokenizer.consume(code);
            (
                State::Fn(Box::new(|tokenizer, code| {
                    complete_attribute_name_after(tokenizer, info, code)
                })),
                None,
            )
        }
        _ => complete_attribute_name_before(tokenizer, info, code),
    }
}

/// Before an unquoted, double quoted, or single quoted attribute value,
/// allowing whitespace.
///
/// ```markdown
/// <x a=|b>
/// <x a=|"c">
/// ```
fn complete_attribute_value_before(
    tokenizer: &mut Tokenizer,
    info: Info,
    code: Code,
) -> StateFnResult {
    match code {
        Code::None | Code::Char('<' | '=' | '>' | '`') => (State::Nok, None),
        Code::Char(char) if char == '"' || char == '\'' => {
            tokenizer.consume(code);
            // To do: life times.
            let mut clone = info;
            clone.quote = if char == '"' {
                QuoteKind::Double
            } else {
                QuoteKind::Single
            };

            (
                State::Fn(Box::new(|tokenizer, code| {
                    complete_attribute_value_quoted(tokenizer, clone, code)
                })),
                None,
            )
        }
        Code::VirtualSpace | Code::Char('\t' | ' ') => {
            tokenizer.consume(code);
            (
                State::Fn(Box::new(|tokenizer, code| {
                    complete_attribute_value_before(tokenizer, info, code)
                })),
                None,
            )
        }
        _ => complete_attribute_value_unquoted(tokenizer, info, code),
    }
}

/// In a double or single quoted attribute value.
///
/// ```markdown
/// <x a="|">
/// <x a='|'>
/// ```
fn complete_attribute_value_quoted(
    tokenizer: &mut Tokenizer,
    info: Info,
    code: Code,
) -> StateFnResult {
    let marker = if info.quote == QuoteKind::Double {
        '"'
    } else {
        '\''
    };

    match code {
        Code::None | Code::CarriageReturnLineFeed | Code::Char('\r' | '\n') => (State::Nok, None),
        Code::Char(char) if char == marker => {
            tokenizer.consume(code);
            (
                State::Fn(Box::new(|tokenizer, code| {
                    complete_attribute_value_quoted_after(tokenizer, info, code)
                })),
                None,
            )
        }
        _ => {
            tokenizer.consume(code);
            (
                State::Fn(Box::new(|tokenizer, code| {
                    complete_attribute_value_quoted(tokenizer, info, code)
                })),
                None,
            )
        }
    }
}

/// In an unquoted attribute value.
///
/// ```markdown
/// <x a=b|c>
/// ```
fn complete_attribute_value_unquoted(
    tokenizer: &mut Tokenizer,
    info: Info,
    code: Code,
) -> StateFnResult {
    match code {
        Code::None
        | Code::CarriageReturnLineFeed
        | Code::VirtualSpace
        | Code::Char('\t' | '\n' | '\r' | ' ' | '"' | '\'' | '/' | '<' | '=' | '>' | '`') => {
            complete_attribute_name_after(tokenizer, info, code)
        }
        Code::Char(_) => {
            tokenizer.consume(code);
            (
                State::Fn(Box::new(|tokenizer, code| {
                    complete_attribute_value_unquoted(tokenizer, info, code)
                })),
                None,
            )
        }
    }
}

/// After a double or single quoted attribute value, before whitespace or the
/// end of the tag.
///
/// ```markdown
/// <x a="b"|>
/// ```
fn complete_attribute_value_quoted_after(
    tokenizer: &mut Tokenizer,
    info: Info,
    code: Code,
) -> StateFnResult {
    match code {
        Code::VirtualSpace | Code::Char('\t' | ' ' | '/' | '>') => {
            complete_attribute_name_before(tokenizer, info, code)
        }
        _ => (State::Nok, None),
    }
}

/// In certain circumstances of a complete tag where only an `>` is allowed.
///
/// ```markdown
/// <x a="b"|>
/// ```
fn complete_end(tokenizer: &mut Tokenizer, info: Info, code: Code) -> StateFnResult {
    match code {
        Code::Char('>') => {
            tokenizer.consume(code);
            (
                State::Fn(Box::new(|tokenizer, code| {
                    complete_after(tokenizer, info, code)
                })),
                None,
            )
        }
        _ => (State::Nok, None),
    }
}

/// After `>` in a complete tag.
///
/// ```markdown
/// <x>|
/// ```
fn complete_after(tokenizer: &mut Tokenizer, info: Info, code: Code) -> StateFnResult {
    match code {
        Code::None | Code::CarriageReturnLineFeed | Code::Char('\n' | '\r') => {
            continuation(tokenizer, info, code)
        }
        Code::VirtualSpace | Code::Char('\t' | ' ') => {
            tokenizer.consume(code);
            (
                State::Fn(Box::new(|tokenizer, code| {
                    complete_after(tokenizer, info, code)
                })),
                None,
            )
        }
        Code::Char(_) => (State::Nok, None),
    }
}

/// Inside continuation of any HTML kind.
///
/// ```markdown
/// <!--x|xx-->
/// ```
fn continuation(tokenizer: &mut Tokenizer, info: Info, code: Code) -> StateFnResult {
    match code {
        Code::Char('-') if info.kind == Kind::Comment => {
            tokenizer.consume(code);
            (
                State::Fn(Box::new(|tokenizer, code| {
                    continuation_comment_inside(tokenizer, info, code)
                })),
                None,
            )
        }
        Code::Char('<') if info.kind == Kind::Raw => {
            tokenizer.consume(code);
            (
                State::Fn(Box::new(|tokenizer, code| {
                    continuation_raw_tag_open(tokenizer, info, code)
                })),
                None,
            )
        }
        Code::Char('>') if info.kind == Kind::Declaration => {
            tokenizer.consume(code);
            (
                State::Fn(Box::new(|tokenizer, code| {
                    continuation_close(tokenizer, info, code)
                })),
                None,
            )
        }
        Code::Char('?') if info.kind == Kind::Instruction => {
            tokenizer.consume(code);
            (
                State::Fn(Box::new(|tokenizer, code| {
                    continuation_declaration_inside(tokenizer, info, code)
                })),
                None,
            )
        }
        Code::Char(']') if info.kind == Kind::Cdata => {
            tokenizer.consume(code);
            (
                State::Fn(Box::new(|tokenizer, code| {
                    continuation_character_data_inside(tokenizer, info, code)
                })),
                None,
            )
        }
        Code::CarriageReturnLineFeed | Code::Char('\n' | '\r')
            if info.kind == Kind::Basic || info.kind == Kind::Complete =>
        {
            let clone = info;

            tokenizer.check(blank_line_before, |ok| {
                if ok {
                    Box::new(|tokenizer, code| continuation_close(tokenizer, clone, code))
                } else {
                    Box::new(|tokenizer, code| continuation_at_line_ending(tokenizer, clone, code))
                }
            })(tokenizer, code)
        }
        Code::None | Code::CarriageReturnLineFeed | Code::Char('\n' | '\r') => {
            continuation_at_line_ending(tokenizer, info, code)
        }
        _ => {
            tokenizer.consume(code);
            (
                State::Fn(Box::new(|tokenizer, code| {
                    continuation(tokenizer, info, code)
                })),
                None,
            )
        }
    }
}

/// In continuation, before an eol or eof.
///
/// ```markdown
/// <x>|
/// ```
fn continuation_at_line_ending(tokenizer: &mut Tokenizer, info: Info, code: Code) -> StateFnResult {
    tokenizer.exit(TokenType::HtmlFlowData);
    html_continue_start(tokenizer, info, code)
}

/// In continuation, after an eol.
///
/// ```markdown
/// <x>|
/// asd
/// ```
fn html_continue_start(tokenizer: &mut Tokenizer, info: Info, code: Code) -> StateFnResult {
    match code {
        Code::None => {
            tokenizer.exit(TokenType::HtmlFlow);
            (State::Ok, Some(vec![code]))
        }
        // To do: do not allow lazy lines.
        Code::CarriageReturnLineFeed | Code::Char('\n' | '\r') => {
            tokenizer.enter(TokenType::LineEnding);
            tokenizer.consume(code);
            tokenizer.exit(TokenType::LineEnding);
            (
                State::Fn(Box::new(|tokenizer, code| {
                    html_continue_start(tokenizer, info, code)
                })),
                None,
            )
        }
        _ => {
            tokenizer.enter(TokenType::HtmlFlowData);
            continuation(tokenizer, info, code)
        }
    }
}

/// In comment continuation, after one `-`, expecting another.
///
/// ```markdown
/// <!--xxx-|->
/// ```
fn continuation_comment_inside(tokenizer: &mut Tokenizer, info: Info, code: Code) -> StateFnResult {
    match code {
        Code::Char('-') if info.kind == Kind::Comment => {
            tokenizer.consume(code);
            (
                State::Fn(Box::new(|tokenizer, code| {
                    continuation_declaration_inside(tokenizer, info, code)
                })),
                None,
            )
        }
        _ => continuation(tokenizer, info, code),
    }
}

/// In raw continuation, after `<`, expecting a `/`.
///
/// ```markdown
/// <script>console.log(1)<|/script>
/// ```
fn continuation_raw_tag_open(tokenizer: &mut Tokenizer, info: Info, code: Code) -> StateFnResult {
    match code {
        Code::Char('/') => {
            tokenizer.consume(code);
            (
                State::Fn(Box::new(|tokenizer, code| {
                    continuation_raw_end_tag(tokenizer, info, code)
                })),
                None,
            )
        }
        _ => continuation(tokenizer, info, code),
    }
}

/// In raw continuation, after `</`, expecting or inside a raw tag name.
///
/// ```markdown
/// <script>console.log(1)</|script>
/// <script>console.log(1)</s|cript>
/// <script>console.log(1)</script|>
/// ```
fn continuation_raw_end_tag(tokenizer: &mut Tokenizer, info: Info, code: Code) -> StateFnResult {
    match code {
        Code::Char('>') => {
            let tag_name_buffer = info.buffer.iter().collect::<String>().to_lowercase();
            // To do: life times.
            let mut clone = info;
            clone.buffer.clear();

            if HTML_RAW_NAMES.contains(&tag_name_buffer.as_str()) {
                tokenizer.consume(code);
                (
                    State::Fn(Box::new(|tokenizer, code| {
                        continuation_close(tokenizer, clone, code)
                    })),
                    None,
                )
            } else {
                continuation(tokenizer, clone, code)
            }
        }
        Code::Char(char) if char.is_ascii_alphabetic() && info.buffer.len() < HTML_RAW_SIZE_MAX => {
            tokenizer.consume(code);
            // To do: life times.
            let mut clone = info;
            clone.buffer.push(char);
            (
                State::Fn(Box::new(|tokenizer, code| {
                    continuation_raw_end_tag(tokenizer, clone, code)
                })),
                None,
            )
        }
        _ => continuation(tokenizer, info, code),
    }
}

/// In cdata continuation, after `]`, expecting `]>`.
///
/// ```markdown
/// <![CDATA[>&<]|]>
/// ```
fn continuation_character_data_inside(
    tokenizer: &mut Tokenizer,
    info: Info,
    code: Code,
) -> StateFnResult {
    match code {
        Code::Char(']') => {
            tokenizer.consume(code);
            (
                State::Fn(Box::new(|tokenizer, code| {
                    continuation_declaration_inside(tokenizer, info, code)
                })),
                None,
            )
        }
        _ => continuation(tokenizer, info, code),
    }
}

/// In declaration or instruction continuation, waiting for `>` to close it.
///
/// ```markdown
/// <!--|>
/// <?ab?|>
/// <?|>
/// <!q|>
/// <!--ab--|>
/// <!--ab--|->
/// <!--ab---|>
/// <![CDATA[>&<]]|>
/// ```
fn continuation_declaration_inside(
    tokenizer: &mut Tokenizer,
    info: Info,
    code: Code,
) -> StateFnResult {
    match code {
        Code::Char('>') => {
            tokenizer.consume(code);
            (
                State::Fn(Box::new(|tokenizer, code| {
                    continuation_close(tokenizer, info, code)
                })),
                None,
            )
        }
        Code::Char('-') if info.kind == Kind::Comment => {
            tokenizer.consume(code);
            (
                State::Fn(Box::new(|tokenizer, code| {
                    continuation_declaration_inside(tokenizer, info, code)
                })),
                None,
            )
        }
        _ => continuation(tokenizer, info, code),
    }
}

/// In closed continuation: everything we get until the eol/eof is part of it.
///
/// ```markdown
/// <!doctype>|
/// ```
fn continuation_close(tokenizer: &mut Tokenizer, info: Info, code: Code) -> StateFnResult {
    match code {
        Code::None | Code::CarriageReturnLineFeed | Code::Char('\n' | '\r') => {
            tokenizer.exit(TokenType::HtmlFlowData);
            tokenizer.exit(TokenType::HtmlFlow);
            (State::Ok, Some(vec![code]))
        }
        _ => {
            tokenizer.consume(code);
            (
                State::Fn(Box::new(|tokenizer, code| {
                    continuation_close(tokenizer, info, code)
                })),
                None,
            )
        }
    }
}

/// Before a line ending, expecting a blank line.
///
/// ```markdown
/// <div>|
///
/// ```
fn blank_line_before(tokenizer: &mut Tokenizer, code: Code) -> StateFnResult {
    tokenizer.enter(TokenType::LineEnding);
    tokenizer.consume(code);
    tokenizer.exit(TokenType::LineEnding);
    (State::Fn(Box::new(blank_line)), None)
}
