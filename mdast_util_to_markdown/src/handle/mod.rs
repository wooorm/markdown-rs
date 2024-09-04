use crate::{message::Message, state::Info, State};
use alloc::string::String;

pub mod emphasis;
mod paragraph;
pub mod strong;
mod text;

pub trait Handle {
    fn handle(&self, state: &mut State, info: &Info) -> Result<String, Message>;
}
