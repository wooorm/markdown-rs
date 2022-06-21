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
use crate::construct::{blank_line::start as blank_line, partial_space_or_tab::space_or_tab_opt};
use crate::tokenizer::{Code, State, StateFnResult, TokenType, Tokenizer};

/// Kind of HTML (flow).
#[derive(Debug, PartialEq)]
enum Kind {
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

/// Type of quote, if we’re in a quoted attribute, in complete (condition 7).
#[derive(Debug, PartialEq)]
enum QuoteKind {
    /// In a double quoted (`"`) attribute value.
    ///
    /// ## Example
    ///
    /// ```markdown
    /// <a b="c" />
    /// ```
    Double,
    /// In a single quoted (`'`) attribute value.
    ///
    /// ## Example
    ///
    /// ```markdown
    /// <a b='c' />
    /// ```
    Single,
}

impl QuoteKind {
    /// Turn the kind into a [char].
    fn as_char(&self) -> char {
        match self {
            QuoteKind::Double => '"',
            QuoteKind::Single => '\'',
        }
    }
    /// Turn a [char] into a kind.
    ///
    /// ## Panics
    ///
    /// Panics if `char` is not `"` or `'`.
    fn from_char(char: char) -> QuoteKind {
        match char {
            '"' => QuoteKind::Double,
            '\'' => QuoteKind::Single,
            _ => unreachable!("invalid char"),
        }
    }
}

/// State needed to parse HTML (flow).
#[derive(Debug)]
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
    quote: Option<QuoteKind>,
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
    tokenizer.go(space_or_tab_opt(), before)(tokenizer, code)
}

/// After optional whitespace, before `<`.
///
/// ```markdown
/// |<x />
/// ```
fn before(tokenizer: &mut Tokenizer, code: Code) -> StateFnResult {
    if Code::Char('<') == code {
        tokenizer.consume(code);
        (State::Fn(Box::new(open)), None)
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
fn open(tokenizer: &mut Tokenizer, code: Code) -> StateFnResult {
    let mut info = Info {
        // Assume basic.
        kind: Kind::Basic,
        start_tag: false,
        buffer: vec![],
        index: 0,
        quote: None,
    };

    match code {
        Code::Char('!') => {
            tokenizer.consume(code);
            (
                State::Fn(Box::new(|t, c| declaration_open(t, c, info))),
                None,
            )
        }
        Code::Char('/') => {
            tokenizer.consume(code);
            (
                State::Fn(Box::new(|t, c| tag_close_start(t, c, info))),
                None,
            )
        }
        Code::Char('?') => {
            info.kind = Kind::Instruction;
            tokenizer.consume(code);
            // While we’re in an instruction instead of a declaration, we’re on a `?`
            // right now, so we do need to search for `>`, similar to declarations.
            (
                State::Fn(Box::new(|t, c| continuation_declaration_inside(t, c, info))),
                None,
            )
        }
        Code::Char(char) if char.is_ascii_alphabetic() => {
            info.start_tag = true;
            tag_name(tokenizer, code, info)
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
fn declaration_open(tokenizer: &mut Tokenizer, code: Code, mut info: Info) -> StateFnResult {
    match code {
        Code::Char('-') => {
            tokenizer.consume(code);
            info.kind = Kind::Comment;
            (
                State::Fn(Box::new(|t, c| comment_open_inside(t, c, info))),
                None,
            )
        }
        Code::Char('[') => {
            tokenizer.consume(code);
            info.kind = Kind::Cdata;
            info.buffer = vec!['C', 'D', 'A', 'T', 'A', '['];
            info.index = 0;
            (
                State::Fn(Box::new(|t, c| cdata_open_inside(t, c, info))),
                None,
            )
        }
        Code::Char(char) if char.is_ascii_alphabetic() => {
            tokenizer.consume(code);
            info.kind = Kind::Declaration;
            (
                State::Fn(Box::new(|t, c| continuation_declaration_inside(t, c, info))),
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
fn comment_open_inside(tokenizer: &mut Tokenizer, code: Code, info: Info) -> StateFnResult {
    match code {
        Code::Char('-') => {
            tokenizer.consume(code);
            (
                State::Fn(Box::new(|t, c| continuation_declaration_inside(t, c, info))),
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
fn cdata_open_inside(tokenizer: &mut Tokenizer, code: Code, mut info: Info) -> StateFnResult {
    match code {
        Code::Char(char) if char == info.buffer[info.index] => {
            info.index += 1;
            tokenizer.consume(code);

            if info.index == info.buffer.len() {
                info.buffer.clear();
                (State::Fn(Box::new(|t, c| continuation(t, c, info))), None)
            } else {
                (
                    State::Fn(Box::new(|t, c| cdata_open_inside(t, c, info))),
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
fn tag_close_start(tokenizer: &mut Tokenizer, code: Code, mut info: Info) -> StateFnResult {
    match code {
        Code::Char(char) if char.is_ascii_alphabetic() => {
            tokenizer.consume(code);
            info.buffer.push(char);
            (State::Fn(Box::new(|t, c| tag_name(t, c, info))), None)
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
fn tag_name(tokenizer: &mut Tokenizer, code: Code, mut info: Info) -> StateFnResult {
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

            info.buffer.clear();

            if !slash && info.start_tag && HTML_RAW_NAMES.contains(&name) {
                info.kind = Kind::Raw;
                continuation(tokenizer, code, info)
            } else if HTML_BLOCK_NAMES.contains(&name) {
                // Basic is assumed, no need to set `kind`.
                if slash {
                    tokenizer.consume(code);
                    (
                        State::Fn(Box::new(|t, c| basic_self_closing(t, c, info))),
                        None,
                    )
                } else {
                    continuation(tokenizer, code, info)
                }
            } else {
                info.kind = Kind::Complete;

                // To do: do not support complete HTML when interrupting.
                if info.start_tag {
                    complete_attribute_name_before(tokenizer, code, info)
                } else {
                    complete_closing_tag_after(tokenizer, code, info)
                }
            }
        }
        Code::Char(char) if char == '-' || char.is_ascii_alphanumeric() => {
            tokenizer.consume(code);
            info.buffer.push(char);
            (State::Fn(Box::new(|t, c| tag_name(t, c, info))), None)
        }
        Code::Char(_) => (State::Nok, None),
    }
}

/// After a closing slash of a basic tag name.
///
/// ```markdown
/// <div/|>
/// ```
fn basic_self_closing(tokenizer: &mut Tokenizer, code: Code, info: Info) -> StateFnResult {
    match code {
        Code::Char('>') => {
            tokenizer.consume(code);
            (State::Fn(Box::new(|t, c| continuation(t, c, info))), None)
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
fn complete_closing_tag_after(tokenizer: &mut Tokenizer, code: Code, info: Info) -> StateFnResult {
    match code {
        Code::VirtualSpace | Code::Char('\t' | ' ') => {
            tokenizer.consume(code);
            (
                State::Fn(Box::new(|t, c| complete_closing_tag_after(t, c, info))),
                None,
            )
        }
        _ => complete_end(tokenizer, code, info),
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
    code: Code,
    info: Info,
) -> StateFnResult {
    match code {
        Code::Char('/') => {
            tokenizer.consume(code);
            (State::Fn(Box::new(|t, c| complete_end(t, c, info))), None)
        }
        Code::Char(char) if char == ':' || char == '_' || char.is_ascii_alphabetic() => {
            tokenizer.consume(code);
            (
                State::Fn(Box::new(|t, c| complete_attribute_name(t, c, info))),
                None,
            )
        }
        Code::VirtualSpace | Code::Char('\t' | ' ') => {
            tokenizer.consume(code);
            (
                State::Fn(Box::new(|t, c| complete_attribute_name_before(t, c, info))),
                None,
            )
        }
        _ => complete_end(tokenizer, code, info),
    }
}

/// In an attribute name.
///
/// ```markdown
/// <x :|>
/// <x _|>
/// <x a|>
/// ```
fn complete_attribute_name(tokenizer: &mut Tokenizer, code: Code, info: Info) -> StateFnResult {
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
                State::Fn(Box::new(|t, c| complete_attribute_name(t, c, info))),
                None,
            )
        }
        _ => complete_attribute_name_after(tokenizer, code, info),
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
    code: Code,
    info: Info,
) -> StateFnResult {
    match code {
        Code::Char('=') => {
            tokenizer.consume(code);
            (
                State::Fn(Box::new(|t, c| complete_attribute_value_before(t, c, info))),
                None,
            )
        }
        Code::VirtualSpace | Code::Char('\t' | ' ') => {
            tokenizer.consume(code);
            (
                State::Fn(Box::new(|t, c| complete_attribute_name_after(t, c, info))),
                None,
            )
        }
        _ => complete_attribute_name_before(tokenizer, code, info),
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
    code: Code,
    mut info: Info,
) -> StateFnResult {
    match code {
        Code::None | Code::Char('<' | '=' | '>' | '`') => (State::Nok, None),
        Code::Char(char) if char == '"' || char == '\'' => {
            tokenizer.consume(code);
            info.quote = Some(QuoteKind::from_char(char));
            (
                State::Fn(Box::new(|t, c| complete_attribute_value_quoted(t, c, info))),
                None,
            )
        }
        Code::VirtualSpace | Code::Char('\t' | ' ') => {
            tokenizer.consume(code);
            (
                State::Fn(Box::new(|t, c| complete_attribute_value_before(t, c, info))),
                None,
            )
        }
        _ => complete_attribute_value_unquoted(tokenizer, code, info),
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
    code: Code,
    info: Info,
) -> StateFnResult {
    match code {
        Code::None | Code::CarriageReturnLineFeed | Code::Char('\r' | '\n') => (State::Nok, None),
        Code::Char(char) if char == info.quote.as_ref().unwrap().as_char() => {
            tokenizer.consume(code);
            (
                State::Fn(Box::new(|t, c| {
                    complete_attribute_value_quoted_after(t, c, info)
                })),
                None,
            )
        }
        _ => {
            tokenizer.consume(code);
            (
                State::Fn(Box::new(|t, c| complete_attribute_value_quoted(t, c, info))),
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
    code: Code,
    info: Info,
) -> StateFnResult {
    match code {
        Code::None
        | Code::CarriageReturnLineFeed
        | Code::VirtualSpace
        | Code::Char('\t' | '\n' | '\r' | ' ' | '"' | '\'' | '/' | '<' | '=' | '>' | '`') => {
            complete_attribute_name_after(tokenizer, code, info)
        }
        Code::Char(_) => {
            tokenizer.consume(code);
            (
                State::Fn(Box::new(|t, c| {
                    complete_attribute_value_unquoted(t, c, info)
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
    code: Code,
    info: Info,
) -> StateFnResult {
    match code {
        Code::VirtualSpace | Code::Char('\t' | ' ' | '/' | '>') => {
            complete_attribute_name_before(tokenizer, code, info)
        }
        _ => (State::Nok, None),
    }
}

/// In certain circumstances of a complete tag where only an `>` is allowed.
///
/// ```markdown
/// <x a="b"|>
/// ```
fn complete_end(tokenizer: &mut Tokenizer, code: Code, info: Info) -> StateFnResult {
    match code {
        Code::Char('>') => {
            tokenizer.consume(code);
            (State::Fn(Box::new(|t, c| complete_after(t, c, info))), None)
        }
        _ => (State::Nok, None),
    }
}

/// After `>` in a complete tag.
///
/// ```markdown
/// <x>|
/// ```
fn complete_after(tokenizer: &mut Tokenizer, code: Code, info: Info) -> StateFnResult {
    match code {
        Code::None | Code::CarriageReturnLineFeed | Code::Char('\n' | '\r') => {
            continuation(tokenizer, code, info)
        }
        Code::VirtualSpace | Code::Char('\t' | ' ') => {
            tokenizer.consume(code);
            (State::Fn(Box::new(|t, c| complete_after(t, c, info))), None)
        }
        Code::Char(_) => (State::Nok, None),
    }
}

/// Inside continuation of any HTML kind.
///
/// ```markdown
/// <!--x|xx-->
/// ```
fn continuation(tokenizer: &mut Tokenizer, code: Code, info: Info) -> StateFnResult {
    match code {
        Code::Char('-') if info.kind == Kind::Comment => {
            tokenizer.consume(code);
            (
                State::Fn(Box::new(|t, c| continuation_comment_inside(t, c, info))),
                None,
            )
        }
        Code::Char('<') if info.kind == Kind::Raw => {
            tokenizer.consume(code);
            (
                State::Fn(Box::new(|t, c| continuation_raw_tag_open(t, c, info))),
                None,
            )
        }
        Code::Char('>') if info.kind == Kind::Declaration => {
            tokenizer.consume(code);
            (
                State::Fn(Box::new(|t, c| continuation_close(t, c, info))),
                None,
            )
        }
        Code::Char('?') if info.kind == Kind::Instruction => {
            tokenizer.consume(code);
            (
                State::Fn(Box::new(|t, c| continuation_declaration_inside(t, c, info))),
                None,
            )
        }
        Code::Char(']') if info.kind == Kind::Cdata => {
            tokenizer.consume(code);
            (
                State::Fn(Box::new(|t, c| {
                    continuation_character_data_inside(t, c, info)
                })),
                None,
            )
        }
        Code::CarriageReturnLineFeed | Code::Char('\n' | '\r')
            if info.kind == Kind::Basic || info.kind == Kind::Complete =>
        {
            tokenizer.check(blank_line_before, |ok| {
                let func = if ok {
                    continuation_close
                } else {
                    continuation_at_line_ending
                };
                Box::new(move |t, c| func(t, c, info))
            })(tokenizer, code)
        }
        Code::None | Code::CarriageReturnLineFeed | Code::Char('\n' | '\r') => {
            continuation_at_line_ending(tokenizer, code, info)
        }
        _ => {
            tokenizer.consume(code);
            (State::Fn(Box::new(|t, c| continuation(t, c, info))), None)
        }
    }
}

/// In continuation, before an eol or eof.
///
/// ```markdown
/// <x>|
/// ```
fn continuation_at_line_ending(tokenizer: &mut Tokenizer, code: Code, info: Info) -> StateFnResult {
    tokenizer.exit(TokenType::HtmlFlowData);
    html_continue_start(tokenizer, code, info)
}

/// In continuation, after an eol.
///
/// ```markdown
/// <x>|
/// asd
/// ```
fn html_continue_start(tokenizer: &mut Tokenizer, code: Code, info: Info) -> StateFnResult {
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
                State::Fn(Box::new(|t, c| html_continue_start(t, c, info))),
                None,
            )
        }
        _ => {
            tokenizer.enter(TokenType::HtmlFlowData);
            continuation(tokenizer, code, info)
        }
    }
}

/// In comment continuation, after one `-`, expecting another.
///
/// ```markdown
/// <!--xxx-|->
/// ```
fn continuation_comment_inside(tokenizer: &mut Tokenizer, code: Code, info: Info) -> StateFnResult {
    match code {
        Code::Char('-') => {
            tokenizer.consume(code);
            (
                State::Fn(Box::new(|t, c| continuation_declaration_inside(t, c, info))),
                None,
            )
        }
        _ => continuation(tokenizer, code, info),
    }
}

/// In raw continuation, after `<`, expecting a `/`.
///
/// ```markdown
/// <script>console.log(1)<|/script>
/// ```
fn continuation_raw_tag_open(tokenizer: &mut Tokenizer, code: Code, info: Info) -> StateFnResult {
    match code {
        Code::Char('/') => {
            tokenizer.consume(code);
            (
                State::Fn(Box::new(|t, c| continuation_raw_end_tag(t, c, info))),
                None,
            )
        }
        _ => continuation(tokenizer, code, info),
    }
}

/// In raw continuation, after `</`, expecting or inside a raw tag name.
///
/// ```markdown
/// <script>console.log(1)</|script>
/// <script>console.log(1)</s|cript>
/// <script>console.log(1)</script|>
/// ```
fn continuation_raw_end_tag(
    tokenizer: &mut Tokenizer,
    code: Code,
    mut info: Info,
) -> StateFnResult {
    match code {
        Code::Char('>') => {
            let tag_name_buffer = info.buffer.iter().collect::<String>().to_lowercase();
            info.buffer.clear();

            if HTML_RAW_NAMES.contains(&tag_name_buffer.as_str()) {
                tokenizer.consume(code);
                (
                    State::Fn(Box::new(|t, c| continuation_close(t, c, info))),
                    None,
                )
            } else {
                continuation(tokenizer, code, info)
            }
        }
        Code::Char(char) if char.is_ascii_alphabetic() && info.buffer.len() < HTML_RAW_SIZE_MAX => {
            tokenizer.consume(code);
            info.buffer.push(char);
            (
                State::Fn(Box::new(|t, c| continuation_raw_end_tag(t, c, info))),
                None,
            )
        }
        _ => {
            info.buffer.clear();
            continuation(tokenizer, code, info)
        }
    }
}

/// In cdata continuation, after `]`, expecting `]>`.
///
/// ```markdown
/// <![CDATA[>&<]|]>
/// ```
fn continuation_character_data_inside(
    tokenizer: &mut Tokenizer,
    code: Code,
    info: Info,
) -> StateFnResult {
    match code {
        Code::Char(']') => {
            tokenizer.consume(code);
            (
                State::Fn(Box::new(|t, c| continuation_declaration_inside(t, c, info))),
                None,
            )
        }
        _ => continuation(tokenizer, code, info),
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
    code: Code,
    info: Info,
) -> StateFnResult {
    match code {
        Code::Char('>') => {
            tokenizer.consume(code);
            (
                State::Fn(Box::new(|t, c| continuation_close(t, c, info))),
                None,
            )
        }
        Code::Char('-') if info.kind == Kind::Comment => {
            tokenizer.consume(code);
            (
                State::Fn(Box::new(|t, c| continuation_declaration_inside(t, c, info))),
                None,
            )
        }
        _ => continuation(tokenizer, code, info),
    }
}

/// In closed continuation: everything we get until the eol/eof is part of it.
///
/// ```markdown
/// <!doctype>|
/// ```
fn continuation_close(tokenizer: &mut Tokenizer, code: Code, info: Info) -> StateFnResult {
    match code {
        Code::None | Code::CarriageReturnLineFeed | Code::Char('\n' | '\r') => {
            tokenizer.exit(TokenType::HtmlFlowData);
            tokenizer.exit(TokenType::HtmlFlow);
            (State::Ok, Some(vec![code]))
        }
        _ => {
            tokenizer.consume(code);
            (
                State::Fn(Box::new(|t, c| continuation_close(t, c, info))),
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
