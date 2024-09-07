use crate::{message::Message, state::Info, State};
use alloc::string::String;

mod r#break;
mod code;
pub mod emphasis;
mod heading;
pub mod html;
mod paragraph;
pub mod strong;
mod text;
mod thematic_break;

pub trait Handle {
    fn handle(&self, state: &mut State, info: &Info) -> Result<String, Message>;
}
