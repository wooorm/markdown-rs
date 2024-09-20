use alloc::{
    format,
    string::{String, ToString},
};
use markdown::mdast::{ListItem, Node};

use crate::{
    configure::IndentOptions,
    construct_name::ConstructName,
    message::Message,
    state::{Info, State},
    util::check_bullet::check_bullet,
};

use super::Handle;

impl Handle for ListItem {
    fn handle(
        &self,
        state: &mut State,
        _info: &Info,
        parent: Option<&Node>,
        node: &Node,
    ) -> Result<alloc::string::String, Message> {
        let list_item_indent = state.options.list_item_indent;
        let mut bullet = state
            .bullet_current
            .unwrap_or(check_bullet(state)?)
            .to_string();

        // This is equal to bullet.len() + 1, since we know bullet is always one byte long we can
        // safely assign 2 to size.
        let mut size = 2;
        if let Some(Node::List(list)) = parent {
            if list.ordered {
                let bullet_number = if let Some(start) = list.start {
                    start as usize
                } else {
                    1
                };

                if state.options.increment_list_marker {
                    if let Some(position_node) = list.children.iter().position(|x| *x == *node) {
                        bullet = format!("{}{}", bullet_number + position_node, bullet);
                    }
                } else {
                    bullet = format!("{}{}", bullet_number, bullet);
                }
            }

            size = bullet.len() + 1;

            if matches!(list_item_indent, IndentOptions::Tab)
                || (matches!(list_item_indent, IndentOptions::Mixed) && list.spread || self.spread)
            {
                size = compute_size(size);
            }
        }

        state.enter(ConstructName::ListItem);

        let value = state.container_flow(node)?;
        let value = state.indent_lines(&value, |line, index, blank| {
            if index > 0 {
                if blank {
                    String::new()
                } else {
                    let blank = " ".repeat(size);
                    let mut result = String::with_capacity(blank.len() + line.len());
                    result.push_str(&blank);
                    result.push_str(line);
                    result
                }
            } else if blank {
                bullet.clone()
            } else {
                // size - bullet.len() will never panic because size > bullet.len() always.
                let blank = " ".repeat(size - bullet.len());
                let mut result = String::with_capacity(blank.len() + line.len() + bullet.len());
                result.push_str(&bullet);
                result.push_str(&blank);
                result.push_str(line);
                result
            }
        });

        Ok(value)
    }
}

fn compute_size(a: usize) -> usize {
    ((a + 4 - 1) / 4) * 4
}
