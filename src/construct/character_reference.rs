//! Character references are a construct that occurs in the [string][] and
//! [text][] content types.
//!
//! They’re formed with the following BNF:
//!
//! ```bnf
//! character_reference ::= '&' (numeric | named) ';'
//!
//! numeric ::= '#' (hexadecimal | decimal)
//! ; Note: Limit of `6` imposed as all bigger numbers are invalid:
//! hexadecimal ::= ('x' | 'X') 1*6(ascii_hexdigit)
//! ; Note: Limit of `7` imposed as all bigger numbers are invalid:
//! decimal ::= 1*7(ascii_digit)
//! ; Note: Limit of `31` imposed by `CounterClockwiseContourIntegral`:
//! ; Note: Limited to any known named character reference (see `constants.rs`)
//! named ::= 1*31(ascii_alphanumeric)
//! ```
//!
//! Like much of markdown, there are no “invalid” character references.
//! However, for security reasons, several numeric character references parse
//! fine but are not rendered as their corresponding character and they are
//! instead replaced by a U+FFFD REPLACEMENT CHARACTER (`�`).
//! See [`decode_numeric`][decode_numeric] for more info.
//!
//! To escape ASCII punctuation characters, use the terser
//! [character escape][character_escape] construct instead (as in, `\&`).
//!
//! Character references in markdown are not the same as character references
//! in HTML.
//! Notably, HTML allows several character references without a closing
//! semicolon.
//! See [*§ 13.2.5.72 Character reference state* in the HTML spec][html] for more info.
//!
//! Character references are parsed insensitive to casing.
//! The casing of hexadecimal numeric character references has no effect.
//! The casing of named character references does not matter when parsing them,
//! but does affect whether they match.
//! Depending on the name, one or more cases are allowed, such as that `AMP`
//! and `amp` are both allowed but other cases are not.
//! See [`CHARACTER_REFERENCES`][character_references] for which
//! names match.
//!
//! ## Tokens
//!
//! *   [`CharacterReference`][Token::CharacterReference]
//! *   [`CharacterReferenceMarker`][Token::CharacterReferenceMarker]
//! *   [`CharacterReferenceMarkerHexadecimal`][Token::CharacterReferenceMarkerHexadecimal]
//! *   [`CharacterReferenceMarkerNumeric`][Token::CharacterReferenceMarkerNumeric]
//! *   [`CharacterReferenceMarkerSemi`][Token::CharacterReferenceMarkerSemi]
//! *   [`CharacterReferenceValue`][Token::CharacterReferenceValue]
//!
//! ## References
//!
//! *   [`character-reference.js` in `micromark`](https://github.com/micromark/micromark/blob/main/packages/micromark-core-commonmark/dev/lib/character-reference.js)
//! *   [*§ 2.5 Entity and numeric character references* in `CommonMark`](https://spec.commonmark.org/0.30/#entity-and-numeric-character-references)
//!
//! [string]: crate::content::string
//! [text]: crate::content::text
//! [character_escape]: crate::construct::character_reference
//! [decode_numeric]: crate::util::decode_character_reference::decode_numeric
//! [character_references]: crate::constant::CHARACTER_REFERENCES
//! [html]: https://html.spec.whatwg.org/multipage/parsing.html#character-reference-state

use crate::constant::{
    CHARACTER_REFERENCES, CHARACTER_REFERENCE_DECIMAL_SIZE_MAX,
    CHARACTER_REFERENCE_HEXADECIMAL_SIZE_MAX, CHARACTER_REFERENCE_NAMED_SIZE_MAX,
};
use crate::token::Token;
use crate::tokenizer::{Code, State, Tokenizer};

/// Kind of a character reference.
#[derive(Debug, Clone, PartialEq)]
pub enum Kind {
    /// Numeric decimal character reference.
    ///
    /// ```markdown
    /// > | a&#x9;b
    ///      ^^^^^
    /// ```
    Decimal,
    /// Numeric hexadecimal character reference.
    ///
    /// ```markdown
    /// > | a&#123;b
    ///      ^^^^^^
    /// ```
    Hexadecimal,
    /// Named character reference.
    ///
    /// ```markdown
    /// > | a&amp;b
    ///      ^^^^^
    /// ```
    Named,
}

impl Kind {
    /// Get the maximum size of characters allowed in the value of a character
    /// reference.
    fn max(&self) -> usize {
        match self {
            Kind::Hexadecimal => CHARACTER_REFERENCE_HEXADECIMAL_SIZE_MAX,
            Kind::Decimal => CHARACTER_REFERENCE_DECIMAL_SIZE_MAX,
            Kind::Named => CHARACTER_REFERENCE_NAMED_SIZE_MAX,
        }
    }

    /// Check if a char is allowed.
    fn allowed(&self, char: char) -> bool {
        let check = match self {
            Kind::Hexadecimal => char::is_ascii_hexdigit,
            Kind::Decimal => char::is_ascii_digit,
            Kind::Named => char::is_ascii_alphanumeric,
        };

        check(&char)
    }
}

/// State needed to parse character references.
#[derive(Debug, Clone)]
struct Info {
    /// All parsed characters.
    buffer: String,
    /// Kind of character reference.
    kind: Kind,
}

/// Start of a character reference.
///
/// ```markdown
/// > | a&amp;b
///      ^
/// > | a&#123;b
///      ^
/// > | a&#x9;b
///      ^
/// ```
pub fn start(tokenizer: &mut Tokenizer, code: Code) -> State {
    match code {
        Code::Char('&') if tokenizer.parse_state.constructs.character_reference => {
            tokenizer.enter(Token::CharacterReference);
            tokenizer.enter(Token::CharacterReferenceMarker);
            tokenizer.consume(code);
            tokenizer.exit(Token::CharacterReferenceMarker);
            State::Fn(Box::new(open))
        }
        _ => State::Nok,
    }
}

/// Inside a character reference, after `&`, before `#` for numeric references
/// or an alphanumeric for named references.
///
/// ```markdown
/// > | a&amp;b
///       ^
/// > | a&#123;b
///       ^
/// > | a&#x9;b
///       ^
/// ```
fn open(tokenizer: &mut Tokenizer, code: Code) -> State {
    let info = Info {
        buffer: String::new(),
        kind: Kind::Named,
    };
    if let Code::Char('#') = code {
        tokenizer.enter(Token::CharacterReferenceMarkerNumeric);
        tokenizer.consume(code);
        tokenizer.exit(Token::CharacterReferenceMarkerNumeric);
        State::Fn(Box::new(|t, c| numeric(t, c, info)))
    } else {
        tokenizer.enter(Token::CharacterReferenceValue);
        value(tokenizer, code, info)
    }
}

/// Inside a numeric character reference, right before `x` for hexadecimals,
/// or a digit for decimals.
///
/// ```markdown
/// > | a&#123;b
///        ^
/// > | a&#x9;b
///        ^
/// ```
fn numeric(tokenizer: &mut Tokenizer, code: Code, mut info: Info) -> State {
    if let Code::Char('x' | 'X') = code {
        tokenizer.enter(Token::CharacterReferenceMarkerHexadecimal);
        tokenizer.consume(code);
        tokenizer.exit(Token::CharacterReferenceMarkerHexadecimal);
        tokenizer.enter(Token::CharacterReferenceValue);
        info.kind = Kind::Hexadecimal;
        State::Fn(Box::new(|t, c| value(t, c, info)))
    } else {
        tokenizer.enter(Token::CharacterReferenceValue);
        info.kind = Kind::Decimal;
        value(tokenizer, code, info)
    }
}

/// Inside a character reference value, after the markers (`&#x`, `&#`, or
/// `&`) that define its kind, but before the `;`.
/// The character reference kind defines what and how many characters are
/// allowed.
///
/// ```markdown
/// > | a&amp;b
///       ^^^
/// > | a&#123;b
///        ^^^
/// > | a&#x9;b
///         ^
/// ```
fn value(tokenizer: &mut Tokenizer, code: Code, mut info: Info) -> State {
    match code {
        Code::Char(';') if !info.buffer.is_empty() => {
            let unknown_named = Kind::Named == info.kind
                && !CHARACTER_REFERENCES.iter().any(|d| d.0 == info.buffer);

            if unknown_named {
                State::Nok
            } else {
                tokenizer.exit(Token::CharacterReferenceValue);
                tokenizer.enter(Token::CharacterReferenceMarkerSemi);
                tokenizer.consume(code);
                tokenizer.exit(Token::CharacterReferenceMarkerSemi);
                tokenizer.exit(Token::CharacterReference);
                State::Ok(0)
            }
        }
        Code::Char(char) => {
            if info.buffer.len() < info.kind.max() && info.kind.allowed(char) {
                info.buffer.push(char);
                tokenizer.consume(code);
                State::Fn(Box::new(|t, c| value(t, c, info)))
            } else {
                State::Nok
            }
        }
        _ => State::Nok,
    }
}
