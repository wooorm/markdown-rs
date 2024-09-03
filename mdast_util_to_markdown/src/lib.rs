#![no_std]

use alloc::string::String;
use markdown::mdast::Node;

extern crate alloc;
mod configure;
pub mod parents;
mod to_markdown;
pub mod r#unsafe;

#[allow(dead_code)]
#[derive(Clone, PartialEq)]
pub enum ConstructName {
    Autolink,
    Blockquote,
    CodeIndented,
    CodeFenced,
    CodeFencedLangGraveAccent,
    CodeFencedLangTilde,
    CodeFencedMetaGraveAccent,
    CodeFencedMetaTilde,
    Definition,
    DestinationLiteral,
    DestinationRaw,
    Emphasis,
    HeadingAtx,
    HeadingSetext,
    Image,
    ImageReference,
    Label,
    Link,
    LinkReference,
    List,
    ListItem,
    Paragraph,
    Phrasing,
    Reference,
    Strong,
    TitleApostrophe,
    TitleQuote,
}

pub fn to_markdown(tree: &Node) -> String {
    to_markdown::serialize(tree)
}
