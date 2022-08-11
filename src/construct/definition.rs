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
//! *   [`Definition`][Token::Definition]
//! *   [`DefinitionDestination`][Token::DefinitionDestination]
//! *   [`DefinitionDestinationLiteral`][Token::DefinitionDestinationLiteral]
//! *   [`DefinitionDestinationLiteralMarker`][Token::DefinitionDestinationLiteralMarker]
//! *   [`DefinitionDestinationRaw`][Token::DefinitionDestinationRaw]
//! *   [`DefinitionDestinationString`][Token::DefinitionDestinationString]
//! *   [`DefinitionLabel`][Token::DefinitionLabel]
//! *   [`DefinitionLabelMarker`][Token::DefinitionLabelMarker]
//! *   [`DefinitionLabelString`][Token::DefinitionLabelString]
//! *   [`DefinitionMarker`][Token::DefinitionMarker]
//! *   [`DefinitionTitle`][Token::DefinitionTitle]
//! *   [`DefinitionTitleMarker`][Token::DefinitionTitleMarker]
//! *   [`DefinitionTitleString`][Token::DefinitionTitleString]
//! *   [`LineEnding`][Token::LineEnding]
//! *   [`SpaceOrTab`][Token::SpaceOrTab]
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

use crate::construct::partial_space_or_tab::{space_or_tab, space_or_tab_eol};
use crate::token::Token;
use crate::tokenizer::{State, StateName, Tokenizer};
use crate::util::skip::opt_back as skip_opt_back;

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
            && tokenizer.events[skip_opt_back(
                &tokenizer.events,
                tokenizer.events.len() - 1,
                &[Token::LineEnding, Token::SpaceOrTab],
            )]
            .token_type
                == Token::Definition);

    if possible && tokenizer.parse_state.constructs.definition {
        tokenizer.enter(Token::Definition);
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
            tokenizer.tokenize_state.token_1 = Token::DefinitionLabel;
            tokenizer.tokenize_state.token_2 = Token::DefinitionLabelMarker;
            tokenizer.tokenize_state.token_3 = Token::DefinitionLabelString;
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
    tokenizer.tokenize_state.token_1 = Token::Data;
    tokenizer.tokenize_state.token_2 = Token::Data;
    tokenizer.tokenize_state.token_3 = Token::Data;

    match tokenizer.current {
        Some(b':') => {
            tokenizer.enter(Token::DefinitionMarker);
            tokenizer.consume();
            tokenizer.exit(Token::DefinitionMarker);
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
    tokenizer.tokenize_state.token_1 = Token::DefinitionDestination;
    tokenizer.tokenize_state.token_2 = Token::DefinitionDestinationLiteral;
    tokenizer.tokenize_state.token_3 = Token::DefinitionDestinationLiteralMarker;
    tokenizer.tokenize_state.token_4 = Token::DefinitionDestinationRaw;
    tokenizer.tokenize_state.token_5 = Token::DefinitionDestinationString;
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
    tokenizer.tokenize_state.token_1 = Token::Data;
    tokenizer.tokenize_state.token_2 = Token::Data;
    tokenizer.tokenize_state.token_3 = Token::Data;
    tokenizer.tokenize_state.token_4 = Token::Data;
    tokenizer.tokenize_state.token_5 = Token::Data;
    tokenizer.tokenize_state.size_b = 0;
    tokenizer.attempt(
        StateName::DefinitionTitleBefore,
        State::Next(StateName::DefinitionAfter),
        State::Next(StateName::DefinitionAfter),
    )
}

/// Without destination.
pub fn destination_missing(tokenizer: &mut Tokenizer) -> State {
    tokenizer.tokenize_state.token_1 = Token::Data;
    tokenizer.tokenize_state.token_2 = Token::Data;
    tokenizer.tokenize_state.token_3 = Token::Data;
    tokenizer.tokenize_state.token_4 = Token::Data;
    tokenizer.tokenize_state.token_5 = Token::Data;
    tokenizer.tokenize_state.size_b = 0;
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
            tokenizer.exit(Token::Definition);
            // You’d be interrupting.
            tokenizer.interrupt = true;
            State::Ok
        }
        _ => State::Nok,
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
    tokenizer.tokenize_state.token_1 = Token::DefinitionTitle;
    tokenizer.tokenize_state.token_2 = Token::DefinitionTitleMarker;
    tokenizer.tokenize_state.token_3 = Token::DefinitionTitleString;
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
    tokenizer.tokenize_state.token_1 = Token::Data;
    tokenizer.tokenize_state.token_2 = Token::Data;
    tokenizer.tokenize_state.token_3 = Token::Data;
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
