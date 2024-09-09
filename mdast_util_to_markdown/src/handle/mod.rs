use crate::{message::Message, state::Info, State};
use alloc::string::String;
use markdown::mdast::Node;

mod blockquote;
mod r#break;
mod code;
pub mod emphasis;
mod heading;
pub mod html;
mod list;
mod list_item;
mod paragraph;
pub mod strong;
mod text;
mod thematic_break;

pub trait Handle {
    fn handle(
        &self,
        state: &mut State,
        info: &Info,
        parent: Option<&Node>,
        _node: &Node,
    ) -> Result<String, Message>;
}
