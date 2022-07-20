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
pub fn encode<S: Into<String>>(value: S) -> String {
    let mut value = value.into();

    // It’ll grow a bit bigger for each dangerous character.
    let mut result = String::with_capacity(value.len());

    while let Some(indice) = value.find(check) {
        let after = value.split_off(indice + 1);
        let dangerous = value.pop().unwrap();
        result.push_str(&value);
        result.push_str(match dangerous {
            '&' => "&amp;",
            '"' => "&quot;",
            '<' => "&lt;",
            '>' => "&gt;",
            _ => unreachable!("xxx"),
        });
        value = after;
    }

    result.push_str(&value);

    result
}

fn check(char: char) -> bool {
    matches!(char, '&' | '"' | '<' | '>')
}
