use alloc::format;

use crate::{message::Message, state::State};

pub fn check_bullet_ordered(state: &mut State) -> Result<char, Message> {
    let marker = state.options.bullet_ordered;

    if marker != '.' && marker != ')' {
        return Err(Message {
            reason: format!(
                "Cannot serialize items with `' {} '` for `options.bullet_ordered`, expected `.` or `)`",
                marker
            ),
        });
    }

    Ok(marker)
}
