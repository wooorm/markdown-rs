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
#![allow(clippy::too_many_lines)]
// Would be nice to use `logo-chromatic`, but that looks horrible on Safari at relatively small sizes currently. 😢
#![doc(
    html_logo_url = "https://raw.githubusercontent.com/wooorm/markdown-rs/8924580/media/logo-monochromatic.svg?sanitize=true"
)]

extern crate alloc;

mod construct;
mod event;
pub mod mdast; // To do: externalize?
mod parser;
mod resolve;
mod state;
mod subtokenize;
mod to_html;
mod to_mdast;
mod tokenizer;
pub mod unist; // To do: externalize.
mod util;

use alloc::{boxed::Box, fmt, string::String};
use mdast::Node;
use parser::parse;

#[doc(hidden)]
// Do not use: exported for quick prototyping, will be removed.
pub use util::identifier::{id_cont, id_start};

#[doc(hidden)]
// Do not use: exported for quick prototyping, will be removed.
pub use util::sanitize_uri::sanitize;

#[doc(hidden)]
// Do not use: exported for quick prototyping, will be removed.
pub use util::location::Location;

/// Type of line endings in markdown.
///
/// Particularly when working with Windows, you might want to use
/// `LineEnding::CarriageReturnLineFeed`.
///
/// ## Examples
///
/// ```
/// use markdown::LineEnding;
/// # fn main() {
///
/// // Use a CR + LF combination:
/// let crlf = LineEnding::CarriageReturnLineFeed;
/// # }
/// ```
#[derive(Clone, Debug, Default, Eq, PartialEq)]
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
    #[default]
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
    /// Turn a string into a line ending.
    ///
    /// ## Panics
    ///
    /// Panics if `code` is not `\r\n`, `\r`, or `\n`.
    fn from_str(str: &str) -> LineEnding {
        match str {
            "\r\n" => LineEnding::CarriageReturnLineFeed,
            "\r" => LineEnding::CarriageReturn,
            "\n" => LineEnding::LineFeed,
            _ => unreachable!("invalid str"),
        }
    }
}

/// Signal used as feedback when parsing MDX ESM/expressions.
#[derive(Clone, Debug)]
pub enum MdxSignal {
    /// A syntax error.
    ///
    /// `markdown-rs` will crash with error message `String`, and convert the
    /// `usize` (byte offset into `&str` passed to `MdxExpressionParse` or
    /// `MdxEsmParse`) to where it happened in the whole document.
    ///
    /// ## Examples
    ///
    /// ```rust ignore
    /// MdxSignal::Error("Unexpected `\"`, expected identifier".into(), 1)
    /// ```
    Error(String, usize),
    /// An error at the end of the (partial?) expression.
    ///
    /// `markdown-rs` will either crash with error message `String` if it
    /// doesn’t have any more text, or it will try again later when more text
    /// is available.
    ///
    /// ## Examples
    ///
    /// ```rust ignore
    /// MdxSignal::Eof("Unexpected end of file in string literal".into())
    /// ```
    Eof(String),
    /// Done, successfully.
    ///
    /// `markdown-rs` knows that this is the end of a valid expression/esm and
    /// continues with markdown.
    ///
    /// ## Examples
    ///
    /// ```rust ignore
    /// MdxSignal::Ok
    /// ```
    Ok,
}

/// Signature of a function that parses MDX ESM.
///
/// Can be passed as `mdx_esm_parse` in [`ParseOptions`][] to support
/// ESM according to a certain grammar (typically, a programming language).
pub type MdxEsmParse = dyn Fn(&str) -> MdxSignal;

/// Expression kind.
#[derive(Clone, Debug)]
pub enum MdxExpressionKind {
    /// Kind of expressions in prose.
    ///
    /// ```mdx
    /// > | # {Math.PI}
    ///       ^^^^^^^^^
    ///   |
    /// > | {Math.PI}
    ///     ^^^^^^^^^
    /// ```
    Expression,
    /// Kind of expressions as attributes.
    ///
    /// ```mdx
    /// > | <a {...b}>
    ///        ^^^^^^
    /// ```
    AttributeExpression,
    /// Kind of expressions as attribute values.
    ///
    /// ```mdx
    /// > | <a b={c}>
    ///          ^^^
    /// ```
    AttributeValueExpression,
}

/// Signature of a function that parses MDX expressions.
///
/// Can be passed as `mdx_expression_parse` in [`ParseOptions`][] to support
/// expressions according to a certain grammar (typically, a programming
/// language).
pub type MdxExpressionParse = dyn Fn(&str, &MdxExpressionKind) -> MdxSignal;

/// Control which constructs are enabled.
///
/// Not all constructs can be configured.
/// Notably, blank lines and paragraphs cannot be turned off.
///
/// ## Examples
///
/// ```
/// use markdown::Constructs;
/// # fn main() {
///
/// // Use the default trait to get `CommonMark` constructs:
/// let commonmark = Constructs::default();
///
/// // To turn on all of GFM, use the `gfm` method:
/// let gfm = Constructs::gfm();
///
/// // Or, mix and match:
/// let custom = Constructs {
///   math_flow: true,
///   math_text: true,
///   ..Constructs::gfm()
/// };
/// # }
/// ```
#[allow(clippy::struct_excessive_bools)]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Constructs {
    /// Attention.
    ///
    /// ```markdown
    /// > | a *b* c **d**.
    ///       ^^^   ^^^^^
    /// ```
    pub attention: bool,
    /// Autolink.
    ///
    /// ```markdown
    /// > | a <https://example.com> b <user@example.org>.
    ///       ^^^^^^^^^^^^^^^^^^^^^   ^^^^^^^^^^^^^^^^^^
    /// ```
    pub autolink: bool,
    /// Block quote.
    ///
    /// ```markdown
    /// > | > a
    ///     ^^^
    /// ```
    pub block_quote: bool,
    /// Character escape.
    ///
    /// ```markdown
    /// > | a \* b
    ///       ^^
    /// ```
    pub character_escape: bool,
    /// Character reference.
    ///
    /// ```markdown
    /// > | a &amp; b
    ///       ^^^^^
    /// ```
    pub character_reference: bool,
    /// Code (indented).
    ///
    /// ```markdown
    /// > |     a
    ///     ^^^^^
    /// ```
    pub code_indented: bool,
    /// Code (fenced).
    ///
    /// ```markdown
    /// > | ~~~js
    ///     ^^^^^
    /// > | console.log(1)
    ///     ^^^^^^^^^^^^^^
    /// > | ~~~
    ///     ^^^
    /// ```
    pub code_fenced: bool,
    /// Code (text).
    ///
    /// ```markdown
    /// > | a `b` c
    ///       ^^^
    /// ```
    pub code_text: bool,
    /// Definition.
    ///
    /// ```markdown
    /// > | [a]: b "c"
    ///     ^^^^^^^^^^
    /// ```
    pub definition: bool,
    /// Frontmatter.
    ///
    /// ````markdown
    /// > | ---
    ///     ^^^
    /// > | title: Neptune
    ///     ^^^^^^^^^^^^^^
    /// > | ---
    ///     ^^^
    /// ````
    pub frontmatter: bool,
    /// GFM: autolink literal.
    ///
    /// ```markdown
    /// > | https://example.com
    ///     ^^^^^^^^^^^^^^^^^^^
    /// ```
    pub gfm_autolink_literal: bool,
    /// GFM: footnote definition.
    ///
    /// ```markdown
    /// > | [^a]: b
    ///     ^^^^^^^
    /// ```
    pub gfm_footnote_definition: bool,
    /// GFM: footnote label start.
    ///
    /// ```markdown
    /// > | a[^b]
    ///      ^^
    /// ```
    pub gfm_label_start_footnote: bool,
    ///
    /// ```markdown
    /// > | a ~b~ c.
    ///       ^^^
    /// ```
    pub gfm_strikethrough: bool,
    /// GFM: table.
    ///
    /// ```markdown
    /// > | | a |
    ///     ^^^^^
    /// > | | - |
    ///     ^^^^^
    /// > | | b |
    ///     ^^^^^
    /// ```
    pub gfm_table: bool,
    /// GFM: task list item.
    ///
    /// ```markdown
    /// > | * [x] y.
    ///       ^^^
    /// ```
    pub gfm_task_list_item: bool,
    /// Hard break (escape).
    ///
    /// ```markdown
    /// > | a\
    ///      ^
    ///   | b
    /// ```
    pub hard_break_escape: bool,
    /// Hard break (trailing).
    ///
    /// ```markdown
    /// > | a␠␠
    ///      ^^
    ///   | b
    /// ```
    pub hard_break_trailing: bool,
    /// Heading (atx).
    ///
    /// ```markdown
    /// > | # a
    ///     ^^^
    /// ```
    pub heading_atx: bool,
    /// Heading (setext).
    ///
    /// ```markdown
    /// > | a
    ///     ^^
    /// > | ==
    ///     ^^
    /// ```
    pub heading_setext: bool,
    /// HTML (flow).
    ///
    /// ```markdown
    /// > | <div>
    ///     ^^^^^
    /// ```
    pub html_flow: bool,
    /// HTML (text).
    ///
    /// ```markdown
    /// > | a <b> c
    ///       ^^^
    /// ```
    pub html_text: bool,
    /// Label start (image).
    ///
    /// ```markdown
    /// > | a ![b](c) d
    ///       ^^
    /// ```
    pub label_start_image: bool,
    /// Label start (link).
    ///
    /// ```markdown
    /// > | a [b](c) d
    ///       ^
    /// ```
    pub label_start_link: bool,
    /// Label end.
    ///
    /// ```markdown
    /// > | a [b](c) d
    ///         ^^^^
    /// ```
    pub label_end: bool,
    /// List items.
    ///
    /// ```markdown
    /// > | * a
    ///     ^^^
    /// ```
    pub list_item: bool,
    /// Math (flow).
    ///
    /// ```markdown
    /// > | $$
    ///     ^^
    /// > | \frac{1}{2}
    ///     ^^^^^^^^^^^
    /// > | $$
    ///     ^^
    /// ```
    pub math_flow: bool,
    /// Math (text).
    ///
    /// ```markdown
    /// > | a $b$ c
    ///       ^^^
    /// ```
    pub math_text: bool,
    /// MDX: ESM.
    ///
    /// ```markdown
    /// > | import a from 'b'
    ///     ^^^^^^^^^^^^^^^^^
    /// ```
    ///
    /// > 👉 **Note**: to support ESM, you *must* pass
    /// > [`mdx_esm_parse`][MdxEsmParse] in [`ParseOptions`][] too.
    /// > Otherwise, ESM is treated as normal markdown.
    pub mdx_esm: bool,
    /// MDX: expression (flow).
    ///
    /// ```markdown
    /// > | {Math.PI}
    ///     ^^^^^^^^^
    /// ```
    ///
    /// > 👉 **Note**: You *can* pass
    /// > [`mdx_expression_parse`][MdxExpressionParse] in [`ParseOptions`][]
    /// > too, to parse expressions according to a certain grammar (typically,
    /// > a programming language).
    /// > Otherwise, expressions are parsed with a basic algorithm that only
    /// > cares about braces.
    pub mdx_expression_flow: bool,
    /// MDX: expression (text).
    ///
    /// ```markdown
    /// > | a {Math.PI} c
    ///       ^^^^^^^^^
    /// ```
    ///
    /// > 👉 **Note**: You *can* pass
    /// > [`mdx_expression_parse`][MdxExpressionParse] in [`ParseOptions`][]
    /// > too, to parse expressions according to a certain grammar (typically,
    /// > a programming language).
    /// > Otherwise, expressions are parsed with a basic algorithm that only
    /// > cares about braces.
    pub mdx_expression_text: bool,
    /// MDX: JSX (flow).
    ///
    /// ```markdown
    /// > | <Component />
    ///     ^^^^^^^^^^^^^
    /// ```
    ///
    /// > 👉 **Note**: You *can* pass
    /// > [`mdx_expression_parse`][MdxExpressionParse] in [`ParseOptions`][]
    /// > too, to parse expressions in JSX according to a certain grammar
    /// > (typically, a programming language).
    /// > Otherwise, expressions are parsed with a basic algorithm that only
    /// > cares about braces.
    pub mdx_jsx_flow: bool,
    /// MDX: JSX (text).
    ///
    /// ```markdown
    /// > | a <Component /> c
    ///       ^^^^^^^^^^^^^
    /// ```
    ///
    /// > 👉 **Note**: You *can* pass
    /// > [`mdx_expression_parse`][MdxExpressionParse] in [`ParseOptions`][]
    /// > too, to parse expressions in JSX according to a certain grammar
    /// > (typically, a programming language).
    /// > Otherwise, expressions are parsed with a basic algorithm that only
    /// > cares about braces.
    pub mdx_jsx_text: bool,
    /// Thematic break.
    ///
    /// ```markdown
    /// > | ***
    ///     ^^^
    /// ```
    pub thematic_break: bool,
}

impl Default for Constructs {
    /// `CommonMark`.
    ///
    /// `CommonMark` is a relatively strong specification of how markdown
    /// works.
    /// Most markdown parsers try to follow it.
    ///
    /// For more information, see the `CommonMark` specification:
    /// <https://spec.commonmark.org>.
    fn default() -> Self {
        Self {
            attention: true,
            autolink: true,
            block_quote: true,
            character_escape: true,
            character_reference: true,
            code_indented: true,
            code_fenced: true,
            code_text: true,
            definition: true,
            frontmatter: false,
            gfm_autolink_literal: false,
            gfm_label_start_footnote: false,
            gfm_footnote_definition: false,
            gfm_strikethrough: false,
            gfm_table: false,
            gfm_task_list_item: false,
            hard_break_escape: true,
            hard_break_trailing: true,
            heading_atx: true,
            heading_setext: true,
            html_flow: true,
            html_text: true,
            label_start_image: true,
            label_start_link: true,
            label_end: true,
            list_item: true,
            math_flow: false,
            math_text: false,
            mdx_esm: false,
            mdx_expression_flow: false,
            mdx_expression_text: false,
            mdx_jsx_flow: false,
            mdx_jsx_text: false,
            thematic_break: true,
        }
    }
}

impl Constructs {
    /// GFM.
    ///
    /// GFM stands for **GitHub flavored markdown**.
    /// GFM extends `CommonMark` and adds support for autolink literals,
    /// footnotes, strikethrough, tables, and tasklists.
    ///
    /// For more information, see the GFM specification:
    /// <https://github.github.com/gfm/>.
    #[must_use]
    pub fn gfm() -> Self {
        Self {
            gfm_autolink_literal: true,
            gfm_footnote_definition: true,
            gfm_label_start_footnote: true,
            gfm_strikethrough: true,
            gfm_table: true,
            gfm_task_list_item: true,
            ..Self::default()
        }
    }

    /// MDX.
    ///
    /// This turns on `CommonMark`, turns off some conflicting constructs
    /// (autolinks, code (indented), and HTML), and turns on MDX (ESM,
    /// expressions, and JSX).
    ///
    /// For more information, see the MDX website:
    /// <https://mdxjs.com>.
    ///
    /// > 👉 **Note**: to support ESM, you *must* pass
    /// > [`mdx_esm_parse`][MdxEsmParse] in [`ParseOptions`][] too.
    /// > Otherwise, ESM is treated as normal markdown.
    /// >
    /// > You *can* pass
    /// > [`mdx_expression_parse`][MdxExpressionParse]
    /// > to parse expressions according to a certain grammar (typically, a
    /// > programming language).
    /// > Otherwise, expressions are parsed with a basic algorithm that only
    /// > cares about braces.
    #[must_use]
    pub fn mdx() -> Self {
        Self {
            autolink: false,
            code_indented: false,
            html_flow: false,
            html_text: false,
            mdx_esm: true,
            mdx_expression_flow: true,
            mdx_expression_text: true,
            mdx_jsx_flow: true,
            mdx_jsx_text: true,
            ..Self::default()
        }
    }
}

/// Configuration that describes how to compile to HTML.
///
/// You likely either want to turn on the dangerous options
/// (`allow_dangerous_html`, `allow_dangerous_protocol`) when dealing with
/// input you trust, or want to customize how GFM footnotes are compiled
/// (typically because the input markdown is not in English).
///
/// ## Examples
///
/// ```
/// use markdown::CompileOptions;
/// # fn main() {
///
/// // Use the default trait to get safe defaults:
/// let safe = CompileOptions::default();
///
/// // Live dangerously / trust the author:
/// let danger = CompileOptions {
///   allow_dangerous_html: true,
///   allow_dangerous_protocol: true,
///   ..CompileOptions::default()
/// };
///
/// // In French:
/// let enFrançais = CompileOptions {
///   gfm_footnote_label: Some("Notes de bas de page".into()),
///   gfm_footnote_back_label: Some("Arrière".into()),
///   ..CompileOptions::default()
/// };
/// # }
/// ```
#[allow(clippy::struct_excessive_bools)]
#[derive(Clone, Debug, Default)]
pub struct CompileOptions {
    /// Whether to allow (dangerous) HTML.
    ///
    /// The default is `false`, which still parses the HTML according to
    /// `CommonMark` but shows the HTML as text instead of as elements.
    ///
    /// Pass `true` for trusted content to get actual HTML elements.
    ///
    /// ## Examples
    ///
    /// ```
    /// use markdown::{to_html, to_html_with_options, CompileOptions, Options};
    /// # fn main() -> Result<(), String> {
    ///
    /// // `markdown-rs` is safe by default:
    /// assert_eq!(
    ///     to_html("Hi, <i>venus</i>!"),
    ///     "<p>Hi, &lt;i&gt;venus&lt;/i&gt;!</p>"
    /// );
    ///
    /// // Turn `allow_dangerous_html` on to allow potentially dangerous HTML:
    /// assert_eq!(
    ///     to_html_with_options(
    ///         "Hi, <i>venus</i>!",
    ///         &Options {
    ///             compile: CompileOptions {
    ///               allow_dangerous_html: true,
    ///               ..CompileOptions::default()
    ///             },
    ///             ..Options::default()
    ///         }
    ///     )?,
    ///     "<p>Hi, <i>venus</i>!</p>"
    /// );
    /// # Ok(())
    /// # }
    /// ```
    pub allow_dangerous_html: bool,

    /// Whether to allow dangerous protocols in links and images.
    ///
    /// The default is `false`, which drops URLs in links and images that use
    /// dangerous protocols.
    ///
    /// Pass `true` for trusted content to support all protocols.
    ///
    /// URLs that have no protocol (which means it’s relative to the current
    /// page, such as `./some/page.html`) and URLs that have a safe protocol
    /// (for images: `http`, `https`; for links: `http`, `https`, `irc`,
    /// `ircs`, `mailto`, `xmpp`), are safe.
    /// All other URLs are dangerous and dropped.
    ///
    /// ## Examples
    ///
    /// ```
    /// use markdown::{to_html, to_html_with_options, CompileOptions, Options};
    /// # fn main() -> Result<(), String> {
    ///
    /// // `markdown-rs` is safe by default:
    /// assert_eq!(
    ///     to_html("<javascript:alert(1)>"),
    ///     "<p><a href=\"\">javascript:alert(1)</a></p>"
    /// );
    ///
    /// // Turn `allow_dangerous_protocol` on to allow potentially dangerous protocols:
    /// assert_eq!(
    ///     to_html_with_options(
    ///         "<javascript:alert(1)>",
    ///         &Options {
    ///             compile: CompileOptions {
    ///               allow_dangerous_protocol: true,
    ///               ..CompileOptions::default()
    ///             },
    ///             ..Options::default()
    ///         }
    ///     )?,
    ///     "<p><a href=\"javascript:alert(1)\">javascript:alert(1)</a></p>"
    /// );
    /// # Ok(())
    /// # }
    /// ```
    pub allow_dangerous_protocol: bool,

    /// Default line ending to use when compiling to HTML, for line endings not
    /// in `value`.
    ///
    /// Generally, `markdown-rs` copies line endings (`\r`, `\n`, `\r\n`) in
    /// the markdown document over to the compiled HTML.
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
    /// ```
    /// use markdown::{to_html, to_html_with_options, CompileOptions, LineEnding, Options};
    /// # fn main() -> Result<(), String> {
    ///
    /// // `markdown-rs` uses `\n` by default:
    /// assert_eq!(
    ///     to_html("> a"),
    ///     "<blockquote>\n<p>a</p>\n</blockquote>"
    /// );
    ///
    /// // Define `default_line_ending` to configure the default:
    /// assert_eq!(
    ///     to_html_with_options(
    ///         "> a",
    ///         &Options {
    ///             compile: CompileOptions {
    ///               default_line_ending: LineEnding::CarriageReturnLineFeed,
    ///               ..CompileOptions::default()
    ///             },
    ///             ..Options::default()
    ///         }
    ///     )?,
    ///     "<blockquote>\r\n<p>a</p>\r\n</blockquote>"
    /// );
    /// # Ok(())
    /// # }
    /// ```
    pub default_line_ending: LineEnding,

    /// Textual label to use for the footnotes section.
    ///
    /// The default value is `"Footnotes"`.
    /// Change it when the markdown is not in English.
    ///
    /// This label is typically hidden visually (assuming a `sr-only` CSS class
    /// is defined that does that), and thus affects screen readers only.
    /// If you do have such a class, but want to show this section to everyone,
    /// pass different attributes with the `gfm_footnote_label_attributes`
    /// option.
    ///
    /// ## Examples
    ///
    /// ```
    /// use markdown::{to_html_with_options, CompileOptions, Options, ParseOptions};
    /// # fn main() -> Result<(), String> {
    ///
    /// // `"Footnotes"` is used by default:
    /// assert_eq!(
    ///     to_html_with_options(
    ///         "[^a]\n\n[^a]: b",
    ///         &Options::gfm()
    ///     )?,
    ///     "<p><sup><a href=\"#user-content-fn-a\" id=\"user-content-fnref-a\" data-footnote-ref=\"\" aria-describedby=\"footnote-label\">1</a></sup></p>\n<section data-footnotes=\"\" class=\"footnotes\"><h2 id=\"footnote-label\" class=\"sr-only\">Footnotes</h2>\n<ol>\n<li id=\"user-content-fn-a\">\n<p>b <a href=\"#user-content-fnref-a\" data-footnote-backref=\"\" aria-label=\"Back to content\" class=\"data-footnote-backref\">↩</a></p>\n</li>\n</ol>\n</section>\n"
    /// );
    ///
    /// // Pass `gfm_footnote_label` to use something else:
    /// assert_eq!(
    ///     to_html_with_options(
    ///         "[^a]\n\n[^a]: b",
    ///         &Options {
    ///             parse: ParseOptions::gfm(),
    ///             compile: CompileOptions {
    ///               gfm_footnote_label: Some("Notes de bas de page".into()),
    ///               ..CompileOptions::gfm()
    ///             }
    ///         }
    ///     )?,
    ///     "<p><sup><a href=\"#user-content-fn-a\" id=\"user-content-fnref-a\" data-footnote-ref=\"\" aria-describedby=\"footnote-label\">1</a></sup></p>\n<section data-footnotes=\"\" class=\"footnotes\"><h2 id=\"footnote-label\" class=\"sr-only\">Notes de bas de page</h2>\n<ol>\n<li id=\"user-content-fn-a\">\n<p>b <a href=\"#user-content-fnref-a\" data-footnote-backref=\"\" aria-label=\"Back to content\" class=\"data-footnote-backref\">↩</a></p>\n</li>\n</ol>\n</section>\n"
    /// );
    /// # Ok(())
    /// # }
    /// ```
    pub gfm_footnote_label: Option<String>,

    /// HTML tag name to use for the footnote label element.
    ///
    /// The default value is `"h2"`.
    /// Change it to match your document structure.
    ///
    /// This label is typically hidden visually (assuming a `sr-only` CSS class
    /// is defined that does that), and thus affects screen readers only.
    /// If you do have such a class, but want to show this section to everyone,
    /// pass different attributes with the `gfm_footnote_label_attributes`
    /// option.
    ///
    /// ## Examples
    ///
    /// ```
    /// use markdown::{to_html_with_options, CompileOptions, Options, ParseOptions};
    /// # fn main() -> Result<(), String> {
    ///
    /// // `"h2"` is used by default:
    /// assert_eq!(
    ///     to_html_with_options(
    ///         "[^a]\n\n[^a]: b",
    ///         &Options::gfm()
    ///     )?,
    ///     "<p><sup><a href=\"#user-content-fn-a\" id=\"user-content-fnref-a\" data-footnote-ref=\"\" aria-describedby=\"footnote-label\">1</a></sup></p>\n<section data-footnotes=\"\" class=\"footnotes\"><h2 id=\"footnote-label\" class=\"sr-only\">Footnotes</h2>\n<ol>\n<li id=\"user-content-fn-a\">\n<p>b <a href=\"#user-content-fnref-a\" data-footnote-backref=\"\" aria-label=\"Back to content\" class=\"data-footnote-backref\">↩</a></p>\n</li>\n</ol>\n</section>\n"
    /// );
    ///
    /// // Pass `gfm_footnote_label_tag_name` to use something else:
    /// assert_eq!(
    ///     to_html_with_options(
    ///         "[^a]\n\n[^a]: b",
    ///         &Options {
    ///             parse: ParseOptions::gfm(),
    ///             compile: CompileOptions {
    ///               gfm_footnote_label_tag_name: Some("h1".into()),
    ///               ..CompileOptions::gfm()
    ///             }
    ///         }
    ///     )?,
    ///     "<p><sup><a href=\"#user-content-fn-a\" id=\"user-content-fnref-a\" data-footnote-ref=\"\" aria-describedby=\"footnote-label\">1</a></sup></p>\n<section data-footnotes=\"\" class=\"footnotes\"><h1 id=\"footnote-label\" class=\"sr-only\">Footnotes</h1>\n<ol>\n<li id=\"user-content-fn-a\">\n<p>b <a href=\"#user-content-fnref-a\" data-footnote-backref=\"\" aria-label=\"Back to content\" class=\"data-footnote-backref\">↩</a></p>\n</li>\n</ol>\n</section>\n"
    /// );
    /// # Ok(())
    /// # }
    /// ```
    pub gfm_footnote_label_tag_name: Option<String>,

    /// Attributes to use on the footnote label.
    ///
    /// The default value is `"class=\"sr-only\""`.
    /// Change it to show the label and add other attributes.
    ///
    /// This label is typically hidden visually (assuming a `sr-only` CSS class
    /// is defined that does that), and thus affects screen readers only.
    /// If you do have such a class, but want to show this section to everyone,
    /// pass an empty string.
    /// You can also add different attributes.
    ///
    /// > 👉 **Note**: `id="footnote-label"` is always added, because footnote
    /// > calls use it with `aria-describedby` to provide an accessible label.
    ///
    /// ## Examples
    ///
    /// ```
    /// use markdown::{to_html_with_options, CompileOptions, Options, ParseOptions};
    /// # fn main() -> Result<(), String> {
    ///
    /// // `"class=\"sr-only\""` is used by default:
    /// assert_eq!(
    ///     to_html_with_options(
    ///         "[^a]\n\n[^a]: b",
    ///         &Options::gfm()
    ///     )?,
    ///     "<p><sup><a href=\"#user-content-fn-a\" id=\"user-content-fnref-a\" data-footnote-ref=\"\" aria-describedby=\"footnote-label\">1</a></sup></p>\n<section data-footnotes=\"\" class=\"footnotes\"><h2 id=\"footnote-label\" class=\"sr-only\">Footnotes</h2>\n<ol>\n<li id=\"user-content-fn-a\">\n<p>b <a href=\"#user-content-fnref-a\" data-footnote-backref=\"\" aria-label=\"Back to content\" class=\"data-footnote-backref\">↩</a></p>\n</li>\n</ol>\n</section>\n"
    /// );
    ///
    /// // Pass `gfm_footnote_label_attributes` to use something else:
    /// assert_eq!(
    ///     to_html_with_options(
    ///         "[^a]\n\n[^a]: b",
    ///         &Options {
    ///             parse: ParseOptions::gfm(),
    ///             compile: CompileOptions {
    ///               gfm_footnote_label_attributes: Some("class=\"footnote-heading\"".into()),
    ///               ..CompileOptions::gfm()
    ///             }
    ///         }
    ///     )?,
    ///     "<p><sup><a href=\"#user-content-fn-a\" id=\"user-content-fnref-a\" data-footnote-ref=\"\" aria-describedby=\"footnote-label\">1</a></sup></p>\n<section data-footnotes=\"\" class=\"footnotes\"><h2 id=\"footnote-label\" class=\"footnote-heading\">Footnotes</h2>\n<ol>\n<li id=\"user-content-fn-a\">\n<p>b <a href=\"#user-content-fnref-a\" data-footnote-backref=\"\" aria-label=\"Back to content\" class=\"data-footnote-backref\">↩</a></p>\n</li>\n</ol>\n</section>\n"
    /// );
    /// # Ok(())
    /// # }
    /// ```
    pub gfm_footnote_label_attributes: Option<String>,

    /// Textual label to describe the backreference back to footnote calls.
    ///
    /// The default value is `"Back to content"`.
    /// Change it when the markdown is not in English.
    ///
    /// This label is used in the `aria-label` attribute on each backreference
    /// (the `↩` links).
    /// It affects users of assistive technology.
    ///
    /// ## Examples
    ///
    /// ```
    /// use markdown::{to_html_with_options, CompileOptions, Options, ParseOptions};
    /// # fn main() -> Result<(), String> {
    ///
    /// // `"Back to content"` is used by default:
    /// assert_eq!(
    ///     to_html_with_options(
    ///         "[^a]\n\n[^a]: b",
    ///         &Options::gfm()
    ///     )?,
    ///     "<p><sup><a href=\"#user-content-fn-a\" id=\"user-content-fnref-a\" data-footnote-ref=\"\" aria-describedby=\"footnote-label\">1</a></sup></p>\n<section data-footnotes=\"\" class=\"footnotes\"><h2 id=\"footnote-label\" class=\"sr-only\">Footnotes</h2>\n<ol>\n<li id=\"user-content-fn-a\">\n<p>b <a href=\"#user-content-fnref-a\" data-footnote-backref=\"\" aria-label=\"Back to content\" class=\"data-footnote-backref\">↩</a></p>\n</li>\n</ol>\n</section>\n"
    /// );
    ///
    /// // Pass `gfm_footnote_back_label` to use something else:
    /// assert_eq!(
    ///     to_html_with_options(
    ///         "[^a]\n\n[^a]: b",
    ///         &Options {
    ///             parse: ParseOptions::gfm(),
    ///             compile: CompileOptions {
    ///               gfm_footnote_back_label: Some("Arrière".into()),
    ///               ..CompileOptions::gfm()
    ///             }
    ///         }
    ///     )?,
    ///     "<p><sup><a href=\"#user-content-fn-a\" id=\"user-content-fnref-a\" data-footnote-ref=\"\" aria-describedby=\"footnote-label\">1</a></sup></p>\n<section data-footnotes=\"\" class=\"footnotes\"><h2 id=\"footnote-label\" class=\"sr-only\">Footnotes</h2>\n<ol>\n<li id=\"user-content-fn-a\">\n<p>b <a href=\"#user-content-fnref-a\" data-footnote-backref=\"\" aria-label=\"Arrière\" class=\"data-footnote-backref\">↩</a></p>\n</li>\n</ol>\n</section>\n"
    /// );
    /// # Ok(())
    /// # }
    /// ```
    pub gfm_footnote_back_label: Option<String>,

    /// Prefix to use before the `id` attribute on footnotes to prevent them
    /// from *clobbering*.
    ///
    /// The default is `"user-content-"`.
    /// Pass `Some("".into())` for trusted markdown and when you are careful
    /// with polyfilling.
    /// You could pass a different prefix.
    ///
    /// DOM clobbering is this:
    ///
    /// ```html
    /// <p id="x"></p>
    /// <script>alert(x) // `x` now refers to the `p#x` DOM element</script>
    /// ```
    ///
    /// The above example shows that elements are made available by browsers,
    /// by their ID, on the `window` object.
    /// This is a security risk because you might be expecting some other
    /// variable at that place.
    /// It can also break polyfills.
    /// Using a prefix solves these problems.
    ///
    /// ## Examples
    ///
    /// ```
    /// use markdown::{to_html_with_options, CompileOptions, Options, ParseOptions};
    /// # fn main() -> Result<(), String> {
    ///
    /// // `"user-content-"` is used by default:
    /// assert_eq!(
    ///     to_html_with_options(
    ///         "[^a]\n\n[^a]: b",
    ///         &Options::gfm()
    ///     )?,
    ///     "<p><sup><a href=\"#user-content-fn-a\" id=\"user-content-fnref-a\" data-footnote-ref=\"\" aria-describedby=\"footnote-label\">1</a></sup></p>\n<section data-footnotes=\"\" class=\"footnotes\"><h2 id=\"footnote-label\" class=\"sr-only\">Footnotes</h2>\n<ol>\n<li id=\"user-content-fn-a\">\n<p>b <a href=\"#user-content-fnref-a\" data-footnote-backref=\"\" aria-label=\"Back to content\" class=\"data-footnote-backref\">↩</a></p>\n</li>\n</ol>\n</section>\n"
    /// );
    ///
    /// // Pass `gfm_footnote_clobber_prefix` to use something else:
    /// assert_eq!(
    ///     to_html_with_options(
    ///         "[^a]\n\n[^a]: b",
    ///         &Options {
    ///             parse: ParseOptions::gfm(),
    ///             compile: CompileOptions {
    ///               gfm_footnote_clobber_prefix: Some("".into()),
    ///               ..CompileOptions::gfm()
    ///             }
    ///         }
    ///     )?,
    ///     "<p><sup><a href=\"#fn-a\" id=\"fnref-a\" data-footnote-ref=\"\" aria-describedby=\"footnote-label\">1</a></sup></p>\n<section data-footnotes=\"\" class=\"footnotes\"><h2 id=\"footnote-label\" class=\"sr-only\">Footnotes</h2>\n<ol>\n<li id=\"fn-a\">\n<p>b <a href=\"#fnref-a\" data-footnote-backref=\"\" aria-label=\"Back to content\" class=\"data-footnote-backref\">↩</a></p>\n</li>\n</ol>\n</section>\n"
    /// );
    /// # Ok(())
    /// # }
    /// ```
    pub gfm_footnote_clobber_prefix: Option<String>,

    /// Whether to support the GFM tagfilter.
    ///
    /// This option does nothing if `allow_dangerous_html` is not turned on.
    /// The default is `false`, which does not apply the GFM tagfilter to HTML.
    /// Pass `true` for output that is a bit closer to GitHub’s actual output.
    ///
    /// The tagfilter is kinda weird and kinda useless.
    /// The tag filter is a naïve attempt at XSS protection.
    /// You should use a proper HTML sanitizing algorithm instead.
    ///
    /// ## Examples
    ///
    /// ```
    /// use markdown::{to_html_with_options, CompileOptions, Options, ParseOptions};
    /// # fn main() -> Result<(), String> {
    ///
    /// // With `allow_dangerous_html`, `markdown-rs` passes HTML through untouched:
    /// assert_eq!(
    ///     to_html_with_options(
    ///         "<iframe>",
    ///         &Options {
    ///             parse: ParseOptions::gfm(),
    ///             compile: CompileOptions {
    ///               allow_dangerous_html: true,
    ///               ..CompileOptions::default()
    ///             }
    ///         }
    ///     )?,
    ///     "<iframe>"
    /// );
    ///
    /// // Pass `gfm_tagfilter: true` to make some of that safe:
    /// assert_eq!(
    ///     to_html_with_options(
    ///         "<iframe>",
    ///         &Options {
    ///             parse: ParseOptions::gfm(),
    ///             compile: CompileOptions {
    ///               allow_dangerous_html: true,
    ///               gfm_tagfilter: true,
    ///               ..CompileOptions::default()
    ///             }
    ///         }
    ///     )?,
    ///     "&lt;iframe>"
    /// );
    /// # Ok(())
    /// # }
    /// ```
    ///
    /// ## References
    ///
    /// *   [*§ 6.1 Disallowed Raw HTML (extension)* in GFM](https://github.github.com/gfm/#disallowed-raw-html-extension-)
    /// *   [`cmark-gfm#extensions/tagfilter.c`](https://github.com/github/cmark-gfm/blob/master/extensions/tagfilter.c)
    pub gfm_tagfilter: bool,
}

impl CompileOptions {
    /// GFM.
    ///
    /// GFM stands for **GitHub flavored markdown**.
    /// On the compilation side, GFM turns on the GFM tag filter.
    /// The tagfilter is useless, but it’s included here for consistency, and
    /// this method exists for parity to parse options.
    ///
    /// For more information, see the GFM specification:
    /// <https://github.github.com/gfm/>.
    #[must_use]
    pub fn gfm() -> Self {
        Self {
            gfm_tagfilter: true,
            ..Self::default()
        }
    }
}

/// Configuration that describes how to parse from markdown.
///
/// You can use this:
///
/// *   To control what markdown constructs are turned on and off
/// *   To control some of those constructs
/// *   To add support for certain programming languages when parsing MDX
///
/// In most cases, you will want to use the default trait or `gfm` method.
///
/// ## Examples
///
/// ```
/// use markdown::ParseOptions;
/// # fn main() {
///
/// // Use the default trait to parse markdown according to `CommonMark`:
/// let commonmark = ParseOptions::default();
///
/// // Use the `gfm` method to parse markdown according to GFM:
/// let gfm = ParseOptions::gfm();
/// # }
/// ```
#[allow(clippy::struct_excessive_bools)]
pub struct ParseOptions {
    // Note: when adding fields, don’t forget to add them to `fmt::Debug` below.
    /// Which constructs to enable and disable.
    ///
    /// The default is to follow `CommonMark`.
    ///
    /// ## Examples
    ///
    /// ```
    /// use markdown::{to_html, to_html_with_options, Constructs, Options, ParseOptions};
    /// # fn main() -> Result<(), String> {
    ///
    /// // `markdown-rs` follows CommonMark by default:
    /// assert_eq!(
    ///     to_html("    indented code?"),
    ///     "<pre><code>indented code?\n</code></pre>"
    /// );
    ///
    /// // Pass `constructs` to choose what to enable and disable:
    /// assert_eq!(
    ///     to_html_with_options(
    ///         "    indented code?",
    ///         &Options {
    ///             parse: ParseOptions {
    ///               constructs: Constructs {
    ///                 code_indented: false,
    ///                 ..Constructs::default()
    ///               },
    ///               ..ParseOptions::default()
    ///             },
    ///             ..Options::default()
    ///         }
    ///     )?,
    ///     "<p>indented code?</p>"
    /// );
    /// # Ok(())
    /// # }
    /// ```
    pub constructs: Constructs,

    /// Whether to support GFM strikethrough with a single tilde
    ///
    /// This option does nothing if `gfm_strikethrough` is not turned on in
    /// `constructs`.
    /// This option does not affect strikethrough with double tildes.
    ///
    /// The default is `true`, which follows how markdown on `github.com`
    /// works, as strikethrough with single tildes is supported.
    /// Pass `false`, to follow the GFM spec more strictly, by not allowing
    /// strikethrough with single tildes.
    ///
    /// ## Examples
    ///
    /// ```
    /// use markdown::{to_html_with_options, Constructs, Options, ParseOptions};
    /// # fn main() -> Result<(), String> {
    ///
    /// // `markdown-rs` supports single tildes by default:
    /// assert_eq!(
    ///     to_html_with_options(
    ///         "~a~",
    ///         &Options {
    ///             parse: ParseOptions {
    ///               constructs: Constructs::gfm(),
    ///               ..ParseOptions::default()
    ///             },
    ///             ..Options::default()
    ///         }
    ///     )?,
    ///     "<p><del>a</del></p>"
    /// );
    ///
    /// // Pass `gfm_strikethrough_single_tilde: false` to turn that off:
    /// assert_eq!(
    ///     to_html_with_options(
    ///         "~a~",
    ///         &Options {
    ///             parse: ParseOptions {
    ///               constructs: Constructs::gfm(),
    ///               gfm_strikethrough_single_tilde: false,
    ///               ..ParseOptions::default()
    ///             },
    ///             ..Options::default()
    ///         }
    ///     )?,
    ///     "<p>~a~</p>"
    /// );
    /// # Ok(())
    /// # }
    /// ```
    pub gfm_strikethrough_single_tilde: bool,

    /// Whether to support math (text) with a single dollar
    ///
    /// This option does nothing if `math_text` is not turned on in
    /// `constructs`.
    /// This option does not affect math (text) with two or more dollars.
    ///
    /// The default is `true`, which is more close to how code (text) and
    /// Pandoc work, as it allows math with a single dollar to form.
    /// However, single dollars can interfere with “normal” dollars in text.
    /// Pass `false`, to only allow math (text) to form when two or more
    /// dollars are used.
    ///
    /// ## Examples
    ///
    /// ```
    /// use markdown::{to_html_with_options, Constructs, Options, ParseOptions};
    /// # fn main() -> Result<(), String> {
    ///
    /// // `markdown-rs` supports single dollars by default:
    /// assert_eq!(
    ///     to_html_with_options(
    ///         "$a$",
    ///         &Options {
    ///             parse: ParseOptions {
    ///               constructs: Constructs {
    ///                 math_text: true,
    ///                 ..Constructs::default()
    ///               },
    ///               ..ParseOptions::default()
    ///             },
    ///             ..Options::default()
    ///         }
    ///     )?,
    ///     "<p><code class=\"language-math math-inline\">a</code></p>"
    /// );
    ///
    /// // Pass `math_text_single_dollar: false` to turn that off:
    /// assert_eq!(
    ///     to_html_with_options(
    ///         "$a$",
    ///         &Options {
    ///             parse: ParseOptions {
    ///               constructs: Constructs {
    ///                 math_text: true,
    ///                 ..Constructs::default()
    ///               },
    ///               math_text_single_dollar: false,
    ///               ..ParseOptions::default()
    ///             },
    ///             ..Options::default()
    ///         }
    ///     )?,
    ///     "<p>$a$</p>"
    /// );
    /// # Ok(())
    /// # }
    /// ```
    pub math_text_single_dollar: bool,

    /// Function to parse expressions with.
    ///
    /// This function can be used to add support for arbitrary programming
    /// languages within expressions.
    ///
    /// It only makes sense to pass this when compiling to a syntax tree
    /// with [`to_mdast()`][].
    ///
    /// For an example that adds support for JavaScript with SWC, see
    /// `tests/test_utils/mod.rs`.
    pub mdx_expression_parse: Option<Box<MdxExpressionParse>>,

    /// Function to parse ESM with.
    ///
    /// This function can be used to add support for arbitrary programming
    /// languages within ESM blocks, however, the keywords (`export`,
    /// `import`) are currently hardcoded JavaScript-specific.
    ///
    /// > 👉 **Note**: please raise an issue if you’re interested in working on
    /// > MDX that is aware of, say, Rust, or other programming languages.
    ///
    /// It only makes sense to pass this when compiling to a syntax tree
    /// with [`to_mdast()`][].
    ///
    /// For an example that adds support for JavaScript with SWC, see
    /// `tests/test_utils/mod.rs`.
    pub mdx_esm_parse: Option<Box<MdxEsmParse>>,
    // Note: when adding fields, don’t forget to add them to `fmt::Debug` below.
}

impl fmt::Debug for ParseOptions {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("ParseOptions")
            .field("constructs", &self.constructs)
            .field(
                "gfm_strikethrough_single_tilde",
                &self.gfm_strikethrough_single_tilde,
            )
            .field("math_text_single_dollar", &self.math_text_single_dollar)
            .field(
                "mdx_expression_parse",
                &self.mdx_expression_parse.as_ref().map(|_d| "[Function]"),
            )
            .field(
                "mdx_esm_parse",
                &self.mdx_esm_parse.as_ref().map(|_d| "[Function]"),
            )
            .finish()
    }
}

impl Default for ParseOptions {
    /// `CommonMark` defaults.
    fn default() -> Self {
        Self {
            constructs: Constructs::default(),
            gfm_strikethrough_single_tilde: true,
            math_text_single_dollar: true,
            mdx_expression_parse: None,
            mdx_esm_parse: None,
        }
    }
}

impl ParseOptions {
    /// GFM.
    ///
    /// GFM stands for GitHub flavored markdown.
    /// GFM extends `CommonMark` and adds support for autolink literals,
    /// footnotes, strikethrough, tables, and tasklists.
    ///
    /// For more information, see the GFM specification:
    /// <https://github.github.com/gfm/>
    #[must_use]
    pub fn gfm() -> Self {
        Self {
            constructs: Constructs::gfm(),
            ..Self::default()
        }
    }

    /// MDX.
    ///
    /// This turns on `CommonMark`, turns off some conflicting constructs
    /// (autolinks, code (indented), and HTML), and turns on MDX (ESM,
    /// expressions, and JSX).
    ///
    /// For more information, see the MDX website:
    /// <https://mdxjs.com>.
    ///
    /// > 👉 **Note**: to support ESM, you *must* pass
    /// > [`mdx_esm_parse`][MdxEsmParse] in [`ParseOptions`][] too.
    /// > Otherwise, ESM is treated as normal markdown.
    /// >
    /// > You *can* pass
    /// > [`mdx_expression_parse`][MdxExpressionParse]
    /// > to parse expressions according to a certain grammar (typically, a
    /// > programming language).
    /// > Otherwise, expressions are parsed with a basic algorithm that only
    /// > cares about braces.
    #[must_use]
    pub fn mdx() -> Self {
        Self {
            constructs: Constructs::mdx(),
            ..Self::default()
        }
    }
}

/// Configuration that describes how to parse from markdown and compile to
/// HTML.
///
/// In most cases, you will want to use the default trait or `gfm` method.
///
/// ## Examples
///
/// ```
/// use markdown::Options;
/// # fn main() {
///
/// // Use the default trait to compile markdown to HTML according to `CommonMark`:
/// let commonmark = Options::default();
///
/// // Use the `gfm` method to compile markdown to HTML according to GFM:
/// let gfm = Options::gfm();
/// # }
/// ```
#[allow(clippy::struct_excessive_bools)]
#[derive(Debug, Default)]
pub struct Options {
    /// Configuration that describes how to parse from markdown.
    pub parse: ParseOptions,
    /// Configuration that describes how to compile to HTML.
    pub compile: CompileOptions,
}

impl Options {
    /// GFM.
    ///
    /// GFM stands for GitHub flavored markdown.
    /// GFM extends `CommonMark` and adds support for autolink literals,
    /// footnotes, strikethrough, tables, and tasklists.
    /// On the compilation side, GFM turns on the GFM tag filter.
    /// The tagfilter is useless, but it’s included here for consistency.
    ///
    /// For more information, see the GFM specification:
    /// <https://github.github.com/gfm/>
    #[must_use]
    pub fn gfm() -> Self {
        Self {
            parse: ParseOptions::gfm(),
            compile: CompileOptions::gfm(),
        }
    }
}

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
#[must_use]
#[allow(clippy::missing_panics_doc)]
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
    let (events, parse_state) = parse(value, &options.parse)?;
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
pub fn to_mdast(value: &str, options: &ParseOptions) -> Result<Node, String> {
    let (events, parse_state) = parse(value, options)?;
    let node = to_mdast::compile(&events, parse_state.bytes)?;
    Ok(node)
}
