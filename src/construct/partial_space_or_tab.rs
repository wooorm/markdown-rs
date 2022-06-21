//! Several helpers to parse whitespace (`space_or_tab`).
//!
//! ## References
//!
//! *   [`micromark-factory-space/index.js` in `micromark`](https://github.com/micromark/micromark/blob/main/packages/micromark-factory-space/dev/index.js)

use crate::tokenizer::{Code, State, StateFn, StateFnResult, TokenType, Tokenizer};

/// Options to parse whitespace.
#[derive(Debug)]
struct Info {
    /// Current size.
    size: usize,
    /// Minimum allowed characters (inclusive).
    min: usize,
    /// Maximum allowed characters (inclusive).
    max: usize,
    /// Token type to use for whitespace events.
    kind: TokenType,
}

/// Optional `space_or_tab`
///
/// ```bnf
/// space_or_tab_opt ::= *( ' ' '\t' )
/// ```
pub fn space_or_tab_opt() -> Box<StateFn> {
    space_or_tab_min_max(0, usize::MAX)
}

/// Between `x` and `y` `space_or_tab`
///
/// ```bnf
/// space_or_tab_min_max ::= x*y( ' ' '\t' )
/// ```
pub fn space_or_tab_min_max(min: usize, max: usize) -> Box<StateFn> {
    space_or_tab(TokenType::Whitespace, min, max)
}

/// Between `x` and `y` `space_or_tab`, with the given token type.
///
/// ```bnf
/// space_or_tab ::= x*y( ' ' '\t' )
/// ```
pub fn space_or_tab(kind: TokenType, min: usize, max: usize) -> Box<StateFn> {
    let info = Info {
        size: 0,
        min,
        max,
        kind,
    };
    Box::new(|t, c| start(t, c, info))
}

/// Before whitespace.
///
/// ```markdown
/// alpha| bravo
/// ```
fn start(tokenizer: &mut Tokenizer, code: Code, mut info: Info) -> StateFnResult {
    match code {
        Code::VirtualSpace | Code::Char('\t' | ' ') if info.max > 0 => {
            tokenizer.enter(info.kind.clone());
            tokenizer.consume(code);
            info.size += 1;
            (State::Fn(Box::new(|t, c| inside(t, c, info))), None)
        }
        _ => (
            if info.min == 0 { State::Ok } else { State::Nok },
            Some(vec![code]),
        ),
    }
}

/// In whitespace.
///
/// ```markdown
/// alpha |bravo
/// alpha | bravo
/// ```
fn inside(tokenizer: &mut Tokenizer, code: Code, mut info: Info) -> StateFnResult {
    match code {
        Code::VirtualSpace | Code::Char('\t' | ' ') if info.size < info.max => {
            tokenizer.consume(code);
            info.size += 1;
            (State::Fn(Box::new(|t, c| inside(t, c, info))), None)
        }
        _ => {
            tokenizer.exit(info.kind.clone());
            (
                if info.size >= info.min {
                    State::Ok
                } else {
                    State::Nok
                },
                Some(vec![code]),
            )
        }
    }
}
