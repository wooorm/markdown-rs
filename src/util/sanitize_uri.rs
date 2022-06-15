//! Utilities to make urls safe.

use crate::util::encode::encode;

/// Make a value safe for injection as a URL.
///
/// This encodes unsafe characters with percent-encoding and skips already
/// encoded sequences (see [`normalize_uri`][] below).
/// Further unsafe characters are encoded as character references (see
/// [`encode`][]).
///
/// Then, a vec of (lowercase) allowed protocols can be given, in which case
/// the URL is sanitized.
///
/// For example, `Some(vec!["http", "https", "irc", "ircs", "mailto", "xmpp"])`
/// can be used for `a[href]`, or `Some(vec!["http", "https"])` for `img[src]`.
/// If the URL includes an unknown protocol (one not matched by `protocol`, such
/// as a dangerous example, `javascript:`), the value is ignored.
///
/// ## Examples
///
/// ```rust ignore
/// use micromark::util::sanitize_url::sanitize_url;
///
/// assert_eq!(sanitize_uri("javascript:alert(1)", &None), "javascript:alert(1)");
/// assert_eq!(sanitize_uri("javascript:alert(1)", &Some(vec!["http", "https"])), "");
/// assert_eq!(sanitize_uri("https://example.com", &Some(vec!["http", "https"])), "https://example.com");
/// assert_eq!(sanitize_uri("https://a👍b.c/%20/%", &Some(vec!["http", "https"])), "https://a%F0%9F%91%8Db.c/%20/%25");
/// ```
///
/// ## References
///
/// *   [`micromark-util-sanitize-uri` in `micromark`](https://github.com/micromark/micromark/tree/main/packages/micromark-util-sanitize-uri)
pub fn sanitize_uri(value: &str, protocols: &Option<Vec<&str>>) -> String {
    let value = encode(&normalize_uri(value));

    if let Some(protocols) = protocols {
        let chars: Vec<char> = value.chars().collect();
        let mut index = 0;
        let mut colon: Option<usize> = None;

        while index < chars.len() {
            let char = chars[index];

            match char {
                ':' => {
                    colon = Some(index);
                    break;
                }
                '?' | '#' | '/' => break,
                _ => {}
            }

            index += 1;
        }

        // If there is no protocol, or the first colon is after `?`, `#`, or `/`, it’s relative.
        // It is a protocol, it should be allowed.
        if let Some(colon) = colon {
            let protocol = chars[0..colon].iter().collect::<String>().to_lowercase();
            if !protocols.contains(&protocol.as_str()) {
                return "".to_string();
            }
        }
    }

    value
}

/// Normalize a URL (such as used in definitions).
///
/// Encode unsafe characters with percent-encoding, skipping already encoded
/// sequences.
///
/// ## Examples
///
/// ```rust ignore
/// use micromark::util::sanitize_url::normalize_uri;
///
/// assert_eq!(sanitize_uri("https://example.com"), "https://example.com");
/// assert_eq!(sanitize_uri("https://a👍b.c/%20/%"), "https://a%F0%9F%91%8Db.c/%20/%25");
/// ```
///
/// ## References
///
/// *   [`micromark-util-sanitize-uri` in `micromark`](https://github.com/micromark/micromark/tree/main/packages/micromark-util-sanitize-uri)
fn normalize_uri(value: &str) -> String {
    let chars: Vec<char> = value.chars().collect();
    let mut result: Vec<String> = vec![];
    let mut index = 0;
    let mut start = 0;
    let mut buff = [0; 4];

    while index < chars.len() {
        let char = chars[index];

        // A correct percent encoded value.
        if char == '%'
            && index + 2 < chars.len()
            && chars[index + 1].is_ascii_alphanumeric()
            && chars[index + 2].is_ascii_alphanumeric()
        {
            index += 3;
            continue;
        }

        // Note: Rust already takes care of lone astral surrogates.
        // Non-ascii or not allowed ascii.
        if char >= '\u{0080}'
            || !matches!(char, '!' | '#' | '$' | '&'..=';' | '=' | '?'..='Z' | '_' | 'a'..='z' | '~')
        {
            result.push(chars[start..index].iter().collect::<String>());

            char.encode_utf8(&mut buff);
            result.push(
                buff[0..char.len_utf8()]
                    .iter()
                    .map(|&byte| format!("%{:X}", byte))
                    .collect::<String>(),
            );

            start = index + 1;
        }

        index += 1;
    }

    result.push(chars[start..].iter().collect::<String>());

    result.join("")
}
