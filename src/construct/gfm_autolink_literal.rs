//! To do.

use crate::event::{Event, Kind, Name};
use crate::tokenizer::Tokenizer;
use crate::util::classify_character::{classify, Kind as CharacterKind};
use crate::util::slice::{Position, Slice};
use alloc::vec::Vec;
use core::str;

// To do: doc al functions.

pub fn resolve(tokenizer: &mut Tokenizer) {
    tokenizer.map.consume(&mut tokenizer.events);

    let mut index = 0;
    let mut links = 0;

    while index < tokenizer.events.len() {
        let event = &tokenizer.events[index];

        if event.kind == Kind::Enter {
            if event.name == Name::Link {
                links += 1;
            }
        } else {
            if event.name == Name::Data && links == 0 {
                let slice = Slice::from_position(
                    tokenizer.parse_state.bytes,
                    &Position::from_exit_event(&tokenizer.events, index),
                );
                let bytes = slice.bytes;
                let mut byte_index = 0;
                let mut replace = Vec::new();
                let mut point = tokenizer.events[index - 1].point.clone();
                let start_index = point.index;
                let mut start = 0;

                while byte_index < bytes.len() {
                    if matches!(bytes[byte_index], b'H' | b'h' | b'W' | b'w' | b'@') {
                        if let Some(autolink) = peek(bytes, byte_index) {
                            byte_index = autolink.1;

                            // If there is something between the last link
                            // (or the start) and this link.
                            if start != autolink.0 {
                                replace.push(Event {
                                    kind: Kind::Enter,
                                    name: Name::Data,
                                    point: point.clone(),
                                    link: None,
                                });
                                point = point.shift_to(
                                    tokenizer.parse_state.bytes,
                                    start_index + autolink.0,
                                );
                                replace.push(Event {
                                    kind: Kind::Exit,
                                    name: Name::Data,
                                    point: point.clone(),
                                    link: None,
                                });
                            }

                            // Add the link.
                            replace.push(Event {
                                kind: Kind::Enter,
                                name: autolink.2.clone(),
                                point: point.clone(),
                                link: None,
                            });
                            point = point
                                .shift_to(tokenizer.parse_state.bytes, start_index + autolink.1);
                            replace.push(Event {
                                kind: Kind::Exit,
                                name: autolink.2.clone(),
                                point: point.clone(),
                                link: None,
                            });
                            start = autolink.1;
                        }
                    }

                    byte_index += 1;
                }

                // If there was a link, and we have more bytes left.
                if start != 0 && start < bytes.len() {
                    replace.push(Event {
                        kind: Kind::Enter,
                        name: Name::Data,
                        point: point.clone(),
                        link: None,
                    });
                    replace.push(Event {
                        kind: Kind::Exit,
                        name: Name::Data,
                        point: event.point.clone(),
                        link: None,
                    });
                }

                // If there were links.
                if !replace.is_empty() {
                    tokenizer.map.add(index - 1, 2, replace);
                }
            }

            if event.name == Name::Link {
                links -= 1;
            }
        }

        index += 1;
    }
}

fn peek(bytes: &[u8], index: usize) -> Option<(usize, usize, Name)> {
    // Protocol.
    if let Some(protocol_end) = peek_protocol(bytes, index) {
        if let Some(domain_end) = peek_domain(bytes, protocol_end, true) {
            let end = truncate(bytes, protocol_end, domain_end);

            // Cannot be empty.
            if end != protocol_end {
                return Some((index, end, Name::GfmAutolinkLiteralProtocol));
            }
        }
    }

    // Www.
    if peek_www(bytes, index).is_some() {
        // Note: we discard the `www.` we parsed, we now try to parse it as a domain.
        let domain_end = peek_domain(bytes, index, false).unwrap_or(index);
        let end = truncate(bytes, index, domain_end);
        return Some((index, end, Name::GfmAutolinkLiteralWww));
    }

    // Email.
    if bytes[index] == b'@' {
        if let Some(start) = peek_atext(bytes, index) {
            if let Some(end) = peek_email_domain(bytes, index + 1) {
                let end = truncate(bytes, start, end);
                return Some((start, end, Name::GfmAutolinkLiteralEmail));
            }
        }
    }

    None
}

/// Move past `http://`, `https://`, case-insensitive.
fn peek_protocol(bytes: &[u8], mut index: usize) -> Option<usize> {
    // `http`
    if index + 3 < bytes.len()
        && matches!(bytes[index], b'H' | b'h')
        && matches!(bytes[index + 1], b'T' | b't')
        && matches!(bytes[index + 2], b'T' | b't')
        && matches!(bytes[index + 3], b'P' | b'p')
    {
        index += 4;

        // `s`, optional.
        if index + 1 < bytes.len() && matches!(bytes[index], b'S' | b's') {
            index += 1;
        }

        // `://`
        if index + 3 < bytes.len()
            && bytes[index] == b':'
            && bytes[index + 1] == b'/'
            && bytes[index + 2] == b'/'
        {
            return Some(index + 3);
        }
    }

    None
}

/// Move past `www.`, case-insensitive.
fn peek_www(bytes: &[u8], index: usize) -> Option<usize> {
    // `www.`
    if index + 3 < bytes.len()
        // Source: <https://github.com/github/cmark-gfm/blob/ef1cfcb/extensions/autolink.c#L156>.
        && (index == 0 || matches!(bytes[index - 1], b'\t' | b'\n' | b'\r' | b' ' | b'(' | b'*' | b'_' | b'~'))
        && matches!(bytes[index], b'W' | b'w')
        && matches!(bytes[index + 1], b'W' | b'w')
        && matches!(bytes[index + 2], b'W' | b'w')
        && bytes[index + 3] == b'.'
    {
        Some(index + 4)
    } else {
        None
    }
}

/// Move past `example.com`.
fn peek_domain(bytes: &[u8], start: usize, allow_short: bool) -> Option<usize> {
    let mut dots = false;
    let mut penultime = false;
    let mut last = false;
    // To do: expose this from slice?
    // To do: do it ourselves? <https://github.com/commonmark/cmark/blob/8a023286198a7e408398e282f293e3b0baebb644/src/utf8.c#L150>, <https://doc.rust-lang.org/core/str/fn.next_code_point.html>, <https://www.reddit.com/r/rust/comments/4g2zu0/lazy_unicode_iterator_from_byte_iteratorslice/>, <http://bjoern.hoehrmann.de/utf-8/decoder/dfa/>.
    let char_indices = str::from_utf8(&bytes[start..])
        .unwrap()
        .char_indices()
        .collect::<Vec<_>>();
    let mut index = 0;

    while index < char_indices.len() {
        match char_indices[index].1 {
            '_' => last = true,
            '.' => {
                penultime = last;
                last = false;
                dots = true;
            }
            '-' => {}
            // Source: <https://github.com/github/cmark-gfm/blob/ef1cfcb/extensions/autolink.c#L12>.
            char if classify(char) == CharacterKind::Other => {}
            _ => break,
        }

        index += 1;
    }

    // No underscores allowed in last two parts.
    // A valid domain needs to have at least a dot.
    if penultime || last || (!allow_short && !dots) {
        None
    } else {
        // Now peek past `/path?search#hash` (anything except whitespace).
        while index < char_indices.len() {
            if classify(char_indices[index].1) == CharacterKind::Whitespace {
                break;
            }

            index += 1;
        }

        Some(if index == char_indices.len() {
            bytes.len()
        } else {
            start + char_indices[index].0
        })
    }
}

/// Move back past `contact`.
fn peek_atext(bytes: &[u8], end: usize) -> Option<usize> {
    let mut index = end;

    // Take simplified atext.
    // See `email_atext` in `autolink.rs` for a similar algorithm.
    // Source: <https://github.com/github/cmark-gfm/blob/ef1cfcb/extensions/autolink.c#L301>.
    while index > 0
        && matches!(bytes[index - 1], b'+' | b'-' | b'.' | b'0'..=b'9' | b'A'..=b'Z' | b'_' | b'a'..=b'z')
    {
        index -= 1;
    }

    // Do not allow a slash “inside” atext.
    // The reference code is a bit weird, but that’s what it results in.
    // Source: <https://github.com/github/cmark-gfm/blob/ef1cfcb/extensions/autolink.c#L307>.
    // Other than slash, every preceding character is allowed.
    if index == end || (index > 0 && bytes[index - 1] == b'/') {
        None
    } else {
        Some(index)
    }
}

/// Move past `example.com`.
fn peek_email_domain(bytes: &[u8], start: usize) -> Option<usize> {
    let mut index = start;
    let mut dot = false;

    // Move past “domain”.
    // The reference code is a bit overly complex as it handles the `@`, of which there may be just one.
    // Source: <https://github.com/github/cmark-gfm/blob/ef1cfcb/extensions/autolink.c#L318>
    while index < bytes.len() {
        match bytes[index] {
            // Alphanumerical, `-`, and `_`.
            b'-' | b'0'..=b'9' | b'A'..=b'Z' | b'_' | b'a'..=b'z' => {}
            // Dot followed by alphanumerical (not `-` or `_`).
            b'.' if index + 1 < bytes.len()
                && matches!(bytes[index + 1], b'0'..=b'9' | b'A'..=b'Z' | b'a'..=b'z') =>
            {
                dot = true;
            }
            _ => break,
        }

        index += 1;
    }

    // Domain must not be empty, must include a dot, and must end in alphabetical or `.`.
    // Source: <https://github.com/github/cmark-gfm/blob/ef1cfcb/extensions/autolink.c#L332>.
    if index > start && dot && matches!(bytes[index - 1], b'.' | b'A'..=b'Z' | b'a'..=b'z') {
        Some(index)
    } else {
        None
    }
}

/// Split trialing stuff from a URL.
fn truncate(bytes: &[u8], start: usize, mut end: usize) -> usize {
    let mut index = start;

    // Source: <https://github.com/github/cmark-gfm/blob/ef1cfcb/extensions/autolink.c#L42>
    while index < end {
        if bytes[index] == b'<' {
            end = index;
            break;
        }
        index += 1;
    }

    let mut split = end;

    // Move before trailing punctuation.
    while split > start {
        match bytes[split - 1] {
            b'!' | b'"' | b'&' | b'\'' | b')' | b',' | b'.' | b':' | b'<' | b'>' | b'?' | b']'
            | b'}' => {}
            // Source: <https://github.com/github/cmark-gfm/blob/ef1cfcb/extensions/autolink.c#L61>.
            // Note: we can’t move across actual references, because those have been parsed already.
            b';' => {
                let mut new_split = split - 1;
                // Move back past alphabeticals.
                while new_split > start && matches!(bytes[new_split - 1], b'A'..=b'Z' | b'a'..=b'z')
                {
                    new_split -= 1;
                }

                // Nonempty character reference:
                if new_split > start && bytes[new_split - 1] == b'&' && new_split < split - 1 {
                    split = new_split - 1;
                    continue;
                }

                // Otherwise it’s just a `;`.
            }
            _ => break,
        }
        split -= 1;
    }

    // If there was trailing punctuation, try to balance parens.
    if split != end {
        let mut open = 0;
        let mut close = 0;
        let mut paren_index = start;

        // Count parens in `url` (not in trail).
        while paren_index < split {
            match bytes[paren_index] {
                b'(' => open += 1,
                b')' => close += 1,
                _ => {}
            }

            paren_index += 1;
        }

        let mut trail_index = split;

        // If there are more opening than closing parens, try to balance them
        // from the trail.
        while open > close && trail_index < end {
            if bytes[trail_index] == b')' {
                split = trail_index;
                close += 1;
            }

            trail_index += 1;
        }
    }

    split
}
