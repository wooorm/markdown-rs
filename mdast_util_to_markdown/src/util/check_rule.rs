use alloc::format;

use crate::{message::Message, state::State};

pub fn check_rule(state: &State) -> Result<char, Message> {
    let marker = state.options.rule;

    if marker != '*' && marker != '-' && marker != '_' {
        return Err(Message {
            reason: format!(
                "Cannot serialize rules with `{}` for `options.rule`, expected `*`, `-`, or `_`",
                marker
            ),
        });
    }

    Ok(marker)
}
