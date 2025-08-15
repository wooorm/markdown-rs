use serde::{Deserialize, Serialize};
use wasm_bindgen::prelude::*;

// Set panic hook for better error messages
#[wasm_bindgen(start)]
pub fn init() {
    console_error_panic_hook::set_once();
}

#[derive(Serialize, Deserialize, Default)]
pub struct Options {
    #[serde(default)]
    pub gfm: bool,

    #[serde(default)]
    pub mdx: bool,

    #[serde(default)]
    pub frontmatter: bool,

    #[serde(rename = "allowDangerousHtml", default)]
    pub allow_dangerous_html: bool,

    #[serde(rename = "allowDangerousProtocol", default)]
    pub allow_dangerous_protocol: bool,
}

/// Convert markdown to HTML with default options
#[wasm_bindgen]
pub fn to_html(input: &str) -> String {
    markdown::to_html(input)
}

/// Convert markdown to HTML with options
#[wasm_bindgen]
pub fn to_html_with_options(input: &str, options: JsValue) -> Result<String, JsValue> {
    // Parse options from JavaScript
    let opts: Options = if options.is_null() || options.is_undefined() {
        Options::default()
    } else {
        serde_wasm_bindgen::from_value(options)
            .map_err(|e| JsValue::from_str(&format!("Invalid options: {}", e)))?
    };

    // Build markdown options
    let mut parse_options = markdown::ParseOptions::default();
    let mut compile_options = markdown::CompileOptions::default();

    // Configure constructs based on options
    if opts.gfm {
        parse_options.constructs = markdown::Constructs::gfm();
    } else if opts.mdx {
        parse_options.constructs = markdown::Constructs::mdx();
    }

    if opts.frontmatter {
        parse_options.constructs.frontmatter = true;
    }

    // Configure compile options
    compile_options.allow_dangerous_html = opts.allow_dangerous_html;
    compile_options.allow_dangerous_protocol = opts.allow_dangerous_protocol;

    let markdown_options = markdown::Options {
        parse: parse_options,
        compile: compile_options,
    };

    // Convert markdown to HTML
    markdown::to_html_with_options(input, &markdown_options)
        .map_err(|e| JsValue::from_str(&format!("Markdown error: {}", e)))
}
