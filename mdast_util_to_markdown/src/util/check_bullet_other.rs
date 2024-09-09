use alloc::format;

use crate::{message::Message, state::State};

use super::check_bullet::check_bullet;

pub fn check_bullet_other(state: &mut State) -> Result<char, Message> {
    let bullet = check_bullet(state)?;
    let bullet_other = state.options.bullet_other;

    if bullet_other != '*' && bullet_other != '+' && bullet_other != '-' {
        return Err(Message {
            reason: format!(
                "Cannot serialize items with `' {} '` for `options.bullet_other`, expected `*`, `+`, or `-`",
                bullet_other
            ),
        });
    }

    if bullet_other == bullet {
        return Err(Message {
            reason: format!(
                "Expected `bullet` (`' {} '`) and `bullet_other` (`' {} '`) to be different",
                bullet, bullet_other
            ),
        });
    }

    Ok(bullet_other)
}
