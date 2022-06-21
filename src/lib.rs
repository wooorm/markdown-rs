//! Public API of micromark.
//!
//! This module exposes [`micromark`][] (and [`micromark_with_options`][]).
//! `micromark` is a safe way to transform (untrusted?) markdown into HTML.
//! `micromark_with_options` allows you to configure how markdown is turned into
//! HTML, such as by allowing dangerous HTML when you trust it.
mod compiler;
mod constant;
mod construct;
mod content;
mod parser;
mod subtokenize;
mod tokenizer;
mod util;

use crate::compiler::compile;
pub use crate::compiler::{LineEnding, Options};
use crate::parser::parse;

/// Turn markdown into HTML.
///
/// ## Examples
///
/// ```rust
/// use micromark::micromark;
///
/// let result = micromark("# Hello, world!");
///
/// assert_eq!(result, "<h1>Hello, world!</h1>");
/// ```
#[must_use]
pub fn micromark(value: &str) -> String {
    micromark_with_options(value, &Options::default())
}

/// Turn markdown into HTML, with configuration.
///
/// ## Examples
///
/// ```rust
/// use micromark::{micromark_with_options, Options};
///
/// let result = micromark_with_options("<div>\n\n# Hello, world!\n\n</div>", &Options {
///     allow_dangerous_html: true,
///     allow_dangerous_protocol: true,
///     default_line_ending: None,
/// });
///
/// assert_eq!(result, "<div>\n<h1>Hello, world!</h1>\n</div>");
/// ```
#[must_use]
pub fn micromark_with_options(value: &str, options: &Options) -> String {
    let (events, codes) = parse(value);
    compile(&events, &codes, options)
}
