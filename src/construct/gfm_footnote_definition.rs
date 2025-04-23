//! GFM: Footnote definition occurs in the [document][] content type.
//!
//! ## Grammar
//!
//! Footnote definitions form with the following BNF
//! (<small>see [construct][crate::construct] for character groups</small>):
//!
//! ```bnf
//! ; Restriction: `label` must start with `^` (and not be empty after it).
//! ; See the `label` construct for the BNF of that part.
//! gfm_footnote_definition_start ::= label ':' *space_or_tab
//!
//! ; Restriction: blank line allowed.
//! gfm_footnote_definition_cont ::= 4(space_or_tab)
//! ```
//!
//! Further lines that are not prefixed with `gfm_footnote_definition_cont`
//! cause the footnote definition to be exited, except when those lines are
//! lazy continuation or blank.
//! Like so many things in markdown, footnote definition too are complex.
//! See [*§ Phase 1: block structure* in `CommonMark`][commonmark_block] for
//! more on parsing details.
//!
//! See [`label`][label] for grammar, notes, and recommendations on that part.
//!
//! The `label` part is interpreted as the [string][] content type.
//! That means that [character escapes][character_escape] and
//! [character references][character_reference] are allowed.
//!
//! Definitions match to calls through identifiers.
//! To match, both labels must be equal after normalizing with
//! [`normalize_identifier`][].
//! One definition can match to multiple calls.
//! Multiple definitions with the same, normalized, identifier are ignored: the
//! first definition is preferred.
//! To illustrate, the definition with the content of `x` wins:
//!
//! ```markdown
//! [^a]: x
//! [^a]: y
//!
//! [^a]
//! ```
//!
//! Importantly, while labels *can* include [string][] content (character
//! escapes and character references), these are not considered when matching.
//! To illustrate, neither definition matches the call:
//!
//! ```markdown
//! [^a&amp;b]: x
//! [^a\&b]: y
//!
//! [^a&b]
//! ```
//!
//! Because footnote definitions are containers (like block quotes and list
//! items), they can contain more footnote definitions, and they can include
//! calls to themselves.
//!
//! ## HTML
//!
//! GFM footnote definitions do not, on their own, relate to anything in HTML.
//! When matched with a [label end][label_end], which in turns matches to a
//! [GFM label start (footnote)][gfm_label_start_footnote], the definition
//! relates to several elements in HTML.
//!
//! When one or more definitions are called, a footnote section is generated
//! at the end of the document, using `<section>`, `<h2>`, and `<ol>` elements:
//!
//! ```html
//! <section data-footnotes="" class="footnotes"><h2 id="footnote-label" class="sr-only">Footnotes</h2>
//! <ol>…</ol>
//! </section>
//! ```
//!
//! Each definition is generated as a `<li>` in the `<ol>`, in the order they
//! were first called:
//!
//! ```html
//! <li id="user-content-fn-1">…</li>
//! ```
//!
//! Backreferences are injected at the end of the first paragraph, or, when
//! there is no paragraph, at the end of the definition.
//! When a definition is called multiple times, multiple backreferences are
//! generated.
//! Further backreferences use an extra counter in the `href` attribute and
//! visually in a `<span>` after `↩`.
//!
//! ```html
//! <a href="#user-content-fnref-1" data-footnote-backref="" class="data-footnote-backref" aria-label="Back to content">↩</a> <a href="#user-content-fnref-1-2" data-footnote-backref="" class="data-footnote-backref" aria-label="Back to content">↩<sup>2</sup></a>
//! ```
//!
//! See
//! [*§ 4.5.1 The `a` element*][html_a],
//! [*§ 4.3.6 The `h1`, `h2`, `h3`, `h4`, `h5`, and `h6` elements*][html_h],
//! [*§ 4.4.8 The `li` element*][html_li],
//! [*§ 4.4.5 The `ol` element*][html_ol],
//! [*§ 4.4.1 The `p` element*][html_p],
//! [*§ 4.3.3 The `section` element*][html_section], and
//! [*§ 4.5.19 The `sub` and `sup` elements*][html_sup]
//! in the HTML spec for more info.
//!
//! ## Recommendation
//!
//! When authoring markdown with footnotes, it’s recommended to use words
//! instead of numbers (or letters or anything with an order) as calls.
//! That makes it easier to reuse and reorder footnotes.
//!
//! It’s recommended to place footnotes definitions at the bottom of the document.
//!
//! ## Bugs
//!
//! GitHub’s own algorithm to parse footnote definitions contains several bugs.
//! These are not present in this project.
//! The issues relating to footnote definitions are:
//!
//! * [Footnote reference call identifiers are trimmed, but definition identifiers aren’t](https://github.com/github/cmark-gfm/issues/237)\
//!   — initial and final whitespace in labels causes them not to match
//! * [Footnotes are matched case-insensitive, but links keep their casing, breaking them](https://github.com/github/cmark-gfm/issues/239)\
//!   — using uppercase (or any character that will be percent encoded) in identifiers breaks links
//! * [Colons in footnotes generate links w/o `href`](https://github.com/github/cmark-gfm/issues/250)\
//!   — colons in identifiers generate broken links
//! * [Character escape of `]` does not work in footnote identifiers](https://github.com/github/cmark-gfm/issues/240)\
//!   — some character escapes don’t work
//! * [Footnotes in links are broken](https://github.com/github/cmark-gfm/issues/249)\
//!   — while `CommonMark` prevents links in links, GitHub does not prevent footnotes (which turn into links) in links
//! * [Footnote-like brackets around image, break that image](https://github.com/github/cmark-gfm/issues/275)\
//!   — images can’t be used in what looks like a footnote call
//! * [GFM footnotes: line ending in footnote definition label causes text to disappear](https://github.com/github/cmark-gfm/issues/282)\
//!   — line endings in footnote definitions cause text to disappear
//!
//! ## Tokens
//!
//! * [`DefinitionMarker`][Name::DefinitionMarker]
//! * [`GfmFootnoteDefinition`][Name::GfmFootnoteDefinition]
//! * [`GfmFootnoteDefinitionLabel`][Name::GfmFootnoteDefinitionLabel]
//! * [`GfmFootnoteDefinitionLabelMarker`][Name::GfmFootnoteDefinitionLabelMarker]
//! * [`GfmFootnoteDefinitionLabelString`][Name::GfmFootnoteDefinitionLabelString]
//! * [`GfmFootnoteDefinitionMarker`][Name::GfmFootnoteDefinitionMarker]
//! * [`GfmFootnoteDefinitionPrefix`][Name::GfmFootnoteDefinitionPrefix]
//! * [`SpaceOrTab`][Name::SpaceOrTab]
//!
//! ## References
//!
//! * [`micromark-extension-gfm-footnote`](https://github.com/micromark/micromark-extension-gfm-footnote)
//!
//! > 👉 **Note**: Footnotes are not specified in GFM yet.
//! > See [`github/cmark-gfm#270`](https://github.com/github/cmark-gfm/issues/270)
//! > for the related issue.
//!
//! [document]: crate::construct::document
//! [string]: crate::construct::string
//! [character_reference]: crate::construct::character_reference
//! [character_escape]: crate::construct::character_escape
//! [label]: crate::construct::partial_label
//! [label_end]: crate::construct::label_end
//! [gfm_label_start_footnote]: crate::construct::gfm_label_start_footnote
//! [commonmark_block]: https://spec.commonmark.org/0.31/#phase-1-block-structure
//! [html_a]: https://html.spec.whatwg.org/multipage/text-level-semantics.html#the-a-element
//! [html_h]: https://html.spec.whatwg.org/multipage/sections.html#the-h1,-h2,-h3,-h4,-h5,-and-h6-elements
//! [html_li]: https://html.spec.whatwg.org/multipage/grouping-content.html#the-li-element
//! [html_ol]: https://html.spec.whatwg.org/multipage/grouping-content.html#the-ol-element
//! [html_p]: https://html.spec.whatwg.org/multipage/grouping-content.html#the-p-element
//! [html_section]: https://html.spec.whatwg.org/multipage/sections.html#the-section-element
//! [html_sup]: https://html.spec.whatwg.org/multipage/text-level-semantics.html#the-sub-and-sup-elements

use crate::construct::partial_space_or_tab::space_or_tab_min_max;
use crate::event::{Content, Link, Name};
use crate::state::{Name as StateName, State};
use crate::tokenizer::Tokenizer;
use crate::util::{
    constant::{LINK_REFERENCE_SIZE_MAX, TAB_SIZE},
    normalize_identifier::normalize_identifier,
    skip,
    slice::{Position, Slice},
};

/// Start of GFM footnote definition.
///
/// ```markdown
/// > | [^a]: b
///     ^
/// ```
pub fn start(tokenizer: &mut Tokenizer) -> State {
    if tokenizer
        .parse_state
        .options
        .constructs
        .gfm_footnote_definition
    {
        tokenizer.enter(Name::GfmFootnoteDefinition);

        if matches!(tokenizer.current, Some(b'\t' | b' ')) {
            tokenizer.attempt(
                State::Next(StateName::GfmFootnoteDefinitionLabelBefore),
                State::Nok,
            );
            State::Retry(space_or_tab_min_max(
                tokenizer,
                1,
                if tokenizer.parse_state.options.constructs.code_indented {
                    TAB_SIZE - 1
                } else {
                    usize::MAX
                },
            ))
        } else {
            State::Retry(StateName::GfmFootnoteDefinitionLabelBefore)
        }
    } else {
        State::Nok
    }
}

/// Before definition label (after optional whitespace).
///
/// ```markdown
/// > | [^a]: b
///     ^
/// ```
pub fn label_before(tokenizer: &mut Tokenizer) -> State {
    match tokenizer.current {
        Some(b'[') => {
            tokenizer.enter(Name::GfmFootnoteDefinitionPrefix);
            tokenizer.enter(Name::GfmFootnoteDefinitionLabel);
            tokenizer.enter(Name::GfmFootnoteDefinitionLabelMarker);
            tokenizer.consume();
            tokenizer.exit(Name::GfmFootnoteDefinitionLabelMarker);
            State::Next(StateName::GfmFootnoteDefinitionLabelAtMarker)
        }
        _ => State::Nok,
    }
}

/// In label, at caret.
///
/// ```markdown
/// > | [^a]: b
///      ^
/// ```
pub fn label_at_marker(tokenizer: &mut Tokenizer) -> State {
    if tokenizer.current == Some(b'^') {
        tokenizer.enter(Name::GfmFootnoteDefinitionMarker);
        tokenizer.consume();
        tokenizer.exit(Name::GfmFootnoteDefinitionMarker);
        tokenizer.enter(Name::GfmFootnoteDefinitionLabelString);
        tokenizer.enter_link(
            Name::Data,
            Link {
                previous: None,
                next: None,
                content: Content::String,
            },
        );
        State::Next(StateName::GfmFootnoteDefinitionLabelInside)
    } else {
        State::Nok
    }
}

/// In label.
///
/// > 👉 **Note**: `cmark-gfm` prevents whitespace from occurring in footnote
/// > definition labels.
///
/// ```markdown
/// > | [^a]: b
///       ^
/// ```
pub fn label_inside(tokenizer: &mut Tokenizer) -> State {
    // Too long.
    if tokenizer.tokenize_state.size > LINK_REFERENCE_SIZE_MAX
        // Space or tab is not supported by GFM for some reason (`\n` and
        // `[` make sense).
        || matches!(tokenizer.current, None | Some(b'\t' | b'\n' | b' ' | b'['))
        // Closing brace with nothing.
        || (matches!(tokenizer.current, Some(b']')) && tokenizer.tokenize_state.size == 0)
    {
        tokenizer.tokenize_state.size = 0;
        State::Nok
    } else if matches!(tokenizer.current, Some(b']')) {
        tokenizer.tokenize_state.size = 0;
        tokenizer.exit(Name::Data);
        tokenizer.exit(Name::GfmFootnoteDefinitionLabelString);
        tokenizer.enter(Name::GfmFootnoteDefinitionLabelMarker);
        tokenizer.consume();
        tokenizer.exit(Name::GfmFootnoteDefinitionLabelMarker);
        tokenizer.exit(Name::GfmFootnoteDefinitionLabel);
        State::Next(StateName::GfmFootnoteDefinitionLabelAfter)
    } else {
        let next = if matches!(tokenizer.current.unwrap(), b'\\') {
            StateName::GfmFootnoteDefinitionLabelEscape
        } else {
            StateName::GfmFootnoteDefinitionLabelInside
        };
        tokenizer.consume();
        tokenizer.tokenize_state.size += 1;
        State::Next(next)
    }
}

/// After `\`, at a special character.
///
/// > 👉 **Note**: `cmark-gfm` currently does not support escaped brackets:
/// > <https://github.com/github/cmark-gfm/issues/240>
///
/// ```markdown
/// > | [^a\*b]: c
///         ^
/// ```
pub fn label_escape(tokenizer: &mut Tokenizer) -> State {
    match tokenizer.current {
        Some(b'[' | b'\\' | b']') => {
            tokenizer.tokenize_state.size += 1;
            tokenizer.consume();
            State::Next(StateName::GfmFootnoteDefinitionLabelInside)
        }
        _ => State::Retry(StateName::GfmFootnoteDefinitionLabelInside),
    }
}

/// After definition label.
///
/// ```markdown
/// > | [^a]: b
///         ^
/// ```
pub fn label_after(tokenizer: &mut Tokenizer) -> State {
    match tokenizer.current {
        Some(b':') => {
            let end = skip::to_back(
                &tokenizer.events,
                tokenizer.events.len() - 1,
                &[Name::GfmFootnoteDefinitionLabelString],
            );

            // Note: we don’t care about virtual spaces, so `as_str` is fine.
            let id = normalize_identifier(
                Slice::from_position(
                    tokenizer.parse_state.bytes,
                    &Position::from_exit_event(&tokenizer.events, end),
                )
                .as_str(),
            );

            // Note: we don’t care about uniqueness.
            // It’s likely that that doesn’t happen very frequently.
            // It is more likely that it wastes precious time.
            tokenizer.tokenize_state.gfm_footnote_definitions.push(id);

            tokenizer.enter(Name::DefinitionMarker);
            tokenizer.consume();
            tokenizer.exit(Name::DefinitionMarker);
            tokenizer.attempt(
                State::Next(StateName::GfmFootnoteDefinitionWhitespaceAfter),
                State::Nok,
            );
            // Any whitespace after the marker is eaten, forming indented code
            // is not possible.
            // No space is also fine, just like a block quote marker.
            State::Next(space_or_tab_min_max(tokenizer, 0, usize::MAX))
        }
        _ => State::Nok,
    }
}

/// After definition prefix.
///
/// ```markdown
/// > | [^a]: b
///           ^
/// ```
pub fn whitespace_after(tokenizer: &mut Tokenizer) -> State {
    tokenizer.exit(Name::GfmFootnoteDefinitionPrefix);
    State::Ok
}

/// Start of footnote definition continuation.
///
/// ```markdown
///   | [^a]: b
/// > |     c
///     ^
/// ```
pub fn cont_start(tokenizer: &mut Tokenizer) -> State {
    tokenizer.check(
        State::Next(StateName::GfmFootnoteDefinitionContBlank),
        State::Next(StateName::GfmFootnoteDefinitionContFilled),
    );
    State::Retry(StateName::BlankLineStart)
}

/// Start of footnote definition continuation, at a blank line.
///
/// ```markdown
///   | [^a]: b
/// > | ␠␠␊
///     ^
/// ```
pub fn cont_blank(tokenizer: &mut Tokenizer) -> State {
    if matches!(tokenizer.current, Some(b'\t' | b' ')) {
        State::Retry(space_or_tab_min_max(tokenizer, 0, TAB_SIZE))
    } else {
        State::Ok
    }
}

/// Start of footnote definition continuation, at a filled line.
///
/// ```markdown
///   | [^a]: b
/// > |     c
///     ^
/// ```
pub fn cont_filled(tokenizer: &mut Tokenizer) -> State {
    if matches!(tokenizer.current, Some(b'\t' | b' ')) {
        // Consume exactly `TAB_SIZE`.
        State::Retry(space_or_tab_min_max(tokenizer, TAB_SIZE, TAB_SIZE))
    } else {
        State::Nok
    }
}
