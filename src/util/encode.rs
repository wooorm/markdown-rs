//! Utilities to encode HTML.

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
    let mut result: Vec<&str> = vec![];
    let mut start = 0;
    let mut index = 0;

    for byte in value.bytes() {
        if let Some(replacement) = match byte {
            b'&' => Some("&amp;"),
            b'"' => Some("&quot;"),
            b'<' => Some("&lt;"),
            b'>' => Some("&gt;"),
            _ => None,
        } {
            if start != index {
                result.push(&value[start..index]);
            }

            result.push(replacement);
            start = index + 1;
        }

        index += 1;
    }

    if start == 0 {
        value.to_string()
    } else {
        if start < index {
            result.push(&value[start..index]);
        }

        result.join("")
    }
}
