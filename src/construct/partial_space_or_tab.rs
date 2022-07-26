//! Several helpers to parse whitespace (`space_or_tab`, `space_or_tab_eol`).
//!
//! ## References
//!
//! *   [`micromark-factory-space/index.js` in `micromark`](https://github.com/micromark/micromark/blob/main/packages/micromark-factory-space/dev/index.js)

use crate::subtokenize::link;
use crate::token::Token;
use crate::tokenizer::{Code, ContentType, State, StateFn, Tokenizer};

/// Options to parse `space_or_tab`.
#[derive(Debug)]
pub struct Options {
    /// Minimum allowed characters (inclusive).
    pub min: usize,
    /// Maximum allowed characters (inclusive).
    pub max: usize,
    /// Token type to use for whitespace events.
    pub kind: Token,
    /// Connect this whitespace to the previous.
    pub connect: bool,
    /// Embedded content type to use.
    pub content_type: Option<ContentType>,
}

/// Options to parse `space_or_tab` and one optional eol, but no blank line.
#[derive(Debug)]
pub struct EolOptions {
    /// Connect this whitespace to the previous.
    pub connect: bool,
    /// Embedded content type to use.
    pub content_type: Option<ContentType>,
}

/// State needed to parse `space_or_tab`.
#[derive(Debug)]
struct Info {
    /// Current size.
    size: usize,
    /// Configuration.
    options: Options,
}

/// State needed to parse `space_or_tab_eol`.
#[derive(Debug)]
struct EolInfo {
    /// Whether to connect the next whitespace to the event before.
    connect: bool,
    /// Whether there was initial whitespace.
    ok: bool,
    /// Configuration.
    options: EolOptions,
}

/// One or more `space_or_tab`.
///
/// ```bnf
/// space_or_tab ::= 1*( ' ' '\t' )
/// ```
pub fn space_or_tab() -> Box<StateFn> {
    space_or_tab_min_max(1, usize::MAX)
}

/// Between `x` and `y` `space_or_tab`.
///
/// ```bnf
/// space_or_tab_min_max ::= x*y( ' ' '\t' )
/// ```
pub fn space_or_tab_min_max(min: usize, max: usize) -> Box<StateFn> {
    space_or_tab_with_options(Options {
        kind: Token::SpaceOrTab,
        min,
        max,
        content_type: None,
        connect: false,
    })
}

/// `space_or_tab`, with the given options.
pub fn space_or_tab_with_options(options: Options) -> Box<StateFn> {
    Box::new(|t| start(t, Info { size: 0, options }))
}

/// `space_or_tab`, or optionally `space_or_tab`, one `eol`, and
/// optionally `space_or_tab`.
///
/// ```bnf
/// space_or_tab_eol ::= 1*( ' ' '\t' ) | 0*( ' ' '\t' ) eol 0*( ' ' '\t' )
/// ```
pub fn space_or_tab_eol() -> Box<StateFn> {
    space_or_tab_eol_with_options(EolOptions {
        content_type: None,
        connect: false,
    })
}

/// `space_or_tab_eol`, with the given options.
pub fn space_or_tab_eol_with_options(options: EolOptions) -> Box<StateFn> {
    Box::new(move |tokenizer| {
        let mut info = EolInfo {
            connect: options.connect,
            ok: false,
            options,
        };

        tokenizer.attempt(
            space_or_tab_with_options(Options {
                kind: Token::SpaceOrTab,
                min: 1,
                max: usize::MAX,
                content_type: info.options.content_type.clone(),
                connect: info.options.connect,
            }),
            move |ok| {
                if ok {
                    info.ok = ok;

                    if info.options.content_type.is_some() {
                        info.connect = true;
                    }
                }

                Box::new(|t| after_space_or_tab(t, info))
            },
        )(tokenizer)
    })
}

/// Before `space_or_tab`.
///
/// ```markdown
/// > | a␠␠b
///      ^
/// ```
fn start(tokenizer: &mut Tokenizer, mut info: Info) -> State {
    match tokenizer.current {
        Code::VirtualSpace | Code::Char('\t' | ' ') if info.options.max > 0 => {
            tokenizer
                .enter_with_content(info.options.kind.clone(), info.options.content_type.clone());

            if info.options.content_type.is_some() {
                let index = tokenizer.events.len() - 1;
                link(&mut tokenizer.events, index);
            }

            tokenizer.consume();
            info.size += 1;
            State::Fn(Box::new(|t| inside(t, info)))
        }
        _ => {
            if info.options.min == 0 {
                State::Ok
            } else {
                State::Nok
            }
        }
    }
}

/// In `space_or_tab`.
///
/// ```markdown
/// > | a␠␠b
///       ^
/// ```
fn inside(tokenizer: &mut Tokenizer, mut info: Info) -> State {
    match tokenizer.current {
        Code::VirtualSpace | Code::Char('\t' | ' ') if info.size < info.options.max => {
            tokenizer.consume();
            info.size += 1;
            State::Fn(Box::new(|t| inside(t, info)))
        }
        _ => {
            tokenizer.exit(info.options.kind.clone());
            if info.size >= info.options.min {
                State::Ok
            } else {
                State::Nok
            }
        }
    }
}

/// `space_or_tab_eol`: after optionally first `space_or_tab`.
///
/// ```markdown
/// > | a
///      ^
///   | b
/// ```
fn after_space_or_tab(tokenizer: &mut Tokenizer, mut info: EolInfo) -> State {
    match tokenizer.current {
        Code::CarriageReturnLineFeed | Code::Char('\n' | '\r') => {
            tokenizer.enter_with_content(Token::LineEnding, info.options.content_type.clone());

            if info.connect {
                let index = tokenizer.events.len() - 1;
                link(&mut tokenizer.events, index);
            } else if info.options.content_type.is_some() {
                info.connect = true;
            }

            tokenizer.consume();
            tokenizer.exit(Token::LineEnding);
            State::Fn(Box::new(|t| after_eol(t, info)))
        }
        _ if info.ok => State::Ok,
        _ => State::Nok,
    }
}

/// `space_or_tab_eol`: after eol.
///
/// ```markdown
///   | a
/// > | b
///     ^
/// ```
#[allow(clippy::needless_pass_by_value)]
fn after_eol(tokenizer: &mut Tokenizer, info: EolInfo) -> State {
    tokenizer.attempt_opt(
        space_or_tab_with_options(Options {
            kind: Token::SpaceOrTab,
            min: 1,
            max: usize::MAX,
            content_type: info.options.content_type,
            connect: info.connect,
        }),
        after_more_space_or_tab,
    )(tokenizer)
}

/// `space_or_tab_eol`: after more (optional) `space_or_tab`.
///
/// ```markdown
///   | a
/// > | b
///     ^
/// ```
fn after_more_space_or_tab(tokenizer: &mut Tokenizer) -> State {
    // Blank line not allowed.
    if matches!(
        tokenizer.current,
        Code::None | Code::CarriageReturnLineFeed | Code::Char('\n' | '\r')
    ) {
        State::Nok
    } else {
        State::Ok
    }
}
