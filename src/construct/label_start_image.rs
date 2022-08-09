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
//! *   [`LabelImage`][Token::LabelImage]
//! *   [`LabelImageMarker`][Token::LabelImageMarker]
//! *   [`LabelMarker`][Token::LabelMarker]
//!
//! ## References
//!
//! *   [`label-start-image.js` in `micromark`](https://github.com/micromark/micromark/blob/main/packages/micromark-core-commonmark/dev/lib/label-start-image.js)
//! *   [*§ 6.4 Images* in `CommonMark`](https://spec.commonmark.org/0.30/#images)
//!
//! [text]: crate::content::text
//! [label_end]: crate::construct::label_end
//! [html-img]: https://html.spec.whatwg.org/multipage/embedded-content.html#the-img-element

use super::label_end::resolve_media;
use crate::token::Token;
use crate::tokenizer::{LabelStart, State, StateName, Tokenizer};

/// Start of label (image) start.
///
/// ```markdown
/// > | a ![b] c
///       ^
/// ```
pub fn start(tokenizer: &mut Tokenizer) -> State {
    match tokenizer.current {
        Some(b'!') if tokenizer.parse_state.constructs.label_start_image => {
            tokenizer.enter(Token::LabelImage);
            tokenizer.enter(Token::LabelImageMarker);
            tokenizer.consume();
            tokenizer.exit(Token::LabelImageMarker);
            State::Fn(StateName::LabelStartImageOpen)
        }
        _ => State::Nok,
    }
}

/// After `!`, before a `[`.
///
/// ```markdown
/// > | a ![b] c
///        ^
/// ```
pub fn open(tokenizer: &mut Tokenizer) -> State {
    match tokenizer.current {
        Some(b'[') => {
            tokenizer.enter(Token::LabelMarker);
            tokenizer.consume();
            tokenizer.exit(Token::LabelMarker);
            tokenizer.exit(Token::LabelImage);
            tokenizer.label_start_stack.push(LabelStart {
                start: (tokenizer.events.len() - 6, tokenizer.events.len() - 1),
                balanced: false,
                inactive: false,
            });
            tokenizer.register_resolver_before("media".to_string(), Box::new(resolve_media));
            State::Ok
        }
        _ => State::Nok,
    }
}
