//! Character classification for the CJK-friendly emphasis amendment.

use super::{
    char::{before_index, classify_opt, Kind},
    unicode_cjk::{CJK, CJK_AMBIGUOUS_PUNCTUATION},
};
use core::cmp::Ordering;

/// Whether a scalar is a CJK character, excluding default emoji presentation.
pub fn is_cjk(value: Option<char>) -> bool {
    value.map_or(false, |value| {
        let cp = u32::from(value);
        CJK.binary_search_by(|&(start, end)| {
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
/// permit an opening delimiter, but are not themselves CJK characters.
pub fn classify_before(bytes: &[u8], index: usize) -> (Kind, bool, bool) {
    let previous = before_index(bytes, index);
    if let Some(selector @ '\u{FE00}'..='\u{FE0E}') = previous {
        let base = before_index(bytes, index - selector.len_utf8());
        let kind = classify_opt(base);
        if kind != Kind::Whitespace && !is_ivs(base) {
            let fullwidth = base.map_or(false, |base| {
                CJK_AMBIGUOUS_PUNCTUATION
                    .binary_search(&(u32::from(base), u32::from(selector)))
                    .is_ok()
            });
            return (kind, is_cjk(base) || fullwidth, false);
        }
    }
    (classify_opt(previous), is_cjk(previous), is_ivs(previous))
}

/// Supplementary ideographic variation selectors are distinct from general selectors.
fn is_ivs(value: Option<char>) -> bool {
    matches!(value, Some('\u{E0100}'..='\u{E01EF}'))
}
