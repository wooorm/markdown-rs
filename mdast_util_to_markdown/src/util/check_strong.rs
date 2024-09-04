use alloc::format;

use crate::{message::Message, state::State};

pub fn check_strong(state: &State) -> Result<char, Message> {
    let marker = state.options.strong;

    if marker != '*' && marker != '_' {
        return Err(Message {
            reason: format!(
                "Cannot serialize strong with `{}` for `options.strong`, expected `*`, or `_`",
                marker
            ),
        });
    }

    Ok(marker)
}
