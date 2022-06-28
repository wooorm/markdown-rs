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
//! *   [`LabelImage`][TokenType::LabelImage]
//! *   [`LabelImageMarker`][TokenType::LabelImageMarker]
//! *   [`LabelMarker`][TokenType::LabelMarker]
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
use crate::tokenizer::{Code, LabelStart, State, StateFnResult, TokenType, Tokenizer};

/// Start of label (image) start.
///
/// ```markdown
/// a |![ b
/// ```
pub fn start(tokenizer: &mut Tokenizer, code: Code) -> StateFnResult {
    match code {
        Code::Char('!') => {
            tokenizer.enter(TokenType::LabelImage);
            tokenizer.enter(TokenType::LabelImageMarker);
            tokenizer.consume(code);
            tokenizer.exit(TokenType::LabelImageMarker);
            (State::Fn(Box::new(open)), None)
        }
        _ => (State::Nok, None),
    }
}

/// After `!`, before a `[`.
///
/// ```markdown
/// a !|[ b
/// ```
pub fn open(tokenizer: &mut Tokenizer, code: Code) -> StateFnResult {
    match code {
        Code::Char('[') => {
            tokenizer.enter(TokenType::LabelMarker);
            tokenizer.consume(code);
            tokenizer.exit(TokenType::LabelMarker);
            tokenizer.exit(TokenType::LabelImage);
            let end = tokenizer.events.len() - 1;
            tokenizer.label_start_stack.push(LabelStart {
                start: (end - 5, end),
                balanced: false,
                inactive: false,
            });
            tokenizer.register_resolver("media".to_string(), Box::new(resolve_media));
            (State::Ok, None)
        }
        _ => (State::Nok, None),
    }
}
