//! Turn a hast (HTML syntax tree) into a string of HTML.
//!
//! JS equivalent: <https://github.com/syntax-tree/hast-util-to-html>.
//!
//! The output style matches [`markdown::to_html`] (void elements end in
//! ` />`, and `&`, `<`, `>`, `"` are encoded), so both paths can be compared.

#![no_std]
#![forbid(unsafe_code)]

extern crate alloc;

use alloc::{borrow::Cow, string::String};
use core::fmt::Write;
use markdown::{SAFE_PROTOCOL_HREF, SAFE_PROTOCOL_SRC};

/// HTML void elements: they have no closing tag.
const VOIDS: [&str; 13] = [
    "area", "base", "br", "col", "embed", "hr", "img", "input", "link", "meta", "source", "track",
    "wbr",
];

/// Configuration.
///
/// The defaults encode raw HTML, and empty `href` and `src` values whose
/// protocol `markdown::to_html` would not allow either. They do not sanitize
/// a tree: tag names, event handler properties, URLs in other properties,
/// and text in `script` and `style` are written as given.
#[derive(Clone, Debug, Default)]
pub struct Options {
    /// Whether to write raw HTML as is, instead of encoding it as text
    /// (default: `false`).
    pub allow_dangerous_html: bool,
    /// Whether to allow any protocol in `href` and `src` (default: `false`).
    ///
    /// Otherwise, `href` allows `http`, `https`, `irc`, `ircs`, `mailto`, and
    /// `xmpp`, and `src` allows `http` and `https`, as `markdown::to_html`
    /// does.
    pub allow_dangerous_protocol: bool,
}

/// Turn a hast tree into HTML.
pub fn to_html(tree: &hast::Node) -> String {
    to_html_with_options(tree, &Options::default())
}

/// Turn a hast tree, with options, into HTML.
///
/// URLs are expected to be percent-encoded already: this only encodes them
/// for HTML.
pub fn to_html_with_options(tree: &hast::Node, options: &Options) -> String {
    let mut out = String::new();
    one(tree, &mut out, options);
    out
}

/// Serialize one node into `out`.
fn one(node: &hast::Node, out: &mut String, options: &Options) {
    match node {
        hast::Node::Root(root) => {
            for child in &root.children {
                one(child, out, options);
            }
        }
        hast::Node::Element(element) => {
            out.push('<');
            out.push_str(&element.tag_name);
            for (name, value) in &element.properties {
                let value: Cow<str> = match value {
                    hast::PropertyValue::Boolean(false) => continue,
                    hast::PropertyValue::Boolean(true) => "".into(),
                    hast::PropertyValue::String(x) => x.as_str().into(),
                    hast::PropertyValue::CommaSeparated(x) => x.join(", ").into(),
                    hast::PropertyValue::SpaceSeparated(x) => x.join(" ").into(),
                };
                out.push(' ');
                attribute_name(name, out);
                out.push_str("=\"");
                // Browsers read attribute names without regard to case.
                let protocols: Option<&[&str]> = if options.allow_dangerous_protocol {
                    None
                } else if name.eq_ignore_ascii_case("href") {
                    Some(&SAFE_PROTOCOL_HREF)
                } else if name.eq_ignore_ascii_case("src") {
                    Some(&SAFE_PROTOCOL_SRC)
                } else {
                    None
                };
                let is_allowed = match protocols {
                    Some(protocols) => has_safe_protocol(&value, protocols),
                    None => true,
                };
                if is_allowed {
                    encode(&value, out);
                }
                out.push('"');
            }
            if VOIDS.contains(&element.tag_name.as_ref()) {
                out.push_str(" />");
                return;
            }
            out.push('>');
            // Like `hast-util-to-html`: script and style hold raw text.
            let is_raw_text = matches!(element.tag_name.as_ref(), "script" | "style");
            for child in &element.children {
                match child {
                    hast::Node::Text(text) if is_raw_text => out.push_str(&text.value),
                    _ => one(child, out, options),
                }
            }
            out.push_str("</");
            out.push_str(&element.tag_name);
            out.push('>');
        }
        hast::Node::Text(text) => encode(&text.value, out),
        hast::Node::Raw(raw) => {
            if options.allow_dangerous_html {
                out.push_str(&raw.value.replace('\0', "\u{FFFD}"));
            } else {
                encode(&raw.value, out);
            }
        }
        hast::Node::Comment(comment) => {
            out.push_str("<!--");
            comment_value(&comment.value, out);
            out.push_str("-->");
        }
        hast::Node::Doctype(_) => out.push_str("<!doctype html>"),
    }
}

/// Write a comment value that cannot end the comment early.
fn comment_value(value: &str, out: &mut String) {
    let value = value
        .replace("<!--", "&lt;!--")
        .replace("-->", "--&gt;")
        .replace("--!>", "--!&gt;");
    let value = if let Some(rest) = value.strip_prefix("->") {
        ["-&gt;", rest].concat()
    } else if let Some(rest) = value.strip_prefix('>') {
        ["&gt;", rest].concat()
    } else {
        value
    };
    if let Some(start) = value.strip_suffix("<!-") {
        out.push_str(start);
        out.push_str("&lt;!-");
    } else {
        out.push_str(&value);
    }
}

/// Write the HTML attribute name of a hast property name.
///
/// Maps `className`, `htmlFor`, `httpEquiv`, `acceptCharset`, `ariaX`, and
/// `dataX` (a subset of `property-information`), and writes other names as
/// given, with characters that cannot occur in an attribute name escaped.
fn attribute_name(name: &str, out: &mut String) {
    let is_upper_after = |prefix: &str| {
        matches!(
            name.strip_prefix(prefix).and_then(|rest| rest.chars().next()),
            Some(char) if char.is_ascii_uppercase()
        )
    };
    let name: Cow<str> = match name {
        "className" => "class".into(),
        "htmlFor" => "for".into(),
        "httpEquiv" => "http-equiv".into(),
        "acceptCharset" => "accept-charset".into(),
        _ if is_upper_after("aria") => ["aria-", &name[4..].to_ascii_lowercase()].concat().into(),
        _ if is_upper_after("data") => {
            let mut result = String::from("data");
            for char in name[4..].chars() {
                if char.is_ascii_uppercase() {
                    result.push('-');
                }
                result.push(char.to_ascii_lowercase());
            }
            result.into()
        }
        _ => name.into(),
    };
    for char in name.chars() {
        if matches!(
            char,
            '\0' | '\t' | '\n' | '\x0C' | '\r' | ' ' | '"' | '\'' | '&' | '/' | '<' | '=' | '>'
        ) {
            // Writing to a `String` cannot fail.
            let _ = write!(out, "&#x{:X};", u32::from(char));
        } else {
            out.push(char);
        }
    }
}

/// Encode `&`, `<`, `>`, `"`, and NUL, like `markdown::to_html` does.
fn encode(value: &str, out: &mut String) {
    let mut start = 0;
    for (index, byte) in value.bytes().enumerate() {
        let replacement = match byte {
            b'\0' => "\u{FFFD}",
            b'&' => "&amp;",
            b'<' => "&lt;",
            b'>' => "&gt;",
            b'"' => "&quot;",
            _ => continue,
        };
        out.push_str(&value[start..index]);
        out.push_str(replacement);
        start = index + 1;
    }
    out.push_str(&value[start..]);
}

/// Whether a URL has no protocol, or one in `protocols`: the protocol check
/// of `markdown::to_html`, on a URL that is not normalized again.
fn has_safe_protocol(value: &str, protocols: &[&str]) -> bool {
    let end = value.find(&['?', '#', '/'][..]).unwrap_or(value.len());
    match value.find(':') {
        Some(colon) if colon < end => {
            let protocol = &value[..colon];
            protocols
                .iter()
                .any(|known| known.eq_ignore_ascii_case(protocol))
        }
        _ => true,
    }
}
