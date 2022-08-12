//! Label start (image) is a construct that occurs in the [text][] content
//! type.
//!
//! It forms with the following BNF:
//!
//! ```bnf
//! label_start_image ::= '!' '['
//! ```
//!
//! Label start (image) does not, on its own, relate to anything in HTML.
//! When matched with a [label end][label_end], they together relate to the
//! `<img>` element in HTML.
//! See [*§ 4.8.3 The `img` element*][html-img] in the HTML spec for more info.
//! Without an end, the characters (`![`) are output.
//!
//! ## Tokens
//!
//! *   [`LabelImage`][Name::LabelImage]
//! *   [`LabelImageMarker`][Name::LabelImageMarker]
//! *   [`LabelMarker`][Name::LabelMarker]
//!
//! ## References
//!
//! *   [`label-start-image.js` in `micromark`](https://github.com/micromark/micromark/blob/main/packages/micromark-core-commonmark/dev/lib/label-start-image.js)
//! *   [*§ 6.4 Images* in `CommonMark`](https://spec.commonmark.org/0.30/#images)
//!
//! [text]: crate::content::text
//! [label_end]: crate::construct::label_end
//! [html-img]: https://html.spec.whatwg.org/multipage/embedded-content.html#the-img-element

use crate::event::Name;
use crate::resolve::Name as ResolveName;
use crate::state::{Name as StateName, State};
use crate::tokenizer::{LabelStart, Tokenizer};

/// Start of label (image) start.
///
/// ```markdown
/// > | a ![b] c
///       ^
/// ```
pub fn start(tokenizer: &mut Tokenizer) -> State {
    if tokenizer.parse_state.constructs.label_start_image && tokenizer.current == Some(b'!') {
        tokenizer.enter(Name::LabelImage);
        tokenizer.enter(Name::LabelImageMarker);
        tokenizer.consume();
        tokenizer.exit(Name::LabelImageMarker);
        State::Next(StateName::LabelStartImageOpen)
    } else {
        State::Nok
    }
}

/// After `!`, at `[`.
///
/// ```markdown
/// > | a ![b] c
///        ^
/// ```
pub fn open(tokenizer: &mut Tokenizer) -> State {
    match tokenizer.current {
        Some(b'[') => {
            tokenizer.enter(Name::LabelMarker);
            tokenizer.consume();
            tokenizer.exit(Name::LabelMarker);
            tokenizer.exit(Name::LabelImage);
            tokenizer.tokenize_state.label_starts.push(LabelStart {
                start: (tokenizer.events.len() - 6, tokenizer.events.len() - 1),
                inactive: false,
            });
            tokenizer.register_resolver_before(ResolveName::Label);
            State::Ok
        }
        _ => State::Nok,
    }
}
