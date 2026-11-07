//! Process markdown with plugins, like unified: markdown → mdast → hast → HTML.
//!
//! JS equivalent: <https://github.com/unifiedjs/unified>.

#![no_std]
#![forbid(unsafe_code)]

extern crate alloc;

pub mod guide;
mod processor;
pub mod visit;

pub use processor::{Plugin, Processor};

/// Check that `markdown::to_html_with_options` and
/// [`Processor::process`] write the same HTML for each of `inputs`, with
/// the processor's options, in the tests of a plugin whose constructs have
/// both an HTML handler and a node for hast (`test-util` feature).
///
/// Line endings are compared as `\n`, as `to_html` writes those of the
/// document, such as `\r\n`, and a processor writes `\n`; and line endings
/// at the end are left out, as `to_html` writes one after a definition at
/// the end, and a processor does not.
/// Transforms run only in a processor, so a plugin that has them can differ.
///
/// ```
/// use markdown_processor::{assert_same_html, Processor};
///
/// let mut processor = Processor::new();
/// assert_same_html(&mut processor, &["*a*", "> b"]);
/// ```
///
/// # Panics
///
/// If the two differ, or either returns an error.
#[cfg(feature = "test-util")]
pub fn assert_same_html(processor: &mut Processor, inputs: &[&str]) {
    let options = markdown::Options {
        parse: core::mem::take(&mut processor.parse),
        compile: markdown::CompileOptions {
            allow_dangerous_html: processor.compile.allow_dangerous_html,
            allow_dangerous_protocol: processor.compile.allow_dangerous_protocol,
            ..markdown::CompileOptions::default()
        },
    };
    let results: alloc::vec::Vec<_> = inputs
        .iter()
        .map(|input| markdown::to_html_with_options(input, &options))
        .collect();
    processor.parse = options.parse;

    for (input, direct) in inputs.iter().zip(results) {
        let direct = direct.expect("expected `to_html_with_options` to work");
        let processed = processor
            .process(input)
            .expect("expected `Processor::process` to work");
        let normalize = |html: &str| html.replace("\r\n", "\n").replace('\r', "\n");
        assert_eq!(
            normalize(direct.trim_end()),
            normalize(processed.trim_end()),
            "expected the same HTML from `to_html_with_options` (left) and `Processor::process` (right) for {:?}",
            input
        );
    }
}

/// GFM, as a plugin: turns on the GFM constructs.
///
/// `hast_util_to_html` has no tag filter, so raw HTML, when allowed, is written
/// as is.
#[derive(Clone, Copy, Debug, Default)]
pub struct Gfm;

impl Plugin for Gfm {
    fn attach(self, processor: &mut Processor) {
        let constructs = &mut processor.parse.constructs;
        constructs.gfm_autolink_literal = true;
        constructs.gfm_footnote_definition = true;
        constructs.gfm_label_start_footnote = true;
        constructs.gfm_strikethrough = true;
        constructs.gfm_table = true;
        constructs.gfm_task_list_item = true;
    }
}
