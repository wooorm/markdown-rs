//! Some utilities helpful when parsing and compiling markdown.

use crate::constant::{CHARACTER_REFERENCE_NAMES, CHARACTER_REFERENCE_VALUES};
use crate::tokenizer::{Code, Event, EventType};

/// Encode dangerous html characters.
///
/// This ensures that certain characters which have special meaning in HTML are
/// dealt with.
/// Technically, we can skip `>` and `"` in many cases, but CM includes them.
///
/// This behavior is not explained in prose in `CommonMark` but can be inferred
/// from the input/output test cases.
///
/// ## Examples
///
/// ```rust ignore
/// use micromark::util::encode;
///
/// assert_eq!(encode("I <3 🦀"), "I &lt;3 🦀");
/// ```
///
/// ## References
///
/// *   [`micromark-util-encode` in `micromark`](https://github.com/micromark/micromark/tree/main/packages/micromark-util-encode)
pub fn encode(value: &str) -> String {
    value
        .replace('&', "&amp;")
        .replace('"', "&quot;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
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
/// use micromark::util::decode_numeric_character_reference;
///
/// assert_eq!(decode_numeric_character_reference("123", 10), '{');
/// assert_eq!(decode_numeric_character_reference("9", 16), '\t');
/// assert_eq!(decode_numeric_character_reference("0", 10), '�'); // Not allowed.
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
pub fn decode_numeric_character_reference(value: &str, radix: u32) -> char {
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
        '�'
    } else {
        char::from_u32(code).expect("expected valid `code`")
    }
}

/// Decode named character references.
///
/// Turn the name coming from a named character reference (without the `&` or
/// `;`) into a string.
/// This looks the given string up in [`CHARACTER_REFERENCE_NAMES`][] and then
/// takes the corresponding value from [`CHARACTER_REFERENCE_VALUES`][].
///
/// The result is `String` instead of `char` because named character references
/// can expand into multiple characters.
///
/// ## Examples
///
/// ```rust ignore
/// use micromark::util::decode_named_character_reference;
///
/// assert_eq!(decode_named_character_reference("amp"), "&");
/// assert_eq!(decode_named_character_reference("AElig"), "Æ");
/// assert_eq!(decode_named_character_reference("aelig"), "æ");
/// ```
///
/// ## Panics
///
/// This function panics if a name not in [`CHARACTER_REFERENCE_NAMES`][] is
/// given.
/// It is expected that figuring out whether a name is allowed is handled in
/// the parser.
/// When `micromark` is used, this function never panics.
///
/// ## References
///
/// *   [`wooorm/decode-named-character-reference`](https://github.com/wooorm/decode-named-character-reference)
/// *   [*§ 2.5 Entity and numeric character references* in `CommonMark`](https://spec.commonmark.org/0.30/#entity-and-numeric-character-references)
pub fn decode_named_character_reference(value: &str) -> String {
    let position = CHARACTER_REFERENCE_NAMES.iter().position(|&x| x == value);
    if let Some(index) = position {
        CHARACTER_REFERENCE_VALUES[index].to_string()
    } else {
        unreachable!("expected valid `name`")
    }
}

/// A struct representing the span of an opening and closing event of a token.
#[derive(Debug)]
pub struct Span {
    // To do: probably needed in the future.
    // start: Point,
    /// Absolute offset (and `index` in `codes`) of where this span starts.
    pub start_index: usize,
    // To do: probably needed in the future.
    // end: Point,
    /// Absolute offset (and `index` in `codes`) of where this span ends.
    pub end_index: usize,
    // To do: probably needed in the future.
    // token_type: TokenType,
}

/// Get a span from an event.
///
/// Get the span of an `exit` event, by looking backwards through the events to
/// find the corresponding `enter` event.
/// This assumes that tokens with the same are not nested.
///
/// ## Panics
///
/// This function panics if an enter event is given.
/// When `micromark` is used, this function never panics.
pub fn get_span(events: &[Event], index: usize) -> Span {
    let exit = &events[index];
    // let end = exit.point.clone();
    let end_index = exit.index;
    let token_type = exit.token_type.clone();
    // To do: support `enter` events if needed and walk forwards?
    assert_eq!(
        exit.event_type,
        EventType::Exit,
        "expected get_span to be called on `exit` event"
    );
    let mut start_index = index - 1;

    loop {
        let enter = &events[start_index];
        if enter.event_type == EventType::Enter && enter.token_type == token_type {
            return Span {
                // start: enter.point.clone(),
                start_index: enter.index,
                // end,
                end_index,
                // token_type,
            };
        }

        start_index -= 1;
    }
}

/// Serialize a span, optionally expanding tabs.
pub fn slice_serialize(codes: &[Code], span: &Span, expand_tabs: bool) -> String {
    serialize_chunks(slice_codes(codes, span), expand_tabs)
}

/// Get a slice of codes from a span.
pub fn slice_codes<'a>(codes: &'a [Code], span: &Span) -> &'a [Code] {
    &codes[span.start_index..span.end_index]
}

/// Serialize a slice of codes, optionally expanding tabs.
pub fn serialize_chunks(codes: &[Code], expand_tabs: bool) -> String {
    let mut at_tab = false;
    let mut index = 0;
    let mut value: Vec<char> = vec![];

    while index < codes.len() {
        let code = codes[index];
        let mut at_tab_next = false;

        match code {
            Code::CarriageReturnLineFeed => {
                value.push('\r');
                value.push('\n');
            }
            Code::Char(char) if char == '\n' || char == '\r' => {
                value.push(char);
            }
            Code::Char(char) if char == '\t' => {
                at_tab_next = true;
                value.push(if expand_tabs { ' ' } else { char });
            }
            Code::VirtualSpace => {
                if !expand_tabs && at_tab {
                    index += 1;
                    continue;
                }
                value.push(' ');
            }
            Code::Char(char) => {
                value.push(char);
            }
            Code::None => {
                unreachable!("unexpected EOF code in codes");
            }
        }

        at_tab = at_tab_next;

        index += 1;
    }

    value.into_iter().collect()
}
