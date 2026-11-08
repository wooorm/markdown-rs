//! Move across lists of events.

use crate::event::{Event, Kind, Name};
use crate::extension::{is_in_content, TokenName};

/// Skip from `index`, optionally past `names`.
pub fn opt(events: &[Event], index: usize, names: &[Name]) -> usize {
    skip_opt_impl(events, index, names, true)
}

/// Skip from `index`, optionally past `names`, backwards.
pub fn opt_back(events: &[Event], index: usize, names: &[Name]) -> usize {
    skip_opt_impl(events, index, names, false)
}

/// Skip from `index`, optionally past `names` and tokens of constructs in
/// content, such as the prefixes of containers of constructs.
pub(crate) fn opt_with_extensions(
    events: &[Event],
    index: usize,
    names: &[Name],
    extension_names: &[TokenName],
) -> usize {
    skip_opt_with_extensions(events, index, names, extension_names, true)
}

/// Skip from `index`, optionally past `names` and tokens of constructs in
/// content, backwards.
pub(crate) fn opt_back_with_extensions(
    events: &[Event],
    index: usize,
    names: &[Name],
    extension_names: &[TokenName],
) -> usize {
    skip_opt_with_extensions(events, index, names, extension_names, false)
}

/// Skip from `index` forwards to `names`.
pub fn to(events: &[Event], index: usize, names: &[Name]) -> usize {
    to_impl(events, index, names, true)
}

/// Skip from `index` backwards to `names`.
pub fn to_back(events: &[Event], index: usize, names: &[Name]) -> usize {
    to_impl(events, index, names, false)
}

/// Skip to something.
fn to_impl(events: &[Event], mut index: usize, names: &[Name], forward: bool) -> usize {
    while index < events.len() {
        let current = &events[index].name;

        if names.contains(current) {
            break;
        }

        index = if forward { index + 1 } else { index - 1 };
    }

    index
}

/// Skip past things, and tokens of constructs in content, one at a time,
/// since the token after one can be of another match.
fn skip_opt_with_extensions(
    events: &[Event],
    mut index: usize,
    names: &[Name],
    extension_names: &[TokenName],
    forward: bool,
) -> usize {
    // Without constructs, as fast as without this.
    if extension_names.is_empty() {
        return skip_opt_impl(events, index, names, forward);
    }

    let open = if forward { Kind::Enter } else { Kind::Exit };

    loop {
        index = skip_opt_impl(events, index, names, forward);

        match events.get(index) {
            Some(event) if event.kind == open && is_in_content(extension_names, event) => {
                let mut balance = 0;
                loop {
                    balance += if events[index].kind == open { 1 } else { -1 };
                    index = if forward { index + 1 } else { index - 1 };
                    if balance == 0 {
                        break;
                    }
                }
            }
            _ => return index,
        }
    }
}

/// Skip past things.
fn skip_opt_impl(events: &[Event], mut index: usize, names: &[Name], forward: bool) -> usize {
    let mut balance = 0;
    let open = if forward { Kind::Enter } else { Kind::Exit };

    while index < events.len() {
        let current = &events[index].name;

        if !names.contains(current) || events[index].kind != open {
            break;
        }

        index = if forward { index + 1 } else { index - 1 };
        balance += 1;

        loop {
            balance = if events[index].kind == open {
                balance + 1
            } else {
                balance - 1
            };

            let next = if forward {
                index + 1
            } else if index > 0 {
                index - 1
            } else {
                index
            };

            if events[index].name == *current && balance == 0 {
                index = next;
                break;
            }

            index = next;
        }
    }

    index
}
