//! Label start (link) is a construct that occurs in the [text][] content
//! type.
//!
//! It forms with the following BNF:
//!
//! ```bnf
//! label_start_link ::= '['
//! ```
//!
//! Label start (link) does not, on its own, relate to anything in HTML.
//! When matched with a [label end][label_end], they together relate to the
//! `<a>` element in HTML.
//! See [*§ 4.5.1 The `a` element*][html-a] in the HTML spec for more info.
//! Without an end, the characters (`[`) are output.
//!
//! ## Tokens
//!
//! *   [`LabelLink`][Token::LabelLink]
//! *   [`LabelMarker`][Token::LabelMarker]
//!
//! ## References
//!
//! *   [`label-start-link.js` in `micromark`](https://github.com/micromark/micromark/blob/main/packages/micromark-core-commonmark/dev/lib/label-start-link.js)
//! *   [*§ 6.3 Links* in `CommonMark`](https://spec.commonmark.org/0.30/#links)
//!
//! [text]: crate::content::text
//! [label_end]: crate::construct::label_end
//! [html-a]: https://html.spec.whatwg.org/multipage/text-level-semantics.html#the-a-element

use crate::event::Name;
use crate::resolve::Name as ResolveName;
use crate::state::State;
use crate::tokenizer::{LabelStart, Tokenizer};

/// Start of label (link) start.
///
/// ```markdown
/// > | a [b] c
///       ^
/// ```
pub fn start(tokenizer: &mut Tokenizer) -> State {
    match tokenizer.current {
        Some(b'[') if tokenizer.parse_state.constructs.label_start_link => {
            let start = tokenizer.events.len();
            tokenizer.enter(Name::LabelLink);
            tokenizer.enter(Name::LabelMarker);
            tokenizer.consume();
            tokenizer.exit(Name::LabelMarker);
            tokenizer.exit(Name::LabelLink);
            tokenizer.tokenize_state.label_start_stack.push(LabelStart {
                start: (start, tokenizer.events.len() - 1),
                balanced: false,
                inactive: false,
            });
            tokenizer.register_resolver_before(ResolveName::Label);
            State::Ok
        }
        _ => State::Nok,
    }
}
