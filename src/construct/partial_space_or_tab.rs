//! Several helpers to parse whitespace (`space_or_tab`).
//!
//! ## References
//!
//! *   [`micromark-factory-space/index.js` in `micromark`](https://github.com/micromark/micromark/blob/main/packages/micromark-factory-space/dev/index.js)

use crate::tokenizer::{Code, State, StateFn, StateFnResult, TokenType, Tokenizer};

/// Options to parse whitespace.
#[derive(Debug)]
pub struct Options {
    /// Minimum allowed characters (inclusive).
    pub min: usize,
    /// Maximum allowed characters (inclusive).
    pub max: usize,
    /// Token type to use for whitespace events.
    pub kind: TokenType,
}

/// Options to parse whitespace.
#[derive(Debug)]
struct Info {
    /// Current size.
    size: usize,
    /// Configuration.
    options: Options,
}

/// One or more `space_or_tab`.
///
/// ```bnf
/// space_or_tab ::= 1*( ' ' '\t' )
/// ```
pub fn space_or_tab() -> Box<StateFn> {
    space_or_tab_min_max(1, usize::MAX)
}

/// Between `x` and `y` `space_or_tab`
///
/// ```bnf
/// space_or_tab_min_max ::= x*y( ' ' '\t' )
/// ```
pub fn space_or_tab_min_max(min: usize, max: usize) -> Box<StateFn> {
    space_or_tab_with_options(Options {
        kind: TokenType::SpaceOrTab,
        min,
        max,
    })
}

/// Between `x` and `y` `space_or_tab`, with the given token type.
///
/// ```bnf
/// space_or_tab ::= x*y( ' ' '\t' )
/// ```
pub fn space_or_tab_with_options(options: Options) -> Box<StateFn> {
    Box::new(|t, c| start(t, c, Info { size: 0, options }))
}

/// Before whitespace.
///
/// ```markdown
/// alpha| bravo
/// ```
fn start(tokenizer: &mut Tokenizer, code: Code, mut info: Info) -> StateFnResult {
    match code {
        Code::VirtualSpace | Code::Char('\t' | ' ') if info.options.max > 0 => {
            tokenizer.enter(info.options.kind.clone());
            tokenizer.consume(code);
            info.size += 1;
            (State::Fn(Box::new(|t, c| inside(t, c, info))), None)
        }
        _ => (
            if info.options.min == 0 {
                State::Ok
            } else {
                State::Nok
            },
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
        Code::VirtualSpace | Code::Char('\t' | ' ') if info.size < info.options.max => {
            tokenizer.consume(code);
            info.size += 1;
            (State::Fn(Box::new(|t, c| inside(t, c, info))), None)
        }
        _ => {
            tokenizer.exit(info.options.kind.clone());
            (
                if info.size >= info.options.min {
                    State::Ok
                } else {
                    State::Nok
                },
                Some(vec![code]),
            )
        }
    }
}
