//! Several helpers to parse whitespace (`space_or_tab`).
//!
//! ## References
//!
//! *   [`micromark-factory-space/index.js` in `micromark`](https://github.com/micromark/micromark/blob/main/packages/micromark-factory-space/dev/index.js)

use crate::subtokenize::link;
use crate::tokenizer::{Code, ContentType, State, StateFn, StateFnResult, TokenType, Tokenizer};

/// Options to parse whitespace.
#[derive(Debug)]
pub struct Options {
    /// Minimum allowed characters (inclusive).
    pub min: usize,
    /// Maximum allowed characters (inclusive).
    pub max: usize,
    /// Token type to use for whitespace events.
    pub kind: TokenType,
    /// To do.
    pub content_type: Option<ContentType>,
    pub connect: bool,
}

#[derive(Debug)]
pub struct OneLineEndingOptions {
    /// To do.
    pub content_type: Option<ContentType>,
    pub connect: bool,
}

/// Options to parse whitespace.
#[derive(Debug)]
struct OneLineInfo {
    /// Whether something was seen.
    connect: bool,
    /// Configuration.
    options: OneLineEndingOptions,
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
        content_type: None,
        connect: false,
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
            tokenizer.enter_with_content(info.options.kind.clone(), info.options.content_type);

            if info.options.content_type.is_some() {
                let index = tokenizer.events.len() - 1;
                link(&mut tokenizer.events, index);
            }

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

pub fn space_or_tab_one_line_ending() -> Box<StateFn> {
    space_or_tab_one_line_ending_with_options(OneLineEndingOptions {
        content_type: None,
        connect: false,
    })
}

pub fn space_or_tab_one_line_ending_with_options(options: OneLineEndingOptions) -> Box<StateFn> {
    Box::new(move |tokenizer, code| {
        let mut info = OneLineInfo {
            connect: false,
            options,
        };

        tokenizer.attempt(
            space_or_tab_with_options(Options {
                kind: TokenType::SpaceOrTab,
                min: 1,
                max: usize::MAX,
                content_type: info.options.content_type,
                connect: info.options.connect,
            }),
            move |ok| {
                if ok && info.options.content_type.is_some() {
                    info.connect = true;
                }

                Box::new(move |tokenizer, code| match code {
                    Code::CarriageReturnLineFeed | Code::Char('\r' | '\n') => {
                        at_eol(tokenizer, code, info)
                    }
                    _ => {
                        if ok {
                            (State::Ok, Some(vec![code]))
                        } else {
                            (State::Nok, None)
                        }
                    }
                })
            },
        )(tokenizer, code)
    })
}

fn at_eol(tokenizer: &mut Tokenizer, code: Code, mut info: OneLineInfo) -> StateFnResult {
    match code {
        Code::CarriageReturnLineFeed | Code::Char('\r' | '\n') => {
            tokenizer.enter_with_content(TokenType::LineEnding, info.options.content_type);

            if info.options.content_type.is_some() {
                if info.connect {
                    let index = tokenizer.events.len() - 1;
                    link(&mut tokenizer.events, index);
                } else {
                    info.connect = true;
                }
            }

            tokenizer.consume(code);
            tokenizer.exit(TokenType::LineEnding);
            (
                State::Fn(Box::new(tokenizer.attempt_opt(
                    space_or_tab_with_options(Options {
                        kind: TokenType::SpaceOrTab,
                        min: 1,
                        max: usize::MAX,
                        content_type: info.options.content_type,
                        connect: info.connect,
                    }),
                    after_eol,
                ))),
                None,
            )
        }
        _ => unreachable!("expected eol"),
    }
}

fn after_eol(_tokenizer: &mut Tokenizer, code: Code) -> StateFnResult {
    // Blank line not allowed.
    if matches!(
        code,
        Code::None | Code::CarriageReturnLineFeed | Code::Char('\r' | '\n')
    ) {
        (State::Nok, None)
    } else {
        (State::Ok, Some(vec![code]))
    }
}
