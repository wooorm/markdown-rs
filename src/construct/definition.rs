//! Definition is a construct that occurs in the [flow] content type.
//!
//! They’re formed with the following BNF:
//!
//! ```bnf
//! definition ::= label ':' whitespace destination [ whitespace title ] [ space_or_tab ]
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
//! The `label`, `destination`, and `title` parts are interpreted as the
//! [string][] content type.
//! That means that [character escapes][character_escape] and
//! [character references][character_reference] are allowed.
//!
//! Definitions match to references through their label.
//! To match, both labels must be equal after normalizing with
//! [`normalize_identifier`][normalize_identifier].
//! One definitions can match to multiple references.
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
//! For info on how to characters are encoded as `href` on `<a>` or `src` on
//! `<img>` when compiling, see
//! [`sanitize_uri`][sanitize_uri].
//!
//! ## Tokens
//!
//! *   [`Definition`][TokenType::Definition]
//! *   [`DefinitionMarker`][TokenType::DefinitionMarker]
//! *   [`DefinitionLabel`][TokenType::DefinitionLabel]
//! *   [`DefinitionLabelMarker`][TokenType::DefinitionLabelMarker]
//! *   [`DefinitionLabelString`][TokenType::DefinitionLabelString]
//! *   [`DefinitionDestination`][TokenType::DefinitionDestination]
//! *   [`DefinitionDestinationLiteral`][TokenType::DefinitionDestinationLiteral]
//! *   [`DefinitionDestinationLiteralMarker`][TokenType::DefinitionDestinationLiteralMarker]
//! *   [`DefinitionDestinationRaw`][TokenType::DefinitionDestinationRaw]
//! *   [`DefinitionDestinationString`][TokenType::DefinitionDestinationString]
//! *   [`DefinitionTitle`][TokenType::DefinitionTitle]
//! *   [`DefinitionTitleMarker`][TokenType::DefinitionTitleMarker]
//! *   [`DefinitionTitleString`][TokenType::DefinitionTitleString]
//! *   [`LineEnding`][TokenType::LineEnding]
//! *   [`SpaceOrTab`][TokenType::SpaceOrTab]
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
    partial_space_or_tab::{space_or_tab, space_or_tab_one_line_ending},
    partial_title::{start as title, Options as TitleOptions},
};
use crate::tokenizer::{Code, State, StateFnResult, TokenType, Tokenizer};

/// At the start of a definition.
///
/// ```markdown
/// |[a]: b "c"
/// ```
pub fn start(tokenizer: &mut Tokenizer, code: Code) -> StateFnResult {
    tokenizer.enter(TokenType::Definition);
    tokenizer.attempt_opt(space_or_tab(), before)(tokenizer, code)
}

/// At the start of a definition, after whitespace.
///
/// ```markdown
/// |[a]: b "c"
/// ```
fn before(tokenizer: &mut Tokenizer, code: Code) -> StateFnResult {
    match code {
        Code::Char('[') => tokenizer.go(
            |t, c| {
                label(
                    t,
                    c,
                    LabelOptions {
                        label: TokenType::DefinitionLabel,
                        marker: TokenType::DefinitionLabelMarker,
                        string: TokenType::DefinitionLabelString,
                    },
                )
            },
            label_after,
        )(tokenizer, code),
        _ => (State::Nok, None),
    }
}

/// After the label of a definition.
///
/// ```markdown
/// [a]|: b "c"
/// ```
fn label_after(tokenizer: &mut Tokenizer, code: Code) -> StateFnResult {
    match code {
        Code::Char(':') => {
            tokenizer.enter(TokenType::DefinitionMarker);
            tokenizer.consume(code);
            tokenizer.exit(TokenType::DefinitionMarker);
            (
                State::Fn(Box::new(
                    tokenizer.go(space_or_tab_one_line_ending(), destination_before),
                )),
                None,
            )
        }
        _ => (State::Nok, None),
    }
}

/// Before a destination.
///
/// ```markdown
/// [a]: |b "c"
///
/// [a]:
///  |b "c"
/// ```
fn destination_before(tokenizer: &mut Tokenizer, code: Code) -> StateFnResult {
    tokenizer.go(
        |t, c| {
            destination(
                t,
                c,
                DestinationOptions {
                    limit: usize::MAX,
                    destination: TokenType::DefinitionDestination,
                    literal: TokenType::DefinitionDestinationLiteral,
                    marker: TokenType::DefinitionDestinationLiteralMarker,
                    raw: TokenType::DefinitionDestinationRaw,
                    string: TokenType::DefinitionDestinationString,
                },
            )
        },
        destination_after,
    )(tokenizer, code)
}

/// After a destination.
///
/// ```markdown
/// [a]: b| "c"
///
/// [a]: b| ␊
///  "c"
/// ```
fn destination_after(tokenizer: &mut Tokenizer, code: Code) -> StateFnResult {
    tokenizer.attempt_opt(title_before, after)(tokenizer, code)
}

/// After a definition.
///
/// ```markdown
/// [a]: b|
/// [a]: b "c"|
/// ```
fn after(tokenizer: &mut Tokenizer, code: Code) -> StateFnResult {
    tokenizer.attempt_opt(space_or_tab(), after_whitespace)(tokenizer, code)
}

/// After a definition, after optional whitespace.
///
/// ```markdown
/// [a]: b |
/// [a]: b "c"|
/// ```
fn after_whitespace(tokenizer: &mut Tokenizer, code: Code) -> StateFnResult {
    match code {
        Code::None | Code::CarriageReturnLineFeed | Code::Char('\r' | '\n') => {
            tokenizer.exit(TokenType::Definition);
            (State::Ok, Some(vec![code]))
        }
        _ => (State::Nok, None),
    }
}

/// After a destination, presumably before a title.
///
/// ```markdown
/// [a]: b| "c"
///
/// [a]: b| ␊
///  "c"
/// ```
fn title_before(tokenizer: &mut Tokenizer, code: Code) -> StateFnResult {
    tokenizer.go(space_or_tab_one_line_ending(), title_before_marker)(tokenizer, code)
}

/// Before a title, after a line ending.
///
/// ```markdown
/// [a]: b␊
/// | "c"
/// ```
fn title_before_marker(tokenizer: &mut Tokenizer, code: Code) -> StateFnResult {
    tokenizer.go(
        |t, c| {
            title(
                t,
                c,
                TitleOptions {
                    title: TokenType::DefinitionTitle,
                    marker: TokenType::DefinitionTitleMarker,
                    string: TokenType::DefinitionTitleString,
                },
            )
        },
        title_after,
    )(tokenizer, code)
}

/// After a title.
///
/// ```markdown
/// [a]: b "c"|
///
/// [a]: b␊
/// "c"|
/// ```
fn title_after(tokenizer: &mut Tokenizer, code: Code) -> StateFnResult {
    tokenizer.attempt_opt(space_or_tab(), title_after_after_optional_whitespace)(tokenizer, code)
}

/// After a title, after optional whitespace.
///
/// ```markdown
/// [a]: b "c"|
///
/// [a]: b "c" |
/// ```
fn title_after_after_optional_whitespace(_tokenizer: &mut Tokenizer, code: Code) -> StateFnResult {
    match code {
        Code::None | Code::CarriageReturnLineFeed | Code::Char('\r' | '\n') => {
            (State::Ok, Some(vec![code]))
        }
        _ => (State::Nok, None),
    }
}
