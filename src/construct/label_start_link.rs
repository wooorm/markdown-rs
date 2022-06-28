//! Label start (link) is a construct that occurs in the [text][] content
//! type.
//!
//! It forms with the following BNF:
//!
//! ```bnf
//! label_start_link ::= '['
//! ```
//!
//! Label start (link) relates to the `<a>` element in HTML.
//! See [*§ 4.5.1 The `a` element*][html-a] in the HTML spec for more info.
//!
//! Whether it contributes a link depends on whether it is followed by a
//! valid [label end][label_end] or not.
//! Without an end, the characters (`[`) are output.
//!
//! ## Tokens
//!
//! *   [`LabelLink`][TokenType::LabelLink]
//! *   [`LabelMarker`][TokenType::LabelMarker]
//!
//! ## References
//!
//! *   [`label-start-link.js` in `micromark`](https://github.com/micromark/micromark/blob/main/packages/micromark-core-commonmark/dev/lib/label-start-link.js)
//! *   [*§ 6.3 Links* in `CommonMark`](https://spec.commonmark.org/0.30/#links)
//!
//! [text]: crate::content::text
//! [label_end]: crate::construct::label_end
//! [html-a]: https://html.spec.whatwg.org/multipage/text-level-semantics.html#the-a-element

use super::label_end::resolve_media;
use crate::tokenizer::{Code, LabelStart, State, StateFnResult, TokenType, Tokenizer};

/// Start of label (link) start.
///
/// ```markdown
/// a |[ b
/// ```
pub fn start(tokenizer: &mut Tokenizer, code: Code) -> StateFnResult {
    match code {
        Code::Char('[') => {
            let start = tokenizer.events.len();
            tokenizer.enter(TokenType::LabelLink);
            tokenizer.enter(TokenType::LabelMarker);
            tokenizer.consume(code);
            tokenizer.exit(TokenType::LabelMarker);
            tokenizer.exit(TokenType::LabelLink);
            tokenizer.label_start_stack.push(LabelStart {
                start: (start, tokenizer.events.len() - 1),
                balanced: false,
                inactive: false,
            });
            tokenizer.register_resolver("media".to_string(), Box::new(resolve_media));
            (State::Ok, None)
        }
        _ => (State::Nok, None),
    }
}
