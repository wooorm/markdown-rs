//! Title occurs in [definition][] and [label end][label_end].
//!
//! They’re formed with the following BNF:
//!
//! ```bnf
//! ; Restriction: no blank lines.
//! ; Restriction: markers must match (in case of `(` with `)`).
//! title ::= marker [  *( code - '\\' | '\\' [ marker ] ) ] marker
//! marker ::= '"' | '\'' | '('
//! ```
//!
//! Titles can be double quoted (`"a"`), single quoted (`'a'`), or
//! parenthesized (`(a)`).
//!
//! Titles can contain line endings and whitespace, but they are not allowed to
//! contain blank lines.
//! They are allowed to be blank themselves.
//!
//! The title is interpreted as the [string][] content type.
//! That means that [character escapes][character_escape] and
//! [character references][character_reference] are allowed.
//!
//! ## References
//!
//! *   [`micromark-factory-title/index.js` in `micromark`](https://github.com/micromark/micromark/blob/main/packages/micromark-factory-title/dev/index.js)
//!
//! [definition]: crate::construct::definition
//! [string]: crate::content::string
//! [character_escape]: crate::construct::character_escape
//! [character_reference]: crate::construct::character_reference
//! [label_end]: crate::construct::label_end

use crate::construct::partial_space_or_tab::{space_or_tab_eol_with_options, EolOptions};
use crate::subtokenize::link;
use crate::token::Token;
use crate::tokenizer::{ContentType, State, StateName, Tokenizer};

/// Before a title.
///
/// ```markdown
/// > | "a"
///     ^
/// ```
pub fn start(tokenizer: &mut Tokenizer) -> State {
    match tokenizer.current {
        Some(b'"' | b'\'' | b'(') => {
            let marker = tokenizer.current.unwrap();
            tokenizer.tokenize_state.marker = if marker == b'(' { b')' } else { marker };
            tokenizer.enter(tokenizer.tokenize_state.token_1.clone());
            tokenizer.enter(tokenizer.tokenize_state.token_2.clone());
            tokenizer.consume();
            tokenizer.exit(tokenizer.tokenize_state.token_2.clone());
            State::Next(StateName::TitleBegin)
        }
        _ => State::Nok,
    }
}

/// After the opening marker.
///
/// This is also used when at the closing marker.
///
/// ```markdown
/// > | "a"
///      ^
/// ```
pub fn begin(tokenizer: &mut Tokenizer) -> State {
    match tokenizer.current {
        Some(b'"' | b'\'' | b')')
            if tokenizer.current.unwrap() == tokenizer.tokenize_state.marker =>
        {
            tokenizer.enter(tokenizer.tokenize_state.token_2.clone());
            tokenizer.consume();
            tokenizer.exit(tokenizer.tokenize_state.token_2.clone());
            tokenizer.exit(tokenizer.tokenize_state.token_1.clone());
            tokenizer.tokenize_state.marker = 0;
            tokenizer.tokenize_state.connect = false;
            State::Ok
        }
        _ => {
            tokenizer.enter(tokenizer.tokenize_state.token_3.clone());
            State::Retry(StateName::TitleAtBreak)
        }
    }
}

/// At something, before something else.
///
/// ```markdown
/// > | "a"
///      ^
/// ```
pub fn at_break(tokenizer: &mut Tokenizer) -> State {
    match tokenizer.current {
        None => {
            tokenizer.tokenize_state.marker = 0;
            tokenizer.tokenize_state.connect = false;
            State::Nok
        }
        Some(b'\n') => {
            let name = space_or_tab_eol_with_options(
                tokenizer,
                EolOptions {
                    content_type: Some(ContentType::String),
                    connect: tokenizer.tokenize_state.connect,
                },
            );

            tokenizer.attempt(
                name,
                State::Next(StateName::TitleAfterEol),
                State::Next(StateName::TitleAtBlankLine),
            )
        }
        Some(b'"' | b'\'' | b')')
            if tokenizer.current.unwrap() == tokenizer.tokenize_state.marker =>
        {
            tokenizer.exit(tokenizer.tokenize_state.token_3.clone());
            State::Retry(StateName::TitleBegin)
        }
        Some(_) => {
            tokenizer.enter_with_content(Token::Data, Some(ContentType::String));

            if tokenizer.tokenize_state.connect {
                let index = tokenizer.events.len() - 1;
                link(&mut tokenizer.events, index);
            } else {
                tokenizer.tokenize_state.connect = true;
            }

            State::Retry(StateName::TitleInside)
        }
    }
}

/// In a title, after whitespace.
///
/// ```markdown
///   | "a␊
/// > | b"
///     ^
/// ```
pub fn after_eol(tokenizer: &mut Tokenizer) -> State {
    tokenizer.tokenize_state.connect = true;
    State::Retry(StateName::TitleAtBreak)
}

/// In a title, at a blank line.
///
/// ```markdown
///   | "a␊
/// > | ␊
///     ^
///   | b"
/// ```
pub fn at_blank_line(tokenizer: &mut Tokenizer) -> State {
    tokenizer.tokenize_state.marker = 0;
    tokenizer.tokenize_state.connect = false;
    State::Nok
}

/// In title text.
///
/// ```markdown
/// > | "a"
///      ^
/// ```
pub fn inside(tokenizer: &mut Tokenizer) -> State {
    match tokenizer.current {
        None | Some(b'\n') => {
            tokenizer.exit(Token::Data);
            State::Retry(StateName::TitleAtBreak)
        }
        Some(b'"' | b'\'' | b')')
            if tokenizer.current.unwrap() == tokenizer.tokenize_state.marker =>
        {
            tokenizer.exit(Token::Data);
            State::Retry(StateName::TitleAtBreak)
        }
        Some(byte) => {
            tokenizer.consume();
            State::Next(if matches!(byte, b'\\') {
                StateName::TitleEscape
            } else {
                StateName::TitleInside
            })
        }
    }
}

/// After `\`, in title text.
///
/// ```markdown
/// > | "a\*b"
///      ^
/// ```
pub fn escape(tokenizer: &mut Tokenizer) -> State {
    match tokenizer.current {
        Some(b'"' | b'\'' | b')') => {
            tokenizer.consume();
            State::Next(StateName::TitleInside)
        }
        _ => State::Retry(StateName::TitleInside),
    }
}
