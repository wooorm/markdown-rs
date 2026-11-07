//! Recipes: one complete construct for each common pattern.
//!
//! Each is a doc-only module with a construct, its tests, and why it is
//! written as it is:
//!
//! 1. [A mention][mention], `@octo-cat`: [`Reach::Bytes`][crate::extension::Reach::Bytes],
//!    and `previous` to start only after whitespace or punctuation
//! 2. [A raw span][raw_span], `{{ user.name }}`: [`Reach::Pair`][crate::extension::Reach::Pair],
//!    on one line, and code spans around it
//! 3. [A span with flanking rules][flanking_span], `$x^2$`: a closer that
//!    looks past itself with an attempt
//! 4. [An inline directive][inline_directive], `:abbr[HTML]{title="…"}`: a
//!    label of markdown with nested brackets, and optional attributes
//! 5. [A delimiter run][delimiter_run], `==mark==`:
//!    [`Reach::Attention`][crate::extension::Reach::Attention]
//! 6. [A block of one line][line_block], `:: note Read this`: a flow
//!    construct
//! 7. [A fenced container][fenced_container], `:::` around markdown: a flow
//!    construct with a body that is a document
//! 8. [A container with a prefix][prefix_container], an admonition with an
//!    indented body
//!
//! Two more, which need `markdown_processor`, are in its `guide` module:
//! rendering one construct three ways (recipe 9), and a transform after
//! parsing (recipe 10).
//! The last, recipe 11, is [testing a construct][testing]: rule panics with
//! `test-util`, and `assert_linear`.

pub mod delimiter_run;
pub mod fenced_container;
pub mod flanking_span;
pub mod inline_directive;
pub mod line_block;
pub mod mention;
pub mod prefix_container;
pub mod raw_span;
pub mod testing;
