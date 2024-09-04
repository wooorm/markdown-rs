use alloc::format;

use crate::{message::Message, state::State};

pub fn check_emphasis(state: &State) -> Result<char, Message> {
    let marker = state.options.emphasis;

    if marker != '*' && marker != '_' {
        return Err(Message {
            reason: format!(
                "Cannot serialize emphasis with `{}` for `options.emphasis`, expected `*`, or `_`",
                marker
            ),
        });
    }

    Ok(marker)
}
