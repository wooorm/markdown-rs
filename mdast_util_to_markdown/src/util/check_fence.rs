use alloc::format;

use crate::{message::Message, state::State};

pub fn check_fence(state: &mut State) -> Result<char, Message> {
    let marker = state.options.fence;

    if marker != '`' && marker != '~' {
        return Err(Message {
            reason: format!(
                "Cannot serialize code with `{}` for `options.fence`, expected `` ` `` or `~`",
                marker
            ),
        });
    }

    Ok(marker)
}
