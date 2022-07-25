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

use crate::token::Token;
use crate::tokenizer::{Code, ContentType, State, Tokenizer};

/// Configuration.
///
/// You must pass the token types in that are used.
#[derive(Debug)]
pub struct Options {
    /// Token for the whole destination.
    pub destination: Token,
    /// Token for a literal (enclosed) destination.
    pub literal: Token,
    /// Token for a literal marker.
    pub marker: Token,
    /// Token for a raw destination.
    pub raw: Token,
    /// Token for a the string.
    pub string: Token,
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
/// > | <aa>
///     ^
/// > | aa
///     ^
/// ```
pub fn start(tokenizer: &mut Tokenizer, options: Options) -> State {
    let info = Info {
        balance: 0,
        options,
    };

    match tokenizer.current {
        Code::Char('<') => {
            tokenizer.enter(info.options.destination.clone());
            tokenizer.enter(info.options.literal.clone());
            tokenizer.enter(info.options.marker.clone());
            tokenizer.consume();
            tokenizer.exit(info.options.marker.clone());
            State::Fn(Box::new(|t| enclosed_before(t, info)))
        }
        Code::None | Code::CarriageReturnLineFeed | Code::VirtualSpace | Code::Char(' ' | ')') => {
            State::Nok
        }
        Code::Char(char) if char.is_ascii_control() => State::Nok,
        Code::Char(_) => {
            tokenizer.enter(info.options.destination.clone());
            tokenizer.enter(info.options.raw.clone());
            tokenizer.enter(info.options.string.clone());
            tokenizer.enter_with_content(Token::Data, Some(ContentType::String));
            raw(tokenizer, info)
        }
    }
}

/// After `<`, before an enclosed destination.
///
/// ```markdown
/// > | <aa>
///      ^
/// ```
fn enclosed_before(tokenizer: &mut Tokenizer, info: Info) -> State {
    if let Code::Char('>') = tokenizer.current {
        tokenizer.enter(info.options.marker.clone());
        tokenizer.consume();
        tokenizer.exit(info.options.marker.clone());
        tokenizer.exit(info.options.literal.clone());
        tokenizer.exit(info.options.destination);
        State::Ok
    } else {
        tokenizer.enter(info.options.string.clone());
        tokenizer.enter_with_content(Token::Data, Some(ContentType::String));
        enclosed(tokenizer, info)
    }
}

/// In an enclosed destination.
///
/// ```markdown
/// > | <aa>
///      ^
/// ```
fn enclosed(tokenizer: &mut Tokenizer, info: Info) -> State {
    match tokenizer.current {
        Code::Char('>') => {
            tokenizer.exit(Token::Data);
            tokenizer.exit(info.options.string.clone());
            enclosed_before(tokenizer, info)
        }
        Code::None | Code::CarriageReturnLineFeed | Code::Char('\n' | '\r' | '<') => State::Nok,
        Code::Char('\\') => {
            tokenizer.consume();
            State::Fn(Box::new(|t| enclosed_escape(t, info)))
        }
        _ => {
            tokenizer.consume();
            State::Fn(Box::new(|t| enclosed(t, info)))
        }
    }
}

/// After `\`, in an enclosed destination.
///
/// ```markdown
/// > | <a\*a>
///        ^
/// ```
fn enclosed_escape(tokenizer: &mut Tokenizer, info: Info) -> State {
    match tokenizer.current {
        Code::Char('<' | '>' | '\\') => {
            tokenizer.consume();
            State::Fn(Box::new(|t| enclosed(t, info)))
        }
        _ => enclosed(tokenizer, info),
    }
}

/// In a raw destination.
///
/// ```markdown
/// > | aa
///     ^
/// ```
fn raw(tokenizer: &mut Tokenizer, mut info: Info) -> State {
    match tokenizer.current {
        Code::Char('(') => {
            if info.balance >= info.options.limit {
                State::Nok
            } else {
                tokenizer.consume();
                info.balance += 1;
                State::Fn(Box::new(move |t| raw(t, info)))
            }
        }
        Code::Char(')') => {
            if info.balance == 0 {
                tokenizer.exit(Token::Data);
                tokenizer.exit(info.options.string.clone());
                tokenizer.exit(info.options.raw.clone());
                tokenizer.exit(info.options.destination);
                State::Ok
            } else {
                tokenizer.consume();
                info.balance -= 1;
                State::Fn(Box::new(move |t| raw(t, info)))
            }
        }
        Code::None
        | Code::CarriageReturnLineFeed
        | Code::VirtualSpace
        | Code::Char('\t' | '\n' | '\r' | ' ') => {
            if info.balance > 0 {
                State::Nok
            } else {
                tokenizer.exit(Token::Data);
                tokenizer.exit(info.options.string.clone());
                tokenizer.exit(info.options.raw.clone());
                tokenizer.exit(info.options.destination);
                State::Ok
            }
        }
        Code::Char(char) if char.is_ascii_control() => State::Nok,
        Code::Char('\\') => {
            tokenizer.consume();
            State::Fn(Box::new(move |t| raw_escape(t, info)))
        }
        Code::Char(_) => {
            tokenizer.consume();
            State::Fn(Box::new(move |t| raw(t, info)))
        }
    }
}

/// After `\`, in a raw destination.
///
/// ```markdown
/// > | a\*a
///       ^
/// ```
fn raw_escape(tokenizer: &mut Tokenizer, info: Info) -> State {
    match tokenizer.current {
        Code::Char('(' | ')' | '\\') => {
            tokenizer.consume();
            State::Fn(Box::new(move |t| raw(t, info)))
        }
        _ => raw(tokenizer, info),
    }
}
