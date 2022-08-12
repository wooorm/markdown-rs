//! Code (fenced) is a construct that occurs in the [flow][] content type.
//!
//! It forms with the following BNF:
//!
//! ```bnf
//! code_fenced ::= fence_open *( eol *code ) [ eol fence_close ]
//!
//! fence_open ::= sequence [ 1*space_or_tab info [ 1*space_or_tab meta ] ] *space_or_tab
//! ; Restriction: the number of markers in the closing fence sequence must be
//! ; equal to or greater than the number of markers in the opening fence
//! ; sequence.
//! ; Restriction: the marker in the closing fence sequence must match the
//! ; marker in the opening fence sequence
//! fence_close ::= sequence *space_or_tab
//! sequence ::= 3*'`' | 3*'~'
//! info ::= 1*text
//! meta ::= 1*text *( *space_or_tab 1*text )
//!
//! ; Restriction: the `` ` `` character cannot occur in `text` if it is the
//! ; marker of the opening fence sequence.
//! text ::= code - eol - space_or_tab
//! eol ::= '\r' | '\r\n' | '\n'
//! space_or_tab ::= ' ' | '\t'
//! code ::= . ; any unicode code point (other than line endings).
//! ```
//!
//! The above grammar does not show how whitespace is handled.
//! To parse code (fenced), let `X` be the number of whitespace characters
//! before the opening fence sequence.
//! Each line of text is then allowed (not required) to be indented with up
//! to `X` spaces or tabs, which are then ignored as an indent instead of being
//! considered as part of the code.
//! This indent does not affect the closing fence.
//! It can be indented up to a separate 3 spaces or tabs.
//! A bigger indent makes it part of the code instead of a fence.
//!
//! Code (fenced) relates to both the `<pre>` and the `<code>` elements in
//! HTML.
//! See [*§ 4.4.3 The `pre` element*][html-pre] and the [*§ 4.5.15 The `code`
//! element*][html-code] in the HTML spec for more info.
//!
//! The optional `meta` part is ignored: it is not used when parsing or
//! rendering.
//! The optional `info` part is used and is expected to specify the programming
//! language that the code is in.
//! Which value it holds depends on what your syntax highlighter supports, if
//! one is used.
//! The `info` is, when rendering to HTML, typically exposed as a class.
//! This behavior stems from the HTML spec ([*§ 4.5.15 The `code`
//! element*][html-code]).
//! For example:
//!
//! ```markdown
//! ~~~css
//! * { color: tomato }
//! ~~~
//! ```
//!
//! Yields:
//!
//! ```html
//! <pre><code class="language-css">* { color: tomato }
//! </code></pre>
//! ```
//!
//! The `info` and `meta` parts are interpreted as the [string][] content type.
//! That means that [character escapes][character_escape] and
//! [character references][character_reference] are allowed.
//!
//! In markdown, it is also possible to use [code (text)][code_text] in the
//! [text][] content type.
//! It is also possible to create code with the
//! [code (indented)][code_indented] construct.
//! That construct is less explicit, different from code (text), and has no
//! support for specifying the programming language, so it is recommended to
//! use code (fenced) instead of code (indented).
//!
//! ## Tokens
//!
//! *   [`CodeFenced`][Name::CodeFenced]
//! *   [`CodeFencedFence`][Name::CodeFencedFence]
//! *   [`CodeFencedFenceInfo`][Name::CodeFencedFenceInfo]
//! *   [`CodeFencedFenceMeta`][Name::CodeFencedFenceMeta]
//! *   [`CodeFencedFenceSequence`][Name::CodeFencedFenceSequence]
//! *   [`CodeFlowChunk`][Name::CodeFlowChunk]
//! *   [`LineEnding`][Name::LineEnding]
//! *   [`SpaceOrTab`][Name::SpaceOrTab]
//!
//! ## References
//!
//! *   [`code-fenced.js` in `micromark`](https://github.com/micromark/micromark/blob/main/packages/micromark-core-commonmark/dev/lib/code-fenced.js)
//! *   [*§ 4.5 Fenced code blocks* in `CommonMark`](https://spec.commonmark.org/0.30/#fenced-code-blocks)
//!
//! [flow]: crate::content::flow
//! [string]: crate::content::string
//! [text]: crate::content::text
//! [code_indented]: crate::construct::code_indented
//! [code_text]: crate::construct::code_text
//! [character_escape]: crate::construct::character_escape
//! [character_reference]: crate::construct::character_reference
//! [html-pre]: https://html.spec.whatwg.org/multipage/grouping-content.html#the-pre-element
//! [html-code]: https://html.spec.whatwg.org/multipage/text-level-semantics.html#the-code-element

use crate::constant::{CODE_FENCED_SEQUENCE_SIZE_MIN, TAB_SIZE};
use crate::construct::partial_space_or_tab::{space_or_tab, space_or_tab_min_max};
use crate::event::{Content, Link, Name};
use crate::state::{Name as StateName, State};
use crate::tokenizer::Tokenizer;
use crate::util::slice::{Position, Slice};

/// Start of fenced code.
///
/// ```markdown
/// > | ~~~js
///     ^
///   | console.log(1)
///   | ~~~
/// ```
pub fn start(tokenizer: &mut Tokenizer) -> State {
    if tokenizer.parse_state.constructs.code_fenced {
        tokenizer.enter(Name::CodeFenced);
        tokenizer.enter(Name::CodeFencedFence);

        tokenizer.attempt(
            State::Next(StateName::CodeFencedBeforeSequenceOpen),
            State::Nok,
        );
        State::Retry(space_or_tab_min_max(
            tokenizer,
            0,
            if tokenizer.parse_state.constructs.code_indented {
                TAB_SIZE - 1
            } else {
                usize::MAX
            },
        ))
    } else {
        State::Nok
    }
}

/// In opening fence, after prefix, at sequence.
///
/// ```markdown
/// > | ~~~js
///     ^
///   | console.log(1)
///   | ~~~
/// ```
pub fn before_sequence_open(tokenizer: &mut Tokenizer) -> State {
    let tail = tokenizer.events.last();
    let mut prefix = 0;

    if let Some(event) = tail {
        if event.name == Name::SpaceOrTab {
            prefix = Slice::from_position(
                tokenizer.parse_state.bytes,
                &Position::from_exit_event(&tokenizer.events, tokenizer.events.len() - 1),
            )
            .len();
        }
    }

    if let Some(b'`' | b'~') = tokenizer.current {
        tokenizer.tokenize_state.marker = tokenizer.current.unwrap();
        tokenizer.tokenize_state.size_c = prefix;
        tokenizer.enter(Name::CodeFencedFenceSequence);
        State::Retry(StateName::CodeFencedSequenceOpen)
    } else {
        State::Nok
    }
}

/// In opening fence sequence.
///
/// ```markdown
/// > | ~~~js
///      ^
///   | console.log(1)
///   | ~~~
/// ```
pub fn sequence_open(tokenizer: &mut Tokenizer) -> State {
    if tokenizer.current == Some(tokenizer.tokenize_state.marker) {
        tokenizer.tokenize_state.size += 1;
        tokenizer.consume();
        State::Next(StateName::CodeFencedSequenceOpen)
    } else if tokenizer.tokenize_state.size >= CODE_FENCED_SEQUENCE_SIZE_MIN {
        tokenizer.exit(Name::CodeFencedFenceSequence);

        tokenizer.attempt(
            State::Next(StateName::CodeFencedInfoBefore),
            State::Next(StateName::CodeFencedInfoBefore),
        );

        State::Retry(space_or_tab(tokenizer))
    } else {
        tokenizer.tokenize_state.marker = 0;
        tokenizer.tokenize_state.size_c = 0;
        tokenizer.tokenize_state.size = 0;
        State::Nok
    }
}

/// In opening fence, after the sequence (and optional whitespace), before info.
///
/// ```markdown
/// > | ~~~js
///        ^
///   | console.log(1)
///   | ~~~
/// ```
pub fn info_before(tokenizer: &mut Tokenizer) -> State {
    match tokenizer.current {
        None | Some(b'\n') => {
            tokenizer.exit(Name::CodeFencedFence);
            // Do not form containers.
            tokenizer.concrete = true;
            tokenizer.check(
                State::Next(StateName::CodeFencedAtNonLazyBreak),
                State::Next(StateName::CodeFencedAfter),
            );
            State::Retry(StateName::NonLazyContinuationStart)
        }
        _ => {
            tokenizer.enter(Name::CodeFencedFenceInfo);
            tokenizer.enter_link(
                Name::Data,
                Link {
                    previous: None,
                    next: None,
                    content: Content::String,
                },
            );
            State::Retry(StateName::CodeFencedInfo)
        }
    }
}

/// In info.
///
/// ```markdown
/// > | ~~~js
///        ^
///   | console.log(1)
///   | ~~~
/// ```
pub fn info(tokenizer: &mut Tokenizer) -> State {
    match tokenizer.current {
        None | Some(b'\n') => {
            tokenizer.exit(Name::Data);
            tokenizer.exit(Name::CodeFencedFenceInfo);
            State::Retry(StateName::CodeFencedInfoBefore)
        }
        Some(b'\t' | b' ') => {
            tokenizer.exit(Name::Data);
            tokenizer.exit(Name::CodeFencedFenceInfo);
            tokenizer.attempt(
                State::Next(StateName::CodeFencedMetaBefore),
                State::Next(StateName::CodeFencedMetaBefore),
            );
            State::Retry(space_or_tab(tokenizer))
        }
        Some(byte) => {
            if tokenizer.tokenize_state.marker == byte && byte == b'`' {
                tokenizer.concrete = false;
                tokenizer.tokenize_state.marker = 0;
                tokenizer.tokenize_state.size_c = 0;
                tokenizer.tokenize_state.size = 0;
                State::Nok
            } else {
                tokenizer.consume();
                State::Next(StateName::CodeFencedInfo)
            }
        }
    }
}

/// In opening fence, after info and whitespace, before meta.
///
/// ```markdown
/// > | ~~~js eval
///           ^
///   | console.log(1)
///   | ~~~
/// ```
pub fn meta_before(tokenizer: &mut Tokenizer) -> State {
    match tokenizer.current {
        None | Some(b'\n') => State::Retry(StateName::CodeFencedInfoBefore),
        _ => {
            tokenizer.enter(Name::CodeFencedFenceMeta);
            tokenizer.enter_link(
                Name::Data,
                Link {
                    previous: None,
                    next: None,
                    content: Content::String,
                },
            );
            State::Retry(StateName::CodeFencedMeta)
        }
    }
}

/// In meta.
///
/// ```markdown
/// > | ~~~js eval
///           ^
///   | console.log(1)
///   | ~~~
/// ```
pub fn meta(tokenizer: &mut Tokenizer) -> State {
    match tokenizer.current {
        None | Some(b'\n') => {
            tokenizer.exit(Name::Data);
            tokenizer.exit(Name::CodeFencedFenceMeta);
            State::Retry(StateName::CodeFencedInfoBefore)
        }
        Some(byte) => {
            if tokenizer.tokenize_state.marker == byte && byte == b'`' {
                tokenizer.concrete = false;
                tokenizer.tokenize_state.marker = 0;
                tokenizer.tokenize_state.size_c = 0;
                tokenizer.tokenize_state.size = 0;
                State::Nok
            } else {
                tokenizer.consume();
                State::Next(StateName::CodeFencedMeta)
            }
        }
    }
}

/// At eol/eof in code, before a non-lazy closing fence or content.
///
/// ```markdown
/// > | ~~~js
///          ^
/// > | console.log(1)
///                   ^
///   | ~~~
/// ```
pub fn at_non_lazy_break(tokenizer: &mut Tokenizer) -> State {
    tokenizer.attempt(
        State::Next(StateName::CodeFencedAfter),
        State::Next(StateName::CodeFencedContentBefore),
    );
    tokenizer.enter(Name::LineEnding);
    tokenizer.consume();
    tokenizer.exit(Name::LineEnding);
    State::Next(StateName::CodeFencedCloseStart)
}

/// Before closing fence, at optional whitespace.
///
/// ```markdown
///   | ~~~js
///   | console.log(1)
/// > | ~~~
///     ^
/// ```
pub fn close_start(tokenizer: &mut Tokenizer) -> State {
    tokenizer.enter(Name::CodeFencedFence);

    tokenizer.attempt(
        State::Next(StateName::CodeFencedBeforeSequenceClose),
        State::Nok,
    );

    State::Retry(space_or_tab_min_max(
        tokenizer,
        0,
        if tokenizer.parse_state.constructs.code_indented {
            TAB_SIZE - 1
        } else {
            usize::MAX
        },
    ))
}

/// In closing fence, after optional whitespace, at sequence.
///
/// ```markdown
///   | ~~~js
///   | console.log(1)
/// > | ~~~
///     ^
/// ```
pub fn before_sequence_close(tokenizer: &mut Tokenizer) -> State {
    if tokenizer.current == Some(tokenizer.tokenize_state.marker) {
        tokenizer.enter(Name::CodeFencedFenceSequence);
        State::Retry(StateName::CodeFencedSequenceClose)
    } else {
        State::Nok
    }
}

/// In closing fence sequence.
///
/// ```markdown
///   | ~~~js
///   | console.log(1)
/// > | ~~~
///     ^
/// ```
pub fn sequence_close(tokenizer: &mut Tokenizer) -> State {
    if tokenizer.current == Some(tokenizer.tokenize_state.marker) {
        tokenizer.tokenize_state.size_b += 1;
        tokenizer.consume();
        State::Next(StateName::CodeFencedSequenceClose)
    } else if tokenizer.tokenize_state.size_b >= CODE_FENCED_SEQUENCE_SIZE_MIN
        && tokenizer.tokenize_state.size_b >= tokenizer.tokenize_state.size
    {
        tokenizer.tokenize_state.size_b = 0;
        tokenizer.exit(Name::CodeFencedFenceSequence);
        tokenizer.attempt(
            State::Next(StateName::CodeFencedAfterSequenceClose),
            State::Next(StateName::CodeFencedAfterSequenceClose),
        );
        State::Retry(space_or_tab(tokenizer))
    } else {
        tokenizer.tokenize_state.size_b = 0;
        State::Nok
    }
}

/// After closing fence sequence, after optional whitespace.
///
/// ```markdown
///   | ~~~js
///   | console.log(1)
/// > | ~~~
///        ^
/// ```
pub fn sequence_close_after(tokenizer: &mut Tokenizer) -> State {
    match tokenizer.current {
        None | Some(b'\n') => {
            tokenizer.exit(Name::CodeFencedFence);
            State::Ok
        }
        _ => State::Nok,
    }
}

/// Before closing fence, at eol.
///
/// ```markdown
///   | ~~~js
/// > | console.log(1)
///                   ^
///   | ~~~
/// ```
pub fn content_before(tokenizer: &mut Tokenizer) -> State {
    tokenizer.enter(Name::LineEnding);
    tokenizer.consume();
    tokenizer.exit(Name::LineEnding);
    State::Next(StateName::CodeFencedContentStart)
}

/// Before code content, definitely not before a closing fence.
///
/// ```markdown
///   | ~~~js
/// > | console.log(1)
///     ^
///   | ~~~
/// ```
pub fn content_start(tokenizer: &mut Tokenizer) -> State {
    tokenizer.attempt(
        State::Next(StateName::CodeFencedBeforeContentChunk),
        State::Nok,
    );
    State::Retry(space_or_tab_min_max(
        tokenizer,
        0,
        tokenizer.tokenize_state.size_c,
    ))
}

/// Before code content, after optional prefix.
///
/// ```markdown
///   | ~~~js
/// > | console.log(1)
///     ^
///   | ~~~
/// ```
pub fn before_content_chunk(tokenizer: &mut Tokenizer) -> State {
    match tokenizer.current {
        None | Some(b'\n') => {
            tokenizer.check(
                State::Next(StateName::CodeFencedAtNonLazyBreak),
                State::Next(StateName::CodeFencedAfter),
            );
            State::Retry(StateName::NonLazyContinuationStart)
        }
        _ => {
            tokenizer.enter(Name::CodeFlowChunk);
            State::Retry(StateName::CodeFencedContentChunk)
        }
    }
}

/// In code content.
///
/// ```markdown
///   | ~~~js
/// > | console.log(1)
///     ^^^^^^^^^^^^^^
///   | ~~~
/// ```
pub fn content_chunk(tokenizer: &mut Tokenizer) -> State {
    match tokenizer.current {
        None | Some(b'\n') => {
            tokenizer.exit(Name::CodeFlowChunk);
            State::Retry(StateName::CodeFencedBeforeContentChunk)
        }
        _ => {
            tokenizer.consume();
            State::Next(StateName::CodeFencedContentChunk)
        }
    }
}

/// After fenced code.
///
/// ```markdown
///   | ~~~js
///   | console.log(1)
/// > | ~~~
///        ^
/// ```
pub fn after(tokenizer: &mut Tokenizer) -> State {
    tokenizer.exit(Name::CodeFenced);
    tokenizer.tokenize_state.marker = 0;
    tokenizer.tokenize_state.size_c = 0;
    tokenizer.tokenize_state.size = 0;
    // Feel free to interrupt.
    tokenizer.interrupt = false;
    // No longer concrete.
    tokenizer.concrete = false;
    State::Ok
}
