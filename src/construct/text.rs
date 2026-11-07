//! The text content type.
//!
//! **Text** contains phrasing content such as
//! [attention][crate::construct::attention] (emphasis, gfm strikethrough, strong),
//! [raw (text)][crate::construct::raw_text] (code (text), math (text)), and actual text.
//!
//! The constructs found in text are:
//!
//! * [Attention][crate::construct::attention] (emphasis, gfm strikethrough, strong)
//! * [Autolink][crate::construct::autolink]
//! * [Character escape][crate::construct::character_escape]
//! * [Character reference][crate::construct::character_reference]
//! * [Raw (text)][crate::construct::raw_text] (code (text), math (text))
//! * [GFM: Label start (footnote)][crate::construct::gfm_label_start_footnote]
//! * [GFM: Task list item check][crate::construct::gfm_task_list_item_check]
//! * [Hard break (escape)][crate::construct::hard_break_escape]
//! * [HTML (text)][crate::construct::html_text]
//! * [Label start (image)][crate::construct::label_start_image]
//! * [Label start (link)][crate::construct::label_start_link]
//! * [Label end][crate::construct::label_end]
//! * [MDX: expression (text)][crate::construct::mdx_expression_text]
//! * [MDX: JSX (text)][crate::construct::mdx_jsx_text]
//!
//! > 👉 **Note**: for performance reasons, hard break (trailing) is formed by
//! > [whitespace][crate::construct::partial_whitespace].

use crate::construct::gfm_autolink_literal::resolve as resolve_gfm_autolink_literal;
use crate::construct::partial_whitespace::resolve_whitespace;
use crate::event::{Kind, Name};
use crate::extension::{
    attention_sizes, can_start_after, ext, ext_mut, is_pair_skipped, markers,
    start as start_construct,
};
use crate::resolve::Name as ResolveName;
use crate::state::{Name as StateName, State};
use crate::subtokenize::Subresult;
use crate::tokenizer::Tokenizer;
use crate::util::char;
use core::convert::TryFrom;

/// Characters that can start something in text.
pub const MARKERS: [u8; 16] = [
    b'!',  // `label_start_image`
    b'$',  // `raw_text` (math (text))
    b'&',  // `character_reference`
    b'*',  // `attention` (emphasis, strong)
    b'<',  // `autolink`, `html_text`, `mdx_jsx_text`
    b'H',  // `gfm_autolink_literal` (`protocol` kind)
    b'W',  // `gfm_autolink_literal` (`www.` kind)
    b'[',  // `label_start_link`
    b'\\', // `character_escape`, `hard_break_escape`
    b']',  // `label_end`, `gfm_label_start_footnote`
    b'_',  // `attention` (emphasis, strong)
    b'`',  // `raw_text` (code (text))
    b'h',  // `gfm_autolink_literal` (`protocol` kind)
    b'w',  // `gfm_autolink_literal` (`www.` kind)
    b'{',  // `mdx_expression_text`
    b'~',  // `attention` (gfm strikethrough)
];

/// Start of text.
///
/// There is a slightly weird case where task list items have their check at
/// the start of the first paragraph.
/// So we start by checking for that.
///
/// ```markdown
/// > | abc
///     ^
/// ```
pub fn start(tokenizer: &mut Tokenizer) -> State {
    let parse_state = tokenizer.parse_state;
    tokenizer.tokenize_state.markers = if parse_state.text_markers.is_empty() {
        &MARKERS
    } else {
        &parse_state.text_markers
    };
    tokenizer.attempt(
        State::Next(StateName::TextBefore),
        State::Next(StateName::TextBefore),
    );
    State::Retry(StateName::GfmTaskListItemCheckStart)
}

/// Before text.
///
/// ```markdown
/// > | abc
///     ^
/// ```
pub fn before(tokenizer: &mut Tokenizer) -> State {
    if tokenizer.parse_state.options.text_constructs.is_empty() {
        before_builtin(tokenizer)
    } else {
        before_construct(tokenizer, 0)
    }
}

/// Before constructs of syntax extensions, trying the one at `index` and
/// later.
///
/// ```markdown
/// > | @a
///     ^
/// ```
pub fn before_construct(tokenizer: &mut Tokenizer, index: u8) -> State {
    let options = tokenizer.parse_state.options;
    let count = options.text_constructs.len();
    let mut index = usize::from(index);

    // Container prefixes are not text, and an escaped character is not
    // markup before a construct, like micromark checking for
    // `characterEscape`.
    let previous = match tokenizer.events.last() {
        _ if tokenizer.at_line_start() => Some('\n'),
        Some(event) if event.kind == Kind::Exit && event.name == Name::CharacterEscape => None,
        _ => match tokenizer.previous {
            // Line endings, tabs, and their virtual spaces are ASCII.
            Some(byte) if byte.is_ascii() => Some(char::from(byte)),
            Some(_) => char::before_index(tokenizer.parse_state.bytes, tokenizer.point.index),
            None => None,
        },
    };

    if let Some(byte) = tokenizer.current {
        while index < count {
            // Parsing checks that there are at most 255 constructs.
            let at = u8::try_from(index).expect("expected at most 255 constructs");
            if byte != b'\n'
                && markers(options, at).contains(&byte)
                && can_start_after(options, at, previous)
                && !is_pair_skipped(tokenizer, at)
            {
                // Parsing checks that there are at most 255 constructs.
                let next = u8::try_from(index + 1).expect("expected at most 255 constructs");
                ext_mut(tokenizer).next = next;
                tokenizer.attempt(
                    State::Next(StateName::TextBefore),
                    State::Next(StateName::TextBeforeConstructNext),
                );

                // A delimiter run, of ASCII, which the attention resolver pairs
                // with runs of the same construct.
                if !attention_sizes(options, at).is_empty() {
                    if !byte.is_ascii() {
                        return State::Nok;
                    }
                    tokenizer.tokenize_state.marker = byte;
                    tokenizer.enter(Name::AttentionSequence);
                    let event = tokenizer.events.last_mut().expect("expected event");
                    event.extension = u16::from(next);
                    return State::Retry(StateName::AttentionInside);
                }

                return start_construct(tokenizer, next - 1);
            }
            index += 1;
        }
    }

    before_builtin(tokenizer)
}

/// Before constructs of syntax extensions, after one did not match.
///
/// ```markdown
/// > | @b
///     ^
/// ```
pub fn before_construct_next(tokenizer: &mut Tokenizer) -> State {
    let next = ext(tokenizer).next;
    before_construct(tokenizer, next)
}

/// Before built-in constructs.
///
/// ```markdown
/// > | abc
///     ^
/// ```
pub fn before_builtin(tokenizer: &mut Tokenizer) -> State {
    match tokenizer.current {
        None => {
            tokenizer.register_resolver(ResolveName::Data);
            tokenizer.register_resolver(ResolveName::Text);
            State::Ok
        }
        Some(b'!') => {
            tokenizer.attempt(
                State::Next(StateName::TextBefore),
                State::Next(StateName::TextBeforeData),
            );
            State::Retry(StateName::LabelStartImageStart)
        }
        // raw (text) (code (text), math (text))
        Some(b'$' | b'`') => {
            tokenizer.attempt(
                State::Next(StateName::TextBefore),
                State::Next(StateName::TextBeforeData),
            );
            State::Retry(StateName::RawTextStart)
        }
        Some(b'&') => {
            tokenizer.attempt(
                State::Next(StateName::TextBefore),
                State::Next(StateName::TextBeforeData),
            );
            State::Retry(StateName::CharacterReferenceStart)
        }
        // attention (emphasis, gfm strikethrough, strong)
        Some(b'*' | b'_' | b'~') => {
            tokenizer.attempt(
                State::Next(StateName::TextBefore),
                State::Next(StateName::TextBeforeData),
            );
            State::Retry(StateName::AttentionStart)
        }
        // `autolink`, `html_text` (order does not matter), `mdx_jsx_text` (order matters).
        Some(b'<') => {
            tokenizer.attempt(
                State::Next(StateName::TextBefore),
                State::Next(StateName::TextBeforeHtml),
            );
            State::Retry(StateName::AutolinkStart)
        }
        Some(b'H' | b'h') => {
            tokenizer.attempt(
                State::Next(StateName::TextBefore),
                State::Next(StateName::TextBeforeData),
            );
            State::Retry(StateName::GfmAutolinkLiteralProtocolStart)
        }
        Some(b'W' | b'w') => {
            tokenizer.attempt(
                State::Next(StateName::TextBefore),
                State::Next(StateName::TextBeforeData),
            );
            State::Retry(StateName::GfmAutolinkLiteralWwwStart)
        }
        Some(b'[') => {
            tokenizer.attempt(
                State::Next(StateName::TextBefore),
                State::Next(StateName::TextBeforeLabelStartLink),
            );
            State::Retry(StateName::GfmLabelStartFootnoteStart)
        }
        Some(b'\\') => {
            tokenizer.attempt(
                State::Next(StateName::TextBefore),
                State::Next(StateName::TextBeforeHardBreakEscape),
            );
            State::Retry(StateName::CharacterEscapeStart)
        }
        Some(b']') => {
            tokenizer.attempt(
                State::Next(StateName::TextBefore),
                State::Next(StateName::TextBeforeData),
            );
            State::Retry(StateName::LabelEndStart)
        }
        Some(b'{') => {
            tokenizer.attempt(
                State::Next(StateName::TextBefore),
                State::Next(StateName::TextBeforeData),
            );
            State::Retry(StateName::MdxExpressionTextStart)
        }
        _ => State::Retry(StateName::TextBeforeData),
    }
}

/// Before html (text).
///
/// At `<`, which wasn’t an autolink.
///
/// ```markdown
/// > | a <b>
///       ^
/// ```
pub fn before_html(tokenizer: &mut Tokenizer) -> State {
    tokenizer.attempt(
        State::Next(StateName::TextBefore),
        State::Next(StateName::TextBeforeMdxJsx),
    );
    State::Retry(StateName::HtmlTextStart)
}

/// Before mdx jsx (text).
///
/// At `<`, which wasn’t an autolink or html.
///
/// ```markdown
/// > | a <b>
///       ^
/// ```
pub fn before_mdx_jsx(tokenizer: &mut Tokenizer) -> State {
    tokenizer.attempt(
        State::Next(StateName::TextBefore),
        State::Next(StateName::TextBeforeData),
    );
    State::Retry(StateName::MdxJsxTextStart)
}

/// Before hard break escape.
///
/// At `\`, which wasn’t a character escape.
///
/// ```markdown
/// > | a \␊
///       ^
/// ```
pub fn before_hard_break_escape(tokenizer: &mut Tokenizer) -> State {
    tokenizer.attempt(
        State::Next(StateName::TextBefore),
        State::Next(StateName::TextBeforeData),
    );
    State::Retry(StateName::HardBreakEscapeStart)
}

/// Before label start (link).
///
/// At `[`, which wasn’t a GFM label start (footnote).
///
/// ```markdown
/// > | [a](b)
///     ^
/// ```
pub fn before_label_start_link(tokenizer: &mut Tokenizer) -> State {
    tokenizer.attempt(
        State::Next(StateName::TextBefore),
        State::Next(StateName::TextBeforeData),
    );
    State::Retry(StateName::LabelStartLinkStart)
}

/// Before data.
///
/// ```markdown
/// > | a
///     ^
/// ```
pub fn before_data(tokenizer: &mut Tokenizer) -> State {
    tokenizer.attempt(State::Next(StateName::TextBefore), State::Nok);
    State::Retry(StateName::DataStart)
}

/// Resolve whitespace.
pub fn resolve(tokenizer: &mut Tokenizer) -> Option<Subresult> {
    let keep_whitespace = tokenizer
        .tokenize_state
        .extension
        .as_ref()
        .map_or(false, |state| state.keep_whitespace);
    resolve_whitespace(
        tokenizer,
        tokenizer.parse_state.options.constructs.hard_break_trailing,
        !keep_whitespace,
    );

    if tokenizer
        .parse_state
        .options
        .constructs
        .gfm_autolink_literal
    {
        resolve_gfm_autolink_literal(tokenizer);
    }

    tokenizer.map.consume(&mut tokenizer.events);
    None
}
