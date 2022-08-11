//! Several helpers to parse whitespace (`space_or_tab`, `space_or_tab_eol`).
//!
//! ## References
//!
//! *   [`micromark-factory-space/index.js` in `micromark`](https://github.com/micromark/micromark/blob/main/packages/micromark-factory-space/dev/index.js)

use crate::event::{Content, Name};
use crate::state::{Name as StateName, State};
use crate::subtokenize::link;
use crate::tokenizer::Tokenizer;

/// Options to parse `space_or_tab`.
#[derive(Debug)]
pub struct Options {
    /// Minimum allowed bytes (inclusive).
    pub min: usize,
    /// Maximum allowed bytes (inclusive).
    pub max: usize,
    /// Token type to use for whitespace events.
    pub kind: Name,
    /// Connect this whitespace to the previous.
    pub connect: bool,
    /// Embedded content type to use.
    pub content_type: Option<Content>,
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
            kind: Name::SpaceOrTab,
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

            State::Retry(StateName::SpaceOrTabInside)
        }
        _ => State::Retry(StateName::SpaceOrTabAfter),
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
            State::Retry(StateName::SpaceOrTabAfter)
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
    tokenizer.tokenize_state.space_or_tab_token = Name::SpaceOrTab;
    state
}
