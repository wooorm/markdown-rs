//! Character classification used by relaxed emphasis boundaries.

use super::{
    char::{before_index, classify_opt, Kind},
    unicode_emphasis::{EAST_ASIAN, FULLWIDTH_PUNCTUATION_VARIANTS},
};
use core::cmp::Ordering;

/// Whether a scalar is a East Asian character, excluding default emoji presentation.
pub fn is_east_asian(value: Option<char>) -> bool {
    value.map_or(false, |value| {
        let cp = u32::from(value);
        EAST_ASIAN
            .binary_search_by(|&(start, end)| {
                if end < cp {
                    Ordering::Less
                } else if start > cp {
                    Ordering::Greater
                } else {
                    Ordering::Equal
                }
            })
            .is_ok()
    })
}

/// Classify the preceding sequence without treating variation selectors as letters.
///
/// FE00..FE0E inherit a valid preceding base character's classification; FE0F
/// deliberately does not. Supplementary ideographic variation selectors can
/// permit an opening delimiter, but are not themselves East Asian characters.
pub fn classify_before(bytes: &[u8], index: usize) -> (Kind, bool, bool) {
    let previous = before_index(bytes, index);
    if let Some(selector @ '\u{FE00}'..='\u{FE0E}') = previous {
        let base = before_index(bytes, index - selector.len_utf8());
        let kind = classify_opt(base);
        if kind != Kind::Whitespace && !is_ivs(base) {
            let fullwidth = base.map_or(false, |base| {
                FULLWIDTH_PUNCTUATION_VARIANTS
                    .binary_search(&(u32::from(base), u32::from(selector)))
                    .is_ok()
            });
            return (kind, is_east_asian(base) || fullwidth, false);
        }
    }
    (
        classify_opt(previous),
        is_east_asian(previous),
        is_ivs(previous),
    )
}

/// Supplementary ideographic variation selectors are distinct from general selectors.
fn is_ivs(value: Option<char>) -> bool {
    matches!(value, Some('\u{E0100}'..='\u{E01EF}'))
}
