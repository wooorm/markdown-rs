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
mod token;
mod tokenizer;
mod unicode;
mod util;

use crate::compiler::compile;
use crate::parser::parse;
use crate::tokenizer::Code;

/// Type of line endings in markdown.
#[derive(Debug, Clone, PartialEq)]
pub enum LineEnding {
    /// Both a carriage return (`\r`) and a line feed (`\n`).
    ///
    /// ## Example
    ///
    /// ```markdown
    /// a␍␊
    /// b
    /// ```
    CarriageReturnLineFeed,
    /// Sole carriage return (`\r`).
    ///
    /// ## Example
    ///
    /// ```markdown
    /// a␍
    /// b
    /// ```
    CarriageReturn,
    /// Sole line feed (`\n`).
    ///
    /// ## Example
    ///
    /// ```markdown
    /// a␊
    /// b
    /// ```
    LineFeed,
}

impl LineEnding {
    /// Turn the line ending into a [str].
    fn as_str(&self) -> &str {
        match self {
            LineEnding::CarriageReturnLineFeed => "\r\n",
            LineEnding::CarriageReturn => "\r",
            LineEnding::LineFeed => "\n",
        }
    }
    /// Turn a [Code] into a line ending.
    ///
    /// ## Panics
    ///
    /// Panics if `code` is not `\r\n`, `\r`, or `\n`.
    fn from_code(code: Code) -> LineEnding {
        match code {
            Code::CarriageReturnLineFeed => LineEnding::CarriageReturnLineFeed,
            Code::Char('\r') => LineEnding::CarriageReturn,
            Code::Char('\n') => LineEnding::LineFeed,
            _ => unreachable!("invalid code"),
        }
    }
}

/// Configuration (optional).
#[derive(Default, Debug)]
pub struct Options {
    /// Whether to allow (dangerous) HTML.
    /// The default is `false`, you can turn it on to `true` for trusted
    /// content.
    ///
    /// ## Examples
    ///
    /// ```rust
    /// use micromark::{micromark, micromark_with_options, Options};
    ///
    /// // micromark is safe by default:
    /// assert_eq!(
    ///     micromark("Hi, <i>venus</i>!"),
    ///     "<p>Hi, &lt;i&gt;venus&lt;/i&gt;!</p>"
    /// );
    ///
    /// // Turn `allow_dangerous_html` on to allow potentially dangerous HTML:
    /// assert_eq!(
    ///     micromark_with_options(
    ///         "Hi, <i>venus</i>!",
    ///         &Options {
    ///             allow_dangerous_html: true,
    ///             allow_dangerous_protocol: false,
    ///             default_line_ending: None,
    ///         }
    ///     ),
    ///     "<p>Hi, <i>venus</i>!</p>"
    /// );
    /// ```
    pub allow_dangerous_html: bool,

    /// Whether to allow (dangerous) protocols in links and images.
    /// The default is `false`, you can turn it on to `true` for trusted
    /// content.
    ///
    /// ## Examples
    ///
    /// ```rust
    /// use micromark::{micromark, micromark_with_options, Options};
    ///
    /// // micromark is safe by default:
    /// assert_eq!(
    ///     micromark("<javascript:alert(1)>"),
    ///     "<p><a href=\"\">javascript:alert(1)</a></p>"
    /// );
    ///
    /// // Turn `allow_dangerous_protocol` on to allow potentially dangerous protocols:
    /// assert_eq!(
    ///     micromark_with_options(
    ///         "<javascript:alert(1)>",
    ///         &Options {
    ///             allow_dangerous_html: false,
    ///             allow_dangerous_protocol: true,
    ///             default_line_ending: None,
    ///         }
    ///     ),
    ///     "<p><a href=\"javascript:alert(1)\">javascript:alert(1)</a></p>"
    /// );
    /// ```
    pub allow_dangerous_protocol: bool,

    /// Default line ending to use, for line endings not in `value`.
    ///
    /// Generally, micromark copies line endings (`\r`, `\n`, `\r\n`) in the
    /// markdown document over to the compiled HTML.
    /// In some cases, such as `> a`, CommonMark requires that extra line
    /// endings are added: `<blockquote>\n<p>a</p>\n</blockquote>`.
    ///
    /// To create that line ending, the document is checked for the first line
    /// ending that is used.
    /// If there is no line ending, `default_line_ending` is used.
    /// If that isn’t configured, `\n` is used.
    ///
    /// ## Examples
    ///
    /// ```rust
    /// use micromark::{micromark, micromark_with_options, Options, LineEnding};
    ///
    /// // micromark uses `\n` by default:
    /// assert_eq!(
    ///     micromark("> a"),
    ///     "<blockquote>\n<p>a</p>\n</blockquote>"
    /// );
    ///
    /// // Define `default_line_ending` to configure the default:
    /// assert_eq!(
    ///     micromark_with_options(
    ///         "> a",
    ///         &Options {
    ///             allow_dangerous_html: false,
    ///             allow_dangerous_protocol: false,
    ///             default_line_ending: Some(LineEnding::CarriageReturnLineFeed),
    ///         }
    ///     ),
    ///     "<blockquote>\r\n<p>a</p>\r\n</blockquote>"
    /// );
    /// ```
    pub default_line_ending: Option<LineEnding>,
}

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
    let (events, result) = parse(value);
    compile(&events, &result.codes, options)
}
