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
//! Each line of content is then allowed (not required) to be indented with up
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
//! That means that character escapes and character reference are allowed.
//!
//! In markdown, it is also possible to use [code (text)][code_text] in the
//! [text][] content type.
//! It is also possible to create code with the
//! [code (indented)][code_indented] construct.
//! That construct is less explicit, different from code (text), and has no
//! support for specifying the programming language, so it is recommended to
//! use code (fenced) instead of code (indented).
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
//! [html-pre]: https://html.spec.whatwg.org/multipage/grouping-content.html#the-pre-element
//! [html-code]: https://html.spec.whatwg.org/multipage/text-level-semantics.html#the-code-element

use crate::constant::{CODE_FENCED_SEQUENCE_SIZE_MIN, TAB_SIZE};
use crate::construct::partial_whitespace::start as whitespace;
use crate::tokenizer::{Code, State, StateFnResult, TokenType, Tokenizer};
use crate::util::span::from_exit_event;

/// Kind of fences.
#[derive(Debug, Clone, PartialEq)]
pub enum Kind {
    /// Grave accent (tick) code.
    GraveAccent,
    /// Tilde code.
    Tilde,
}

/// State needed to parse code (fenced).
#[derive(Debug, Clone)]
struct Info {
    /// Number of markers on the opening fence sequence.
    size: usize,
    /// Number of tabs or spaces of indentation before the opening fence
    /// sequence.
    prefix: usize,
    /// Kind of fences.
    kind: Kind,
}

/// Start of fenced code.
///
/// ```markdown
/// | ~~~js
///  console.log(1);
///  ~~~
/// ```
///
/// Parsing note: normally, the prefix is already stripped.
/// `flow.rs` makes sure that that doesn’t happen for code (fenced), as we need
/// it.
pub fn start(tokenizer: &mut Tokenizer, code: Code) -> StateFnResult {
    tokenizer.enter(TokenType::CodeFenced);
    tokenizer.enter(TokenType::CodeFencedFence);
    tokenizer.attempt(
        |tokenizer, code| whitespace(tokenizer, code, TokenType::Whitespace),
        |_ok| Box::new(before_sequence_open),
    )(tokenizer, code)
}

/// Inside the opening fence, after an optional prefix, before a sequence.
///
/// ```markdown
/// |~~~js
/// console.log(1);
/// ~~~
/// ```
fn before_sequence_open(tokenizer: &mut Tokenizer, code: Code) -> StateFnResult {
    let tail = tokenizer.events.last();
    let mut prefix = 0;

    if let Some(event) = tail {
        if event.token_type == TokenType::Whitespace {
            let span = from_exit_event(&tokenizer.events, tokenizer.events.len() - 1);
            prefix = span.end_index - span.start_index;
        }
    }

    match code {
        Code::Char(char) if char == '`' || char == '~' => {
            tokenizer.enter(TokenType::CodeFencedFenceSequence);
            sequence_open(
                tokenizer,
                Info {
                    prefix,
                    size: 0,
                    kind: if char == '`' {
                        Kind::GraveAccent
                    } else {
                        Kind::Tilde
                    },
                },
                code,
            )
        }
        _ => (State::Nok, None),
    }
}

/// Inside the opening fence sequence.
///
/// ```markdown
/// ~|~~js
/// console.log(1);
/// ~~~
/// ```
fn sequence_open(tokenizer: &mut Tokenizer, info: Info, code: Code) -> StateFnResult {
    let marker = if info.kind == Kind::GraveAccent {
        '`'
    } else {
        '~'
    };

    match code {
        Code::Char(char) if char == marker => {
            tokenizer.consume(code);
            (
                State::Fn(Box::new(|tokenizer, code| {
                    let mut info = info;
                    info.size += 1;
                    sequence_open(tokenizer, info, code)
                })),
                None,
            )
        }
        _ => {
            if info.size < CODE_FENCED_SEQUENCE_SIZE_MIN {
                (State::Nok, None)
            } else {
                tokenizer.exit(TokenType::CodeFencedFenceSequence);
                tokenizer.attempt(
                    |tokenizer, code| {
                        whitespace(tokenizer, code, TokenType::CodeFencedFenceWhitespace)
                    },
                    |_ok| Box::new(|tokenizer, code| info_before(tokenizer, info, code)),
                )(tokenizer, code)
            }
        }
    }
}

/// Inside the opening fence, after the sequence (and optional whitespace), before the info.
///
/// ```markdown
/// ~~~|js
/// console.log(1);
/// ~~~
/// ```
fn info_before(tokenizer: &mut Tokenizer, info: Info, code: Code) -> StateFnResult {
    match code {
        Code::None | Code::CarriageReturnLineFeed | Code::Char('\n' | '\r') => {
            tokenizer.exit(TokenType::CodeFencedFence);
            at_break(tokenizer, info, code)
        }
        _ => {
            tokenizer.enter(TokenType::CodeFencedFenceInfo);
            tokenizer.enter(TokenType::ChunkString);
            info_inside(tokenizer, info, code, vec![])
        }
    }
}

/// Inside the opening fence info.
///
/// ```markdown
/// ~~~j|s
/// console.log(1);
/// ~~~
/// ```
fn info_inside(
    tokenizer: &mut Tokenizer,
    info: Info,
    code: Code,
    codes: Vec<Code>,
) -> StateFnResult {
    match code {
        Code::None | Code::CarriageReturnLineFeed | Code::Char('\n' | '\r') => {
            tokenizer.exit(TokenType::ChunkString);
            tokenizer.exit(TokenType::CodeFencedFenceInfo);
            tokenizer.exit(TokenType::CodeFencedFence);
            at_break(tokenizer, info, code)
        }
        Code::VirtualSpace | Code::Char('\t' | ' ') => {
            tokenizer.exit(TokenType::ChunkString);
            tokenizer.exit(TokenType::CodeFencedFenceInfo);
            tokenizer.attempt(
                |tokenizer, code| whitespace(tokenizer, code, TokenType::CodeFencedFenceWhitespace),
                |_ok| Box::new(|tokenizer, code| meta_before(tokenizer, info, code)),
            )(tokenizer, code)
        }
        Code::Char(char) if char == '`' && info.kind == Kind::GraveAccent => (State::Nok, None),
        Code::Char(_) => {
            let mut codes = codes;
            codes.push(code);
            tokenizer.consume(code);
            (
                State::Fn(Box::new(|tokenizer, code| {
                    info_inside(tokenizer, info, code, codes)
                })),
                None,
            )
        }
    }
}

/// Inside the opening fence, after the info and whitespace, before the meta.
///
/// ```markdown
/// ~~~js |eval
/// console.log(1);
/// ~~~
/// ```
fn meta_before(tokenizer: &mut Tokenizer, info: Info, code: Code) -> StateFnResult {
    match code {
        Code::None | Code::CarriageReturnLineFeed | Code::Char('\n' | '\r') => {
            tokenizer.exit(TokenType::CodeFencedFence);
            at_break(tokenizer, info, code)
        }
        _ => {
            tokenizer.enter(TokenType::CodeFencedFenceMeta);
            tokenizer.enter(TokenType::ChunkString);
            meta(tokenizer, info, code)
        }
    }
}

/// Inside the opening fence meta.
///
/// ```markdown
/// ~~~js e|val
/// console.log(1);
/// ~~~
/// ```
fn meta(tokenizer: &mut Tokenizer, info: Info, code: Code) -> StateFnResult {
    match code {
        Code::None | Code::CarriageReturnLineFeed | Code::Char('\n' | '\r') => {
            tokenizer.exit(TokenType::ChunkString);
            tokenizer.exit(TokenType::CodeFencedFenceMeta);
            tokenizer.exit(TokenType::CodeFencedFence);
            at_break(tokenizer, info, code)
        }
        Code::Char(char) if char == '`' && info.kind == Kind::GraveAccent => (State::Nok, None),
        _ => {
            tokenizer.consume(code);
            (
                State::Fn(Box::new(|tokenizer, code| meta(tokenizer, info, code))),
                None,
            )
        }
    }
}

/// At an eol/eof in code, before a closing fence or before content.
///
/// ```markdown
/// ~~~js|
/// aa|
/// ~~~
/// ```
fn at_break(tokenizer: &mut Tokenizer, info: Info, code: Code) -> StateFnResult {
    let clone = info.clone();

    match code {
        Code::None => after(tokenizer, code),
        Code::CarriageReturnLineFeed | Code::Char('\n' | '\r') => tokenizer.attempt(
            |tokenizer, code| {
                tokenizer.enter(TokenType::LineEnding);
                tokenizer.consume(code);
                tokenizer.exit(TokenType::LineEnding);
                (
                    State::Fn(Box::new(|tokenizer, code| {
                        close_before(tokenizer, info, code)
                    })),
                    None,
                )
            },
            |ok| {
                if ok {
                    Box::new(after)
                } else {
                    Box::new(|tokenizer, code| {
                        tokenizer.enter(TokenType::LineEnding);
                        tokenizer.consume(code);
                        tokenizer.exit(TokenType::LineEnding);
                        (
                            State::Fn(Box::new(|tokenizer, code| {
                                content_start(tokenizer, clone, code)
                            })),
                            None,
                        )
                    })
                }
            },
        )(tokenizer, code),
        _ => unreachable!("unexpected non-eol/eof after `at_break` `{:?}`", code),
    }
}

/// Before a closing fence, before optional whitespace.
///
/// ```markdown
/// ~~~js
/// console.log('1')
/// |~~~
///
/// ~~~js
/// console.log('1')
/// |  ~~~
/// ```
fn close_before(tokenizer: &mut Tokenizer, info: Info, code: Code) -> StateFnResult {
    tokenizer.enter(TokenType::CodeFencedFence);
    tokenizer.attempt(
        |tokenizer, code| whitespace(tokenizer, code, TokenType::Whitespace),
        |_ok| Box::new(|tokenizer, code| close_sequence_before(tokenizer, info, code)),
    )(tokenizer, code)
}

/// In a closing fence, after optional whitespace, before sequence.
///
/// ```markdown
/// ~~~js
/// console.log('1')
/// |~~~
///
/// ~~~js
/// console.log('1')
///   |~~~
/// ```
fn close_sequence_before(tokenizer: &mut Tokenizer, info: Info, code: Code) -> StateFnResult {
    let tail = tokenizer.events.last();
    let mut prefix = 0;
    let marker = if info.kind == Kind::GraveAccent {
        '`'
    } else {
        '~'
    };

    if let Some(event) = tail {
        if event.token_type == TokenType::Whitespace {
            let span = from_exit_event(&tokenizer.events, tokenizer.events.len() - 1);
            prefix = span.end_index - span.start_index;
        }
    }

    // To do: 4+ should be okay if code (indented) is turned off!
    if prefix >= TAB_SIZE {
        return (State::Nok, None);
    }

    match code {
        Code::Char(char) if char == marker => {
            tokenizer.enter(TokenType::CodeFencedFenceSequence);
            close_sequence(tokenizer, info, code, 0)
        }
        _ => (State::Nok, None),
    }
}

/// In the closing fence sequence.
///
/// ```markdown
/// ~~~js
/// console.log('1')
/// ~|~~
/// ```
fn close_sequence(tokenizer: &mut Tokenizer, info: Info, code: Code, size: usize) -> StateFnResult {
    let marker = if info.kind == Kind::GraveAccent {
        '`'
    } else {
        '~'
    };

    match code {
        Code::Char(char) if char == marker => {
            tokenizer.consume(code);
            (
                State::Fn(Box::new(move |tokenizer, code| {
                    close_sequence(tokenizer, info, code, size + 1)
                })),
                None,
            )
        }
        _ if size >= CODE_FENCED_SEQUENCE_SIZE_MIN && size >= info.size => {
            tokenizer.exit(TokenType::CodeFencedFenceSequence);
            tokenizer.attempt(
                |tokenizer, code| whitespace(tokenizer, code, TokenType::CodeFencedFenceWhitespace),
                |_ok| Box::new(close_whitespace_after),
            )(tokenizer, code)
        }
        _ => (State::Nok, None),
    }
}

/// After the closing fence sequence after optional whitespace.
///
/// ```markdown
/// ~~~js
/// console.log('1')
/// ~~~ |
/// ```
fn close_whitespace_after(tokenizer: &mut Tokenizer, code: Code) -> StateFnResult {
    match code {
        Code::None | Code::CarriageReturnLineFeed | Code::Char('\n' | '\r') => {
            tokenizer.exit(TokenType::CodeFencedFence);
            (State::Ok, Some(vec![code]))
        }
        _ => (State::Nok, None),
    }
}

/// Before code content, definitely not before a closing fence.
///
/// ```markdown
/// ~~~js
/// |aa
/// ~~~
/// ```
fn content_start(tokenizer: &mut Tokenizer, info: Info, code: Code) -> StateFnResult {
    match code {
        Code::None | Code::CarriageReturnLineFeed | Code::Char('\n' | '\r') => {
            at_break(tokenizer, info, code)
        }
        Code::VirtualSpace | Code::Char('\t' | ' ') if info.prefix > 0 => {
            tokenizer.enter(TokenType::Whitespace);
            content_prefix(tokenizer, info, 0, code)
        }
        _ => {
            tokenizer.enter(TokenType::CodeFlowChunk);
            content_continue(tokenizer, info, code)
        }
    }
}

/// Before code content, in a prefix.
///
/// ```markdown
///   ~~~js
///  | aa
///   ~~~
/// ```
fn content_prefix(
    tokenizer: &mut Tokenizer,
    info: Info,
    prefix: usize,
    code: Code,
) -> StateFnResult {
    match code {
        Code::VirtualSpace | Code::Char('\t' | ' ') if info.prefix > prefix => {
            tokenizer.consume(code);
            (
                State::Fn(Box::new(move |tokenizer, code| {
                    content_prefix(tokenizer, info, prefix + 1, code)
                })),
                None,
            )
        }
        Code::None | Code::CarriageReturnLineFeed | Code::Char('\n' | '\r') => {
            tokenizer.exit(TokenType::Whitespace);
            at_break(tokenizer, info, code)
        }
        _ => {
            tokenizer.exit(TokenType::Whitespace);
            tokenizer.enter(TokenType::CodeFlowChunk);
            content_continue(tokenizer, info, code)
        }
    }
}

/// In code content.
///
/// ```markdown
/// ~~~js
/// |ab
/// a|b
/// ab|
/// ~~~
/// ```
fn content_continue(tokenizer: &mut Tokenizer, info: Info, code: Code) -> StateFnResult {
    match code {
        Code::None | Code::CarriageReturnLineFeed | Code::Char('\n' | '\r') => {
            tokenizer.exit(TokenType::CodeFlowChunk);
            at_break(tokenizer, info, code)
        }
        _ => {
            tokenizer.consume(code);
            (
                State::Fn(Box::new(|tokenizer, code| {
                    content_continue(tokenizer, info, code)
                })),
                None,
            )
        }
    }
}

/// After fenced code.
///
/// ```markdown
/// ~~~js
/// console.log('1')
/// ~~~|
/// ```
fn after(tokenizer: &mut Tokenizer, code: Code) -> StateFnResult {
    tokenizer.exit(TokenType::CodeFenced);
    (State::Ok, Some(vec![code]))
}
