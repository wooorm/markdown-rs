//! Guide: syntax extensions in a processor.
//!
//! A [`Processor`][crate::Processor] takes the constructs of
//! [`markdown::extension`], with
//! [`add_text_construct`][crate::Processor::add_text_construct] and its
//! siblings, and turns their nodes into HTML through hast, not through
//! [`Construct::html`][markdown::extension::Construct::html].
//! Start with [`markdown::extension::guide`], which builds constructs; the
//! recipes here are about what happens to their nodes after parsing.
//!
//! * [Recipe 9][rendering]: one construct rendered three ways
//! * [Recipe 10][transform]: a transform that changes nodes after parsing

pub mod rendering;
pub mod transform;
