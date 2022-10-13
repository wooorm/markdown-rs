use crate::test_utils::{
    hast_util_to_swc::hast_util_to_swc,
    mdast_util_to_hast::mdast_util_to_hast,
    mdx_plugin_recma_document::{mdx_plugin_recma_document, Options as DocumentOptions},
    mdx_plugin_recma_jsx_rewrite::{mdx_plugin_recma_jsx_rewrite, Options as RewriteOptions},
    swc::{parse_esm, parse_expression, serialize},
};
use markdown::{to_mdast, Constructs, Location, ParseOptions};

pub use super::mdx_plugin_recma_document::JsxRuntime;

use swc_common::comments::{Comments, SingleThreadedComments};
// use swc_ecma_transforms_react::{jsx, Options as JsxBuildOptions};
// use swc_ecma_visit::VisitMutWith;

// To do: use `Format`.
/// Format the file is in (default: `Format::Detect`).
#[allow(dead_code)]
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub enum Format {
    /// Use `Format::Markdown` for files with an extension in `md_extensions`
    /// and `Format::Mdx` otherwise.
    #[default]
    Detect,
    /// Treat file as MDX.
    Mdx,
    /// Treat file as plain vanilla markdown.
    Markdown,
}

// To do: use `OutputFormat`.
/// Output format to generate (default: `OutputFormat::Program`).
#[allow(dead_code)]
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub enum OutputFormat {
    /// The `Program` format will use import statements to import the JSX
    /// runtime (and optionally provider) and use an export statement to yield
    /// the `MDXContent` component.
    #[default]
    Program,
    /// The `FunctionBody` format will get the JSX runtime (and optionally
    /// provider) from `arguments[0]`, rewrite export statements, and use a
    /// return statement to yield what was exported.
    /// Normally, this output format will throw on `import` (and
    /// `export … from`) statements, but you can support them by setting
    /// `options.useDynamicImport`.
    FunctionBody,
}

/// Configuration (optional).
#[derive(Clone, Debug, Default)]
pub struct Options {
    // /// List of markdown extensions, with dot.
    // ///
    // /// Default: `vec![".md".into(), ".markdown".into(), ".mdown".into(), ".mkdn".into(), ".mkd".into(), ".mdwn".into(), ".mkdown".into(), ".ron".into()]`.
    // pub md_extensions: Option<Vec<String>>,
    // /// List of MDX extensions, with dot.
    // ///
    // /// Default: `vec![".mdx".into()]`.
    // pub mdx_extensions: Option<Vec<String>>,
    // /// Format the file is in (default: `Format::Detect`).
    // pub format: Format,
    // /// To do: support `output_format: FunctionBody
    // /// Output format to generate (default: `OutputFormat::Program`).
    // ///
    // /// In most cases `OutputFormat::Program` should be used, as it results in a
    // /// whole program.
    // /// `OutputFormat::FunctionBody` can be used to compile to code that can be
    // /// `eval`ed.
    // /// In some cases, you might want to do that, such as when compiling on the
    // /// server and running on the client.
    // pub output_format: OutputFormat,
    // /// Whether to compile to dynamic import expressions (default:
    // /// `false`).
    // ///
    // /// This option applies when `options.output_format` is
    // /// `OutputFormat::FunctionBody`.
    // ///
    // /// This project can turn import statements (`import x from 'y'`) into
    // /// dynamic imports (`const {x} = await import('y')`).
    // /// This is useful because import statements only work at the top level of
    // /// JavaScript modules, whereas `import()` is available inside function
    // /// bodies.
    // ///
    // /// When you turn `use_dynamic_import` on, you should probably set
    // /// `options.base_url` too.
    // pub use_dynamic_import: bool,
    // /// Resolve `import`s (and `export … from`, `import.meta.url`) from this
    // /// URL (default: `None`, example: `Some("https://example.com/".into())`).
    // ///
    // /// Relative specifiers are non-absolute URLs that start with `/`, `./`, or
    // /// `../`.
    // /// For example: `/index.js`, `./folder/file.js`, or `../main.js`.
    // ///
    // /// This option is useful when code will run in a different place.
    // /// One example is when `.mdx` files are in path *a* but compiled to path
    // /// *b* and imports should run relative the path *b*.
    // /// Another example is when evaluating code, whether in Node or a browser.
    // pub base_url: Option<String>,
    /// Whether to add extra information to error messages in generated code
    /// (default: `false`).
    pub development: bool,

    // To do: some alternative to generate source maps.
    // SourceMapGenerator
    /// Place to import a provider from (default: `None`, example:
    /// `Some("@mdx-js/react").into()`).
    ///
    /// Useful for runtimes that support context (React, Preact).
    /// The provider must export a `useMDXComponents`, which is called to
    /// access an object of components.
    pub provider_import_source: Option<String>,

    /// Whether to keep JSX (default: `false`).
    ///
    /// The default is to compile JSX away so that the resulting file is
    /// immediately runnable.
    pub jsx: bool,

    /// JSX runtime to use (default: `Some(JsxRuntime::Automatic)`).
    ///
    /// The classic runtime compiles to calls such as `h('p')`, the automatic
    /// runtime compiles to
    /// `import _jsx from '$importSource/jsx-runtime'\n_jsx('p')`.
    pub jsx_runtime: Option<JsxRuntime>,

    /// Place to import automatic JSX runtimes from (`Option<String>`, default:
    /// `Some("react".into())`).
    ///
    /// When in the automatic runtime, this is used to define an import for
    /// `_Fragment`, `_jsx`, and `_jsxs`.
    pub jsx_import_source: Option<String>,

    /// Pragma for JSX (default: `Some("React.createElement".into())`).
    ///
    /// When in the classic runtime, this is used as an identifier for function
    /// calls: `<x />` to `React.createElement('x')`.
    ///
    /// You should most probably define `pragma_frag` and `pragma_import_source`
    /// too when changing this.
    pub pragma: Option<String>,

    /// Pragma for JSX fragments (default: `Some("React.Fragment".into())`).
    ///
    /// When in the classic runtime, this is used as an identifier for
    /// fragments: `<>` to `React.createElement(React.Fragment)`.
    ///
    /// You should most probably define `pragma` and `pragma_import_source`
    /// too when changing this.
    pub pragma_frag: Option<String>,

    /// Where to import the identifier of `pragma` from (default:
    /// `Some("react".into())`).
    ///
    /// When in the classic runtime, this is used to import the `pragma`
    /// function.
    /// To illustrate with an example: when `pragma` is `"a.b"` and
    /// `pragma_import_source` is `"c"`, the following will be generated:
    /// `import a from 'c'`.
    pub pragma_import_source: Option<String>,
}

#[allow(dead_code)]
pub fn mdx(value: &str, filepath: Option<String>, options: &Options) -> Result<String, String> {
    let parse_options = ParseOptions {
        constructs: Constructs::mdx(),
        mdx_esm_parse: Some(Box::new(parse_esm)),
        mdx_expression_parse: Some(Box::new(parse_expression)),
        ..ParseOptions::default()
    };
    let document_options = DocumentOptions {
        pragma: options.pragma.clone(),
        pragma_frag: options.pragma_frag.clone(),
        pragma_import_source: options.pragma_import_source.clone(),
        jsx_import_source: options.jsx_import_source.clone(),
        jsx_runtime: options.jsx_runtime,
    };
    let rewrite_options = RewriteOptions {
        development: options.development,
        provider_import_source: options.provider_import_source.clone(),
    };

    let location = Location::new(value.as_bytes());
    let mdast = to_mdast(value, &parse_options)?;
    let hast = mdast_util_to_hast(&mdast);
    let mut program = hast_util_to_swc(&hast, filepath, Some(&location))?;
    mdx_plugin_recma_document(&mut program, &document_options, Some(&location))?;
    mdx_plugin_recma_jsx_rewrite(&mut program, &rewrite_options, Some(&location));

    // Add our comments.
    let comments = SingleThreadedComments::default();

    for c in program.comments {
        comments.add_leading(c.span.lo, c);
    }

    println!("comments for swc: {:?}", comments);

    if !options.jsx {
        // let cm = Lrc::new(SourceMap::new(FilePathMapping::empty()));
        // let build_options = JsxBuildOptions {
        //     next: Some(true),
        //     throw_if_namespace: Some(false),
        //     development: Some(options.development),
        //     use_spread: Some(true),
        //     refresh: None,
        //     runtime: None,
        //     import_source: None,
        //     pragma: None,
        //     pragma_frag: None,
        //     use_builtins: None,
        // };
        // let mark = Mark::fresh(Mark::default());
        // program
        //     .module
        //     .visit_mut_with(&mut jsx(cm, Some(comments), build_options, mark));
    }

    // To do: include comments.
    Ok(serialize(&program.module))
}
