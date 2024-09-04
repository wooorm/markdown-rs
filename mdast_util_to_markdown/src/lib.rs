#![no_std]

use alloc::string::String;
pub use configure::Options;
use markdown::mdast::Node;
use message::Message;
use state::{Info, State};

extern crate alloc;
mod configure;
mod construct_name;
mod handle;
mod message;
mod parents;
mod state;
mod r#unsafe;
mod util;

pub fn to_markdown(tree: &Node) -> Result<String, Message> {
    to_markdown_with_options(tree, &Options::default())
}

pub fn to_markdown_with_options(tree: &Node, options: &Options) -> Result<String, Message> {
    let mut state = State::new(options);
    let mut result = state.handle(tree, &Info::new("\n", "\n"))?;
    if !result.is_empty() {
        let last_char = result.chars().last().unwrap();
        if last_char != '\n' && last_char != '\r' {
            result.push('\n');
        }
    }
    Ok(result)
}
