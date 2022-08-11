//! HTML (flow) is a construct that occurs in the [flow][] cont&ent type.
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
//! ## Tokens
//!
//! *   [`HtmlFlow`][Token::HtmlFlow]
//! *   [`HtmlFlowData`][Token::HtmlFlowData]
//! *   [`LineEnding`][Token::LineEnding]
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

use crate::constant::{
    HTML_BLOCK_NAMES, HTML_CDATA_PREFIX, HTML_RAW_NAMES, HTML_RAW_SIZE_MAX, TAB_SIZE,
};
use crate::construct::partial_space_or_tab::{
    space_or_tab_with_options, Options as SpaceOrTabOptions,
};
use crate::state::{Name, State};
use crate::token::Token;
use crate::tokenizer::Tokenizer;
use crate::util::slice::Slice;

/// Symbol for `<script>` (condition 1).
const RAW: u8 = 1;
/// Symbol for `<!---->` (condition 2).
const COMMENT: u8 = 2;
/// Symbol for `<?php?>` (condition 3).
const INSTRUCTION: u8 = 3;
/// Symbol for `<!doctype>` (condition 4).
const DECLARATION: u8 = 4;
/// Symbol for `<![CDATA[]]>` (condition 5).
const CDATA: u8 = 5;
/// Symbol for `<div` (condition 6).
const BASIC: u8 = 6;
/// Symbol for `<x>` (condition 7).
const COMPLETE: u8 = 7;

/// Start of HTML (flow), before optional whitespace.
///
/// ```markdown
/// > | <x />
///     ^
/// ```
pub fn start(tokenizer: &mut Tokenizer) -> State {
    if tokenizer.parse_state.constructs.html_flow {
        tokenizer.enter(Token::HtmlFlow);
        let name = space_or_tab_with_options(
            tokenizer,
            SpaceOrTabOptions {
                kind: Token::HtmlFlowData,
                min: 0,
                max: if tokenizer.parse_state.constructs.code_indented {
                    TAB_SIZE - 1
                } else {
                    usize::MAX
                },
                connect: false,
                content_type: None,
            },
        );

        tokenizer.attempt(name, State::Next(Name::HtmlFlowBefore), State::Nok)
    } else {
        State::Nok
    }
}

/// After optional whitespace, before `<`.
///
/// ```markdown
/// > | <x />
///     ^
/// ```
pub fn before(tokenizer: &mut Tokenizer) -> State {
    if Some(b'<') == tokenizer.current {
        tokenizer.enter(Token::HtmlFlowData);
        tokenizer.consume();
        State::Next(Name::HtmlFlowOpen)
    } else {
        State::Nok
    }
}

/// After `<`, before a tag name or other stuff.
///
/// ```markdown
/// > | <x />
///      ^
/// > | <!doctype>
///      ^
/// > | <!--xxx-->
///      ^
/// ```
pub fn open(tokenizer: &mut Tokenizer) -> State {
    match tokenizer.current {
        Some(b'!') => {
            tokenizer.consume();
            State::Next(Name::HtmlFlowDeclarationOpen)
        }
        Some(b'/') => {
            tokenizer.consume();
            tokenizer.tokenize_state.seen = true;
            tokenizer.tokenize_state.start = tokenizer.point.index;
            State::Next(Name::HtmlFlowTagCloseStart)
        }
        Some(b'?') => {
            tokenizer.tokenize_state.marker = INSTRUCTION;
            tokenizer.consume();
            // Do not form containers.
            tokenizer.concrete = true;
            // While we’re in an instruction instead of a declaration, we’re on a `?`
            // right now, so we do need to search for `>`, similar to declarations.
            State::Next(Name::HtmlFlowContinuationDeclarationInside)
        }
        // ASCII alphabetical.
        Some(b'A'..=b'Z' | b'a'..=b'z') => {
            tokenizer.tokenize_state.start = tokenizer.point.index;
            State::Retry(Name::HtmlFlowTagName)
        }
        _ => State::Nok,
    }
}

/// After `<!`, so inside a declaration, comment, or CDATA.
///
/// ```markdown
/// > | <!doctype>
///       ^
/// > | <!--xxx-->
///       ^
/// > | <![CDATA[>&<]]>
///       ^
/// ```
pub fn declaration_open(tokenizer: &mut Tokenizer) -> State {
    match tokenizer.current {
        Some(b'-') => {
            tokenizer.consume();
            tokenizer.tokenize_state.marker = COMMENT;
            State::Next(Name::HtmlFlowCommentOpenInside)
        }
        Some(b'A'..=b'Z' | b'a'..=b'z') => {
            tokenizer.consume();
            tokenizer.tokenize_state.marker = DECLARATION;
            // Do not form containers.
            tokenizer.concrete = true;
            State::Next(Name::HtmlFlowContinuationDeclarationInside)
        }
        Some(b'[') => {
            tokenizer.consume();
            tokenizer.tokenize_state.marker = CDATA;
            State::Next(Name::HtmlFlowCdataOpenInside)
        }
        _ => State::Nok,
    }
}

/// After `<!-`, inside a comment, before another `-`.
///
/// ```markdown
/// > | <!--xxx-->
///        ^
/// ```
pub fn comment_open_inside(tokenizer: &mut Tokenizer) -> State {
    if let Some(b'-') = tokenizer.current {
        tokenizer.consume();
        // Do not form containers.
        tokenizer.concrete = true;
        State::Next(Name::HtmlFlowContinuationDeclarationInside)
    } else {
        tokenizer.tokenize_state.marker = 0;
        State::Nok
    }
}

/// After `<![`, inside CDATA, expecting `CDATA[`.
///
/// ```markdown
/// > | <![CDATA[>&<]]>
///        ^^^^^^
/// ```
pub fn cdata_open_inside(tokenizer: &mut Tokenizer) -> State {
    if tokenizer.current == Some(HTML_CDATA_PREFIX[tokenizer.tokenize_state.size]) {
        tokenizer.tokenize_state.size += 1;
        tokenizer.consume();

        if tokenizer.tokenize_state.size == HTML_CDATA_PREFIX.len() {
            tokenizer.tokenize_state.size = 0;
            // Do not form containers.
            tokenizer.concrete = true;
            State::Next(Name::HtmlFlowContinuation)
        } else {
            State::Next(Name::HtmlFlowCdataOpenInside)
        }
    } else {
        tokenizer.tokenize_state.marker = 0;
        tokenizer.tokenize_state.size = 0;
        State::Nok
    }
}

/// After `</`, in a closing tag, before a tag name.
///
/// ```markdown
/// > | </x>
///       ^
/// ```
pub fn tag_close_start(tokenizer: &mut Tokenizer) -> State {
    if let Some(b'A'..=b'Z' | b'a'..=b'z') = tokenizer.current {
        tokenizer.consume();
        State::Next(Name::HtmlFlowTagName)
    } else {
        tokenizer.tokenize_state.seen = false;
        tokenizer.tokenize_state.start = 0;
        State::Nok
    }
}

/// In a tag name.
///
/// ```markdown
/// > | <ab>
///      ^^
/// > | </ab>
///       ^^
/// ```
pub fn tag_name(tokenizer: &mut Tokenizer) -> State {
    match tokenizer.current {
        None | Some(b'\t' | b'\n' | b' ' | b'/' | b'>') => {
            let closing_tag = tokenizer.tokenize_state.seen;
            let slash = matches!(tokenizer.current, Some(b'/'));
            // Guaranteed to be valid ASCII bytes.
            let slice = Slice::from_indices(
                tokenizer.parse_state.bytes,
                tokenizer.tokenize_state.start,
                tokenizer.point.index,
            );
            let name = slice
                .as_str()
                // The line ending case might result in a `\r` that is already accounted for.
                .trim()
                .to_ascii_lowercase();
            tokenizer.tokenize_state.seen = false;
            tokenizer.tokenize_state.start = 0;

            if !slash && !closing_tag && HTML_RAW_NAMES.contains(&name.as_str()) {
                tokenizer.tokenize_state.marker = RAW;
                // Do not form containers.
                tokenizer.concrete = true;
                State::Retry(Name::HtmlFlowContinuation)
            } else if HTML_BLOCK_NAMES.contains(&name.as_str()) {
                tokenizer.tokenize_state.marker = BASIC;

                if slash {
                    tokenizer.consume();
                    State::Next(Name::HtmlFlowBasicSelfClosing)
                } else {
                    // Do not form containers.
                    tokenizer.concrete = true;
                    State::Retry(Name::HtmlFlowContinuation)
                }
            } else {
                tokenizer.tokenize_state.marker = COMPLETE;

                // Do not support complete HTML when interrupting.
                if tokenizer.interrupt && !tokenizer.lazy {
                    tokenizer.tokenize_state.marker = 0;
                    State::Nok
                } else if closing_tag {
                    State::Retry(Name::HtmlFlowCompleteClosingTagAfter)
                } else {
                    State::Retry(Name::HtmlFlowCompleteAttributeNameBefore)
                }
            }
        }
        // ASCII alphanumerical and `-`.
        Some(b'-' | b'0'..=b'9' | b'A'..=b'Z' | b'a'..=b'z') => {
            tokenizer.consume();
            State::Next(Name::HtmlFlowTagName)
        }
        Some(_) => {
            tokenizer.tokenize_state.seen = false;
            State::Nok
        }
    }
}

/// After a closing slash of a basic tag name.
///
/// ```markdown
/// > | <div/>
///          ^
/// ```
pub fn basic_self_closing(tokenizer: &mut Tokenizer) -> State {
    if let Some(b'>') = tokenizer.current {
        tokenizer.consume();
        // Do not form containers.
        tokenizer.concrete = true;
        State::Next(Name::HtmlFlowContinuation)
    } else {
        tokenizer.tokenize_state.marker = 0;
        State::Nok
    }
}

/// After a closing slash of a complete tag name.
///
/// ```markdown
/// > | <x/>
///        ^
/// ```
pub fn complete_closing_tag_after(tokenizer: &mut Tokenizer) -> State {
    match tokenizer.current {
        Some(b'\t' | b' ') => {
            tokenizer.consume();
            State::Next(Name::HtmlFlowCompleteClosingTagAfter)
        }
        _ => State::Retry(Name::HtmlFlowCompleteEnd),
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
/// > | <a />
///        ^
/// > | <a :b>
///        ^
/// > | <a _b>
///        ^
/// > | <a b>
///        ^
/// > | <a >
///        ^
/// ```
pub fn complete_attribute_name_before(tokenizer: &mut Tokenizer) -> State {
    match tokenizer.current {
        Some(b'\t' | b' ') => {
            tokenizer.consume();
            State::Next(Name::HtmlFlowCompleteAttributeNameBefore)
        }
        Some(b'/') => {
            tokenizer.consume();
            State::Next(Name::HtmlFlowCompleteEnd)
        }
        // ASCII alphanumerical and `:` and `_`.
        Some(b'0'..=b'9' | b':' | b'A'..=b'Z' | b'_' | b'a'..=b'z') => {
            tokenizer.consume();
            State::Next(Name::HtmlFlowCompleteAttributeName)
        }
        _ => State::Retry(Name::HtmlFlowCompleteEnd),
    }
}

/// In an attribute name.
///
/// ```markdown
/// > | <a :b>
///         ^
/// > | <a _b>
///         ^
/// > | <a b>
///         ^
/// ```
pub fn complete_attribute_name(tokenizer: &mut Tokenizer) -> State {
    match tokenizer.current {
        // ASCII alphanumerical and `-`, `.`, `:`, and `_`.
        Some(b'-' | b'.' | b'0'..=b'9' | b':' | b'A'..=b'Z' | b'_' | b'a'..=b'z') => {
            tokenizer.consume();
            State::Next(Name::HtmlFlowCompleteAttributeName)
        }
        _ => State::Retry(Name::HtmlFlowCompleteAttributeNameAfter),
    }
}

/// After an attribute name, before an attribute initializer, the end of the
/// tag, or whitespace.
///
/// ```markdown
/// > | <a b>
///         ^
/// > | <a b=c>
///         ^
/// ```
pub fn complete_attribute_name_after(tokenizer: &mut Tokenizer) -> State {
    match tokenizer.current {
        Some(b'\t' | b' ') => {
            tokenizer.consume();
            State::Next(Name::HtmlFlowCompleteAttributeNameAfter)
        }
        Some(b'=') => {
            tokenizer.consume();
            State::Next(Name::HtmlFlowCompleteAttributeValueBefore)
        }
        _ => State::Retry(Name::HtmlFlowCompleteAttributeNameBefore),
    }
}

/// Before an unquoted, double quoted, or single quoted attribute value,
/// allowing whitespace.
///
/// ```markdown
/// > | <a b=c>
///          ^
/// > | <a b="c">
///          ^
/// ```
pub fn complete_attribute_value_before(tokenizer: &mut Tokenizer) -> State {
    match tokenizer.current {
        None | Some(b'<' | b'=' | b'>' | b'`') => {
            tokenizer.tokenize_state.marker = 0;
            State::Nok
        }
        Some(b'\t' | b' ') => {
            tokenizer.consume();
            State::Next(Name::HtmlFlowCompleteAttributeValueBefore)
        }
        Some(b'"' | b'\'') => {
            tokenizer.tokenize_state.marker_b = tokenizer.current.unwrap();
            tokenizer.consume();
            State::Next(Name::HtmlFlowCompleteAttributeValueQuoted)
        }
        _ => State::Retry(Name::HtmlFlowCompleteAttributeValueUnquoted),
    }
}

/// In a double or single quoted attribute value.
///
/// ```markdown
/// > | <a b="c">
///           ^
/// > | <a b='c'>
///           ^
/// ```
pub fn complete_attribute_value_quoted(tokenizer: &mut Tokenizer) -> State {
    match tokenizer.current {
        None | Some(b'\n') => {
            tokenizer.tokenize_state.marker = 0;
            tokenizer.tokenize_state.marker_b = 0;
            State::Nok
        }
        Some(b'"' | b'\'') if tokenizer.current.unwrap() == tokenizer.tokenize_state.marker_b => {
            tokenizer.tokenize_state.marker_b = 0;
            tokenizer.consume();
            State::Next(Name::HtmlFlowCompleteAttributeValueQuotedAfter)
        }
        _ => {
            tokenizer.consume();
            State::Next(Name::HtmlFlowCompleteAttributeValueQuoted)
        }
    }
}

/// In an unquoted attribute value.
///
/// ```markdown
/// > | <a b=c>
///          ^
/// ```
pub fn complete_attribute_value_unquoted(tokenizer: &mut Tokenizer) -> State {
    match tokenizer.current {
        None | Some(b'\t' | b'\n' | b' ' | b'"' | b'\'' | b'/' | b'<' | b'=' | b'>' | b'`') => {
            State::Retry(Name::HtmlFlowCompleteAttributeNameAfter)
        }
        Some(_) => {
            tokenizer.consume();
            State::Next(Name::HtmlFlowCompleteAttributeValueUnquoted)
        }
    }
}

/// After a double or single quoted attribute value, before whitespace or the
/// end of the tag.
///
/// ```markdown
/// > | <a b="c">
///            ^
/// ```
pub fn complete_attribute_value_quoted_after(tokenizer: &mut Tokenizer) -> State {
    if let Some(b'\t' | b' ' | b'/' | b'>') = tokenizer.current {
        State::Retry(Name::HtmlFlowCompleteAttributeNameBefore)
    } else {
        tokenizer.tokenize_state.marker = 0;
        State::Nok
    }
}

/// In certain circumstances of a complete tag where only an `>` is allowed.
///
/// ```markdown
/// > | <a b="c">
///             ^
/// ```
pub fn complete_end(tokenizer: &mut Tokenizer) -> State {
    if let Some(b'>') = tokenizer.current {
        tokenizer.consume();
        State::Next(Name::HtmlFlowCompleteAfter)
    } else {
        tokenizer.tokenize_state.marker = 0;
        State::Nok
    }
}

/// After `>` in a complete tag.
///
/// ```markdown
/// > | <x>
///        ^
/// ```
pub fn complete_after(tokenizer: &mut Tokenizer) -> State {
    match tokenizer.current {
        None | Some(b'\n') => {
            // Do not form containers.
            tokenizer.concrete = true;
            State::Retry(Name::HtmlFlowContinuation)
        }
        Some(b'\t' | b' ') => {
            tokenizer.consume();
            State::Next(Name::HtmlFlowCompleteAfter)
        }
        Some(_) => {
            tokenizer.tokenize_state.marker = 0;
            State::Nok
        }
    }
}

/// Inside continuation of any HTML kind.
///
/// ```markdown
/// > | <!--xxx-->
///          ^
/// ```
pub fn continuation(tokenizer: &mut Tokenizer) -> State {
    match tokenizer.current {
        Some(b'\n')
            if tokenizer.tokenize_state.marker == BASIC
                || tokenizer.tokenize_state.marker == COMPLETE =>
        {
            tokenizer.exit(Token::HtmlFlowData);
            tokenizer.check(
                Name::HtmlFlowBlankLineBefore,
                State::Next(Name::HtmlFlowContinuationAfter),
                State::Next(Name::HtmlFlowContinuationStart),
            )
        }
        // Note: important that this is after the basic/complete case.
        None | Some(b'\n') => {
            tokenizer.exit(Token::HtmlFlowData);
            State::Retry(Name::HtmlFlowContinuationStart)
        }
        Some(b'-') if tokenizer.tokenize_state.marker == COMMENT => {
            tokenizer.consume();
            State::Next(Name::HtmlFlowContinuationCommentInside)
        }
        Some(b'<') if tokenizer.tokenize_state.marker == RAW => {
            tokenizer.consume();
            State::Next(Name::HtmlFlowContinuationRawTagOpen)
        }
        Some(b'>') if tokenizer.tokenize_state.marker == DECLARATION => {
            tokenizer.consume();
            State::Next(Name::HtmlFlowContinuationClose)
        }
        Some(b'?') if tokenizer.tokenize_state.marker == INSTRUCTION => {
            tokenizer.consume();
            State::Next(Name::HtmlFlowContinuationDeclarationInside)
        }
        Some(b']') if tokenizer.tokenize_state.marker == CDATA => {
            tokenizer.consume();
            State::Next(Name::HtmlFlowContinuationCdataInside)
        }
        _ => {
            tokenizer.consume();
            State::Next(Name::HtmlFlowContinuation)
        }
    }
}

/// In continuation, at an eol.
///
/// ```markdown
/// > | <x>
///        ^
///   | asd
/// ```
pub fn continuation_start(tokenizer: &mut Tokenizer) -> State {
    tokenizer.check(
        Name::NonLazyContinuationStart,
        State::Next(Name::HtmlFlowContinuationStartNonLazy),
        State::Next(Name::HtmlFlowContinuationAfter),
    )
}

/// In continuation, at an eol, before non-lazy content.
///
/// ```markdown
/// > | <x>
///        ^
///   | asd
/// ```
pub fn continuation_start_non_lazy(tokenizer: &mut Tokenizer) -> State {
    match tokenizer.current {
        Some(b'\n') => {
            tokenizer.enter(Token::LineEnding);
            tokenizer.consume();
            tokenizer.exit(Token::LineEnding);
            State::Next(Name::HtmlFlowContinuationBefore)
        }
        _ => unreachable!("expected eol"),
    }
}

/// In continuation, after an eol, before non-lazy content.
///
/// ```markdown
///   | <x>
/// > | asd
///     ^
/// ```
pub fn continuation_before(tokenizer: &mut Tokenizer) -> State {
    match tokenizer.current {
        None | Some(b'\n') => State::Retry(Name::HtmlFlowContinuationStart),
        _ => {
            tokenizer.enter(Token::HtmlFlowData);
            State::Retry(Name::HtmlFlowContinuation)
        }
    }
}

/// In comment continuation, after one `-`, expecting another.
///
/// ```markdown
/// > | <!--xxx-->
///             ^
/// ```
pub fn continuation_comment_inside(tokenizer: &mut Tokenizer) -> State {
    match tokenizer.current {
        Some(b'-') => {
            tokenizer.consume();
            State::Next(Name::HtmlFlowContinuationDeclarationInside)
        }
        _ => State::Retry(Name::HtmlFlowContinuation),
    }
}

/// In raw continuation, after `<`, expecting a `/`.
///
/// ```markdown
/// > | <script>console.log(1)</script>
///                            ^
/// ```
pub fn continuation_raw_tag_open(tokenizer: &mut Tokenizer) -> State {
    match tokenizer.current {
        Some(b'/') => {
            tokenizer.consume();
            tokenizer.tokenize_state.start = tokenizer.point.index;
            State::Next(Name::HtmlFlowContinuationRawEndTag)
        }
        _ => State::Retry(Name::HtmlFlowContinuation),
    }
}

/// In raw continuation, after `</`, expecting or inside a raw tag name.
///
/// ```markdown
/// > | <script>console.log(1)</script>
///                             ^^^^^^
/// ```
pub fn continuation_raw_end_tag(tokenizer: &mut Tokenizer) -> State {
    match tokenizer.current {
        Some(b'>') => {
            // Guaranteed to be valid ASCII bytes.
            let slice = Slice::from_indices(
                tokenizer.parse_state.bytes,
                tokenizer.tokenize_state.start,
                tokenizer.point.index,
            );
            let name = slice.as_str().to_ascii_lowercase();

            tokenizer.tokenize_state.start = 0;

            if HTML_RAW_NAMES.contains(&name.as_str()) {
                tokenizer.consume();
                State::Next(Name::HtmlFlowContinuationClose)
            } else {
                State::Retry(Name::HtmlFlowContinuation)
            }
        }
        Some(b'A'..=b'Z' | b'a'..=b'z')
            if tokenizer.point.index - tokenizer.tokenize_state.start < HTML_RAW_SIZE_MAX =>
        {
            tokenizer.consume();
            State::Next(Name::HtmlFlowContinuationRawEndTag)
        }
        _ => {
            tokenizer.tokenize_state.start = 0;
            State::Retry(Name::HtmlFlowContinuation)
        }
    }
}

/// In cdata continuation, after `]`, expecting `]>`.
///
/// ```markdown
/// > | <![CDATA[>&<]]>
///                  ^
/// ```
pub fn continuation_cdata_inside(tokenizer: &mut Tokenizer) -> State {
    match tokenizer.current {
        Some(b']') => {
            tokenizer.consume();
            State::Next(Name::HtmlFlowContinuationDeclarationInside)
        }
        _ => State::Retry(Name::HtmlFlowContinuation),
    }
}

/// In declaration or instruction continuation, waiting for `>` to close it.
///
/// ```markdown
/// > | <!-->
///         ^
/// > | <?>
///       ^
/// > | <!q>
///        ^
/// > | <!--ab-->
///             ^
/// > | <![CDATA[>&<]]>
///                   ^
/// ```
pub fn continuation_declaration_inside(tokenizer: &mut Tokenizer) -> State {
    match tokenizer.current {
        Some(b'>') => {
            tokenizer.consume();
            State::Next(Name::HtmlFlowContinuationClose)
        }
        Some(b'-') if tokenizer.tokenize_state.marker == COMMENT => {
            tokenizer.consume();
            State::Next(Name::HtmlFlowContinuationDeclarationInside)
        }
        _ => State::Retry(Name::HtmlFlowContinuation),
    }
}

/// In closed continuation: everything we get until the eol/eof is part of it.
///
/// ```markdown
/// > | <!doctype>
///               ^
/// ```
pub fn continuation_close(tokenizer: &mut Tokenizer) -> State {
    match tokenizer.current {
        None | Some(b'\n') => {
            tokenizer.exit(Token::HtmlFlowData);
            State::Retry(Name::HtmlFlowContinuationAfter)
        }
        _ => {
            tokenizer.consume();
            State::Next(Name::HtmlFlowContinuationClose)
        }
    }
}

/// Done.
///
/// ```markdown
/// > | <!doctype>
///               ^
/// ```
pub fn continuation_after(tokenizer: &mut Tokenizer) -> State {
    tokenizer.exit(Token::HtmlFlow);
    tokenizer.tokenize_state.marker = 0;
    // Feel free to interrupt.
    tokenizer.interrupt = false;
    // No longer concrete.
    tokenizer.concrete = false;
    State::Ok
}

/// Before a line ending, expecting a blank line.
///
/// ```markdown
/// > | <div>
///          ^
///   |
/// ```
pub fn blank_line_before(tokenizer: &mut Tokenizer) -> State {
    tokenizer.enter(Token::LineEnding);
    tokenizer.consume();
    tokenizer.exit(Token::LineEnding);
    State::Next(Name::BlankLineStart)
}
