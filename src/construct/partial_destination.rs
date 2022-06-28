//! Destination occurs in [definition][] and [label end][label_end].
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
//! before it.
//! Escaped parens do not count in balancing.
//!
//! The destination is interpreted as the [string][] content type.
//! That means that [character escapes][character_escape] and
//! [character references][character_reference] are allowed.
//!
//! The grammar for enclosed destinations (`<x>`) prohibits the use of `<`,
//! `>`, and line endings to form URLs.
//! The angle brackets can be encoded as a character reference, character
//! escape, or percent encoding: for `<` as `&lt;`, `\<`, or `%3c` and for
//! `>` as `&gt;`, `\>`, or `%3e`.
//!
//! The grammar for raw destinations (`x`) prohibits space (` `) and all
//! [ASCII control][char::is_ascii_control] characters, which thus must be
//! encoded.
//! Unbalanced arens can be encoded as a character reference, character escape,
//! or percent encoding: for `(` as `&lpar;`, `\(`, or `%28` and for `)` as
//! `&rpar;`, `\)`, or `%29`.
//!
//! It is recommended to use the enclosed variant of destinations, as it allows
//! the most characters, including arbitrary parens, in URLs.
//!
//! There are several cases where incorrect encoding of URLs would, in other
//! languages, result in a parse error.
//! In markdown, there are no errors, and URLs are normalized.
//! In addition, unicode characters are percent encoded
//! ([`sanitize_uri`][sanitize_uri]).
//! For example:
//!
//! ```markdown
//! [x]
//!
//! [x]: <https://a👍b%>
//! ```
//!
//! Yields:
//!
//! ```html
//! <p><a href="https://a%F0%9F%91%8Db%25">x</a></p>
//! ```
//!
//! ## References
//!
//! *   [`micromark-factory-destination/index.js` in `micromark`](https://github.com/micromark/micromark/blob/main/packages/micromark-factory-destination/dev/index.js)
//!
//! [definition]: crate::construct::definition
//! [string]: crate::content::string
//! [character_escape]: crate::construct::character_escape
//! [character_reference]: crate::construct::character_reference
//! [label_end]: crate::construct::label_end
//! [sanitize_uri]: crate::util::sanitize_uri

use crate::tokenizer::{Code, ContentType, State, StateFnResult, TokenType, Tokenizer};

/// Configuration.
///
/// You must pass the token types in that are used.
#[derive(Debug)]
pub struct Options {
    /// Token for the whole destination.
    pub destination: TokenType,
    /// Token for a literal (enclosed) destination.
    pub literal: TokenType,
    /// Token for a literal marker.
    pub marker: TokenType,
    /// Token for a raw destination.
    pub raw: TokenType,
    /// Token for a the string.
    pub string: TokenType,
    /// Maximum unbalanced parens.
    pub limit: usize,
}

/// State needed to parse destination.
#[derive(Debug)]
struct Info {
    /// Paren balance (used in raw).
    balance: usize,
    /// Configuration.
    options: Options,
}

/// Before a destination.
///
/// ```markdown
/// |<ab>
/// |ab
/// ```
pub fn start(tokenizer: &mut Tokenizer, code: Code, options: Options) -> StateFnResult {
    let info = Info {
        balance: 0,
        options,
    };

    match code {
        Code::Char('<') => {
            tokenizer.enter(info.options.destination.clone());
            tokenizer.enter(info.options.literal.clone());
            tokenizer.enter(info.options.marker.clone());
            tokenizer.consume(code);
            tokenizer.exit(info.options.marker.clone());
            (
                State::Fn(Box::new(|t, c| enclosed_before(t, c, info))),
                None,
            )
        }
        Code::None | Code::CarriageReturnLineFeed | Code::VirtualSpace | Code::Char(' ' | ')') => {
            (State::Nok, None)
        }
        Code::Char(char) if char.is_ascii_control() => (State::Nok, None),
        Code::Char(_) => {
            tokenizer.enter(info.options.destination.clone());
            tokenizer.enter(info.options.raw.clone());
            tokenizer.enter(info.options.string.clone());
            tokenizer.enter_with_content(TokenType::Data, Some(ContentType::String));
            raw(tokenizer, code, info)
        }
    }
}

/// After `<`, before an enclosed destination.
///
/// ```markdown
/// <|ab>
/// ```
fn enclosed_before(tokenizer: &mut Tokenizer, code: Code, info: Info) -> StateFnResult {
    if let Code::Char('>') = code {
        tokenizer.enter(info.options.marker.clone());
        tokenizer.consume(code);
        tokenizer.exit(info.options.marker.clone());
        tokenizer.exit(info.options.literal.clone());
        tokenizer.exit(info.options.destination);
        (State::Ok, None)
    } else {
        tokenizer.enter(info.options.string.clone());
        tokenizer.enter_with_content(TokenType::Data, Some(ContentType::String));
        enclosed(tokenizer, code, info)
    }
}

/// In an enclosed destination.
///
/// ```markdown
/// <u|rl>
/// ```
fn enclosed(tokenizer: &mut Tokenizer, code: Code, info: Info) -> StateFnResult {
    match code {
        Code::Char('>') => {
            tokenizer.exit(TokenType::Data);
            tokenizer.exit(info.options.string.clone());
            enclosed_before(tokenizer, code, info)
        }
        Code::None | Code::CarriageReturnLineFeed | Code::Char('\r' | '\n' | '<') => {
            (State::Nok, None)
        }
        Code::Char('\\') => {
            tokenizer.consume(code);
            (
                State::Fn(Box::new(|t, c| enclosed_escape(t, c, info))),
                None,
            )
        }
        _ => {
            tokenizer.consume(code);
            (State::Fn(Box::new(|t, c| enclosed(t, c, info))), None)
        }
    }
}

/// After `\`, in an enclosed destination.
///
/// ```markdown
/// <a\|>b>
/// ```
fn enclosed_escape(tokenizer: &mut Tokenizer, code: Code, info: Info) -> StateFnResult {
    match code {
        Code::Char('<' | '>' | '\\') => {
            tokenizer.consume(code);
            (State::Fn(Box::new(|t, c| enclosed(t, c, info))), None)
        }
        _ => enclosed(tokenizer, code, info),
    }
}

/// In a raw destination.
///
/// ```markdown
/// a|b
/// ```
fn raw(tokenizer: &mut Tokenizer, code: Code, mut info: Info) -> StateFnResult {
    match code {
        Code::Char('(') => {
            if info.balance >= info.options.limit {
                (State::Nok, None)
            } else {
                tokenizer.consume(code);
                info.balance += 1;
                (State::Fn(Box::new(move |t, c| raw(t, c, info))), None)
            }
        }
        Code::Char(')') => {
            if info.balance == 0 {
                tokenizer.exit(TokenType::Data);
                tokenizer.exit(info.options.string.clone());
                tokenizer.exit(info.options.raw.clone());
                tokenizer.exit(info.options.destination);
                (State::Ok, Some(vec![code]))
            } else {
                tokenizer.consume(code);
                info.balance -= 1;
                (State::Fn(Box::new(move |t, c| raw(t, c, info))), None)
            }
        }
        Code::None
        | Code::CarriageReturnLineFeed
        | Code::VirtualSpace
        | Code::Char('\t' | '\r' | '\n' | ' ') => {
            if info.balance > 0 {
                (State::Nok, None)
            } else {
                tokenizer.exit(TokenType::Data);
                tokenizer.exit(info.options.string.clone());
                tokenizer.exit(info.options.raw.clone());
                tokenizer.exit(info.options.destination);
                (State::Ok, Some(vec![code]))
            }
        }
        Code::Char(char) if char.is_ascii_control() => (State::Nok, None),
        Code::Char('\\') => {
            tokenizer.consume(code);
            (
                State::Fn(Box::new(move |t, c| raw_escape(t, c, info))),
                None,
            )
        }
        Code::Char(_) => {
            tokenizer.consume(code);
            (State::Fn(Box::new(move |t, c| raw(t, c, info))), None)
        }
    }
}

/// After `\`, in a raw destination.
///
/// ```markdown
/// a\|)b
/// ```
fn raw_escape(tokenizer: &mut Tokenizer, code: Code, info: Info) -> StateFnResult {
    match code {
        Code::Char('(' | ')' | '\\') => {
            tokenizer.consume(code);
            (State::Fn(Box::new(move |t, c| raw(t, c, info))), None)
        }
        _ => raw(tokenizer, code, info),
    }
}
