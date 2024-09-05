use crate::{message::Message, state::Info, State};
use alloc::string::String;

mod r#break;
pub mod emphasis;
mod heading;
mod paragraph;
pub mod strong;
mod text;

pub trait Handle {
    fn handle(&self, state: &mut State, info: &Info) -> Result<String, Message>;
}
