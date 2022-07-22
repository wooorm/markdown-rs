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
use crate::construct::{
    partial_non_lazy_continuation::start as partial_non_lazy_continuation,
    partial_space_or_tab::{space_or_tab, space_or_tab_min_max},
};
use crate::token::Token;
use crate::tokenizer::{Code, ContentType, State, Tokenizer};
use crate::util::span::from_exit_event;

/// Kind of fences.
#[derive(Debug, Clone, PartialEq)]
pub enum Kind {
    /// Grave accent (tick) code.
    ///
    /// ## Example
    ///
    /// ````markdown
    /// ```rust
    /// println!("I <3 🦀");
    /// ```
    /// ````
    GraveAccent,
    /// Tilde code.
    ///
    /// ## Example
    ///
    /// ```markdown
    /// ~~~rust
    /// println!("I <3 🦀");
    /// ~~~
    /// ```
    Tilde,
}

impl Kind {
    /// Turn the kind into a [char].
    fn as_char(&self) -> char {
        match self {
            Kind::GraveAccent => '`',
            Kind::Tilde => '~',
        }
    }
    /// Turn a [char] into a kind.
    ///
    /// ## Panics
    ///
    /// Panics if `char` is not `~` or `` ` ``.
    fn from_char(char: char) -> Kind {
        match char {
            '`' => Kind::GraveAccent,
            '~' => Kind::Tilde,
            _ => unreachable!("invalid char"),
        }
    }
    /// Turn [Code] into a kind.
    ///
    /// ## Panics
    ///
    /// Panics if `code` is not ``Code::Char('~' | '`')``.
    fn from_code(code: Code) -> Kind {
        match code {
            Code::Char(char) => Kind::from_char(char),
            _ => unreachable!("invalid code"),
        }
    }
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
/// > | ~~~js
///     ^
///   | console.log(1)
///   | ~~~
/// ```
pub fn start(tokenizer: &mut Tokenizer, code: Code) -> State {
    let max = if tokenizer.parse_state.constructs.code_indented {
        TAB_SIZE - 1
    } else {
        usize::MAX
    };
    if tokenizer.parse_state.constructs.code_fenced {
        tokenizer.enter(Token::CodeFenced);
        tokenizer.enter(Token::CodeFencedFence);
        tokenizer.go(space_or_tab_min_max(0, max), before_sequence_open)(tokenizer, code)
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
fn before_sequence_open(tokenizer: &mut Tokenizer, code: Code) -> State {
    let tail = tokenizer.events.last();
    let mut prefix = 0;

    if let Some(event) = tail {
        if event.token_type == Token::SpaceOrTab {
            let span = from_exit_event(&tokenizer.events, tokenizer.events.len() - 1);
            prefix = span.end_index - span.start_index;
        }
    }

    match code {
        Code::Char('`' | '~') => {
            tokenizer.enter(Token::CodeFencedFenceSequence);
            sequence_open(
                tokenizer,
                code,
                Info {
                    prefix,
                    size: 0,
                    kind: Kind::from_code(code),
                },
            )
        }
        _ => State::Nok,
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
fn sequence_open(tokenizer: &mut Tokenizer, code: Code, mut info: Info) -> State {
    match code {
        Code::Char(char) if char == info.kind.as_char() => {
            tokenizer.consume(code);
            State::Fn(Box::new(|t, c| {
                info.size += 1;
                sequence_open(t, c, info)
            }))
        }
        _ if info.size >= CODE_FENCED_SEQUENCE_SIZE_MIN => {
            tokenizer.exit(Token::CodeFencedFenceSequence);
            tokenizer.attempt_opt(space_or_tab(), |t, c| info_before(t, c, info))(tokenizer, code)
        }
        _ => State::Nok,
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
fn info_before(tokenizer: &mut Tokenizer, code: Code, info: Info) -> State {
    match code {
        Code::None | Code::CarriageReturnLineFeed | Code::Char('\n' | '\r') => {
            tokenizer.exit(Token::CodeFencedFence);
            // Do not form containers.
            tokenizer.concrete = true;
            at_break(tokenizer, code, info)
        }
        _ => {
            tokenizer.enter(Token::CodeFencedFenceInfo);
            tokenizer.enter_with_content(Token::Data, Some(ContentType::String));
            info_inside(tokenizer, code, info, vec![])
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
fn info_inside(tokenizer: &mut Tokenizer, code: Code, info: Info, mut codes: Vec<Code>) -> State {
    match code {
        Code::None | Code::CarriageReturnLineFeed | Code::Char('\n' | '\r') => {
            tokenizer.exit(Token::Data);
            tokenizer.exit(Token::CodeFencedFenceInfo);
            tokenizer.exit(Token::CodeFencedFence);
            // Do not form containers.
            tokenizer.concrete = true;
            at_break(tokenizer, code, info)
        }
        Code::VirtualSpace | Code::Char('\t' | ' ') => {
            tokenizer.exit(Token::Data);
            tokenizer.exit(Token::CodeFencedFenceInfo);
            tokenizer.attempt_opt(space_or_tab(), |t, c| meta_before(t, c, info))(tokenizer, code)
        }
        Code::Char('`') if info.kind == Kind::GraveAccent => State::Nok,
        Code::Char(_) => {
            codes.push(code);
            tokenizer.consume(code);
            State::Fn(Box::new(|t, c| info_inside(t, c, info, codes)))
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
fn meta_before(tokenizer: &mut Tokenizer, code: Code, info: Info) -> State {
    match code {
        Code::None | Code::CarriageReturnLineFeed | Code::Char('\n' | '\r') => {
            tokenizer.exit(Token::CodeFencedFence);
            // Do not form containers.
            tokenizer.concrete = true;
            at_break(tokenizer, code, info)
        }
        _ => {
            tokenizer.enter(Token::CodeFencedFenceMeta);
            tokenizer.enter_with_content(Token::Data, Some(ContentType::String));
            meta(tokenizer, code, info)
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
fn meta(tokenizer: &mut Tokenizer, code: Code, info: Info) -> State {
    match code {
        Code::None | Code::CarriageReturnLineFeed | Code::Char('\n' | '\r') => {
            tokenizer.exit(Token::Data);
            tokenizer.exit(Token::CodeFencedFenceMeta);
            tokenizer.exit(Token::CodeFencedFence);
            // Do not form containers.
            tokenizer.concrete = true;
            at_break(tokenizer, code, info)
        }
        Code::Char('`') if info.kind == Kind::GraveAccent => State::Nok,
        _ => {
            tokenizer.consume(code);
            State::Fn(Box::new(|t, c| meta(t, c, info)))
        }
    }
}

/// At an eol/eof in code, before a closing fence or before content.
///
/// ```markdown
/// > | ~~~js
///          ^
/// > | console.log(1)
///                   ^
///   | ~~~
/// ```
fn at_break(tokenizer: &mut Tokenizer, code: Code, info: Info) -> State {
    tokenizer.check(partial_non_lazy_continuation, |ok| {
        if ok {
            Box::new(move |t, c| at_non_lazy_break(t, c, info))
        } else {
            Box::new(after)
        }
    })(tokenizer, code)
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
fn at_non_lazy_break(tokenizer: &mut Tokenizer, code: Code, info: Info) -> State {
    let clone = info.clone();

    tokenizer.attempt(
        |t, c| close_begin(t, c, info),
        |ok| {
            if ok {
                Box::new(after)
            } else {
                Box::new(|t, c| content_before(t, c, clone))
            }
        },
    )(tokenizer, code)
}

/// Before a closing fence, at the line ending.
///
/// ```markdown
///   | ~~~js
/// > | console.log(1)
///                   ^
///   | ~~~
/// ```
fn close_begin(tokenizer: &mut Tokenizer, code: Code, info: Info) -> State {
    match code {
        Code::CarriageReturnLineFeed | Code::Char('\n' | '\r') => {
            tokenizer.enter(Token::LineEnding);
            tokenizer.consume(code);
            tokenizer.exit(Token::LineEnding);
            State::Fn(Box::new(|t, c| close_start(t, c, info)))
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
fn close_start(tokenizer: &mut Tokenizer, code: Code, info: Info) -> State {
    let max = if tokenizer.parse_state.constructs.code_indented {
        TAB_SIZE - 1
    } else {
        usize::MAX
    };

    tokenizer.enter(Token::CodeFencedFence);
    tokenizer.go(space_or_tab_min_max(0, max), |t, c| {
        close_before(t, c, info)
    })(tokenizer, code)
}

/// In a closing fence, after optional whitespace, before sequence.
///
/// ```markdown
///   | ~~~js
///   | console.log(1)
/// > | ~~~
///     ^
/// ```
fn close_before(tokenizer: &mut Tokenizer, code: Code, info: Info) -> State {
    match code {
        Code::Char(char) if char == info.kind.as_char() => {
            tokenizer.enter(Token::CodeFencedFenceSequence);
            close_sequence(tokenizer, code, info, 0)
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
fn close_sequence(tokenizer: &mut Tokenizer, code: Code, info: Info, size: usize) -> State {
    match code {
        Code::Char(char) if char == info.kind.as_char() => {
            tokenizer.consume(code);
            State::Fn(Box::new(move |t, c| close_sequence(t, c, info, size + 1)))
        }
        _ if size >= CODE_FENCED_SEQUENCE_SIZE_MIN && size >= info.size => {
            tokenizer.exit(Token::CodeFencedFenceSequence);
            tokenizer.attempt_opt(space_or_tab(), close_sequence_after)(tokenizer, code)
        }
        _ => State::Nok,
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
fn close_sequence_after(tokenizer: &mut Tokenizer, code: Code) -> State {
    match code {
        Code::None | Code::CarriageReturnLineFeed | Code::Char('\n' | '\r') => {
            tokenizer.exit(Token::CodeFencedFence);
            State::Ok(if matches!(code, Code::None) { 0 } else { 1 })
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
fn content_before(tokenizer: &mut Tokenizer, code: Code, info: Info) -> State {
    tokenizer.enter(Token::LineEnding);
    tokenizer.consume(code);
    tokenizer.exit(Token::LineEnding);
    State::Fn(Box::new(|t, c| content_start(t, c, info)))
}
/// Before code content, definitely not before a closing fence.
///
/// ```markdown
///   | ~~~js
/// > | console.log(1)
///     ^
///   | ~~~
/// ```
fn content_start(tokenizer: &mut Tokenizer, code: Code, info: Info) -> State {
    tokenizer.go(space_or_tab_min_max(0, info.prefix), |t, c| {
        content_begin(t, c, info)
    })(tokenizer, code)
}

/// Before code content, after a prefix.
///
/// ```markdown
///   | ~~~js
/// > | console.log(1)
///     ^
///   | ~~~
/// ```
fn content_begin(tokenizer: &mut Tokenizer, code: Code, info: Info) -> State {
    match code {
        Code::None | Code::CarriageReturnLineFeed | Code::Char('\n' | '\r') => {
            at_break(tokenizer, code, info)
        }
        _ => {
            tokenizer.enter(Token::CodeFlowChunk);
            content_continue(tokenizer, code, info)
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
fn content_continue(tokenizer: &mut Tokenizer, code: Code, info: Info) -> State {
    match code {
        Code::None | Code::CarriageReturnLineFeed | Code::Char('\n' | '\r') => {
            tokenizer.exit(Token::CodeFlowChunk);
            at_break(tokenizer, code, info)
        }
        _ => {
            tokenizer.consume(code);
            State::Fn(Box::new(|t, c| content_continue(t, c, info)))
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
fn after(tokenizer: &mut Tokenizer, code: Code) -> State {
    tokenizer.exit(Token::CodeFenced);
    // Feel free to interrupt.
    tokenizer.interrupt = false;
    // No longer concrete.
    tokenizer.concrete = false;
    State::Ok(if matches!(code, Code::None) { 0 } else { 1 })
}
