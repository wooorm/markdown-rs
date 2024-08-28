//! Utilities used when processing markdown.

pub mod char;
pub mod character_reference;
pub mod constant;
pub mod edit_map;
pub mod encode;
pub mod format_code_as_indented;
pub mod format_heading_as_setext;
pub mod gfm_tagfilter;
pub mod identifier;
pub mod infer;
pub mod line_ending;
pub mod location;
pub mod mdx;
pub mod mdx_collect;
pub mod normalize_identifier;
pub mod sanitize_uri;
pub mod skip;
pub mod slice;
pub mod unicode;
