//! The text content type.
//!
//! **Text** contains phrasing content such as attention (emphasis, strong),
//! media (links, images), and actual text.
//!
//! The constructs found in text are:
//!
//! *   [Autolink][crate::construct::autolink]
//! *   Attention
//! *   [HTML (text)][crate::construct::html_text]
//! *   [Hard break (escape)][crate::construct::hard_break_escape]
//! *   [Hard break (trailing)][crate::construct::hard_break_trailing]
//! *   [Code (text)][crate::construct::code_text]
//! *   Label start (image)
//! *   Label start (link)
//! *   [Character escape][crate::construct::character_escape]
//! *   [Character reference][crate::construct::character_reference]

use crate::construct::{
    autolink::start as autolink, character_escape::start as character_escape,
    character_reference::start as character_reference, code_text::start as code_text,
    hard_break_escape::start as hard_break_escape,
    hard_break_trailing::start as hard_break_trailing, html_text::start as html_text,
    partial_data::start as data,
};
use crate::tokenizer::{Code, State, StateFnResult, Tokenizer};

const MARKERS: [Code; 5] = [
    Code::Char(' '),  // `hard_break_trailing`
    Code::Char('&'),  // `character_reference`
    Code::Char('<'),  // `autolink`, `html_text`
    Code::Char('\\'), // `character_escape`, `hard_break_escape`
    Code::Char('`'),  // `code_text`
];

/// Before text.
///
/// First we assume character reference.
///
/// ```markdown
/// |&amp;
/// |\&
/// |qwe
/// ```
pub fn start(tokenizer: &mut Tokenizer, code: Code) -> StateFnResult {
    match code {
        Code::None => (State::Ok, None),
        _ => tokenizer.attempt_n(
            vec![
                Box::new(character_reference),
                Box::new(character_escape),
                Box::new(hard_break_escape),
                Box::new(hard_break_trailing),
                Box::new(autolink),
                Box::new(html_text),
                Box::new(code_text),
            ],
            |ok| Box::new(if ok { start } else { before_data }),
        )(tokenizer, code),
    }
}

/// At data.
///
/// ```markdown
/// |qwe
/// ```
fn before_data(tokenizer: &mut Tokenizer, code: Code) -> StateFnResult {
    tokenizer.go(|t, c| data(t, c, MARKERS.to_vec()), start)(tokenizer, code)
}
