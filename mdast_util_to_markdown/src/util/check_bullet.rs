use alloc::format;

use crate::{message::Message, state::State};

pub fn check_bullet(state: &mut State) -> Result<char, Message> {
    let marker = state.options.bullet;

    if marker != '*' && marker != '+' && marker != '-' {
        return Err(Message {
            reason: format!(
                "Cannot serialize items with `' {} '` for `options.bullet`, expected `*`, `+`, or `-`",
                marker
            ),
        });
    }

    Ok(marker)
}
