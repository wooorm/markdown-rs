use markdown::mdast::Node;
use regex::Regex;

use crate::state::State;

pub fn format_code_as_indented(node: &Node, _state: &State) -> bool {
    if let Node::Code(code) = node {
        let white_space = Regex::new(r"[^ \r\n]").unwrap();
        let blank = Regex::new(r"^[\t ]*(?:[\r\n]|$)|(?:^|[\r\n])[\t ]*$").unwrap();

        return !code.value.is_empty()
            && code.lang.is_none()
            && white_space.is_match(&code.value)
            && !blank.is_match(&code.value);
    }

    false
}
