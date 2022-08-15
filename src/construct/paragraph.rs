//! Paragraph is a construct that occurs in the [flow] content type.
//!
//! They’re formed with the following BNF:
//!
//! ```bnf
//! ; Restriction: lines cannot start other flow constructs.
//! ; Restriction: lines cannot be blank.
//! paragraph ::= 1*line *( eol 1*line )
//! ```
//!
//! Paragraphs in markdown relate to the `<p>` element in HTML.
//! See [*§ 4.4.1 The `p` element* in the HTML spec][html] for more info.
//!
//! Paragraphs can contain line endings and whitespace, but they are not
//! allowed to contain blank lines, or to be blank themselves.
//!
//! The paragraph is interpreted as the [text][] content type.
//! That means that [autolinks][autolink], [code (text)][code_text], etc are allowed.
//!
//! ## Tokens
//!
//! *   [`Paragraph`][Name::Paragraph]
//!
//! ## References
//!
//! *   [`content.js` in `micromark`](https://github.com/micromark/micromark/blob/main/packages/micromark-core-commonmark/dev/lib/content.js)
//! *   [*§ 4.8 Paragraphs* in `CommonMark`](https://spec.commonmark.org/0.30/#paragraphs)
//!
//! [flow]: crate::construct::flow
//! [text]: crate::construct::text
//! [autolink]: crate::construct::autolink
//! [code_text]: crate::construct::code_text
//! [html]: https://html.spec.whatwg.org/multipage/grouping-content.html#the-p-element

use crate::event::{Content, Kind, Link, Name};
use crate::resolve::Name as ResolveName;
use crate::state::{Name as StateName, State};
use crate::tokenizer::Tokenizer;

/// Before paragraph.
///
/// ```markdown
/// > | abc
///     ^
/// ```
pub fn start(tokenizer: &mut Tokenizer) -> State {
    match tokenizer.current {
        None | Some(b'\n') => unreachable!("unexpected eol/eof"),
        _ => {
            tokenizer.enter(Name::Paragraph);
            tokenizer.enter_link(
                Name::Data,
                Link {
                    previous: None,
                    next: None,
                    content: Content::Text,
                },
            );
            State::Retry(StateName::ParagraphInside)
        }
    }
}

/// In paragraph.
///
/// ```markdown
/// > | abc
///     ^^^
/// ```
pub fn inside(tokenizer: &mut Tokenizer) -> State {
    match tokenizer.current {
        None | Some(b'\n') => {
            tokenizer.exit(Name::Data);
            tokenizer.exit(Name::Paragraph);
            tokenizer.register_resolver_before(ResolveName::Paragraph);
            // You’d be interrupting.
            tokenizer.interrupt = true;
            State::Ok
        }
        _ => {
            tokenizer.consume();
            State::Next(StateName::ParagraphInside)
        }
    }
}

/// Merge “`Paragraph`”s, which currently span a single line, into actual
/// `Paragraph`s that span multiple lines.
pub fn resolve(tokenizer: &mut Tokenizer) {
    let mut index = 0;

    while index < tokenizer.events.len() {
        let event = &tokenizer.events[index];

        if event.kind == Kind::Enter && event.name == Name::Paragraph {
            // Exit:Paragraph
            let mut exit_index = index + 3;

            loop {
                let mut enter_index = exit_index + 1;

                if enter_index == tokenizer.events.len()
                    || tokenizer.events[enter_index].name != Name::LineEnding
                {
                    break;
                }

                enter_index += 2;

                while enter_index < tokenizer.events.len() {
                    let event = &tokenizer.events[enter_index];

                    if event.name != Name::SpaceOrTab
                        && event.name != Name::BlockQuotePrefix
                        && event.name != Name::BlockQuoteMarker
                    {
                        break;
                    }

                    enter_index += 1;
                }

                if enter_index == tokenizer.events.len()
                    || tokenizer.events[enter_index].name != Name::Paragraph
                {
                    break;
                }

                // Remove Exit:Paragraph, Enter:LineEnding, Exit:LineEnding.
                tokenizer.map.add(exit_index, 3, vec![]);

                // Remove Enter:Paragraph.
                tokenizer.map.add(enter_index, 1, vec![]);

                // Add Exit:LineEnding position info to Exit:Data.
                tokenizer.events[exit_index - 1].point =
                    tokenizer.events[exit_index + 2].point.clone();

                // Link Enter:Data on the previous line to Enter:Data on this line.
                if let Some(link) = &mut tokenizer.events[exit_index - 2].link {
                    link.next = Some(enter_index + 1);
                }
                if let Some(link) = &mut tokenizer.events[enter_index + 1].link {
                    link.previous = Some(exit_index - 2);
                }

                // Potential next start.
                exit_index = enter_index + 3;
            }

            // Move to `Exit:Paragraph`.
            index = exit_index;
        }

        index += 1;
    }

    tokenizer.map.consume(&mut tokenizer.events);
}
