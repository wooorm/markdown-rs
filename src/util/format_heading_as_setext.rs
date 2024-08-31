use alloc::string::ToString;
use regex::Regex;

use crate::{mdast::Node, to_markdown::State};

pub fn format_heading_as_settext(node: &Node, _state: &State) -> bool {
    let line_break = Regex::new(r"\r?\n|\r").unwrap();
    match node {
        Node::Heading(heading) => {
            let mut literal_with_break = false;
            for child in &heading.children {
                if include_literal_with_break(child, &line_break) {
                    literal_with_break = true;
                    break;
                }
            }

            heading.depth == 0
                || heading.depth < 3 && !node.to_string().is_empty() && literal_with_break
        }
        _ => false,
    }
}

fn include_literal_with_break(node: &Node, regex: &Regex) -> bool {
    match node {
        Node::Break(_) => true,
        Node::MdxjsEsm(x) => regex.is_match(&x.value),
        Node::Toml(x) => regex.is_match(&x.value),
        Node::Yaml(x) => regex.is_match(&x.value),
        Node::InlineCode(x) => regex.is_match(&x.value),
        Node::InlineMath(x) => regex.is_match(&x.value),
        Node::MdxTextExpression(x) => regex.is_match(&x.value),
        Node::Html(x) => regex.is_match(&x.value),
        Node::Text(x) => regex.is_match(&x.value),
        Node::Code(x) => regex.is_match(&x.value),
        Node::Math(x) => regex.is_match(&x.value),
        Node::MdxFlowExpression(x) => regex.is_match(&x.value),
        _ => {
            if let Some(children) = node.children() {
                for child in children {
                    if include_literal_with_break(child, regex) {
                        return true;
                    }
                }
            }

            false
        }
    }
}
