#![no_std]

use alloc::string::String;
use markdown::mdast::Node;

extern crate alloc;
mod configure;
mod to_markdown;

pub fn to_markdown(tree: &Node) -> String {
    to_markdown::serialize(tree)
}
