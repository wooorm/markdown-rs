use markdown::{MdxExpressionKind, MdxSignal};
use napi::{bindgen_prelude::*, Error};
use napi_derive::napi;
use serde_json::Value;

#[napi(object)]
pub struct ParseOptions {
    /// Whether to parse as MDX
    pub mdx: Option<bool>,
    /// Whether to enable GitHub Flavored Markdown (GFM) constructs
    pub gfm: Option<bool>,
    /// Whether to support GFM strikethrough with a single tilde
    pub gfm_strikethrough_single_tilde: Option<bool>,
    /// Whether to support math (text) with a single dollar
    pub math_text_single_dollar: Option<bool>,
    /// Whether to enable basic MDX expression parsing
    pub mdx_expression_parse: Option<bool>,
    /// Whether to enable basic MDX ESM parsing
    pub mdx_esm_parse: Option<bool>,
    /// Whether to support frontmatter
    pub frontmatter: Option<bool>,
}

#[napi(object)]
pub struct CompileOptions {
    /// Whether to allow (dangerous) HTML
    pub allow_dangerous_html: Option<bool>,
    /// Whether to allow dangerous protocols in links and images
    pub allow_dangerous_protocol: Option<bool>,
    /// Whether to allow all values in images
    pub allow_any_img_src: Option<bool>,
    /// Default line ending to use when compiling to HTML
    pub default_line_ending: Option<String>,
    /// Textual label to describe the backreference back to footnote calls
    pub gfm_footnote_back_label: Option<String>,
    /// Prefix to use before the `id` attribute on footnotes
    pub gfm_footnote_clobber_prefix: Option<String>,
    /// Textual label to use for the footnotes section
    pub gfm_footnote_label: Option<String>,
    /// Attributes to use on the footnote label
    pub gfm_footnote_label_attributes: Option<String>,
    /// HTML tag name to use for the footnote label element
    pub gfm_footnote_label_tag_name: Option<String>,
    /// Whether or not GFM task list html `<input>` items are enabled
    pub gfm_task_list_item_checkable: Option<bool>,
    /// Whether to support the GFM tagfilter
    pub gfm_tagfilter: Option<bool>,
}

#[napi(object)]
pub struct HtmlOptions {
    /// Configuration that describes how to parse from markdown
    pub parse: Option<ParseOptions>,
    /// Configuration that describes how to compile to HTML
    pub compile: Option<CompileOptions>,
}

impl ParseOptions {
    fn to_rust_parse_options(&self) -> markdown::ParseOptions {
        let mut constructs = if self.mdx.unwrap_or(false) {
            markdown::Constructs::mdx()
        } else {
            markdown::Constructs::default()
        };

        // Enable GFM constructs if requested
        if self.gfm.unwrap_or(false) {
            constructs.gfm_autolink_literal = true;
            constructs.gfm_label_start_footnote = true;
            constructs.gfm_footnote_definition = true;
            constructs.gfm_strikethrough = true;
            constructs.gfm_table = true;
            constructs.gfm_task_list_item = true;
        }

        // Enable frontmatter if requested
        if self.frontmatter.unwrap_or(false) {
            constructs.frontmatter = true;
        }

        markdown::ParseOptions {
            constructs,
            gfm_strikethrough_single_tilde: self.gfm_strikethrough_single_tilde.unwrap_or(true),
            math_text_single_dollar: self.math_text_single_dollar.unwrap_or(true),
            mdx_expression_parse: if self.mdx_expression_parse.unwrap_or(false) {
                Some(Box::new(|_value: &str, _kind: &MdxExpressionKind| {
                    // Basic expression parsing that just returns success
                    MdxSignal::Ok
                }))
            } else {
                None
            },
            mdx_esm_parse: if self.mdx_esm_parse.unwrap_or(false) {
                Some(Box::new(|_value: &str| {
                    // Basic ESM parsing that just returns success
                    MdxSignal::Ok
                }))
            } else {
                None
            },
            ..Default::default()
        }
    }
}

impl CompileOptions {
    fn to_rust_compile_options(&self) -> markdown::CompileOptions {
        let mut opts = markdown::CompileOptions::default();

        if let Some(val) = self.allow_dangerous_html {
            opts.allow_dangerous_html = val;
        }
        if let Some(val) = self.allow_dangerous_protocol {
            opts.allow_dangerous_protocol = val;
        }
        if let Some(val) = self.allow_any_img_src {
            opts.allow_any_img_src = val;
        }
        if let Some(ref val) = self.default_line_ending {
            opts.default_line_ending = match val.as_str() {
                "lf" | "\n" => markdown::LineEnding::LineFeed,
                "cr" | "\r" => markdown::LineEnding::CarriageReturn,
                "crlf" | "\r\n" => markdown::LineEnding::CarriageReturnLineFeed,
                _ => markdown::LineEnding::LineFeed,
            };
        }
        if let Some(ref val) = self.gfm_footnote_back_label {
            opts.gfm_footnote_back_label = Some(val.clone());
        }
        if let Some(ref val) = self.gfm_footnote_clobber_prefix {
            opts.gfm_footnote_clobber_prefix = Some(val.clone());
        }
        if let Some(ref val) = self.gfm_footnote_label {
            opts.gfm_footnote_label = Some(val.clone());
        }
        if let Some(ref val) = self.gfm_footnote_label_attributes {
            opts.gfm_footnote_label_attributes = Some(val.clone());
        }
        if let Some(ref val) = self.gfm_footnote_label_tag_name {
            opts.gfm_footnote_label_tag_name = Some(val.clone());
        }
        if let Some(val) = self.gfm_task_list_item_checkable {
            opts.gfm_task_list_item_checkable = val;
        }
        if let Some(val) = self.gfm_tagfilter {
            opts.gfm_tagfilter = val;
        }

        opts
    }
}

impl HtmlOptions {
    fn to_rust_options(&self) -> markdown::Options {
        let parse = match &self.parse {
            Some(opts) => opts.to_rust_parse_options(),
            None => markdown::ParseOptions::default(),
        };

        let compile = match &self.compile {
            Some(opts) => opts.to_rust_compile_options(),
            None => markdown::CompileOptions::default(),
        };

        markdown::Options { parse, compile }
    }
}

#[napi]
pub fn parse(input: String, options: Option<ParseOptions>) -> Result<Value> {
    let parse_options = match options {
        Some(opts) => opts.to_rust_parse_options(),
        None => markdown::ParseOptions::default(),
    };

    let tree = markdown::to_mdast(&input, &parse_options)
        .map_err(|e| Error::from_reason(format!("{:?}", e)))?;
    serde_json::to_value(&tree).map_err(|e| Error::from_reason(e.to_string()))
}

#[napi]
pub fn to_html(input: String, options: Option<HtmlOptions>) -> Result<String> {
    let opts = match options {
        Some(opts) => opts.to_rust_options(),
        None => markdown::Options::default(),
    };

    markdown::to_html_with_options(&input, &opts)
        .map_err(|e| Error::from_reason(format!("{:?}", e)))
}