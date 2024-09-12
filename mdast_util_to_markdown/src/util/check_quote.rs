use alloc::format;

use crate::{message::Message, state::State};

pub fn check_quote(state: &State) -> Result<char, Message> {
    let marker = state.options.quote;

    if marker != '"' && marker != '\'' {
        return Err(Message {
            reason: format!(
                "Cannot serialize title with `' {} '`  for `options.quote`, expected `\"`, or `'`",
                marker
            ),
        });
    }

    Ok(marker)
}
