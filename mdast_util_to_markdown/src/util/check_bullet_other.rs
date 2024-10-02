use alloc::{boxed::Box, format};
use markdown::message::Message;

use crate::state::State;

use super::check_bullet::check_bullet;

pub fn check_bullet_other(state: &mut State) -> Result<char, Message> {
    let bullet = check_bullet(state)?;
    let mut bullet_other = state.options.bullet_other;

    if bullet != '*' {
        bullet_other = '*';
    }

    if bullet_other != '*' && bullet_other != '+' && bullet_other != '-' {
        return Err(Message {
            reason: format!(
                "Cannot serialize items with `' {} '` for `options.bullet_other`, expected `*`, `+`, or `-`",
                bullet_other
            ),
            rule_id: Box::new("unexpected-marker".into()),
            source: Box::new("mdast-util-to-markdown".into()),
            place: None,
        });
    }

    if bullet_other == bullet {
        return Err(Message {
            reason: format!(
                "Expected `bullet` (`' {} '`) and `bullet_other` (`' {} '`) to be different",
                bullet, bullet_other
            ),
            rule_id: Box::new("bullet-match-bullet_other".into()),
            source: Box::new("mdast-util-to_markdown".into()),
            place: None,
        });
    }

    Ok(bullet_other)
}
