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
//! *   [`CodeFenced`][Token::CodeFenced]
//! *   [`CodeFencedFence`][Token::CodeFencedFence]
//! *   [`CodeFencedFenceInfo`][Token::CodeFencedFenceInfo]
//! *   [`CodeFencedFenceMeta`][Token::CodeFencedFenceMeta]
//! *   [`CodeFencedFenceSequence`][Token::CodeFencedFenceSequence]
//! *   [`CodeFlowChunk`][Token::CodeFlowChunk]
//! *   [`LineEnding`][Token::LineEnding]
//! *   [`SpaceOrTab`][Token::SpaceOrTab]
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
use crate::token::Token;
use crate::tokenizer::{ContentType, State, StateName, Tokenizer};
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
        tokenizer.enter(Token::CodeFenced);
        tokenizer.enter(Token::CodeFencedFence);
        let name = space_or_tab_min_max(
            tokenizer,
            0,
            if tokenizer.parse_state.constructs.code_indented {
                TAB_SIZE - 1
            } else {
                usize::MAX
            },
        );
        tokenizer.attempt(
            name,
            State::Next(StateName::CodeFencedBeforeSequenceOpen),
            State::Nok,
        )
    } else {
        State::Nok
    }
}

/// Inside the opening fence, after an optional prefix, before a sequence.
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
        if event.token_type == Token::SpaceOrTab {
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
        tokenizer.enter(Token::CodeFencedFenceSequence);
        State::Retry(StateName::CodeFencedSequenceOpen)
    } else {
        State::Nok
    }
}

/// Inside the opening fence sequence.
///
/// ```markdown
/// > | ~~~js
///      ^
///   | console.log(1)
///   | ~~~
/// ```
pub fn sequence_open(tokenizer: &mut Tokenizer) -> State {
    match tokenizer.current {
        Some(b'`' | b'~') if tokenizer.current.unwrap() == tokenizer.tokenize_state.marker => {
            tokenizer.tokenize_state.size += 1;
            tokenizer.consume();
            State::Next(StateName::CodeFencedSequenceOpen)
        }
        _ if tokenizer.tokenize_state.size >= CODE_FENCED_SEQUENCE_SIZE_MIN => {
            tokenizer.exit(Token::CodeFencedFenceSequence);
            let name = space_or_tab(tokenizer);
            tokenizer.attempt(
                name,
                State::Next(StateName::CodeFencedInfoBefore),
                State::Next(StateName::CodeFencedInfoBefore),
            )
        }
        _ => {
            tokenizer.tokenize_state.marker = 0;
            tokenizer.tokenize_state.size_c = 0;
            tokenizer.tokenize_state.size = 0;
            State::Nok
        }
    }
}

/// Inside the opening fence, after the sequence (and optional whitespace), before the info.
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
            tokenizer.exit(Token::CodeFencedFence);
            // Do not form containers.
            tokenizer.concrete = true;
            tokenizer.check(
                StateName::NonLazyContinuationStart,
                State::Next(StateName::CodeFencedAtNonLazyBreak),
                State::Next(StateName::CodeFencedAfter),
            )
        }
        _ => {
            tokenizer.enter(Token::CodeFencedFenceInfo);
            tokenizer.enter_with_content(Token::Data, Some(ContentType::String));
            State::Retry(StateName::CodeFencedInfo)
        }
    }
}

/// Inside the opening fence info.
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
            tokenizer.exit(Token::Data);
            tokenizer.exit(Token::CodeFencedFenceInfo);
            State::Retry(StateName::CodeFencedInfoBefore)
        }
        Some(b'\t' | b' ') => {
            tokenizer.exit(Token::Data);
            tokenizer.exit(Token::CodeFencedFenceInfo);
            let name = space_or_tab(tokenizer);
            tokenizer.attempt(
                name,
                State::Next(StateName::CodeFencedMetaBefore),
                State::Next(StateName::CodeFencedMetaBefore),
            )
        }
        Some(b'`') if tokenizer.tokenize_state.marker == b'`' => {
            tokenizer.concrete = false;
            tokenizer.tokenize_state.marker = 0;
            tokenizer.tokenize_state.size_c = 0;
            tokenizer.tokenize_state.size = 0;
            State::Nok
        }
        Some(_) => {
            tokenizer.consume();
            State::Next(StateName::CodeFencedInfo)
        }
    }
}

/// Inside the opening fence, after the info and whitespace, before the meta.
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
            tokenizer.enter(Token::CodeFencedFenceMeta);
            tokenizer.enter_with_content(Token::Data, Some(ContentType::String));
            State::Retry(StateName::CodeFencedMeta)
        }
    }
}

/// Inside the opening fence meta.
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
            tokenizer.exit(Token::Data);
            tokenizer.exit(Token::CodeFencedFenceMeta);
            State::Retry(StateName::CodeFencedInfoBefore)
        }
        Some(b'`') if tokenizer.tokenize_state.marker == b'`' => {
            tokenizer.concrete = false;
            tokenizer.tokenize_state.marker = 0;
            tokenizer.tokenize_state.size_c = 0;
            tokenizer.tokenize_state.size = 0;
            State::Nok
        }
        _ => {
            tokenizer.consume();
            State::Next(StateName::CodeFencedMeta)
        }
    }
}

/// At an eol/eof in code, before a non-lazy closing fence or content.
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
        StateName::CodeFencedCloseBefore,
        State::Next(StateName::CodeFencedAfter),
        State::Next(StateName::CodeFencedContentBefore),
    )
}

/// Before a closing fence, at the line ending.
///
/// ```markdown
///   | ~~~js
/// > | console.log(1)
///                   ^
///   | ~~~
/// ```
pub fn close_before(tokenizer: &mut Tokenizer) -> State {
    match tokenizer.current {
        Some(b'\n') => {
            tokenizer.enter(Token::LineEnding);
            tokenizer.consume();
            tokenizer.exit(Token::LineEnding);
            State::Next(StateName::CodeFencedCloseStart)
        }
        _ => unreachable!("expected eol"),
    }
}

/// Before a closing fence, before optional whitespace.
///
/// ```markdown
///   | ~~~js
///   | console.log(1)
/// > | ~~~
///     ^
/// ```
pub fn close_start(tokenizer: &mut Tokenizer) -> State {
    tokenizer.enter(Token::CodeFencedFence);
    let name = space_or_tab_min_max(
        tokenizer,
        0,
        if tokenizer.parse_state.constructs.code_indented {
            TAB_SIZE - 1
        } else {
            usize::MAX
        },
    );
    tokenizer.attempt(
        name,
        State::Next(StateName::CodeFencedBeforeSequenceClose),
        State::Nok,
    )
}

/// In a closing fence, after optional whitespace, before sequence.
///
/// ```markdown
///   | ~~~js
///   | console.log(1)
/// > | ~~~
///     ^
/// ```
pub fn before_sequence_close(tokenizer: &mut Tokenizer) -> State {
    match tokenizer.current {
        Some(b'`' | b'~') if tokenizer.current.unwrap() == tokenizer.tokenize_state.marker => {
            tokenizer.enter(Token::CodeFencedFenceSequence);
            State::Retry(StateName::CodeFencedSequenceClose)
        }
        _ => State::Nok,
    }
}

/// In the closing fence sequence.
///
/// ```markdown
///   | ~~~js
///   | console.log(1)
/// > | ~~~
///     ^
/// ```
pub fn sequence_close(tokenizer: &mut Tokenizer) -> State {
    match tokenizer.current {
        Some(b'`' | b'~') if tokenizer.current.unwrap() == tokenizer.tokenize_state.marker => {
            tokenizer.tokenize_state.size_b += 1;
            tokenizer.consume();
            State::Next(StateName::CodeFencedSequenceClose)
        }
        _ if tokenizer.tokenize_state.size_b >= CODE_FENCED_SEQUENCE_SIZE_MIN
            && tokenizer.tokenize_state.size_b >= tokenizer.tokenize_state.size =>
        {
            tokenizer.tokenize_state.size_b = 0;
            tokenizer.exit(Token::CodeFencedFenceSequence);
            let name = space_or_tab(tokenizer);
            tokenizer.attempt(
                name,
                State::Next(StateName::CodeFencedAfterSequenceClose),
                State::Next(StateName::CodeFencedAfterSequenceClose),
            )
        }
        _ => {
            tokenizer.tokenize_state.size_b = 0;
            State::Nok
        }
    }
}

/// After the closing fence sequence after optional whitespace.
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
            tokenizer.exit(Token::CodeFencedFence);
            State::Ok
        }
        _ => State::Nok,
    }
}

/// Before a closing fence, at the line ending.
///
/// ```markdown
///   | ~~~js
/// > | console.log(1)
///                   ^
///   | ~~~
/// ```
pub fn content_before(tokenizer: &mut Tokenizer) -> State {
    tokenizer.enter(Token::LineEnding);
    tokenizer.consume();
    tokenizer.exit(Token::LineEnding);
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
    let name = space_or_tab_min_max(tokenizer, 0, tokenizer.tokenize_state.size_c);
    tokenizer.attempt(
        name,
        State::Next(StateName::CodeFencedBeforeContentChunk),
        State::Nok,
    )
}

/// Before code content, after a prefix.
///
/// ```markdown
///   | ~~~js
/// > | console.log(1)
///     ^
///   | ~~~
/// ```
pub fn before_content_chunk(tokenizer: &mut Tokenizer) -> State {
    match tokenizer.current {
        None | Some(b'\n') => tokenizer.check(
            StateName::NonLazyContinuationStart,
            State::Next(StateName::CodeFencedAtNonLazyBreak),
            State::Next(StateName::CodeFencedAfter),
        ),
        _ => {
            tokenizer.enter(Token::CodeFlowChunk);
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
            tokenizer.exit(Token::CodeFlowChunk);
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
    tokenizer.exit(Token::CodeFenced);
    tokenizer.tokenize_state.marker = 0;
    tokenizer.tokenize_state.size_c = 0;
    tokenizer.tokenize_state.size = 0;
    // Feel free to interrupt.
    tokenizer.interrupt = false;
    // No longer concrete.
    tokenizer.concrete = false;
    State::Ok
}
