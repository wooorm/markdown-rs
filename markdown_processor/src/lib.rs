//! Process markdown with plugins, like unified: markdown → mdast → hast → HTML.
//!
//! JS equivalent: <https://github.com/unifiedjs/unified>.

#![no_std]
#![forbid(unsafe_code)]

extern crate alloc;

mod processor;
pub mod visit;

pub use processor::{Plugin, Processor};

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
