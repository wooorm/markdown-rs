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
use crate::state::{Name, State};
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
    State::Retry(Name::TextBefore)
}

/// Before text.
pub fn before(tokenizer: &mut Tokenizer) -> State {
    match tokenizer.current {
        None => State::Ok,
        Some(b'!') => tokenizer.attempt(
            Name::LabelStartImageStart,
            State::Next(Name::TextBefore),
            State::Next(Name::TextBeforeData),
        ),
        Some(b'&') => tokenizer.attempt(
            Name::CharacterReferenceStart,
            State::Next(Name::TextBefore),
            State::Next(Name::TextBeforeData),
        ),
        Some(b'*' | b'_') => tokenizer.attempt(
            Name::AttentionStart,
            State::Next(Name::TextBefore),
            State::Next(Name::TextBeforeData),
        ),
        // `autolink`, `html_text` (order does not matter)
        Some(b'<') => tokenizer.attempt(
            Name::AutolinkStart,
            State::Next(Name::TextBefore),
            State::Next(Name::TextBeforeHtml),
        ),
        Some(b'[') => tokenizer.attempt(
            Name::LabelStartLinkStart,
            State::Next(Name::TextBefore),
            State::Next(Name::TextBeforeData),
        ),
        Some(b'\\') => tokenizer.attempt(
            Name::CharacterEscapeStart,
            State::Next(Name::TextBefore),
            State::Next(Name::TextBeforeHardBreakEscape),
        ),
        Some(b']') => tokenizer.attempt(
            Name::LabelEndStart,
            State::Next(Name::TextBefore),
            State::Next(Name::TextBeforeData),
        ),
        Some(b'`') => tokenizer.attempt(
            Name::CodeTextStart,
            State::Next(Name::TextBefore),
            State::Next(Name::TextBeforeData),
        ),
        _ => State::Retry(Name::TextBeforeData),
    }
}

/// At `<`, which wasn’t an autolink: before HTML?
pub fn before_html(tokenizer: &mut Tokenizer) -> State {
    tokenizer.attempt(
        Name::HtmlTextStart,
        State::Next(Name::TextBefore),
        State::Next(Name::TextBeforeData),
    )
}

/// At `\`, which wasn’t a character escape: before a hard break?
pub fn before_hard_break_escape(tokenizer: &mut Tokenizer) -> State {
    tokenizer.attempt(
        Name::HardBreakEscapeStart,
        State::Next(Name::TextBefore),
        State::Next(Name::TextBeforeData),
    )
}

/// At data.
pub fn before_data(tokenizer: &mut Tokenizer) -> State {
    tokenizer.attempt(Name::DataStart, State::Next(Name::TextBefore), State::Nok)
}

/// Resolve whitespace.
pub fn resolve(tokenizer: &mut Tokenizer) {
    resolve_whitespace(
        tokenizer,
        tokenizer.parse_state.constructs.hard_break_trailing,
        true,
    );
}
