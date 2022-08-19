//! Heading (setext) occurs in the [flow][] content type.
//!
//! ## Grammar
//!
//! Heading (setext) forms with the following BNF
//! (<small>see [construct][crate::construct] for character groups</small>):
//!
//! ```bnf
//! heading_setext ::= paragraph eol *space_or_tab (1*'-' | 1*'=')  *space_or_tab
//!
//! ; See the `paragraph` construct for the BNF of that part.
//! ```
//!
//! As this construct occurs in flow, like all flow constructs, it must be
//! followed by an eol (line ending) or eof (end of file).
//!
//! See [`paragraph`][paragraph] for grammar, notes, and recommendations on
//! that part.
//!
//! In markdown, it is also possible to create headings with a
//! [heading (atx)][heading_atx] construct.
//! The benefit of setext headings is that their text can include line endings,
//! and by extensions also hard breaks (e.g., with
//! [hard break (escape)][hard_break_escape]).
//! However, their limit is that they cannot form `<h3>` through `<h6>`
//! headings.
//!
//! [Thematic breaks][thematic_break] formed with dashes and without whitespace
//! could be interpreted as a heading (setext).
//! Which one forms depends on whether there is text directly in fron of the
//! sequence.
//!
//! > 🏛 **Background**: the word *setext* originates from a small markup
//! > language by Ian Feldman from 1991.
//! > See [*§ Setext* on Wikipedia][wiki_setext] for more info.
//! > The word *atx* originates from a tiny markup language by Aaron Swartz
//! > from 2002.
//! > See [*§ atx, the true structured text format* on `aaronsw.com`][atx] for
//! > more info.
//!
//! ## HTML
//!
//! Heading (setext) in markdown relates to the `<h1>` and `<h2>` elements in
//! HTML.
//! See [*§ 4.3.6 The `h1`, `h2`, `h3`, `h4`, `h5`, and `h6` elements* in the
//! HTML spec][html] for more info.
//!
//! ## Recommendation
//!
//! Always use heading (atx), never heading (setext).
//!
//! ## Tokens
//!
//! *   [`HeadingSetext`][Name::HeadingSetext]
//! *   [`HeadingSetextText`][Name::HeadingSetextText]
//! *   [`HeadingSetextUnderline`][Name::HeadingSetextUnderline]
//!
//! ## References
//!
//! *   [`setext-underline.js` in `micromark`](https://github.com/micromark/micromark/blob/main/packages/micromark-core-commonmark/dev/lib/setext-underline.js)
//! *   [*§ 4.3 Setext headings* in `CommonMark`](https://spec.commonmark.org/0.30/#setext-headings)
//!
//! [flow]: crate::construct::flow
//! [paragraph]: crate::construct::paragraph
//! [heading_atx]: crate::construct::heading_atx
//! [thematic_break]: crate::construct::thematic_break
//! [hard_break_escape]: crate::construct::hard_break_escape
//! [html]: https://html.spec.whatwg.org/multipage/sections.html#the-h1,-h2,-h3,-h4,-h5,-and-h6-elements
//! [wiki_setext]: https://en.wikipedia.org/wiki/Setext
//! [atx]: http://www.aaronsw.com/2002/atx/

use crate::construct::partial_space_or_tab::{space_or_tab, space_or_tab_min_max};
use crate::event::{Kind, Name};
use crate::resolve::Name as ResolveName;
use crate::state::{Name as StateName, State};
use crate::tokenizer::Tokenizer;
use crate::util::{constant::TAB_SIZE, skip::opt_back as skip_opt_back};
use alloc::vec;

/// At start of heading (setext) underline.
///
/// ```markdown
///   | aa
/// > | ==
///     ^
/// ```
pub fn start(tokenizer: &mut Tokenizer) -> State {
    if tokenizer.parse_state.constructs.heading_setext
        && !tokenizer.lazy
        // Require a paragraph before.
        && (!tokenizer.events.is_empty()
            && tokenizer.events[skip_opt_back(
                &tokenizer.events,
                tokenizer.events.len() - 1,
                &[Name::LineEnding, Name::SpaceOrTab],
            )]
            .name
                == Name::Paragraph)
    {
        if matches!(tokenizer.current, Some(b'\t' | b' ')) {
            tokenizer.attempt(State::Next(StateName::HeadingSetextBefore), State::Nok);
            State::Retry(space_or_tab_min_max(
                tokenizer,
                0,
                if tokenizer.parse_state.constructs.code_indented {
                    TAB_SIZE - 1
                } else {
                    usize::MAX
                },
            ))
        } else {
            State::Retry(StateName::HeadingSetextBefore)
        }
    } else {
        State::Nok
    }
}

/// After optional whitespace, at `-` or `=`.
///
/// ```markdown
///   | aa
/// > | ==
///     ^
/// ```
pub fn before(tokenizer: &mut Tokenizer) -> State {
    match tokenizer.current {
        Some(b'-' | b'=') => {
            tokenizer.tokenize_state.marker = tokenizer.current.unwrap();
            tokenizer.enter(Name::HeadingSetextUnderline);
            State::Retry(StateName::HeadingSetextInside)
        }
        _ => State::Nok,
    }
}

/// In sequence.
///
/// ```markdown
///   | aa
/// > | ==
///     ^
/// ```
pub fn inside(tokenizer: &mut Tokenizer) -> State {
    if tokenizer.current == Some(tokenizer.tokenize_state.marker) {
        tokenizer.consume();
        State::Next(StateName::HeadingSetextInside)
    } else {
        tokenizer.tokenize_state.marker = 0;
        tokenizer.exit(Name::HeadingSetextUnderline);

        if matches!(tokenizer.current, Some(b'\t' | b' ')) {
            tokenizer.attempt(State::Next(StateName::HeadingSetextAfter), State::Nok);
            State::Retry(space_or_tab(tokenizer))
        } else {
            State::Retry(StateName::HeadingSetextAfter)
        }
    }
}

/// After sequence, after optional whitespace.
///
/// ```markdown
///   | aa
/// > | ==
///       ^
/// ```
pub fn after(tokenizer: &mut Tokenizer) -> State {
    match tokenizer.current {
        None | Some(b'\n') => {
            // Feel free to interrupt.
            tokenizer.interrupt = false;
            tokenizer.register_resolver(ResolveName::HeadingSetext);
            State::Ok
        }
        _ => State::Nok,
    }
}

/// Resolve heading (setext).
pub fn resolve(tokenizer: &mut Tokenizer) {
    let mut index = 0;
    let mut paragraph_enter = None;
    let mut paragraph_exit = None;

    while index < tokenizer.events.len() {
        let event = &tokenizer.events[index];

        // Find paragraphs.
        if event.kind == Kind::Enter {
            if event.name == Name::Paragraph {
                paragraph_enter = Some(index);
            }
        } else if event.name == Name::Paragraph {
            paragraph_exit = Some(index);
        }
        // We know this is preceded by a paragraph.
        // Otherwise we don’t parse.
        else if event.name == Name::HeadingSetextUnderline {
            let enter = paragraph_enter.take().unwrap();
            let exit = paragraph_exit.take().unwrap();

            // Change types of Enter:Paragraph, Exit:Paragraph.
            tokenizer.events[enter].name = Name::HeadingSetextText;
            tokenizer.events[exit].name = Name::HeadingSetextText;

            // Add Enter:HeadingSetext, Exit:HeadingSetext.
            let mut heading_enter = tokenizer.events[enter].clone();
            heading_enter.name = Name::HeadingSetext;
            let mut heading_exit = tokenizer.events[index].clone();
            heading_exit.name = Name::HeadingSetext;

            tokenizer.map.add(enter, 0, vec![heading_enter]);
            tokenizer.map.add(index + 1, 0, vec![heading_exit]);
        }

        index += 1;
    }
}
