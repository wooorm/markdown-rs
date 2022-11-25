//! List item occurs in the [document][] content type.
//!
//! ## Grammar
//!
//! List item forms with the following BNF
//! (<small>see [construct][crate::construct] for character groups</small>):
//!
//! ```bnf
//! ; Restriction: if there is no space after the marker, the start must be followed by an `eol`.
//! ; Restriction: if the first line after the marker is not blank and starts with `5(space_or_tab)`,
//! ; only the first `space_or_tab` is part of the start.
//! list_item_start ::= '*' | '+' | '-' | 1*9(ascii_decimal) ('.' | ')') [1*4 space_or_tab]
//!
//! ; Restriction: blank line allowed, except when this is the first continuation after a blank start.
//! ; Restriction: if not blank, the line must be indented, exactly `n` times.
//! list_item_cont ::= [n(space_or_tab)]
//! ```
//!
//! Further lines that are not prefixed with `list_item_cont` cause the list
//! item to be exited, except when those lines are lazy continuation or blank.
//! Like so many things in markdown, list items too are complex.
//! See [*§ Phase 1: block structure* in `CommonMark`][commonmark_block] for
//! more on parsing details.
//!
//! As list item is a container, it takes several bytes from the start of the
//! line, while the rest of the line includes more containers or flow.
//!
//! ## HTML
//!
//! List item relates to the `<li>`, `<ol>`, and `<ul>` elements in HTML.
//! See [*§ 4.4.8 The `li` element*][html_li],
//! [*§ 4.4.5 The `ol` element*][html_ol], and
//! [*§ 4.4.7 The `ul` element*][html_ul] in the HTML spec for more info.
//!
//! ## Recommendation
//!
//! Use a single space after a marker.
//! Never use lazy continuation.
//!
//! ## Tokens
//!
//! *   [`ListItem`][Name::ListItem]
//! *   [`ListItemMarker`][Name::ListItemMarker]
//! *   [`ListItemPrefix`][Name::ListItemPrefix]
//! *   [`ListItemValue`][Name::ListItemValue]
//! *   [`ListOrdered`][Name::ListOrdered]
//! *   [`ListUnordered`][Name::ListUnordered]
//!
//! ## References
//!
//! *   [`list.js` in `micromark`](https://github.com/micromark/micromark/blob/main/packages/micromark-core-commonmark/dev/lib/list.js)
//! *   [*§ 5.2 List items* in `CommonMark`](https://spec.commonmark.org/0.30/#list-items)
//! *   [*§ 5.3 Lists* in `CommonMark`](https://spec.commonmark.org/0.30/#lists)
//!
//! [document]: crate::construct::document
//! [html_li]: https://html.spec.whatwg.org/multipage/grouping-content.html#the-li-element
//! [html_ol]: https://html.spec.whatwg.org/multipage/grouping-content.html#the-ol-element
//! [html_ul]: https://html.spec.whatwg.org/multipage/grouping-content.html#the-ul-element
//! [commonmark_block]: https://spec.commonmark.org/0.30/#phase-1-block-structure

use crate::construct::partial_space_or_tab::space_or_tab_min_max;
use crate::event::{Kind, Name};
use crate::resolve::Name as ResolveName;
use crate::state::{Name as StateName, State};
use crate::subtokenize::Subresult;
use crate::tokenizer::Tokenizer;
use crate::util::{
    constant::{LIST_ITEM_VALUE_SIZE_MAX, TAB_SIZE},
    skip,
    slice::{Position, Slice},
};
use alloc::{vec, vec::Vec};

/// Start of list item.
///
/// ```markdown
/// > | * a
///     ^
/// ```
pub fn start(tokenizer: &mut Tokenizer) -> State {
    if tokenizer.parse_state.options.constructs.list_item {
        tokenizer.enter(Name::ListItem);

        if matches!(tokenizer.current, Some(b'\t' | b' ')) {
            tokenizer.attempt(State::Next(StateName::ListItemBefore), State::Nok);
            State::Retry(space_or_tab_min_max(
                tokenizer,
                0,
                if tokenizer.parse_state.options.constructs.code_indented {
                    TAB_SIZE - 1
                } else {
                    usize::MAX
                },
            ))
        } else {
            State::Retry(StateName::ListItemBefore)
        }
    } else {
        State::Nok
    }
}

/// After optional whitespace, at list item prefix.
///
/// ```markdown
/// > | * a
///     ^
/// ```
pub fn before(tokenizer: &mut Tokenizer) -> State {
    // Unordered.
    if matches!(tokenizer.current, Some(b'*' | b'-')) {
        tokenizer.check(State::Nok, State::Next(StateName::ListItemBeforeUnordered));
        State::Retry(StateName::ThematicBreakStart)
    } else if tokenizer.current == Some(b'+') {
        State::Retry(StateName::ListItemBeforeUnordered)
    }
    // Ordered.
    else if tokenizer.current == Some(b'1')
        || (matches!(tokenizer.current, Some(b'0'..=b'9')) && !tokenizer.interrupt)
    {
        State::Retry(StateName::ListItemBeforeOrdered)
    } else {
        State::Nok
    }
}

/// At unordered list item marker.
///
/// The line is not a thematic break.
///
/// ```markdown
/// > | * a
///     ^
/// ```
pub fn before_unordered(tokenizer: &mut Tokenizer) -> State {
    tokenizer.enter(Name::ListItemPrefix);
    State::Retry(StateName::ListItemMarker)
}

/// At ordered list item value.
///
/// ```markdown
/// > | * a
///     ^
/// ```
pub fn before_ordered(tokenizer: &mut Tokenizer) -> State {
    tokenizer.enter(Name::ListItemPrefix);
    tokenizer.enter(Name::ListItemValue);
    State::Retry(StateName::ListItemValue)
}

/// In ordered list item value.
///
/// ```markdown
/// > | 1. a
///     ^
/// ```
pub fn value(tokenizer: &mut Tokenizer) -> State {
    if matches!(tokenizer.current, Some(b'.' | b')'))
        && (!tokenizer.interrupt || tokenizer.tokenize_state.size < 2)
    {
        tokenizer.exit(Name::ListItemValue);
        State::Retry(StateName::ListItemMarker)
    } else if matches!(tokenizer.current, Some(b'0'..=b'9'))
        && tokenizer.tokenize_state.size + 1 < LIST_ITEM_VALUE_SIZE_MAX
    {
        tokenizer.tokenize_state.size += 1;
        tokenizer.consume();
        State::Next(StateName::ListItemValue)
    } else {
        tokenizer.tokenize_state.size = 0;
        State::Nok
    }
}

/// At list item marker.
///
/// ```markdown
/// > | * a
///     ^
/// > | 1. b
///      ^
/// ```
pub fn marker(tokenizer: &mut Tokenizer) -> State {
    tokenizer.enter(Name::ListItemMarker);
    tokenizer.consume();
    tokenizer.exit(Name::ListItemMarker);
    State::Next(StateName::ListItemMarkerAfter)
}

/// After list item marker.
///
/// ```markdown
/// > | * a
///      ^
/// > | 1. b
///       ^
/// ```
pub fn marker_after(tokenizer: &mut Tokenizer) -> State {
    tokenizer.tokenize_state.size = 1;
    tokenizer.check(
        State::Next(StateName::ListItemAfter),
        State::Next(StateName::ListItemMarkerAfterFilled),
    );
    State::Retry(StateName::BlankLineStart)
}

/// After list item marker.
///
/// The marker is not followed by a blank line.
///
/// ```markdown
/// > | * a
///      ^
/// ```
pub fn marker_after_filled(tokenizer: &mut Tokenizer) -> State {
    tokenizer.tokenize_state.size = 0;

    // Attempt to parse up to the largest allowed indent, `nok` if there is more whitespace.
    tokenizer.attempt(
        State::Next(StateName::ListItemAfter),
        State::Next(StateName::ListItemPrefixOther),
    );
    State::Retry(StateName::ListItemWhitespace)
}

/// After marker, at whitespace.
///
/// ```markdown
/// > | * a
///      ^
/// ```
pub fn whitespace(tokenizer: &mut Tokenizer) -> State {
    tokenizer.attempt(State::Next(StateName::ListItemWhitespaceAfter), State::Nok);
    State::Retry(space_or_tab_min_max(tokenizer, 1, TAB_SIZE))
}

/// After acceptable whitespace.
///
/// ```markdown
/// > | * a
///      ^
/// ```
pub fn whitespace_after(tokenizer: &mut Tokenizer) -> State {
    if let Some(b'\t' | b' ') = tokenizer.current {
        State::Nok
    } else {
        State::Ok
    }
}

/// After marker, followed by no indent or more indent that needed.
///
/// ```markdown
/// > | * a
///      ^
/// ```
pub fn prefix_other(tokenizer: &mut Tokenizer) -> State {
    match tokenizer.current {
        Some(b'\t' | b' ') => {
            tokenizer.enter(Name::SpaceOrTab);
            tokenizer.consume();
            tokenizer.exit(Name::SpaceOrTab);
            State::Next(StateName::ListItemAfter)
        }
        _ => State::Nok,
    }
}

/// After list item prefix.
///
/// ```markdown
/// > | * a
///       ^
/// ```
pub fn after(tokenizer: &mut Tokenizer) -> State {
    let blank = tokenizer.tokenize_state.size == 1;
    tokenizer.tokenize_state.size = 0;

    if blank && tokenizer.interrupt {
        State::Nok
    } else {
        let start = skip::to_back(
            &tokenizer.events,
            tokenizer.events.len() - 1,
            &[Name::ListItem],
        );
        let mut prefix = Slice::from_position(
            tokenizer.parse_state.bytes,
            &Position {
                start: &tokenizer.events[start].point,
                end: &tokenizer.point,
            },
        )
        .len();

        if blank {
            prefix += 1;
        }

        let container = &mut tokenizer.tokenize_state.document_container_stack
            [tokenizer.tokenize_state.document_continued];

        container.blank_initial = blank;
        container.size = prefix;

        tokenizer.exit(Name::ListItemPrefix);
        tokenizer.register_resolver_before(ResolveName::ListItem);
        State::Ok
    }
}

/// Start of list item continuation.
///
/// ```markdown
///   | * a
/// > |   b
///     ^
/// ```
pub fn cont_start(tokenizer: &mut Tokenizer) -> State {
    tokenizer.check(
        State::Next(StateName::ListItemContBlank),
        State::Next(StateName::ListItemContFilled),
    );
    State::Retry(StateName::BlankLineStart)
}

/// Start of blank list item continuation.
///
/// ```markdown
///   | * a
/// > |
///     ^
///   |   b
/// ```
pub fn cont_blank(tokenizer: &mut Tokenizer) -> State {
    let container = &mut tokenizer.tokenize_state.document_container_stack
        [tokenizer.tokenize_state.document_continued];
    let size = container.size;

    if container.blank_initial {
        State::Nok
    } else if matches!(tokenizer.current, Some(b'\t' | b' ')) {
        // Consume, optionally, at most `size`.
        State::Retry(space_or_tab_min_max(tokenizer, 0, size))
    } else {
        State::Ok
    }
}

/// Start of non-blank list item continuation.
///
/// ```markdown
///   | * a
/// > |   b
///     ^
/// ```
pub fn cont_filled(tokenizer: &mut Tokenizer) -> State {
    let container = &mut tokenizer.tokenize_state.document_container_stack
        [tokenizer.tokenize_state.document_continued];
    let size = container.size;

    container.blank_initial = false;

    if matches!(tokenizer.current, Some(b'\t' | b' ')) {
        // Consume exactly `size`.
        State::Retry(space_or_tab_min_max(tokenizer, size, size))
    } else {
        State::Nok
    }
}

/// Find adjacent list items with the same marker.
pub fn resolve(tokenizer: &mut Tokenizer) -> Option<Subresult> {
    let mut lists_wip: Vec<(u8, usize, usize, usize)> = vec![];
    let mut lists: Vec<(u8, usize, usize, usize)> = vec![];
    let mut index = 0;
    let mut balance = 0;

    // Merge list items.
    while index < tokenizer.events.len() {
        let event = &tokenizer.events[index];

        if event.name == Name::ListItem {
            if event.kind == Kind::Enter {
                let end = skip::opt(&tokenizer.events, index, &[Name::ListItem]) - 1;
                let marker = skip::to(&tokenizer.events, index, &[Name::ListItemMarker]);
                // Guaranteed to be a valid ASCII byte.
                let marker = tokenizer.parse_state.bytes[tokenizer.events[marker].point.index];
                let current = (marker, balance, index, end);

                let mut list_index = lists_wip.len();
                let mut matched = false;

                while list_index > 0 {
                    list_index -= 1;
                    let previous = &lists_wip[list_index];
                    let before = skip::opt(
                        &tokenizer.events,
                        previous.3 + 1,
                        &[
                            Name::SpaceOrTab,
                            Name::LineEnding,
                            Name::BlankLineEnding,
                            Name::BlockQuotePrefix,
                        ],
                    );

                    if previous.0 == current.0 && previous.1 == current.1 && before == current.2 {
                        let previous_mut = &mut lists_wip[list_index];
                        previous_mut.3 = current.3;
                        lists.append(&mut lists_wip.split_off(list_index + 1));
                        matched = true;
                        break;
                    }
                }

                if !matched {
                    let mut index = lists_wip.len();
                    let mut exit = None;

                    while index > 0 {
                        index -= 1;

                        // If the current (new) item starts after where this
                        // item on the stack ends, we can remove it from the
                        // stack.
                        if current.2 > lists_wip[index].3 {
                            exit = Some(index);
                        } else {
                            break;
                        }
                    }

                    if let Some(exit) = exit {
                        lists.append(&mut lists_wip.split_off(exit));
                    }

                    lists_wip.push(current);
                }

                balance += 1;
            } else {
                balance -= 1;
            }
        }

        index += 1;
    }

    lists.append(&mut lists_wip);

    // Inject events.
    let mut index = 0;
    while index < lists.len() {
        let list_item = &lists[index];
        let mut list_start = tokenizer.events[list_item.2].clone();
        let mut list_end = tokenizer.events[list_item.3].clone();
        let name = match list_item.0 {
            b'.' | b')' => Name::ListOrdered,
            _ => Name::ListUnordered,
        };
        list_start.name = name.clone();
        list_end.name = name;

        tokenizer.map.add(list_item.2, 0, vec![list_start]);
        tokenizer.map.add(list_item.3 + 1, 0, vec![list_end]);

        index += 1;
    }

    tokenizer.map.consume(&mut tokenizer.events);
    None
}
