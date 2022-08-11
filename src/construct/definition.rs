//! Definition is a construct that occurs in the [flow] content type.
//!
//! They’re formed with the following BNF:
//!
//! ```bnf
//! definition ::= label ':' [ whitespace ] destination [ whitespace title ] [ space_or_tab ]
//!
//! ; See the `destination`, `title`, and `label` constructs for the BNF of
//! ; those parts.
//! ```
//!
//! See [`destination`][destination], [`label`][label], and [`title`][title]
//! for grammar, notes, and recommendations.
//!
//! Definitions in markdown do not, on their own, relate to anything in HTML.
//! When matched with a [label end (reference)][label_end], they together
//! relate to the `<a>` or `<img>` elements in HTML.
//! The definition forms its `href` or `src`, and optionally `title`,
//! attributes.
//! See [*§ 4.5.1 The `a` element*][html-a] and
//! [*§ 4.8.3 The `img` element*][html-img] in the HTML spec for more info.
//!
//! The `destination`, `label`, and `title` parts are interpreted as the
//! [string][] content type.
//! That means that [character escapes][character_escape] and
//! [character references][character_reference] are allowed.
//!
//! Definitions match to references through their label.
//! To match, both labels must be equal after normalizing with
//! [`normalize_identifier`][normalize_identifier].
//! One definition can match to multiple references.
//! Multiple definitions with the same, normalized, identifier are ignored: the
//! first definition is preferred.
//! To illustrate, the definition with a destination of `x` wins:
//!
//! ```markdown
//! [a]: x
//! [a]: y
//!
//! [a]
//! ```
//!
//! Importantly, while labels *can* include [string][] content (character
//! escapes and character references), these are not considered when matching.
//! To illustrate, neither definition matches the reference:
//!
//! ```markdown
//! [a&amp;b]: x
//! [a\&b]: y
//!
//! [a&b]
//! ```
//!
//! For info on how to encode characters in URLs, see
//! [`destination`][destination].
//! For info on how characters are encoded as `href` on `<a>` or `src` on
//! `<img>` when compiling, see
//! [`sanitize_uri`][sanitize_uri].
//!
//! ## Tokens
//!
//! *   [`Definition`][Name::Definition]
//! *   [`DefinitionDestination`][Name::DefinitionDestination]
//! *   [`DefinitionDestinationLiteral`][Name::DefinitionDestinationLiteral]
//! *   [`DefinitionDestinationLiteralMarker`][Name::DefinitionDestinationLiteralMarker]
//! *   [`DefinitionDestinationRaw`][Name::DefinitionDestinationRaw]
//! *   [`DefinitionDestinationString`][Name::DefinitionDestinationString]
//! *   [`DefinitionLabel`][Name::DefinitionLabel]
//! *   [`DefinitionLabelMarker`][Name::DefinitionLabelMarker]
//! *   [`DefinitionLabelString`][Name::DefinitionLabelString]
//! *   [`DefinitionMarker`][Name::DefinitionMarker]
//! *   [`DefinitionTitle`][Name::DefinitionTitle]
//! *   [`DefinitionTitleMarker`][Name::DefinitionTitleMarker]
//! *   [`DefinitionTitleString`][Name::DefinitionTitleString]
//! *   [`LineEnding`][Name::LineEnding]
//! *   [`SpaceOrTab`][Name::SpaceOrTab]
//!
//! ## References
//!
//! *   [`definition.js` in `micromark`](https://github.com/micromark/micromark/blob/main/packages/micromark-core-commonmark/dev/lib/definition.js)
//! *   [*§ 4.7 Link reference definitions* in `CommonMark`](https://spec.commonmark.org/0.30/#link-reference-definitions)
//!
//! [flow]: crate::content::flow
//! [string]: crate::content::string
//! [character_escape]: crate::construct::character_escape
//! [character_reference]: crate::construct::character_reference
//! [label_end]: crate::construct::label_end
//! [destination]: crate::construct::partial_destination
//! [title]: crate::construct::partial_title
//! [label]: crate::construct::partial_label
//! [sanitize_uri]: crate::util::sanitize_uri::sanitize_uri
//! [normalize_identifier]: crate::util::normalize_identifier
//! [html-a]: https://html.spec.whatwg.org/multipage/text-level-semantics.html#the-a-element
//! [html-img]: https://html.spec.whatwg.org/multipage/embedded-content.html#the-img-element

use crate::construct::partial_space_or_tab::space_or_tab;
use crate::construct::partial_space_or_tab_eol::space_or_tab_eol;
use crate::event::Name;
use crate::state::{Name as StateName, State};
use crate::tokenizer::Tokenizer;
use crate::util::{
    normalize_identifier::normalize_identifier,
    skip,
    slice::{Position, Slice},
};

/// At the start of a definition.
///
/// ```markdown
/// > | [a]: b "c"
///     ^
/// ```
pub fn start(tokenizer: &mut Tokenizer) -> State {
    // Do not interrupt paragraphs (but do follow definitions).
    let possible = !tokenizer.interrupt
        || (!tokenizer.events.is_empty()
            && tokenizer.events[skip::opt_back(
                &tokenizer.events,
                tokenizer.events.len() - 1,
                &[Name::LineEnding, Name::SpaceOrTab],
            )]
            .name
                == Name::Definition);

    if possible && tokenizer.parse_state.constructs.definition {
        tokenizer.enter(Name::Definition);
        // Note: arbitrary whitespace allowed even if code (indented) is on.
        let name = space_or_tab(tokenizer);
        tokenizer.attempt(
            name,
            State::Next(StateName::DefinitionBefore),
            State::Next(StateName::DefinitionBefore),
        )
    } else {
        State::Nok
    }
}

/// At the start of a definition, after whitespace.
///
/// ```markdown
/// > | [a]: b "c"
///     ^
/// ```
pub fn before(tokenizer: &mut Tokenizer) -> State {
    match tokenizer.current {
        Some(b'[') => {
            tokenizer.tokenize_state.token_1 = Name::DefinitionLabel;
            tokenizer.tokenize_state.token_2 = Name::DefinitionLabelMarker;
            tokenizer.tokenize_state.token_3 = Name::DefinitionLabelString;
            tokenizer.attempt(
                StateName::LabelStart,
                State::Next(StateName::DefinitionLabelAfter),
                State::Nok,
            )
        }
        _ => State::Nok,
    }
}

/// After the label of a definition.
///
/// ```markdown
/// > | [a]: b "c"
///        ^
/// ```
pub fn label_after(tokenizer: &mut Tokenizer) -> State {
    tokenizer.tokenize_state.token_1 = Name::Data;
    tokenizer.tokenize_state.token_2 = Name::Data;
    tokenizer.tokenize_state.token_3 = Name::Data;

    tokenizer.tokenize_state.end = skip::to_back(
        &tokenizer.events,
        tokenizer.events.len() - 1,
        &[Name::DefinitionLabelString],
    );

    match tokenizer.current {
        Some(b':') => {
            tokenizer.enter(Name::DefinitionMarker);
            tokenizer.consume();
            tokenizer.exit(Name::DefinitionMarker);
            State::Next(StateName::DefinitionMarkerAfter)
        }
        _ => State::Nok,
    }
}

/// After the marker.
///
/// ```markdown
/// > | [a]: b "c"
///         ^
/// ```
pub fn marker_after(tokenizer: &mut Tokenizer) -> State {
    let name = space_or_tab_eol(tokenizer);
    tokenizer.attempt(
        name,
        State::Next(StateName::DefinitionDestinationBefore),
        State::Next(StateName::DefinitionDestinationBefore),
    )
}

/// Before a destination.
///
/// ```markdown
/// > | [a]: b "c"
///          ^
/// ```
pub fn destination_before(tokenizer: &mut Tokenizer) -> State {
    tokenizer.tokenize_state.token_1 = Name::DefinitionDestination;
    tokenizer.tokenize_state.token_2 = Name::DefinitionDestinationLiteral;
    tokenizer.tokenize_state.token_3 = Name::DefinitionDestinationLiteralMarker;
    tokenizer.tokenize_state.token_4 = Name::DefinitionDestinationRaw;
    tokenizer.tokenize_state.token_5 = Name::DefinitionDestinationString;
    tokenizer.tokenize_state.size_b = usize::MAX;
    tokenizer.attempt(
        StateName::DestinationStart,
        State::Next(StateName::DefinitionDestinationAfter),
        State::Next(StateName::DefinitionDestinationMissing),
    )
}

/// After a destination.
///
/// ```markdown
/// > | [a]: b "c"
///           ^
/// ```
pub fn destination_after(tokenizer: &mut Tokenizer) -> State {
    tokenizer.tokenize_state.token_1 = Name::Data;
    tokenizer.tokenize_state.token_2 = Name::Data;
    tokenizer.tokenize_state.token_3 = Name::Data;
    tokenizer.tokenize_state.token_4 = Name::Data;
    tokenizer.tokenize_state.token_5 = Name::Data;
    tokenizer.tokenize_state.size_b = 0;
    tokenizer.attempt(
        StateName::DefinitionTitleBefore,
        State::Next(StateName::DefinitionAfter),
        State::Next(StateName::DefinitionAfter),
    )
}

/// Without destination.
pub fn destination_missing(tokenizer: &mut Tokenizer) -> State {
    tokenizer.tokenize_state.token_1 = Name::Data;
    tokenizer.tokenize_state.token_2 = Name::Data;
    tokenizer.tokenize_state.token_3 = Name::Data;
    tokenizer.tokenize_state.token_4 = Name::Data;
    tokenizer.tokenize_state.token_5 = Name::Data;
    tokenizer.tokenize_state.size_b = 0;
    tokenizer.tokenize_state.end = 0;
    State::Nok
}

/// After a definition.
///
/// ```markdown
/// > | [a]: b
///           ^
/// > | [a]: b "c"
///               ^
/// ```
pub fn after(tokenizer: &mut Tokenizer) -> State {
    let name = space_or_tab(tokenizer);
    tokenizer.attempt(
        name,
        State::Next(StateName::DefinitionAfterWhitespace),
        State::Next(StateName::DefinitionAfterWhitespace),
    )
}

/// After a definition, after optional whitespace.
///
/// ```markdown
/// > | [a]: b
///           ^
/// > | [a]: b "c"
///               ^
/// ```
pub fn after_whitespace(tokenizer: &mut Tokenizer) -> State {
    match tokenizer.current {
        None | Some(b'\n') => {
            tokenizer.exit(Name::Definition);

            // Note: we don’t care about uniqueness.
            // It’s likely that that doesn’t happen very frequently.
            // It is more likely that it wastes precious time.
            tokenizer.tokenize_state.definitions.push(
                // Note: we don’t care about virtual spaces, so `as_str` is fine.
                normalize_identifier(
                    Slice::from_position(
                        tokenizer.parse_state.bytes,
                        &Position::from_exit_event(&tokenizer.events, tokenizer.tokenize_state.end),
                    )
                    .as_str(),
                ),
            );

            tokenizer.tokenize_state.end = 0;

            // You’d be interrupting.
            tokenizer.interrupt = true;
            State::Ok
        }
        _ => {
            tokenizer.tokenize_state.end = 0;
            State::Nok
        },
    }
}

/// After a destination, presumably before a title.
///
/// ```markdown
/// > | [a]: b
///           ^
/// > | [a]: b "c"
///           ^
/// ```
pub fn title_before(tokenizer: &mut Tokenizer) -> State {
    let name = space_or_tab_eol(tokenizer);
    tokenizer.attempt(
        name,
        State::Next(StateName::DefinitionTitleBeforeMarker),
        State::Nok,
    )
}

/// Before a title, after a line ending.
///
/// ```markdown
///   | [a]: b
/// > | "c"
///     ^
/// ```
pub fn title_before_marker(tokenizer: &mut Tokenizer) -> State {
    tokenizer.tokenize_state.token_1 = Name::DefinitionTitle;
    tokenizer.tokenize_state.token_2 = Name::DefinitionTitleMarker;
    tokenizer.tokenize_state.token_3 = Name::DefinitionTitleString;
    tokenizer.attempt(
        StateName::TitleStart,
        State::Next(StateName::DefinitionTitleAfter),
        State::Nok,
    )
}

/// After a title.
///
/// ```markdown
/// > | [a]: b "c"
///               ^
/// ```
pub fn title_after(tokenizer: &mut Tokenizer) -> State {
    tokenizer.tokenize_state.token_1 = Name::Data;
    tokenizer.tokenize_state.token_2 = Name::Data;
    tokenizer.tokenize_state.token_3 = Name::Data;
    let name = space_or_tab(tokenizer);
    tokenizer.attempt(
        name,
        State::Next(StateName::DefinitionTitleAfterOptionalWhitespace),
        State::Next(StateName::DefinitionTitleAfterOptionalWhitespace),
    )
}

/// After a title, after optional whitespace.
///
/// ```markdown
/// > | [a]: b "c"
///               ^
/// ```
pub fn title_after_optional_whitespace(tokenizer: &mut Tokenizer) -> State {
    match tokenizer.current {
        None | Some(b'\n') => State::Ok,
        _ => State::Nok,
    }
}
