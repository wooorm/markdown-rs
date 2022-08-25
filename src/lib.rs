//! Public API of micromark.
//!
//! This module exposes [`micromark`][] (and [`micromark_with_options`][]).
//! `micromark` is a safe way to transform (untrusted?) markdown into HTML.
//! `micromark_with_options` allows you to configure how markdown is turned into
//! HTML, such as by allowing dangerous HTML when you trust it.
#![no_std]

extern crate alloc;

mod compiler;
mod construct;
mod event;
mod parser;
mod resolve;
mod state;
mod subtokenize;
mod tokenizer;
mod util;

use crate::compiler::compile;
use crate::parser::parse;
use alloc::string::String;

/// Type of line endings in markdown.
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

/// Control which constructs are enabled.
///
/// Not all constructs can be configured.
/// Notably, blank lines and paragraphs cannot be turned off.
#[allow(clippy::struct_excessive_bools)]
#[derive(Clone, Debug)]
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
            thematic_break: true,
        }
    }
}

impl Constructs {
    /// GFM.
    ///
    /// This turns on `CommonMark` + GFM.
    #[must_use]
    pub fn gfm() -> Self {
        Self {
            gfm_autolink_literal: true,
            gfm_footnote_definition: true,
            gfm_label_start_footnote: true,
            gfm_strikethrough: true,
            gfm_task_list_item: true,
            ..Self::default()
        }
    }
}

/// Configuration (optional).
#[derive(Clone, Debug)]
pub struct Options {
    /// Whether to allow (dangerous) HTML.
    /// The default is `false`, you can turn it on to `true` for trusted
    /// content.
    ///
    /// ## Examples
    ///
    /// ```
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
    ///             ..Options::default()
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
    /// ```
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
    ///             allow_dangerous_protocol: true,
    ///             ..Options::default()
    ///         }
    ///     ),
    ///     "<p><a href=\"javascript:alert(1)\">javascript:alert(1)</a></p>"
    /// );
    /// ```
    pub allow_dangerous_protocol: bool,

    /// Label to use for the footnotes section.
    ///
    /// Change it when the markdown is not in English.
    /// Typically affects screen readers (change `gfm_footnote_label_attributes`
    /// to make it visible).
    ///
    /// ## Examples
    ///
    /// ```
    /// use micromark::{micromark, micromark_with_options, Options, Constructs};
    ///
    /// // `"Footnotes"` is used by default:
    /// assert_eq!(
    ///     micromark_with_options(
    ///         "[^a]\n\n[^a]: b",
    ///         &Options {
    ///             constructs: Constructs::gfm(),
    ///             ..Options::default()
    ///         }
    ///     ),
    ///     "<p><sup><a href=\"#user-content-fn-a\" id=\"user-content-fnref-a\" data-footnote-ref=\"\" aria-describedby=\"footnote-label\">1</a></sup></p>\n<section data-footnotes=\"\" class=\"footnotes\"><h2 id=\"footnote-label\" class=\"sr-only\">Footnotes</h2>\n<ol>\n<li id=\"user-content-fn-a\">\n<p>b <a href=\"#user-content-fnref-a\" data-footnote-backref=\"\" class=\"data-footnote-backref\" aria-label=\"Back to content\">↩</a></p>\n</li>\n</ol>\n</section>\n"
    /// );
    ///
    /// // Pass `gfm_footnote_label` to use something else:
    /// assert_eq!(
    ///     micromark_with_options(
    ///         "[^a]\n\n[^a]: b",
    ///         &Options {
    ///             constructs: Constructs::gfm(),
    ///             gfm_footnote_label: Some("Notes de bas de page".to_string()),
    ///             ..Options::default()
    ///         }
    ///     ),
    ///     "<p><sup><a href=\"#user-content-fn-a\" id=\"user-content-fnref-a\" data-footnote-ref=\"\" aria-describedby=\"footnote-label\">1</a></sup></p>\n<section data-footnotes=\"\" class=\"footnotes\"><h2 id=\"footnote-label\" class=\"sr-only\">Notes de bas de page</h2>\n<ol>\n<li id=\"user-content-fn-a\">\n<p>b <a href=\"#user-content-fnref-a\" data-footnote-backref=\"\" class=\"data-footnote-backref\" aria-label=\"Back to content\">↩</a></p>\n</li>\n</ol>\n</section>\n"
    /// );
    /// ```
    pub gfm_footnote_label: Option<String>,

    /// HTML tag to use for the footnote label.
    ///
    /// Change it to match your document structure and play well with your CSS.
    ///
    /// ## Examples
    ///
    /// ```
    /// use micromark::{micromark, micromark_with_options, Options, Constructs};
    ///
    /// // `"h2"` is used by default:
    /// assert_eq!(
    ///     micromark_with_options(
    ///         "[^a]\n\n[^a]: b",
    ///         &Options {
    ///             constructs: Constructs::gfm(),
    ///             ..Options::default()
    ///         }
    ///     ),
    ///     "<p><sup><a href=\"#user-content-fn-a\" id=\"user-content-fnref-a\" data-footnote-ref=\"\" aria-describedby=\"footnote-label\">1</a></sup></p>\n<section data-footnotes=\"\" class=\"footnotes\"><h2 id=\"footnote-label\" class=\"sr-only\">Footnotes</h2>\n<ol>\n<li id=\"user-content-fn-a\">\n<p>b <a href=\"#user-content-fnref-a\" data-footnote-backref=\"\" class=\"data-footnote-backref\" aria-label=\"Back to content\">↩</a></p>\n</li>\n</ol>\n</section>\n"
    /// );
    ///
    /// // Pass `gfm_footnote_label_tag_name` to use something else:
    /// assert_eq!(
    ///     micromark_with_options(
    ///         "[^a]\n\n[^a]: b",
    ///         &Options {
    ///             constructs: Constructs::gfm(),
    ///             gfm_footnote_label_tag_name: Some("h1".to_string()),
    ///             ..Options::default()
    ///         }
    ///     ),
    ///     "<p><sup><a href=\"#user-content-fn-a\" id=\"user-content-fnref-a\" data-footnote-ref=\"\" aria-describedby=\"footnote-label\">1</a></sup></p>\n<section data-footnotes=\"\" class=\"footnotes\"><h1 id=\"footnote-label\" class=\"sr-only\">Footnotes</h1>\n<ol>\n<li id=\"user-content-fn-a\">\n<p>b <a href=\"#user-content-fnref-a\" data-footnote-backref=\"\" class=\"data-footnote-backref\" aria-label=\"Back to content\">↩</a></p>\n</li>\n</ol>\n</section>\n"
    /// );
    /// ```
    pub gfm_footnote_label_tag_name: Option<String>,

    /// Attributes to use on the footnote label.
    ///
    /// > 👉 **Note**: `id="footnote-label"` is always added, because footnote
    /// > calls use it with `aria-describedby` to provide an accessible label.
    ///
    /// A `class="sr-only"` is added by default to hide the label from sighted
    /// users.
    /// Change it to make the label visible, or add other classes or other
    /// attributes.
    ///
    /// ## Examples
    ///
    /// ```
    /// use micromark::{micromark, micromark_with_options, Options, Constructs};
    ///
    /// // `"class=\"sr-only\""` is used by default:
    /// assert_eq!(
    ///     micromark_with_options(
    ///         "[^a]\n\n[^a]: b",
    ///         &Options {
    ///             constructs: Constructs::gfm(),
    ///             ..Options::default()
    ///         }
    ///     ),
    ///     "<p><sup><a href=\"#user-content-fn-a\" id=\"user-content-fnref-a\" data-footnote-ref=\"\" aria-describedby=\"footnote-label\">1</a></sup></p>\n<section data-footnotes=\"\" class=\"footnotes\"><h2 id=\"footnote-label\" class=\"sr-only\">Footnotes</h2>\n<ol>\n<li id=\"user-content-fn-a\">\n<p>b <a href=\"#user-content-fnref-a\" data-footnote-backref=\"\" class=\"data-footnote-backref\" aria-label=\"Back to content\">↩</a></p>\n</li>\n</ol>\n</section>\n"
    /// );
    ///
    /// // Pass `gfm_footnote_label_attributes` to use something else:
    /// assert_eq!(
    ///     micromark_with_options(
    ///         "[^a]\n\n[^a]: b",
    ///         &Options {
    ///             constructs: Constructs::gfm(),
    ///             gfm_footnote_label_attributes: Some("class=\"footnote-heading\"".to_string()),
    ///             ..Options::default()
    ///         }
    ///     ),
    ///     "<p><sup><a href=\"#user-content-fn-a\" id=\"user-content-fnref-a\" data-footnote-ref=\"\" aria-describedby=\"footnote-label\">1</a></sup></p>\n<section data-footnotes=\"\" class=\"footnotes\"><h2 id=\"footnote-label\" class=\"footnote-heading\">Footnotes</h2>\n<ol>\n<li id=\"user-content-fn-a\">\n<p>b <a href=\"#user-content-fnref-a\" data-footnote-backref=\"\" class=\"data-footnote-backref\" aria-label=\"Back to content\">↩</a></p>\n</li>\n</ol>\n</section>\n"
    /// );
    /// ```
    pub gfm_footnote_label_attributes: Option<String>,

    /// Label to use from backreferences back to their footnote call.
    ///
    /// Change it when the markdown is not in English.
    /// Affects screen readers.
    ///
    /// ## Examples
    ///
    /// ```
    /// use micromark::{micromark, micromark_with_options, Options, Constructs};
    ///
    /// // `"Back to content"` is used by default:
    /// assert_eq!(
    ///     micromark_with_options(
    ///         "[^a]\n\n[^a]: b",
    ///         &Options {
    ///             constructs: Constructs::gfm(),
    ///             ..Options::default()
    ///         }
    ///     ),
    ///     "<p><sup><a href=\"#user-content-fn-a\" id=\"user-content-fnref-a\" data-footnote-ref=\"\" aria-describedby=\"footnote-label\">1</a></sup></p>\n<section data-footnotes=\"\" class=\"footnotes\"><h2 id=\"footnote-label\" class=\"sr-only\">Footnotes</h2>\n<ol>\n<li id=\"user-content-fn-a\">\n<p>b <a href=\"#user-content-fnref-a\" data-footnote-backref=\"\" class=\"data-footnote-backref\" aria-label=\"Back to content\">↩</a></p>\n</li>\n</ol>\n</section>\n"
    /// );
    ///
    /// // Pass `gfm_footnote_back_label` to use something else:
    /// assert_eq!(
    ///     micromark_with_options(
    ///         "[^a]\n\n[^a]: b",
    ///         &Options {
    ///             constructs: Constructs::gfm(),
    ///             gfm_footnote_back_label: Some("Arrière".to_string()),
    ///             ..Options::default()
    ///         }
    ///     ),
    ///     "<p><sup><a href=\"#user-content-fn-a\" id=\"user-content-fnref-a\" data-footnote-ref=\"\" aria-describedby=\"footnote-label\">1</a></sup></p>\n<section data-footnotes=\"\" class=\"footnotes\"><h2 id=\"footnote-label\" class=\"sr-only\">Footnotes</h2>\n<ol>\n<li id=\"user-content-fn-a\">\n<p>b <a href=\"#user-content-fnref-a\" data-footnote-backref=\"\" class=\"data-footnote-backref\" aria-label=\"Arrière\">↩</a></p>\n</li>\n</ol>\n</section>\n"
    /// );
    /// ```
    pub gfm_footnote_back_label: Option<String>,

    /// Prefix to use before the `id` attribute on footnotes to prevent them
    /// from *clobbering*.
    ///
    /// DOM clobbering is this:
    ///
    /// ```html
    /// <p id=x></p>
    /// <script>alert(x) // `x` now refers to the DOM `p#x` element</script>
    /// ```
    ///
    /// The above example shows that elements are made available by browsers,
    /// by their ID, on the `window` object, which is a security risk because
    /// you might be expecting some other variable at that place.
    /// Using a prefix solves this problem.
    ///
    /// ## Examples
    ///
    /// ```
    /// use micromark::{micromark, micromark_with_options, Options, Constructs};
    ///
    /// // `"user-content-"` is used by default:
    /// assert_eq!(
    ///     micromark_with_options(
    ///         "[^a]\n\n[^a]: b",
    ///         &Options {
    ///             constructs: Constructs::gfm(),
    ///             ..Options::default()
    ///         }
    ///     ),
    ///     "<p><sup><a href=\"#user-content-fn-a\" id=\"user-content-fnref-a\" data-footnote-ref=\"\" aria-describedby=\"footnote-label\">1</a></sup></p>\n<section data-footnotes=\"\" class=\"footnotes\"><h2 id=\"footnote-label\" class=\"sr-only\">Footnotes</h2>\n<ol>\n<li id=\"user-content-fn-a\">\n<p>b <a href=\"#user-content-fnref-a\" data-footnote-backref=\"\" class=\"data-footnote-backref\" aria-label=\"Back to content\">↩</a></p>\n</li>\n</ol>\n</section>\n"
    /// );
    ///
    /// // Pass `gfm_footnote_clobber_prefix` to use something else:
    /// assert_eq!(
    ///     micromark_with_options(
    ///         "[^a]\n\n[^a]: b",
    ///         &Options {
    ///             constructs: Constructs::gfm(),
    ///             gfm_footnote_clobber_prefix: Some("".to_string()),
    ///             ..Options::default()
    ///         }
    ///     ),
    ///     "<p><sup><a href=\"#fn-a\" id=\"fnref-a\" data-footnote-ref=\"\" aria-describedby=\"footnote-label\">1</a></sup></p>\n<section data-footnotes=\"\" class=\"footnotes\"><h2 id=\"footnote-label\" class=\"sr-only\">Footnotes</h2>\n<ol>\n<li id=\"fn-a\">\n<p>b <a href=\"#fnref-a\" data-footnote-backref=\"\" class=\"data-footnote-backref\" aria-label=\"Back to content\">↩</a></p>\n</li>\n</ol>\n</section>\n"
    /// );
    /// ```
    pub gfm_footnote_clobber_prefix: Option<String>,

    /// Whether to support GFM strikethrough (if enabled in `constructs`) with
    /// a single tilde (default: true).
    ///
    /// Single tildes work on github.com but are technically prohibited by GFM.
    ///
    /// ## Examples
    ///
    /// ```
    /// use micromark::{micromark, micromark_with_options, Options, Constructs};
    ///
    /// // micromark supports single tildes by default:
    /// assert_eq!(
    ///     micromark_with_options(
    ///         "~a~",
    ///         &Options {
    ///             constructs: Constructs::gfm(),
    ///             ..Options::default()
    ///         }
    ///     ),
    ///     "<p><del>a</del></p>"
    /// );
    ///
    /// // Pass `gfm_strikethrough_single_tilde: false` to turn that off:
    /// assert_eq!(
    ///     micromark_with_options(
    ///         "~a~",
    ///         &Options {
    ///             constructs: Constructs::gfm(),
    ///             gfm_strikethrough_single_tilde: false,
    ///             ..Options::default()
    ///         }
    ///     ),
    ///     "<p>~a~</p>"
    /// );
    /// ```
    pub gfm_strikethrough_single_tilde: bool,

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
    /// ```
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
    ///             default_line_ending: LineEnding::CarriageReturnLineFeed,
    ///             ..Options::default()
    ///         }
    ///     ),
    ///     "<blockquote>\r\n<p>a</p>\r\n</blockquote>"
    /// );
    /// ```
    pub default_line_ending: LineEnding,

    /// Which constructs to enable and disable.
    /// The default is to follow `CommonMark`.
    ///
    /// ## Examples
    ///
    /// ```
    /// use micromark::{micromark, micromark_with_options, Options, Constructs};
    ///
    /// // micromark follows CommonMark by default:
    /// assert_eq!(
    ///     micromark("    indented code?"),
    ///     "<pre><code>indented code?\n</code></pre>"
    /// );
    ///
    /// // Pass `constructs` to choose what to enable and disable:
    /// assert_eq!(
    ///     micromark_with_options(
    ///         "    indented code?",
    ///         &Options {
    ///             constructs: Constructs {
    ///                 code_indented: false,
    ///                 ..Constructs::default()
    ///             },
    ///             ..Options::default()
    ///         }
    ///     ),
    ///     "<p>indented code?</p>"
    /// );
    /// ```
    pub constructs: Constructs,
}

impl Default for Options {
    /// Safe `CommonMark` defaults.
    fn default() -> Self {
        Self {
            allow_dangerous_html: false,
            allow_dangerous_protocol: false,
            gfm_footnote_label: None,
            gfm_footnote_label_tag_name: None,
            gfm_footnote_label_attributes: None,
            gfm_footnote_back_label: None,
            gfm_footnote_clobber_prefix: None,
            gfm_strikethrough_single_tilde: true,
            default_line_ending: LineEnding::default(),
            constructs: Constructs::default(),
        }
    }
}

/// Turn markdown into HTML.
///
/// ## Examples
///
/// ```
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
/// ```
/// use micromark::{micromark_with_options, Options};
///
/// let result = micromark_with_options("<div>\n\n# Hello, world!\n\n</div>", &Options {
///     allow_dangerous_html: true,
///     allow_dangerous_protocol: true,
///     ..Options::default()
/// });
///
/// assert_eq!(result, "<div>\n<h1>Hello, world!</h1>\n</div>");
/// ```
#[must_use]
pub fn micromark_with_options(value: &str, options: &Options) -> String {
    let (events, bytes) = parse(value, options);
    compile(&events, bytes, options)
}
