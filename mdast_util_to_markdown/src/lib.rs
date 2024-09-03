#![no_std]

use alloc::string::String;
pub use configure::Options;
use markdown::mdast::Node;
use state::{Info, State};

extern crate alloc;
mod configure;
mod construct_name;
mod handle;
mod parents;
mod state;
mod r#unsafe;
mod util;

pub fn to_markdown(tree: &Node, _options: &Options) -> Result<String, String> {
    let mut state = State::new();
    let result = state.handle(tree, &Info::new("\n", "\n"))?;
    Ok(result)
}

#[cfg(test)]
mod init_tests {
    use super::*;
    use alloc::{string::String, vec};

    use markdown::mdast::{Node, Paragraph, Strong, Text};

    #[test]
    fn it_works_for_simple_text() {
        let text_a = Node::Text(Text {
            value: String::from("a"),
            position: None,
        });
        let text_b = Node::Text(Text {
            value: String::from("b"),
            position: None,
        });
        let paragraph = Node::Paragraph(Paragraph {
            children: vec![text_a, text_b],
            position: None,
        });
        let actual = to_markdown(&paragraph, &Default::default()).unwrap();
        assert_eq!(actual, String::from("ab"));
    }

    #[test]
    fn it_escape() {
        let text_a = Node::Text(Text {
            value: String::from("![](a.jpg)"),
            position: None,
        });
        let paragraph = Node::Paragraph(Paragraph {
            children: vec![text_a],
            position: None,
        });
        let actual = to_markdown(&paragraph, &Default::default()).unwrap();
        assert_eq!(actual, "!\\[]\\(a.jpg)");
    }

    #[test]
    fn it_will_strong() {
        let text_a = Node::Text(Text {
            value: String::from("a"),
            position: None,
        });

        let text_b = Node::Text(Text {
            value: String::from("b"),
            position: None,
        });
        let strong = Node::Strong(Strong {
            children: vec![text_a, text_b],
            position: None,
        });
        let actual = to_markdown(&strong, &Default::default()).unwrap();
        assert_eq!(actual, "**ab**");
    }
}
