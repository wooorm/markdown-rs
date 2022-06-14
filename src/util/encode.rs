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
    value
        .replace('&', "&amp;")
        .replace('"', "&quot;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
}
