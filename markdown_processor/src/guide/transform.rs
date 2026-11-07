//! Recipe 10: a transform that changes nodes after parsing.
//!
//! Some changes depend on what is around a match, which a construct does
//! not see: a span alone in its paragraph, such as an image, can become a
//! block, here a `<figure>` with the alt text as its caption.
//! An mdast transform, added with
//! [`add_mdast_transform`][crate::Processor::add_mdast_transform], gets the
//! whole tree after parsing, and [`visit_mut`][crate::visit::visit_mut]
//! walks it.
//! The same works for the [`Custom`][markdown::mdast::Custom] node of a
//! construct: match it by its `name`.
//!
//! A transform returns a [`Message`][markdown::message::Message] to fail
//! [`process`][crate::Processor::process].
//!
//! ```
//! use markdown::mdast::{Custom, Node};
//! use markdown_processor::{visit::visit_mut, Processor};
//!
//! let processor = Processor::new().plugin(|processor: &mut Processor| {
//!     processor.add_mdast_transform(|tree| {
//!         visit_mut(tree, &mut |node| {
//!             let image = match node {
//!                 Node::Paragraph(paragraph) => match paragraph.children.as_slice() {
//!                     [Node::Image(image)] => image.clone(),
//!                     _ => return,
//!                 },
//!                 _ => return,
//!             };
//!             let caption = Custom {
//!                 name: "figcaption".into(),
//!                 h_name: Some("figcaption".into()),
//!                 value: Some(image.alt.clone()),
//!                 ..Custom::default()
//!             };
//!             *node = Node::Custom(Custom {
//!                 name: "figure".into(),
//!                 h_name: Some("figure".into()),
//!                 children: vec![Node::Image(image), Node::Custom(caption)],
//!                 ..Custom::default()
//!             });
//!         });
//!         Ok(())
//!     });
//! });
//!
//! assert_eq!(
//!     processor.process("![A cat](cat.png)").unwrap(),
//!     "<figure><img src=\"cat.png\" alt=\"A cat\" /><figcaption>A cat</figcaption></figure>"
//! );
//! // Not alone in its paragraph.
//! assert_eq!(
//!     processor.process("See ![A cat](cat.png)").unwrap(),
//!     "<p>See <img src=\"cat.png\" alt=\"A cat\" /></p>"
//! );
//! ```
