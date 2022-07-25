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

use crate::construct::{
    partial_destination::{start as destination, Options as DestinationOptions},
    partial_label::{start as label, Options as LabelOptions},
    partial_space_or_tab::{space_or_tab, space_or_tab_eol},
    partial_title::{start as title, Options as TitleOptions},
};
use crate::token::Token;
use crate::tokenizer::{Code, State, Tokenizer};
use crate::util::skip::opt_back as skip_opt_back;

/// At the start of a definition.
///
/// ```markdown
/// > | [a]: b "c"
///     ^
/// ```
pub fn start(tokenizer: &mut Tokenizer, code: Code) -> State {
    let definition_before = !tokenizer.events.is_empty()
        && tokenizer.events[skip_opt_back(
            &tokenizer.events,
            tokenizer.events.len() - 1,
            &[Token::LineEnding, Token::SpaceOrTab],
        )]
        .token_type
            == Token::Definition;

    // Do not interrupt paragraphs (but do follow definitions).
    if (!tokenizer.interrupt || definition_before) && tokenizer.parse_state.constructs.definition {
        tokenizer.enter(Token::Definition);
        // Note: arbitrary whitespace allowed even if code (indented) is on.
        tokenizer.attempt_opt(space_or_tab(), before)(tokenizer, code)
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
fn before(tokenizer: &mut Tokenizer, code: Code) -> State {
    match code {
        Code::Char('[') => tokenizer.go(
            |t, c| {
                label(
                    t,
                    c,
                    LabelOptions {
                        label: Token::DefinitionLabel,
                        marker: Token::DefinitionLabelMarker,
                        string: Token::DefinitionLabelString,
                    },
                )
            },
            label_after,
        )(tokenizer, code),
        _ => State::Nok,
    }
}

/// After the label of a definition.
///
/// ```markdown
/// > | [a]: b "c"
///        ^
/// ```
fn label_after(tokenizer: &mut Tokenizer, code: Code) -> State {
    match code {
        Code::Char(':') => {
            tokenizer.enter(Token::DefinitionMarker);
            tokenizer.consume(code);
            tokenizer.exit(Token::DefinitionMarker);
            State::Fn(Box::new(
                tokenizer.attempt_opt(space_or_tab_eol(), destination_before),
            ))
        }
        _ => State::Nok,
    }
}

/// Before a destination.
///
/// ```markdown
/// > | [a]: b "c"
///          ^
/// ```
fn destination_before(tokenizer: &mut Tokenizer, code: Code) -> State {
    tokenizer.go(
        |t, c| {
            destination(
                t,
                c,
                DestinationOptions {
                    limit: usize::MAX,
                    destination: Token::DefinitionDestination,
                    literal: Token::DefinitionDestinationLiteral,
                    marker: Token::DefinitionDestinationLiteralMarker,
                    raw: Token::DefinitionDestinationRaw,
                    string: Token::DefinitionDestinationString,
                },
            )
        },
        destination_after,
    )(tokenizer, code)
}

/// After a destination.
///
/// ```markdown
/// > | [a]: b "c"
///           ^
/// ```
fn destination_after(tokenizer: &mut Tokenizer, code: Code) -> State {
    tokenizer.attempt_opt(title_before, after)(tokenizer, code)
}

/// After a definition.
///
/// ```markdown
/// > | [a]: b
///           ^
/// > | [a]: b "c"
///               ^
/// ```
fn after(tokenizer: &mut Tokenizer, code: Code) -> State {
    tokenizer.attempt_opt(space_or_tab(), after_whitespace)(tokenizer, code)
}

/// After a definition, after optional whitespace.
///
/// ```markdown
/// > | [a]: b
///           ^
/// > | [a]: b "c"
///               ^
/// ```
fn after_whitespace(tokenizer: &mut Tokenizer, code: Code) -> State {
    match code {
        Code::None | Code::CarriageReturnLineFeed | Code::Char('\n' | '\r') => {
            tokenizer.exit(Token::Definition);
            // You’d be interrupting.
            tokenizer.interrupt = true;
            State::Ok(0)
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
fn title_before(tokenizer: &mut Tokenizer, code: Code) -> State {
    tokenizer.go(space_or_tab_eol(), title_before_marker)(tokenizer, code)
}

/// Before a title, after a line ending.
///
/// ```markdown
///   | [a]: b
/// > | "c"
///     ^
/// ```
fn title_before_marker(tokenizer: &mut Tokenizer, code: Code) -> State {
    tokenizer.go(
        |t, c| {
            title(
                t,
                c,
                TitleOptions {
                    title: Token::DefinitionTitle,
                    marker: Token::DefinitionTitleMarker,
                    string: Token::DefinitionTitleString,
                },
            )
        },
        title_after,
    )(tokenizer, code)
}

/// After a title.
///
/// ```markdown
/// > | [a]: b "c"
///               ^
/// ```
fn title_after(tokenizer: &mut Tokenizer, code: Code) -> State {
    tokenizer.attempt_opt(space_or_tab(), title_after_after_optional_whitespace)(tokenizer, code)
}

/// After a title, after optional whitespace.
///
/// ```markdown
/// > | [a]: b "c"
///               ^
/// ```
fn title_after_after_optional_whitespace(_tokenizer: &mut Tokenizer, code: Code) -> State {
    match code {
        Code::None | Code::CarriageReturnLineFeed | Code::Char('\n' | '\r') => State::Ok(0),
        _ => State::Nok,
    }
}
