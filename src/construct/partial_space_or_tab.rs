//! Several helpers to parse whitespace (`space_or_tab`).
//!
//! ## References
//!
//! *   [`micromark-factory-space/index.js` in `micromark`](https://github.com/micromark/micromark/blob/main/packages/micromark-factory-space/dev/index.js)

use crate::tokenizer::{Code, State, StateFn, StateFnResult, TokenType, Tokenizer};

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
    Box::new(move |t, c| start(t, c, kind, min, max))
}

/// Before whitespace.
///
/// ```markdown
/// alpha| bravo
/// ```
fn start(
    tokenizer: &mut Tokenizer,
    code: Code,
    kind: TokenType,
    min: usize,
    max: usize,
) -> StateFnResult {
    match code {
        Code::VirtualSpace | Code::Char('\t' | ' ') if max > 0 => {
            tokenizer.enter(kind.clone());
            tokenizer.consume(code);
            (
                State::Fn(Box::new(move |tokenizer, code| {
                    inside(tokenizer, code, kind, min, max, 1)
                })),
                None,
            )
        }
        _ => (
            if min == 0 { State::Ok } else { State::Nok },
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
fn inside(
    tokenizer: &mut Tokenizer,
    code: Code,
    kind: TokenType,
    min: usize,
    max: usize,
    size: usize,
) -> StateFnResult {
    match code {
        Code::VirtualSpace | Code::Char('\t' | ' ') if size < max => {
            tokenizer.consume(code);
            (
                State::Fn(Box::new(move |tokenizer, code| {
                    inside(tokenizer, code, kind, min, max, size + 1)
                })),
                None,
            )
        }
        _ => {
            tokenizer.exit(kind);
            (
                if size >= min { State::Ok } else { State::Nok },
                Some(vec![code]),
            )
        }
    }
}
