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

use crate::construct::{
    attention::start as attention, autolink::start as autolink,
    character_escape::start as character_escape, character_reference::start as character_reference,
    code_text::start as code_text, hard_break_escape::start as hard_break_escape,
    html_text::start as html_text, label_end::start as label_end,
    label_start_image::start as label_start_image, label_start_link::start as label_start_link,
    partial_data::start as data, partial_whitespace::create_resolve_whitespace,
};
use crate::tokenizer::{State, Tokenizer};

const MARKERS: [char; 9] = [
    '!',  // `label_start_image`
    '&',  // `character_reference`
    '*',  // `attention`
    '<',  // `autolink`, `html_text`
    '[',  // `label_start_link`
    '\\', // `character_escape`, `hard_break_escape`
    ']',  // `label_end`
    '_',  // `attention`
    '`',  // `code_text`
];

/// Start of text.
pub fn start(tokenizer: &mut Tokenizer) -> State {
    tokenizer.register_resolver(
        "whitespace".to_string(),
        Box::new(create_resolve_whitespace(
            tokenizer.parse_state.constructs.hard_break_trailing,
            true,
        )),
    );
    before(tokenizer)
}

/// Before text.
pub fn before(tokenizer: &mut Tokenizer) -> State {
    match tokenizer.current {
        None => State::Ok,
        _ => tokenizer.attempt_n(
            vec![
                Box::new(attention),
                Box::new(autolink),
                Box::new(character_escape),
                Box::new(character_reference),
                Box::new(code_text),
                Box::new(hard_break_escape),
                Box::new(html_text),
                Box::new(label_end),
                Box::new(label_start_image),
                Box::new(label_start_link),
            ],
            |ok| Box::new(if ok { before } else { before_data }),
        )(tokenizer),
    }
}

/// At data.
///
/// ```markdown
/// |qwe
/// ```
fn before_data(tokenizer: &mut Tokenizer) -> State {
    tokenizer.go(|t| data(t, &MARKERS), before)(tokenizer)
}
