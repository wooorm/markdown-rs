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

use crate::construct::partial_whitespace::resolve_whitespace;
use crate::state::{Name as StateName, State};
use crate::tokenizer::Tokenizer;

const MARKERS: [u8; 9] = [
    b'!',  // `label_start_image`
    b'&',  // `character_reference`
    b'*',  // `attention`
    b'<',  // `autolink`, `html_text`
    b'[',  // `label_start_link`
    b'\\', // `character_escape`, `hard_break_escape`
    b']',  // `label_end`
    b'_',  // `attention`
    b'`',  // `code_text`
];

/// Start of text.
pub fn start(tokenizer: &mut Tokenizer) -> State {
    tokenizer.register_resolver("whitespace".to_string(), Box::new(resolve));
    tokenizer.tokenize_state.markers = &MARKERS;
    State::Retry(StateName::TextBefore)
}

/// Before text.
pub fn before(tokenizer: &mut Tokenizer) -> State {
    match tokenizer.current {
        None => State::Ok,
        Some(b'!') => tokenizer.attempt(
            StateName::LabelStartImageStart,
            State::Next(StateName::TextBefore),
            State::Next(StateName::TextBeforeData),
        ),
        Some(b'&') => tokenizer.attempt(
            StateName::CharacterReferenceStart,
            State::Next(StateName::TextBefore),
            State::Next(StateName::TextBeforeData),
        ),
        Some(b'*' | b'_') => tokenizer.attempt(
            StateName::AttentionStart,
            State::Next(StateName::TextBefore),
            State::Next(StateName::TextBeforeData),
        ),
        // `autolink`, `html_text` (order does not matter)
        Some(b'<') => tokenizer.attempt(
            StateName::AutolinkStart,
            State::Next(StateName::TextBefore),
            State::Next(StateName::TextBeforeHtml),
        ),
        Some(b'[') => tokenizer.attempt(
            StateName::LabelStartLinkStart,
            State::Next(StateName::TextBefore),
            State::Next(StateName::TextBeforeData),
        ),
        Some(b'\\') => tokenizer.attempt(
            StateName::CharacterEscapeStart,
            State::Next(StateName::TextBefore),
            State::Next(StateName::TextBeforeHardBreakEscape),
        ),
        Some(b']') => tokenizer.attempt(
            StateName::LabelEndStart,
            State::Next(StateName::TextBefore),
            State::Next(StateName::TextBeforeData),
        ),
        Some(b'`') => tokenizer.attempt(
            StateName::CodeTextStart,
            State::Next(StateName::TextBefore),
            State::Next(StateName::TextBeforeData),
        ),
        _ => State::Retry(StateName::TextBeforeData),
    }
}

/// At `<`, which wasn’t an autolink: before HTML?
pub fn before_html(tokenizer: &mut Tokenizer) -> State {
    tokenizer.attempt(
        StateName::HtmlTextStart,
        State::Next(StateName::TextBefore),
        State::Next(StateName::TextBeforeData),
    )
}

/// At `\`, which wasn’t a character escape: before a hard break?
pub fn before_hard_break_escape(tokenizer: &mut Tokenizer) -> State {
    tokenizer.attempt(
        StateName::HardBreakEscapeStart,
        State::Next(StateName::TextBefore),
        State::Next(StateName::TextBeforeData),
    )
}

/// At data.
pub fn before_data(tokenizer: &mut Tokenizer) -> State {
    tokenizer.attempt(
        StateName::DataStart,
        State::Next(StateName::TextBefore),
        State::Nok,
    )
}

/// Resolve whitespace.
pub fn resolve(tokenizer: &mut Tokenizer) {
    resolve_whitespace(
        tokenizer,
        tokenizer.parse_state.constructs.hard_break_trailing,
        true,
    );
}
