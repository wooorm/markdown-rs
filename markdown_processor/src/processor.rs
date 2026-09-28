//! The processor and the plugin trait.

use alloc::{boxed::Box, string::String, vec::Vec};
use core::fmt;
use markdown::{mdast, message::Message, to_mdast, ParseOptions};
use mdast_util_to_hast::to_hast_with_options;

/// A plugin: attaches parser config, transforms, and handlers to a processor.
pub trait Plugin {
    /// Configure `processor`.
    fn attach(self, processor: &mut Processor);
}

/// Any `FnOnce(&mut Processor)` is a plugin, like an inline unified attacher.
impl<F: FnOnce(&mut Processor)> Plugin for F {
    fn attach(self, processor: &mut Processor) {
        self(processor);
    }
}

type MdastTransform = Box<dyn Fn(&mut mdast::Node) -> Result<(), Message>>;
type HastTransform = Box<dyn Fn(&mut hast::Node) -> Result<(), Message>>;

/// Markdown → mdast → hast → HTML, with plugins.
pub struct Processor {
    /// Parse options; plugins such as [`Gfm`][crate::Gfm] change them.
    pub parse: ParseOptions,
    /// Serialize options: whether raw HTML and any URL protocol are allowed.
    pub compile: hast_util_to_html::Options,
    mdast_transforms: Vec<MdastTransform>,
    hast_transforms: Vec<HastTransform>,
    to_hast: mdast_util_to_hast::Options,
}

impl Processor {
    /// Create a `CommonMark` processor without plugins.
    pub fn new() -> Self {
        Processor {
            parse: ParseOptions::default(),
            compile: hast_util_to_html::Options::default(),
            mdast_transforms: Vec::new(),
            hast_transforms: Vec::new(),
            to_hast: mdast_util_to_hast::Options::default(),
        }
    }

    /// Attach a plugin, right away: a plugin attached twice runs its setup twice.
    #[must_use]
    pub fn plugin(mut self, plugin: impl Plugin) -> Self {
        plugin.attach(&mut self);
        self
    }

    /// Add a transform that runs on mdast, in the order added.
    pub fn add_mdast_transform(
        &mut self,
        transform: impl Fn(&mut mdast::Node) -> Result<(), Message> + 'static,
    ) {
        self.mdast_transforms.push(Box::new(transform));
    }

    /// Add a transform that runs on hast, in the order added.
    pub fn add_hast_transform(
        &mut self,
        transform: impl Fn(&mut hast::Node) -> Result<(), Message> + 'static,
    ) {
        self.hast_transforms.push(Box::new(transform));
    }

    /// Set how custom nodes named `name` turn into hast, from their converted
    /// children; a later handler for a name replaces an earlier one.
    pub fn add_hast_handler(
        &mut self,
        name: &str,
        handler: impl Fn(&mdast::Custom, Vec<hast::Node>) -> Vec<hast::Node> + 'static,
    ) {
        self.to_hast.handlers.insert(name.into(), Box::new(handler));
    }

    /// Turn markdown into HTML.
    ///
    /// MDX nodes are dropped, with their children: MDX compiles to JavaScript,
    /// not HTML.
    ///
    /// ## Errors
    ///
    /// Errors when parsing fails (only with MDX) or when a transform fails.
    pub fn process(&self, value: &str) -> Result<String, Message> {
        let mut mdast = to_mdast(value, &self.parse)?;
        for transform in &self.mdast_transforms {
            transform(&mut mdast)?;
        }
        let mut hast = to_hast_with_options(&mdast, &self.to_hast);
        for transform in &self.hast_transforms {
            transform(&mut hast)?;
        }
        Ok(hast_util_to_html::to_html_with_options(
            &hast,
            &self.compile,
        ))
    }
}

impl fmt::Debug for Processor {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("Processor")
            .field("parse", &self.parse)
            .field("compile", &self.compile)
            .field("mdast_transforms", &self.mdast_transforms.len())
            .field("hast_transforms", &self.hast_transforms.len())
            .field("to_hast", &self.to_hast)
            .finish()
    }
}

impl Default for Processor {
    fn default() -> Self {
        Self::new()
    }
}
