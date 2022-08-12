//! Several helpers to parse whitespace (`space_or_tab`, `space_or_tab_eol`).
//!
//! ## References
//!
//! *   [`micromark-factory-space/index.js` in `micromark`](https://github.com/micromark/micromark/blob/main/packages/micromark-factory-space/dev/index.js)

use crate::construct::partial_space_or_tab::{
    space_or_tab_with_options, Options as SpaceOrTabOptions,
};
use crate::event::{Content, Link, Name};
use crate::state::{Name as StateName, State};
use crate::subtokenize::link;
use crate::tokenizer::Tokenizer;

/// Options to parse `space_or_tab` and one optional eol, but no blank line.
#[derive(Debug)]
pub struct Options {
    /// Connect this whitespace to the previous.
    pub connect: bool,
    /// Embedded content type to use.
    pub content: Option<Content>,
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
        Options {
            content: None,
            connect: false,
        },
    )
}

/// `space_or_tab_eol`, with the given options.
pub fn space_or_tab_eol_with_options(tokenizer: &mut Tokenizer, options: Options) -> StateName {
    tokenizer.tokenize_state.space_or_tab_eol_content = options.content;
    tokenizer.tokenize_state.space_or_tab_eol_connect = options.connect;
    StateName::SpaceOrTabEolStart
}

/// Start of whitespace with at most one eol.
///
/// ```markdown
/// > | a␠␠b
///      ^
/// > | a␠␠␊
///      ^
///   | ␠␠b
/// ```
pub fn start(tokenizer: &mut Tokenizer) -> State {
    if matches!(tokenizer.current, Some(b'\t' | b'\n' | b' ')) {
        tokenizer.attempt(
            State::Next(StateName::SpaceOrTabEolAfterFirst),
            State::Next(StateName::SpaceOrTabEolAtEol),
        );

        State::Retry(space_or_tab_with_options(
            tokenizer,
            SpaceOrTabOptions {
                kind: Name::SpaceOrTab,
                min: 1,
                max: usize::MAX,
                content: tokenizer.tokenize_state.space_or_tab_eol_content.clone(),
                connect: tokenizer.tokenize_state.space_or_tab_eol_connect,
            },
        ))
    } else {
        State::Nok
    }
}

/// After initial whitespace, at optional eol.
///
/// ```markdown
/// > | a␠␠b
///        ^
/// > | a␠␠␊
///        ^
///   | ␠␠b
/// ```
pub fn after_first(tokenizer: &mut Tokenizer) -> State {
    tokenizer.tokenize_state.space_or_tab_eol_ok = true;

    if tokenizer.tokenize_state.space_or_tab_eol_content.is_some() {
        tokenizer.tokenize_state.space_or_tab_eol_connect = true;
    }

    State::Retry(StateName::SpaceOrTabEolAtEol)
}

/// After optional whitespace, at eol.
///
/// ```markdown
/// > | a␠␠b
///        ^
/// > | a␠␠␊
///        ^
///   | ␠␠b
/// > | a␊
///      ^
///   | ␠␠b
/// ```
pub fn at_eol(tokenizer: &mut Tokenizer) -> State {
    if let Some(b'\n') = tokenizer.current {
        if let Some(ref content) = tokenizer.tokenize_state.space_or_tab_eol_content {
            tokenizer.enter_link(
                Name::LineEnding,
                Link {
                    previous: None,
                    next: None,
                    content: content.clone(),
                },
            );
        } else {
            tokenizer.enter(Name::LineEnding);
        }

        if tokenizer.tokenize_state.space_or_tab_eol_connect {
            let index = tokenizer.events.len() - 1;
            link(&mut tokenizer.events, index);
        } else if tokenizer.tokenize_state.space_or_tab_eol_content.is_some() {
            tokenizer.tokenize_state.space_or_tab_eol_connect = true;
        }

        tokenizer.consume();
        tokenizer.exit(Name::LineEnding);
        State::Next(StateName::SpaceOrTabEolAfterEol)
    } else {
        let ok = tokenizer.tokenize_state.space_or_tab_eol_ok;
        tokenizer.tokenize_state.space_or_tab_eol_content = None;
        tokenizer.tokenize_state.space_or_tab_eol_connect = false;
        tokenizer.tokenize_state.space_or_tab_eol_ok = false;
        if ok {
            State::Ok
        } else {
            State::Nok
        }
    }
}

/// After eol.
///
/// ```markdown
///   | a␠␠␊
/// > | ␠␠b
///     ^
///   | a␊
/// > | ␠␠b
///     ^
/// ```
pub fn after_eol(tokenizer: &mut Tokenizer) -> State {
    if matches!(tokenizer.current, Some(b'\t' | b' ')) {
        tokenizer.attempt(State::Next(StateName::SpaceOrTabEolAfterMore), State::Nok);
        State::Retry(space_or_tab_with_options(
            tokenizer,
            SpaceOrTabOptions {
                kind: Name::SpaceOrTab,
                min: 1,
                max: usize::MAX,
                content: tokenizer.tokenize_state.space_or_tab_eol_content.clone(),
                connect: tokenizer.tokenize_state.space_or_tab_eol_connect,
            },
        ))
    } else {
        State::Retry(StateName::SpaceOrTabEolAfterMore)
    }
}

/// After optional final whitespace.
///
/// ```markdown
///   | a␠␠␊
/// > | ␠␠b
///       ^
///   | a␊
/// > | ␠␠b
///       ^
/// ```
pub fn after_more(tokenizer: &mut Tokenizer) -> State {
    tokenizer.tokenize_state.space_or_tab_eol_content = None;
    tokenizer.tokenize_state.space_or_tab_eol_connect = false;
    tokenizer.tokenize_state.space_or_tab_eol_ok = false;

    // Blank line not allowed.
    if matches!(tokenizer.current, None | Some(b'\n')) {
        State::Nok
    } else {
        State::Ok
    }
}
