//! Definition is a construct that occurs in the [flow] content type.
//!
//! They’re formed with the following BNF:
//!
//! ```bnf
//! definition ::= label ':' whitespace destination [ whitespace title ] [ space_or_tab ]
//!
//! ; Restriction: maximum `999` codes allowed between brackets.
//! ; Restriction: no blank lines.
//! ; Restriction: at least 1 non-space and non-eol code must exist.
//! label ::= '[' *( label_text | label_escape ) ']'
//! label_text ::= code - '[' - '\\' - ']'
//! label_escape ::= '\\' [ '[' | '\\' | ']' ]
//!
//! destination ::= destination_enclosed | destination_raw
//! destination_enclosed ::= '<' *( destination_enclosed_text | destination_enclosed_escape ) '>'
//! destination_enclosed_text ::= code - '<' - '\\' - eol
//! destination_enclosed_escape ::= '\\' [ '<' | '\\' | '>' ]
//! destination_raw ::= 1*( destination_raw_text | destination_raw_escape )
//! ; Restriction: unbalanced `)` characters are not allowed.
//! destination_raw_text ::= code - '\\' - ascii_control - space_or_tab - eol
//! destination_raw_escape ::= '\\' [ '(' | ')' | '\\' ]
//!
//! ; Restriction: no blank lines.
//! ; Restriction: markers must match (in case of `(` with `)`).
//! title ::= marker [  *( code - '\\' | '\\' [ marker ] ) ] marker
//! marker ::= '"' | '\'' | '('
//!
//! whitespace ::= eol *whitespace | 1*space_or_tab [ eol *whitespace ]
//! space_or_tab ::= ' ' | '\t'
//! ```
//!
//! Definitions in markdown to not, on their own, relate to anything in HTML.
//! When connected with a link (reference), they together relate to the `<a>`
//! element in HTML.
//! The definition forms its `href`, and optionally `title`, attributes.
//! See [*§ 4.5.1 The `a` element*][html] in the HTML spec for more info.
//!
//! The `label`, `destination`, and `title` parts are interpreted as the
//! [string][] content type.
//! That means that character escapes and character reference are allowed.
//!
//! ## References
//!
//! *   [`definition.js` in `micromark`](https://github.com/micromark/micromark/blob/main/packages/micromark-core-commonmark/dev/lib/definition.js)
//! *   [*§ 4.7 Link reference definitions* in `CommonMark`](https://spec.commonmark.org/0.30/#link-reference-definitions)
//!
//! [flow]: crate::content::flow
//! [string]: crate::content::string
//! [html]: https://html.spec.whatwg.org/multipage/text-level-semantics.html#the-a-element
//!
//! <!-- To do: link link (reference) -->
//!
//! <!-- To do: describe how references and definitions match -->

use crate::construct::{
    partial_destination::start as destination, partial_label::start as label,
    partial_title::start as title, partial_whitespace::start as whitespace,
};
use crate::tokenizer::{Code, State, StateFnResult, TokenType, Tokenizer};

/// At the start of a definition.
///
/// ```markdown
/// |[a]: b "c"
/// ```
pub fn start(tokenizer: &mut Tokenizer, code: Code) -> StateFnResult {
    match code {
        Code::Char('[') => {
            tokenizer.enter(TokenType::Definition);
            tokenizer.go(label, label_after)(tokenizer, code)
        }
        _ => (State::Nok, None),
    }
}

/// After the label of a definition.
///
/// ```markdown
/// [a]|: b "c"
/// ```
pub fn label_after(tokenizer: &mut Tokenizer, code: Code) -> StateFnResult {
    // To do: get the identifier:
    // identifier = normalizeIdentifier(
    //   self.sliceSerialize(self.events[self.events.length - 1][1]).slice(1, -1)
    // )

    match code {
        Code::Char(':') => {
            tokenizer.enter(TokenType::DefinitionMarker);
            tokenizer.consume(code);
            tokenizer.exit(TokenType::DefinitionMarker);
            (State::Fn(Box::new(marker_after)), None)
        }
        _ => (State::Nok, None),
    }
}

/// After the marker of a definition.
///
/// ```markdown
/// [a]:| b "c"
///
/// [a]:| ␊
///  b "c"
/// ```
pub fn marker_after(tokenizer: &mut Tokenizer, code: Code) -> StateFnResult {
    tokenizer.attempt(
        |t, c| whitespace(t, c, TokenType::Whitespace),
        |_ok| Box::new(marker_after_optional_whitespace),
    )(tokenizer, code)
}

/// After the marker, after whitespace.
///
/// ```markdown
/// [a]: |b "c"
///
/// [a]: |␊
///  b "c"
/// ```
pub fn marker_after_optional_whitespace(tokenizer: &mut Tokenizer, code: Code) -> StateFnResult {
    match code {
        Code::CarriageReturnLineFeed | Code::Char('\r' | '\n') => {
            tokenizer.enter(TokenType::LineEnding);
            tokenizer.consume(code);
            tokenizer.exit(TokenType::LineEnding);
            (State::Fn(Box::new(marker_after_optional_line_ending)), None)
        }
        _ => destination_before(tokenizer, code),
    }
}

/// After the marker, after a line ending.
///
/// ```markdown
/// [a]:
/// | b "c"
/// ```
pub fn marker_after_optional_line_ending(tokenizer: &mut Tokenizer, code: Code) -> StateFnResult {
    tokenizer.attempt(
        |t, c| whitespace(t, c, TokenType::Whitespace),
        |_ok| Box::new(destination_before),
    )(tokenizer, code)
}

/// Before a destination.
///
/// ```markdown
/// [a]: |b "c"
///
/// [a]:
///  |b "c"
/// ```
pub fn destination_before(tokenizer: &mut Tokenizer, code: Code) -> StateFnResult {
    let event = tokenizer.events.last().unwrap();
    // Blank line not ok.
    let char_nok = matches!(
        code,
        Code::None | Code::CarriageReturnLineFeed | Code::Char('\r' | '\n')
    );

    if !char_nok
        && (event.token_type == TokenType::LineEnding || event.token_type == TokenType::Whitespace)
    {
        tokenizer.go(destination, destination_after)(tokenizer, code)
    } else {
        (State::Nok, None)
    }
}

/// After a destination.
///
/// ```markdown
/// [a]: b| "c"
///
/// [a]: b| ␊
///  "c"
/// ```
pub fn destination_after(tokenizer: &mut Tokenizer, code: Code) -> StateFnResult {
    tokenizer.attempt(title_before, |_ok| Box::new(after))(tokenizer, code)
}

/// After a definition.
///
/// ```markdown
/// [a]: b|
/// [a]: b "c"|
/// ```
pub fn after(tokenizer: &mut Tokenizer, code: Code) -> StateFnResult {
    tokenizer.attempt(
        |t, c| whitespace(t, c, TokenType::Whitespace),
        |_ok| Box::new(after_whitespace),
    )(tokenizer, code)
}

/// After a definition, after optional whitespace.
///
/// ```markdown
/// [a]: b |
/// [a]: b "c"|
/// ```
pub fn after_whitespace(tokenizer: &mut Tokenizer, code: Code) -> StateFnResult {
    match code {
        Code::None | Code::CarriageReturnLineFeed | Code::Char('\r' | '\n') => {
            tokenizer.exit(TokenType::Definition);
            (State::Ok, Some(vec![code]))
        }
        _ => (State::Nok, None),
    }
}

/// After a destination, presumably before a title.
///
/// ```markdown
/// [a]: b| "c"
///
/// [a]: b| ␊
///  "c"
/// ```
pub fn title_before(tokenizer: &mut Tokenizer, code: Code) -> StateFnResult {
    tokenizer.attempt(
        |t, c| whitespace(t, c, TokenType::Whitespace),
        |_ok| Box::new(title_before_after_optional_whitespace),
    )(tokenizer, code)
}

/// Before a title, after optional whitespace.
///
/// ```markdown
/// [a]: b |"c"
///
/// [a]: b |␊
///  "c"
/// ```
pub fn title_before_after_optional_whitespace(
    tokenizer: &mut Tokenizer,
    code: Code,
) -> StateFnResult {
    match code {
        Code::CarriageReturnLineFeed | Code::Char('\r' | '\n') => {
            tokenizer.enter(TokenType::LineEnding);
            tokenizer.consume(code);
            tokenizer.exit(TokenType::LineEnding);
            (
                State::Fn(Box::new(title_before_after_optional_line_ending)),
                None,
            )
        }
        _ => title_before_marker(tokenizer, code),
    }
}

/// Before a title, after a line ending.
///
/// ```markdown
/// [a]: b␊
/// | "c"
/// ```
pub fn title_before_after_optional_line_ending(
    tokenizer: &mut Tokenizer,
    code: Code,
) -> StateFnResult {
    tokenizer.attempt(
        |t, c| whitespace(t, c, TokenType::Whitespace),
        |_ok| Box::new(title_before_marker),
    )(tokenizer, code)
}

/// Before a title, after a line ending.
///
/// ```markdown
/// [a]: b␊
/// | "c"
/// ```
pub fn title_before_marker(tokenizer: &mut Tokenizer, code: Code) -> StateFnResult {
    let event = tokenizer.events.last().unwrap();

    if event.token_type == TokenType::LineEnding || event.token_type == TokenType::Whitespace {
        tokenizer.go(title, title_after)(tokenizer, code)
    } else {
        (State::Nok, None)
    }
}

/// After a title.
///
/// ```markdown
/// [a]: b "c"|
///
/// [a]: b␊
/// "c"|
/// ```
pub fn title_after(tokenizer: &mut Tokenizer, code: Code) -> StateFnResult {
    tokenizer.attempt(
        |t, c| whitespace(t, c, TokenType::Whitespace),
        |_ok| Box::new(title_after_after_optional_whitespace),
    )(tokenizer, code)
}

/// After a title, after optional whitespace.
///
/// ```markdown
/// [a]: b "c"|
///
/// [a]: b "c" |
/// ```
pub fn title_after_after_optional_whitespace(
    _tokenizer: &mut Tokenizer,
    code: Code,
) -> StateFnResult {
    match code {
        Code::None | Code::CarriageReturnLineFeed | Code::Char('\r' | '\n') => {
            (State::Ok, Some(vec![code]))
        }
        _ => (State::Nok, None),
    }
}
