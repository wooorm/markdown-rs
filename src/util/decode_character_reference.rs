//! Decode character references.

use crate::constant::CHARACTER_REFERENCES;
use alloc::string::{String, ToString};

/// Decode named character references.
///
/// Turn the name coming from a named character reference (without the `&` or
/// `;`) into a string.
/// This looks the given string up at `0` in the tuples of
/// [`CHARACTER_REFERENCES`][] and then takes the corresponding value from `1`.
///
/// The result is `String` instead of `char` because named character references
/// can expand into multiple characters.
///
/// ## Examples
///
/// ```rust ignore
/// use micromark::util::decode_character_reference::decode_named;
///
/// assert_eq!(decode_named("amp"), "&");
/// assert_eq!(decode_named("AElig"), "Æ");
/// assert_eq!(decode_named("aelig"), "æ");
/// ```
///
/// ## Panics
///
/// This function panics if a name not in [`CHARACTER_REFERENCES`][] is
/// given.
/// It is expected that figuring out whether a name is allowed is handled in
/// the parser.
/// When `micromark` is used, this function never panics.
///
/// ## References
///
/// *   [`wooorm/decode-named-character-reference`](https://github.com/wooorm/decode-named-character-reference)
/// *   [*§ 2.5 Entity and numeric character references* in `CommonMark`](https://spec.commonmark.org/0.30/#entity-and-numeric-character-references)
pub fn decode_named(value: &str) -> String {
    let entry = CHARACTER_REFERENCES.iter().find(|d| d.0 == value);
    let tuple = entry.expect("expected valid `name`");
    tuple.1.to_string()
}

/// Decode numeric character references.
///
/// Turn the number (in string form as either hexadecimal or decimal) coming
/// from a numeric character reference into a string.
/// The base of the string form must be passed as the `radix` parameter, as
/// `10` (decimal) or `16` (hexadecimal).
///
/// This returns a `String` form of the associated character or a replacement
/// character for C0 control characters (except for ASCII whitespace), C1
/// control characters, lone surrogates, noncharacters, and out of range
/// characters.
///
/// ## Examples
///
/// ```rust ignore
/// use micromark::util::decode_character_reference::decode_numeric;
///
/// assert_eq!(decode_numeric("123", 10), "{");
/// assert_eq!(decode_numeric("9", 16), "\t");
/// assert_eq!(decode_numeric("0", 10), "�"); // Not allowed.
/// ```
///
/// ## Panics
///
/// This function panics if a invalid string or an out of bounds valid string
/// is given.
/// It is expected that figuring out whether a number is allowed is handled in
/// the parser.
/// When `micromark` is used, this function never panics.
///
/// ## References
///
/// *   [`micromark-util-decode-numeric-character-reference` in `micromark`](https://github.com/micromark/micromark/tree/main/packages/micromark-util-decode-numeric-character-reference)
/// *   [*§ 2.5 Entity and numeric character references* in `CommonMark`](https://spec.commonmark.org/0.30/#entity-and-numeric-character-references)
pub fn decode_numeric(value: &str, radix: u32) -> String {
    if let Some(char) = char::from_u32(u32::from_str_radix(value, radix).unwrap()) {
        if !matches!(char,
            // C0 except for HT, LF, FF, CR, space
            '\0'..='\u{08}' | '\u{0B}' | '\u{0E}'..='\u{1F}' |
            // Control character (DEL) of c0, and C1 controls.
            '\u{7F}'..='\u{9F}'
            // Lone surrogates, noncharacters, and out of range are handled by
            // Rust.
        ) {
            return char.to_string();
        }
    }

    char::REPLACEMENT_CHARACTER.to_string()
}
