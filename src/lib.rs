//! Public API of `markdown-rs`.
//!
//! This module exposes primarily [`to_html()`][].
//! It also exposes [`to_html_with_options()`][] and [`to_mdast()`][].
//!
//! *   [`to_html()`][]
//!     — safe way to transform (untrusted?) markdown into HTML
//! *   [`to_html_with_options()`][]
//!     — like `to_html` but lets you configure how markdown is turned into
//!     HTML, such as allowing dangerous HTML or turning on/off different
//!     constructs (GFM, MDX, and the like)
//! *   [`to_mdast()`][]
//!     — turn markdown into a syntax tree
#![no_std]
#![deny(clippy::pedantic)]
#![allow(clippy::doc_link_with_quotes)]
#![allow(clippy::missing_panics_doc)]
#![allow(clippy::must_use_candidate)]
#![allow(clippy::too_many_lines)]
#![doc(
    html_logo_url = "https://raw.githubusercontent.com/wooorm/markdown-rs/8924580/media/logo-monochromatic.svg?sanitize=true"
)]

extern crate alloc;
mod configuration;
mod construct;
mod event;
mod parser;
mod resolve;
mod state;
mod subtokenize;
mod to_html;
mod to_mdast;
mod tokenizer;
mod util;

pub mod mdast; // To do: externalize?
pub mod unist; // To do: externalize.

#[doc(hidden)]
pub use util::identifier::{id_cont, id_start};

#[doc(hidden)]
pub use util::sanitize_uri::sanitize;

#[doc(hidden)]
pub use util::location::Location;

pub use util::line_ending::LineEnding;

pub use util::mdx::{
    EsmParse as MdxEsmParse, ExpressionKind as MdxExpressionKind,
    ExpressionParse as MdxExpressionParse, Signal as MdxSignal,
};

pub use configuration::{CompileOptions, Constructs, Options, ParseOptions};

use alloc::string::String;

/// Turn markdown into HTML.
///
/// Compiles markdown to HTML according to `CommonMark`.
/// Use [`to_html_with_options()`][] to configure how markdown is turned into
/// HTML.
///
/// ## Examples
///
/// ```
/// use markdown::to_html;
///
/// assert_eq!(to_html("# Hello, world!"), "<h1>Hello, world!</h1>");
/// ```
pub fn to_html(value: &str) -> String {
    to_html_with_options(value, &Options::default()).unwrap()
}

/// Turn markdown into HTML, with configuration.
///
/// ## Errors
///
/// `to_html_with_options()` never errors with normal markdown because markdown
/// does not have syntax errors, so feel free to `unwrap()`.
/// However, MDX does have syntax errors.
/// When MDX is turned on, there are several errors that can occur with how
/// expressions, ESM, and JSX are written.
///
/// ## Examples
///
/// ```
/// use markdown::{to_html_with_options, CompileOptions, Options};
/// # fn main() -> Result<(), String> {
///
/// // Use GFM:
/// let result = to_html_with_options("~hi~hello!", &Options::gfm())?;
///
/// assert_eq!(result, "<p><del>hi</del>hello!</p>");
///
/// // Live dangerously / trust the author:
/// let result = to_html_with_options("<div>\n\n# Hello, world!\n\n</div>", &Options {
///     compile: CompileOptions {
///       allow_dangerous_html: true,
///       allow_dangerous_protocol: true,
///       ..CompileOptions::default()
///     },
///     ..Options::default()
/// })?;
///
/// assert_eq!(result, "<div>\n<h1>Hello, world!</h1>\n</div>");
/// # Ok(())
/// # }
/// ```
pub fn to_html_with_options(value: &str, options: &Options) -> Result<String, String> {
    let (events, parse_state) = parser::parse(value, &options.parse)?;
    Ok(to_html::compile(
        &events,
        parse_state.bytes,
        &options.compile,
    ))
}

/// Turn markdown into a syntax tree.
///
/// ## Errors
///
/// `to_mdast()` never errors with normal markdown because markdown does not
/// have syntax errors, so feel free to `unwrap()`.
/// However, MDX does have syntax errors.
/// When MDX is turned on, there are several errors that can occur with how
/// JSX, expressions, or ESM are written.
///
/// ## Examples
///
/// ```
/// use markdown::{to_mdast, ParseOptions};
/// # fn main() -> Result<(), String> {
///
/// let tree = to_mdast("# Hey, *you*!", &ParseOptions::default())?;
///
/// println!("{:?}", tree);
/// // => Root { children: [Heading { children: [Text { value: "Hey, ", position: Some(1:3-1:8 (2-7)) }, Emphasis { children: [Text { value: "you", position: Some(1:9-1:12 (8-11)) }], position: Some(1:8-1:13 (7-12)) }, Text { value: "!", position: Some(1:13-1:14 (12-13)) }], position: Some(1:1-1:14 (0-13)), depth: 1 }], position: Some(1:1-1:14 (0-13)) }
/// # Ok(())
/// # }
/// ```
pub fn to_mdast(value: &str, options: &ParseOptions) -> Result<mdast::Node, String> {
    let (events, parse_state) = parser::parse(value, options)?;
    let node = to_mdast::compile(&events, parse_state.bytes)?;
    Ok(node)
}

/// Turn markdown into a JSON representation of its syntax tree.
///
/// ### Errors
///
/// `to_json()` errors under the same conditions as [`to_mdast`]. Additionally, an error may be
/// returned if the syntax tree cannot be serialized to JSON.
#[cfg(feature = "json")]
pub fn to_json(value: &str, options: &ParseOptions) -> Result<serde_json::Value, String> {
    let (events, parse_state) = parser::parse(value, options)?;
    let node = to_mdast::compile(&events, parse_state.bytes)?;

    let value = serde_json::to_value(&node).map_err(|_| "Failed to serialize mdast to json")?;

    Ok(value)
}
