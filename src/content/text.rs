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
//! *   [Hard break (trailing)][crate::construct::hard_break_trailing]
//! *   [HTML (text)][crate::construct::html_text]
//! *   [Label start (image)][crate::construct::label_start_image]
//! *   [Label start (link)][crate::construct::label_start_link]
//! *   [Label end][crate::construct::label_end]

use crate::construct::{
    attention::start as attention, autolink::start as autolink,
    character_escape::start as character_escape, character_reference::start as character_reference,
    code_text::start as code_text, hard_break_escape::start as hard_break_escape,
    hard_break_trailing::start as hard_break_trailing, html_text::start as html_text,
    label_end::start as label_end, label_start_image::start as label_start_image,
    label_start_link::start as label_start_link, partial_data::start as data,
    partial_whitespace::whitespace,
};
use crate::tokenizer::{Code, State, StateFnResult, Tokenizer};

/// Before text.
pub fn start(tokenizer: &mut Tokenizer, code: Code) -> StateFnResult {
    let mut markers = vec![
        Code::VirtualSpace, // `whitespace`
        Code::Char('\t'),   // `whitespace`
        Code::Char(' '),    // `hard_break_trailing`, `whitespace`
    ];

    if tokenizer.parse_state.constructs.label_start_image {
        markers.push(Code::Char('!'));
    }
    if tokenizer.parse_state.constructs.character_reference {
        markers.push(Code::Char('&'));
    }
    if tokenizer.parse_state.constructs.attention {
        markers.push(Code::Char('*'));
    }
    if tokenizer.parse_state.constructs.autolink || tokenizer.parse_state.constructs.html_text {
        markers.push(Code::Char('<'));
    }
    if tokenizer.parse_state.constructs.label_start_link {
        markers.push(Code::Char('['));
    }
    if tokenizer.parse_state.constructs.character_escape
        || tokenizer.parse_state.constructs.hard_break_escape
    {
        markers.push(Code::Char('\\'));
    }
    if tokenizer.parse_state.constructs.label_end {
        markers.push(Code::Char(']'));
    }
    if tokenizer.parse_state.constructs.attention {
        markers.push(Code::Char('_'));
    }
    if tokenizer.parse_state.constructs.code_text {
        markers.push(Code::Char('`'));
    }

    before_marker(tokenizer, code, markers)
}

/// Before text.
fn before_marker(tokenizer: &mut Tokenizer, code: Code, markers: Vec<Code>) -> StateFnResult {
    match code {
        Code::None => (State::Ok, None),
        _ => tokenizer.attempt_n(
            vec![
                Box::new(attention),
                Box::new(autolink),
                Box::new(character_escape),
                Box::new(character_reference),
                Box::new(code_text),
                Box::new(hard_break_escape),
                Box::new(hard_break_trailing),
                Box::new(html_text),
                Box::new(label_end),
                Box::new(label_start_image),
                Box::new(label_start_link),
                Box::new(whitespace),
            ],
            |ok| {
                let func = if ok { before_marker } else { before_data };
                Box::new(move |t, c| func(t, c, markers))
            },
        )(tokenizer, code),
    }
}

/// At data.
///
/// ```markdown
/// |qwe
/// ```
fn before_data(tokenizer: &mut Tokenizer, code: Code, markers: Vec<Code>) -> StateFnResult {
    let copy = markers.clone();
    tokenizer.go(|t, c| data(t, c, copy), |t, c| before_marker(t, c, markers))(tokenizer, code)
}
