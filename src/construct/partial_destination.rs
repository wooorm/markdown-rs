//! Destination occurs in [definition][] and label end.
//!
//! They’re formed with the following BNF:
//!
//! ```bnf
//! destination ::= destination_enclosed | destination_raw
//!
//! destination_enclosed ::= '<' *( destination_enclosed_text | destination_enclosed_escape ) '>'
//! destination_enclosed_text ::= code - '<' - '\\' - '>' - eol
//! destination_enclosed_escape ::= '\\' [ '<' | '\\' | '>' ]
//! destination_raw ::= 1*( destination_raw_text | destination_raw_escape )
//! ; Restriction: unbalanced `)` characters are not allowed.
//! destination_raw_text ::= code - '\\' - ascii_control - space_or_tab - eol
//! destination_raw_escape ::= '\\' [ '(' | ')' | '\\' ]
//! ```
//!
//! Balanced parens allowed in raw destinations.
//! They are counted with a counter that starts at `0`, and is incremented
//! every time `(` occurs and decremented every time `)` occurs.
//! If `)` is found when the counter is `0`, the destination closes immediately
//! after it.
//! Escaped parens do not count.
//!
//! It is recommended to use the enclosed variant of destinations, as it allows
//! arbitrary parens, and also allows for whitespace and other characters in
//! URLs.
//!
//! The destination is interpreted as the [string][] content type.
//! That means that [character escapes][character_escape] and
//! [character references][character_reference] are allowed.
//!
//! ## References
//!
//! *   [`micromark-factory-destination/index.js` in `micromark`](https://github.com/micromark/micromark/blob/main/packages/micromark-factory-destination/dev/index.js)
//!
//! [definition]: crate::construct::definition
//! [string]: crate::content::string
//! [character_escape]: crate::construct::character_escape
//! [character_reference]: crate::construct::character_reference
//!
//! <!-- To do: link label end. -->

// To do: pass token types in.

use crate::tokenizer::{Code, State, StateFnResult, TokenType, Tokenizer};

/// Before a destination.
///
/// ```markdown
/// |<ab>
/// |ab
/// ```
pub fn start(tokenizer: &mut Tokenizer, code: Code) -> StateFnResult {
    match code {
        Code::Char('<') => {
            tokenizer.enter(TokenType::DefinitionDestination);
            tokenizer.enter(TokenType::DefinitionDestinationLiteral);
            tokenizer.enter(TokenType::DefinitionDestinationLiteralMarker);
            tokenizer.consume(code);
            tokenizer.exit(TokenType::DefinitionDestinationLiteralMarker);
            (State::Fn(Box::new(enclosed_before)), None)
        }
        Code::None | Code::CarriageReturnLineFeed | Code::VirtualSpace | Code::Char(')') => {
            (State::Nok, None)
        }
        Code::Char(char) if char.is_ascii_control() => (State::Nok, None),
        Code::Char(_) => {
            tokenizer.enter(TokenType::DefinitionDestination);
            tokenizer.enter(TokenType::DefinitionDestinationRaw);
            tokenizer.enter(TokenType::DefinitionDestinationString);
            // To do: link.
            tokenizer.enter(TokenType::ChunkString);
            raw(tokenizer, code, 0)
        }
    }
}

/// After `<`, before an enclosed destination.
///
/// ```markdown
/// <|ab>
/// ```
fn enclosed_before(tokenizer: &mut Tokenizer, code: Code) -> StateFnResult {
    if let Code::Char('>') = code {
        tokenizer.enter(TokenType::DefinitionDestinationLiteralMarker);
        tokenizer.consume(code);
        tokenizer.exit(TokenType::DefinitionDestinationLiteralMarker);
        tokenizer.exit(TokenType::DefinitionDestinationLiteral);
        tokenizer.exit(TokenType::DefinitionDestination);
        (State::Ok, None)
    } else {
        tokenizer.enter(TokenType::DefinitionDestinationString);
        // To do: link.
        tokenizer.enter(TokenType::ChunkString);
        enclosed(tokenizer, code)
    }
}

/// In an enclosed destination.
///
/// ```markdown
/// <u|rl>
/// ```
fn enclosed(tokenizer: &mut Tokenizer, code: Code) -> StateFnResult {
    match code {
        Code::Char('>') => {
            tokenizer.exit(TokenType::ChunkString);
            tokenizer.exit(TokenType::DefinitionDestinationString);
            enclosed_before(tokenizer, code)
        }
        Code::None | Code::CarriageReturnLineFeed | Code::Char('\r' | '\n' | '<') => {
            (State::Nok, None)
        }
        Code::Char('\\') => {
            tokenizer.consume(code);
            (State::Fn(Box::new(enclosed_escape)), None)
        }
        _ => {
            tokenizer.consume(code);
            (State::Fn(Box::new(enclosed)), None)
        }
    }
}

/// After `\`, in an enclosed destination.
///
/// ```markdown
/// <a\|>b>
/// ```
fn enclosed_escape(tokenizer: &mut Tokenizer, code: Code) -> StateFnResult {
    match code {
        Code::Char('<' | '>' | '\\') => {
            tokenizer.consume(code);
            (State::Fn(Box::new(enclosed)), None)
        }
        _ => enclosed(tokenizer, code),
    }
}

/// In a raw destination.
///
/// ```markdown
/// a|b
/// ```
// To do: these arms can be improved?
fn raw(tokenizer: &mut Tokenizer, code: Code, balance: usize) -> StateFnResult {
    // To do: configurable.
    let limit = usize::MAX;

    match code {
        Code::Char('(') if balance >= limit => (State::Nok, None),
        Code::Char('(') => {
            tokenizer.consume(code);
            (
                State::Fn(Box::new(move |t, c| raw(t, c, balance + 1))),
                None,
            )
        }
        Code::Char(')') if balance == 0 => {
            tokenizer.exit(TokenType::ChunkString);
            tokenizer.exit(TokenType::DefinitionDestinationString);
            tokenizer.exit(TokenType::DefinitionDestinationRaw);
            tokenizer.exit(TokenType::DefinitionDestination);
            (State::Ok, Some(vec![code]))
        }
        Code::Char(')') => {
            tokenizer.consume(code);
            (
                State::Fn(Box::new(move |t, c| raw(t, c, balance - 1))),
                None,
            )
        }
        Code::None
        | Code::CarriageReturnLineFeed
        | Code::VirtualSpace
        | Code::Char('\t' | '\r' | '\n' | ' ')
            if balance > 0 =>
        {
            (State::Nok, None)
        }
        Code::None
        | Code::CarriageReturnLineFeed
        | Code::VirtualSpace
        | Code::Char('\t' | '\r' | '\n' | ' ') => {
            tokenizer.exit(TokenType::ChunkString);
            tokenizer.exit(TokenType::DefinitionDestinationString);
            tokenizer.exit(TokenType::DefinitionDestinationRaw);
            tokenizer.exit(TokenType::DefinitionDestination);
            (State::Ok, Some(vec![code]))
        }
        Code::Char(char) if char.is_ascii_control() => (State::Nok, None),
        Code::Char('\\') => {
            tokenizer.consume(code);
            (
                State::Fn(Box::new(move |t, c| raw_escape(t, c, balance))),
                None,
            )
        }
        Code::Char(_) => {
            tokenizer.consume(code);
            (State::Fn(Box::new(move |t, c| raw(t, c, balance))), None)
        }
    }
}

/// After `\`, in a raw destination.
///
/// ```markdown
/// a\|)b
/// ```
fn raw_escape(tokenizer: &mut Tokenizer, code: Code, balance: usize) -> StateFnResult {
    match code {
        Code::Char('(' | ')' | '\\') => {
            tokenizer.consume(code);
            (
                State::Fn(Box::new(move |t, c| raw(t, c, balance + 1))),
                None,
            )
        }
        _ => raw(tokenizer, code, balance),
    }
}
