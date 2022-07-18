//! Utilities to decode character references.

use crate::constant::CHARACTER_REFERENCES;

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
/// from a numeric character reference into a character.
/// Whether the base of the string form is `10` (decimal) or `16` (hexadecimal)
/// must be passed as the `radix` parameter.
///
/// This returns the `char` associated with that number or a replacement
/// character for C0 control characters (except for ASCII whitespace), C1
/// control characters, lone surrogates, noncharacters, and out of range
/// characters.
///
/// ## Examples
///
/// ```rust ignore
/// use micromark::util::decode_character_reference::decode_numeric;
///
/// assert_eq!(decode_numeric("123", 10), '{');
/// assert_eq!(decode_numeric("9", 16), '\t');
/// assert_eq!(decode_numeric("0", 10), '�'); // Not allowed.
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
pub fn decode_numeric(value: &str, radix: u32) -> char {
    let code = u32::from_str_radix(value, radix).expect("expected `value` to be an int");

    if
    // C0 except for HT, LF, FF, CR, space
    code < 0x09 ||
    code == 0x0B ||
    (code > 0x0D && code < 0x20) ||
    // Control character (DEL) of the basic block and C1 controls.
    (code > 0x7E && code < 0xA0) ||
    // Lone high surrogates and low surrogates.
    (code > 0xd7ff && code < 0xe000) ||
    // Noncharacters.
    (code > 0xfdcf && code < 0xfdf0) ||
    ((code & 0xffff) == 0xffff) ||
    ((code & 0xffff) == 0xfffe) ||
    // Out of range
    code > 0x0010_ffff
    {
        char::REPLACEMENT_CHARACTER
    } else {
        char::from_u32(code).expect("expected valid `code`")
    }
}
