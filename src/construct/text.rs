//! The text content type.
//!
//! **Text** contains phrasing content such as
//! [attention][crate::construct::attention] (emphasis, strong),
//! [code (text)][crate::construct::code_text], and actual text.
//!
//! The constructs found in text are:
//!
//! *   [Attention][crate::construct::attention]
//! *   [Autolink][crate::construct::autolink]
//! *   [Character escape][crate::construct::character_escape]
//! *   [Character reference][crate::construct::character_reference]
//! *   [Code (text)][crate::construct::code_text]
//! *   [Hard break (escape)][crate::construct::hard_break_escape]
//! *   [HTML (text)][crate::construct::html_text]
//! *   [Label start (image)][crate::construct::label_start_image]
//! *   [Label start (link)][crate::construct::label_start_link]
//! *   [Label end][crate::construct::label_end]
//!
//! > 👉 **Note**: for performance reasons, hard break (trailing) is formed by
//! > [whitespace][crate::construct::partial_whitespace].

use crate::construct::gfm_autolink_literal::resolve as resolve_gfm_autolink_literal;
use crate::construct::partial_whitespace::resolve_whitespace;
use crate::resolve::Name as ResolveName;
use crate::state::{Name as StateName, State};
use crate::tokenizer::Tokenizer;

/// Characters that can start something in text.
const MARKERS: [u8; 10] = [
    b'!',  // `label_start_image`
    b'&',  // `character_reference`
    b'*',  // `attention`
    b'<',  // `autolink`, `html_text`
    b'[',  // `label_start_link`
    b'\\', // `character_escape`, `hard_break_escape`
    b']',  // `label_end`
    b'_',  // `attention`
    b'`',  // `code_text`
    b'~',  // `attention` (w/ `gfm_strikethrough`)
];

/// Start of text.
///
/// ```markdown
/// > | abc
///     ^
/// ```
pub fn start(tokenizer: &mut Tokenizer) -> State {
    tokenizer.tokenize_state.markers = &MARKERS;
    State::Retry(StateName::TextBefore)
}

/// Before text.
///
/// ```markdown
/// > | abc
///     ^
/// ```
pub fn before(tokenizer: &mut Tokenizer) -> State {
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
        Some(b'&') => {
            tokenizer.attempt(
                State::Next(StateName::TextBefore),
                State::Next(StateName::TextBeforeData),
            );
            State::Retry(StateName::CharacterReferenceStart)
        }
        Some(b'*' | b'_' | b'~') => {
            tokenizer.attempt(
                State::Next(StateName::TextBefore),
                State::Next(StateName::TextBeforeData),
            );
            State::Retry(StateName::AttentionStart)
        }
        // `autolink`, `html_text` (order does not matter)
        Some(b'<') => {
            tokenizer.attempt(
                State::Next(StateName::TextBefore),
                State::Next(StateName::TextBeforeHtml),
            );
            State::Retry(StateName::AutolinkStart)
        }
        Some(b'[') => {
            tokenizer.attempt(
                State::Next(StateName::TextBefore),
                State::Next(StateName::TextBeforeData),
            );
            State::Retry(StateName::LabelStartLinkStart)
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
        Some(b'`') => {
            tokenizer.attempt(
                State::Next(StateName::TextBefore),
                State::Next(StateName::TextBeforeData),
            );
            State::Retry(StateName::CodeTextStart)
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
        State::Next(StateName::TextBeforeData),
    );
    State::Retry(StateName::HtmlTextStart)
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
pub fn resolve(tokenizer: &mut Tokenizer) {
    resolve_whitespace(
        tokenizer,
        tokenizer.parse_state.options.constructs.hard_break_trailing,
        true,
    );

    if tokenizer
        .parse_state
        .options
        .constructs
        .gfm_autolink_literal
    {
        resolve_gfm_autolink_literal(tokenizer);
    }
}
