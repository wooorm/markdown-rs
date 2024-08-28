use regex::Regex;

use crate::{mdast::Node, to_markdown::State};

pub fn format_code_as_indented(node: &Node, _state: &State) -> bool {
    match node {
        Node::Code(code) => {
            let white_space = Regex::new(r"[^ \r\n]").unwrap();
            let blank = Regex::new(r"^[\t ]*(?:[\r\n]|$)|(?:^|[\r\n])[\t ]*$").unwrap();
            !code.value.is_empty()
                && code.lang.is_none()
                && white_space.is_match(&code.value)
                && !blank.is_match(&code.value)
        }
        _ => false,
    }
}
