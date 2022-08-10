//! Several helpers to parse whitespace (`space_or_tab`, `space_or_tab_eol`).
//!
//! ## References
//!
//! *   [`micromark-factory-space/index.js` in `micromark`](https://github.com/micromark/micromark/blob/main/packages/micromark-factory-space/dev/index.js)

use crate::subtokenize::link;
use crate::token::Token;
use crate::tokenizer::{ContentType, State, StateName, Tokenizer};

/// Options to parse `space_or_tab`.
#[derive(Debug)]
pub struct Options {
    /// Minimum allowed bytes (inclusive).
    pub min: usize,
    /// Maximum allowed bytes (inclusive).
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

/// One or more `space_or_tab`.
///
/// ```bnf
/// space_or_tab ::= 1*( ' ' '\t' )
/// ```
pub fn space_or_tab(tokenizer: &mut Tokenizer) -> StateName {
    space_or_tab_min_max(tokenizer, 1, usize::MAX)
}

/// Between `x` and `y` `space_or_tab`.
///
/// ```bnf
/// space_or_tab_min_max ::= x*y( ' ' '\t' )
/// ```
pub fn space_or_tab_min_max(tokenizer: &mut Tokenizer, min: usize, max: usize) -> StateName {
    space_or_tab_with_options(
        tokenizer,
        Options {
            kind: Token::SpaceOrTab,
            min,
            max,
            content_type: None,
            connect: false,
        },
    )
}

/// `space_or_tab`, with the given options.
pub fn space_or_tab_with_options(tokenizer: &mut Tokenizer, options: Options) -> StateName {
    tokenizer.tokenize_state.space_or_tab_connect = options.connect;
    tokenizer.tokenize_state.space_or_tab_content_type = options.content_type;
    tokenizer.tokenize_state.space_or_tab_min = options.min;
    tokenizer.tokenize_state.space_or_tab_max = options.max;
    tokenizer.tokenize_state.space_or_tab_token = options.kind;
    StateName::SpaceOrTabStart
}

/// `space_or_tab`, or optionally `space_or_tab`, one `eol`, and
/// optionally `space_or_tab`.
///
/// ```bnf
/// space_or_tab_eol ::= 1*( ' ' '\t' ) | 0*( ' ' '\t' ) eol 0*( ' ' '\t' )
/// ```
pub fn space_or_tab_eol(tokenizer: &mut Tokenizer) -> StateName {
    space_or_tab_eol_with_options(
        tokenizer,
        EolOptions {
            content_type: None,
            connect: false,
        },
    )
}

/// `space_or_tab_eol`, with the given options.
pub fn space_or_tab_eol_with_options(tokenizer: &mut Tokenizer, options: EolOptions) -> StateName {
    tokenizer.tokenize_state.space_or_tab_eol_content_type = options.content_type;
    tokenizer.tokenize_state.space_or_tab_eol_connect = options.connect;
    StateName::SpaceOrTabEolStart
}

/// Before `space_or_tab`.
///
/// ```markdown
/// > | a␠␠b
///      ^
/// ```
pub fn start(tokenizer: &mut Tokenizer) -> State {
    match tokenizer.current {
        Some(b'\t' | b' ') if tokenizer.tokenize_state.space_or_tab_max > 0 => {
            tokenizer.enter_with_content(
                tokenizer.tokenize_state.space_or_tab_token.clone(),
                tokenizer.tokenize_state.space_or_tab_content_type.clone(),
            );

            if tokenizer.tokenize_state.space_or_tab_connect {
                let index = tokenizer.events.len() - 1;
                link(&mut tokenizer.events, index);
            } else if tokenizer.tokenize_state.space_or_tab_content_type.is_some() {
                tokenizer.tokenize_state.space_or_tab_connect = true;
            }

            inside(tokenizer)
        }
        _ => after(tokenizer),
    }
}

/// In `space_or_tab`.
///
/// ```markdown
/// > | a␠␠b
///       ^
/// ```
pub fn inside(tokenizer: &mut Tokenizer) -> State {
    match tokenizer.current {
        Some(b'\t' | b' ')
            if tokenizer.tokenize_state.space_or_tab_size
                < tokenizer.tokenize_state.space_or_tab_max =>
        {
            tokenizer.consume();
            tokenizer.tokenize_state.space_or_tab_size += 1;
            State::Next(StateName::SpaceOrTabInside)
        }
        _ => {
            tokenizer.exit(tokenizer.tokenize_state.space_or_tab_token.clone());
            after(tokenizer)
        }
    }
}

/// After `space_or_tab`.
///
/// ```markdown
/// > | a␠␠b
///        ^
/// ```
pub fn after(tokenizer: &mut Tokenizer) -> State {
    let state = if tokenizer.tokenize_state.space_or_tab_size
        >= tokenizer.tokenize_state.space_or_tab_min
    {
        State::Ok
    } else {
        State::Nok
    };
    tokenizer.tokenize_state.space_or_tab_connect = false;
    tokenizer.tokenize_state.space_or_tab_content_type = None;
    tokenizer.tokenize_state.space_or_tab_size = 0;
    tokenizer.tokenize_state.space_or_tab_max = 0;
    tokenizer.tokenize_state.space_or_tab_min = 0;
    tokenizer.tokenize_state.space_or_tab_token = Token::SpaceOrTab;
    state
}

pub fn eol_start(tokenizer: &mut Tokenizer) -> State {
    let name = space_or_tab_with_options(
        tokenizer,
        Options {
            kind: Token::SpaceOrTab,
            min: 1,
            max: usize::MAX,
            content_type: tokenizer
                .tokenize_state
                .space_or_tab_eol_content_type
                .clone(),
            connect: tokenizer.tokenize_state.space_or_tab_eol_connect,
        },
    );

    tokenizer.attempt(
        name,
        State::Next(StateName::SpaceOrTabEolAfterFirst),
        State::Next(StateName::SpaceOrTabEolAtEol),
    )
}

pub fn eol_after_first(tokenizer: &mut Tokenizer) -> State {
    tokenizer.tokenize_state.space_or_tab_eol_ok = true;

    if tokenizer
        .tokenize_state
        .space_or_tab_eol_content_type
        .is_some()
    {
        tokenizer.tokenize_state.space_or_tab_eol_connect = true;
    }

    eol_at_eol(tokenizer)
}

/// `space_or_tab_eol`: after optionally first `space_or_tab`.
///
/// ```markdown
/// > | a
///      ^
///   | b
/// ```
pub fn eol_at_eol(tokenizer: &mut Tokenizer) -> State {
    if let Some(b'\n') = tokenizer.current {
        tokenizer.enter_with_content(
            Token::LineEnding,
            tokenizer
                .tokenize_state
                .space_or_tab_eol_content_type
                .clone(),
        );

        if tokenizer.tokenize_state.space_or_tab_eol_connect {
            let index = tokenizer.events.len() - 1;
            link(&mut tokenizer.events, index);
        } else if tokenizer
            .tokenize_state
            .space_or_tab_eol_content_type
            .is_some()
        {
            tokenizer.tokenize_state.space_or_tab_eol_connect = true;
        }

        tokenizer.consume();
        tokenizer.exit(Token::LineEnding);
        State::Next(StateName::SpaceOrTabEolAfterEol)
    } else {
        let ok = tokenizer.tokenize_state.space_or_tab_eol_ok;
        tokenizer.tokenize_state.space_or_tab_eol_content_type = None;
        tokenizer.tokenize_state.space_or_tab_eol_connect = false;
        tokenizer.tokenize_state.space_or_tab_eol_ok = false;
        if ok {
            State::Ok
        } else {
            State::Nok
        }
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
pub fn eol_after_eol(tokenizer: &mut Tokenizer) -> State {
    let name = space_or_tab_with_options(
        tokenizer,
        Options {
            kind: Token::SpaceOrTab,
            min: 1,
            max: usize::MAX,
            content_type: tokenizer
                .tokenize_state
                .space_or_tab_eol_content_type
                .clone(),
            connect: tokenizer.tokenize_state.space_or_tab_eol_connect,
        },
    );
    tokenizer.attempt(
        name,
        State::Next(StateName::SpaceOrTabEolAfterMore),
        State::Next(StateName::SpaceOrTabEolAfterMore),
    )
}

/// `space_or_tab_eol`: after more (optional) `space_or_tab`.
///
/// ```markdown
///   | a
/// > | b
///     ^
/// ```
pub fn eol_after_more(tokenizer: &mut Tokenizer) -> State {
    tokenizer.tokenize_state.space_or_tab_eol_content_type = None;
    tokenizer.tokenize_state.space_or_tab_eol_connect = false;
    tokenizer.tokenize_state.space_or_tab_eol_ok = false;

    // Blank line not allowed.
    if matches!(tokenizer.current, None | Some(b'\n')) {
        State::Nok
    } else {
        State::Ok
    }
}
