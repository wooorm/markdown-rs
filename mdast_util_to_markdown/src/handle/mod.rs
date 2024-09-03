use crate::{state::Info, State};
use alloc::string::String;

mod paragraph;
pub mod strong;
mod text;

pub trait Handle {
    type Error;
    fn handle(&self, state: &mut State, info: &Info) -> Result<String, Self::Error>;
}
