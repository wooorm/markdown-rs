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
use crate::event::Name;
use crate::state::{Name as StateName, State};
use crate::tokenizer::Tokenizer;
use crate::util::slice::Slice;

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
pub fn start(tokenizer: &mut Tokenizer) -> State {
    match tokenizer.current {
        Some(b'&') if tokenizer.parse_state.constructs.character_reference => {
            tokenizer.enter(Name::CharacterReference);
            tokenizer.enter(Name::CharacterReferenceMarker);
            tokenizer.consume();
            tokenizer.exit(Name::CharacterReferenceMarker);
            State::Next(StateName::CharacterReferenceOpen)
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
pub fn open(tokenizer: &mut Tokenizer) -> State {
    if let Some(b'#') = tokenizer.current {
        tokenizer.enter(Name::CharacterReferenceMarkerNumeric);
        tokenizer.consume();
        tokenizer.exit(Name::CharacterReferenceMarkerNumeric);
        State::Next(StateName::CharacterReferenceNumeric)
    } else {
        tokenizer.tokenize_state.marker = b'&';
        tokenizer.enter(Name::CharacterReferenceValue);
        State::Retry(StateName::CharacterReferenceValue)
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
pub fn numeric(tokenizer: &mut Tokenizer) -> State {
    if let Some(b'x' | b'X') = tokenizer.current {
        tokenizer.enter(Name::CharacterReferenceMarkerHexadecimal);
        tokenizer.consume();
        tokenizer.exit(Name::CharacterReferenceMarkerHexadecimal);
        tokenizer.enter(Name::CharacterReferenceValue);
        tokenizer.tokenize_state.marker = b'x';
        State::Next(StateName::CharacterReferenceValue)
    } else {
        tokenizer.enter(Name::CharacterReferenceValue);
        tokenizer.tokenize_state.marker = b'#';
        State::Retry(StateName::CharacterReferenceValue)
    }
}

/// Inside a character reference value, after the markers (`&#x`, `&#`, or
/// `&`) that define its kind, but before the `;`.
///
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
pub fn value(tokenizer: &mut Tokenizer) -> State {
    if matches!(tokenizer.current, Some(b';')) && tokenizer.tokenize_state.size > 0 {
        // Named.
        if tokenizer.tokenize_state.marker == b'&' {
            // Guaranteed to be valid ASCII bytes.
            let slice = Slice::from_indices(
                tokenizer.parse_state.bytes,
                tokenizer.point.index - tokenizer.tokenize_state.size,
                tokenizer.point.index,
            );
            let name = slice.as_str();

            if !CHARACTER_REFERENCES.iter().any(|d| d.0 == name) {
                tokenizer.tokenize_state.marker = 0;
                tokenizer.tokenize_state.size = 0;
                return State::Nok;
            }
        }

        tokenizer.exit(Name::CharacterReferenceValue);
        tokenizer.enter(Name::CharacterReferenceMarkerSemi);
        tokenizer.consume();
        tokenizer.exit(Name::CharacterReferenceMarkerSemi);
        tokenizer.exit(Name::CharacterReference);
        tokenizer.tokenize_state.marker = 0;
        tokenizer.tokenize_state.size = 0;
        return State::Ok;
    }

    let max = match tokenizer.tokenize_state.marker {
        b'&' => CHARACTER_REFERENCE_NAMED_SIZE_MAX,
        b'x' => CHARACTER_REFERENCE_HEXADECIMAL_SIZE_MAX,
        b'#' => CHARACTER_REFERENCE_DECIMAL_SIZE_MAX,
        _ => unreachable!("Unexpected marker `{}`", tokenizer.tokenize_state.marker),
    };
    let test = match tokenizer.tokenize_state.marker {
        b'&' => u8::is_ascii_alphanumeric,
        b'x' => u8::is_ascii_hexdigit,
        b'#' => u8::is_ascii_digit,
        _ => unreachable!("Unexpected marker `{}`", tokenizer.tokenize_state.marker),
    };

    if let Some(byte) = tokenizer.current {
        if tokenizer.tokenize_state.size < max && test(&byte) {
            tokenizer.tokenize_state.size += 1;
            tokenizer.consume();
            return State::Next(StateName::CharacterReferenceValue);
        }
    }

    tokenizer.tokenize_state.marker = 0;
    tokenizer.tokenize_state.size = 0;
    State::Nok
}
