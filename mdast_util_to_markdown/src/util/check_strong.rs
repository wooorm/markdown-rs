use alloc::string::String;

use crate::state::State;

pub fn check_strong(_state: &State) -> Result<char, String> {
    Ok('*')
}
