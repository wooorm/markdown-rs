//! List is a construct that occurs in the [document][] content type.
//!
//! It forms with, roughly, the following BNF:
//!
//! ```bnf
//! ; Restriction: there must be `eol | space_or_tab` after the start.
//! ; Restriction: if the first line after the marker is not blank and starts with `5( space_or_tab )`,
//! ; only the first `space_or_tab` is part of the start.
//! list_item_start ::= '*' | '+' | '-' | 1*9( ascii_decimal ) ( '.' | ')' ) [ 1*4 space_or_tab ]
//! ; Restriction: blank line allowed, except when this is the first continuation after a blank start.
//! ; Restriction: if not blank, the line must be indented, exactly `n` times.
//! list_item_cont ::= [ n( space_or_tab ) ]
//! ```
//!
//! Further lines that are not prefixed with `list_item_cont` cause the item
//! to be exited, except when those lines are lazy continuation.
//! Like so many things in markdown, list (items) too, are very complex.
//! See [*§ Phase 1: block structure*][commonmark-block] for more on parsing
//! details.
//!
//! Lists relates to the `<li>`, `<ol>`, and `<ul>` elements in HTML.
//! See [*§ 4.4.8 The `li` element*][html-li],
//! [*§ 4.4.5 The `ol` element*][html-ol], and
//! [*§ 4.4.7 The `ul` element*][html-ul] in the HTML spec for more info.
//!
//! ## Tokens
//!
//! *   [`ListItem`][Token::ListItem]
//! *   [`ListItemMarker`][Token::ListItemMarker]
//! *   [`ListItemPrefix`][Token::ListItemPrefix]
//! *   [`ListItemValue`][Token::ListItemValue]
//! *   [`ListOrdered`][Token::ListOrdered]
//! *   [`ListUnordered`][Token::ListUnordered]
//!
//! ## References
//!
//! *   [`list.js` in `micromark`](https://github.com/micromark/micromark/blob/main/packages/micromark-core-commonmark/dev/lib/list.js)
//! *   [*§ 5.2 List items* in `CommonMark`](https://spec.commonmark.org/0.30/#list-items)
//! *   [*§ 5.3 Lists* in `CommonMark`](https://spec.commonmark.org/0.30/#lists)
//!
//! [document]: crate::content::document
//! [html-li]: https://html.spec.whatwg.org/multipage/grouping-content.html#the-li-element
//! [html-ol]: https://html.spec.whatwg.org/multipage/grouping-content.html#the-ol-element
//! [html-ul]: https://html.spec.whatwg.org/multipage/grouping-content.html#the-ul-element
//! [commonmark-block]: https://spec.commonmark.org/0.30/#phase-1-block-structure

use crate::constant::{LIST_ITEM_VALUE_SIZE_MAX, TAB_SIZE};
use crate::construct::partial_space_or_tab::space_or_tab_min_max;
use crate::token::Token;
use crate::tokenizer::{EventType, State, StateName, Tokenizer};
use crate::util::{
    skip,
    slice::{Position, Slice},
};

/// Start of list item.
///
/// ```markdown
/// > | * a
///     ^
/// ```
pub fn start(tokenizer: &mut Tokenizer) -> State {
    if tokenizer.parse_state.constructs.list {
        tokenizer.enter(Token::ListItem);
        let name = space_or_tab_min_max(
            tokenizer,
            0,
            if tokenizer.parse_state.constructs.code_indented {
                TAB_SIZE - 1
            } else {
                usize::MAX
            },
        );
        tokenizer.attempt(name, State::Next(StateName::ListBefore), State::Nok)
    } else {
        State::Nok
    }
}

/// Start of list item, after whitespace.
///
/// ```markdown
/// > | * a
///     ^
/// ```
pub fn before(tokenizer: &mut Tokenizer) -> State {
    match tokenizer.current {
        // Unordered.
        Some(b'*' | b'-') => tokenizer.check(
            StateName::ThematicBreakStart,
            State::Next(StateName::ListNok),
            State::Next(StateName::ListBeforeUnordered),
        ),
        Some(b'+') => before_unordered(tokenizer),
        // Ordered.
        Some(b'0'..=b'9') if !tokenizer.interrupt => before_ordered(tokenizer),
        Some(b'1') => before_ordered(tokenizer),
        _ => State::Nok,
    }
}

/// Start of an unordered list item.
///
/// The line is not a thematic break.
///
/// ```markdown
/// > | * a
///     ^
/// ```
pub fn before_unordered(tokenizer: &mut Tokenizer) -> State {
    tokenizer.enter(Token::ListItemPrefix);
    marker(tokenizer)
}

/// Start of an ordered list item.
///
/// ```markdown
/// > | * a
///     ^
/// ```
pub fn before_ordered(tokenizer: &mut Tokenizer) -> State {
    tokenizer.enter(Token::ListItemPrefix);
    tokenizer.enter(Token::ListItemValue);
    value(tokenizer)
}

/// In an ordered list item value.
///
/// ```markdown
/// > | 1. a
///     ^
/// ```
pub fn value(tokenizer: &mut Tokenizer) -> State {
    match tokenizer.current {
        Some(b'.' | b')') if !tokenizer.interrupt || tokenizer.tokenize_state.size < 2 => {
            tokenizer.exit(Token::ListItemValue);
            marker(tokenizer)
        }
        Some(b'0'..=b'9') if tokenizer.tokenize_state.size + 1 < LIST_ITEM_VALUE_SIZE_MAX => {
            tokenizer.tokenize_state.size += 1;
            tokenizer.consume();
            State::Next(StateName::ListValue)
        }
        _ => {
            tokenizer.tokenize_state.size = 0;
            State::Nok
        }
    }
}

/// At a list item marker.
///
/// ```markdown
/// > | * a
///     ^
/// > | 1. b
///      ^
/// ```
pub fn marker(tokenizer: &mut Tokenizer) -> State {
    tokenizer.enter(Token::ListItemMarker);
    tokenizer.consume();
    tokenizer.exit(Token::ListItemMarker);
    State::Next(StateName::ListMarkerAfter)
}

/// After a list item marker.
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
        StateName::BlankLineStart,
        State::Next(StateName::ListAfter),
        State::Next(StateName::ListMarkerAfterFilled),
    )
}

/// After a list item marker, not followed by a blank line.
///
/// ```markdown
/// > | * a
///      ^
/// ```
pub fn marker_after_filled(tokenizer: &mut Tokenizer) -> State {
    tokenizer.tokenize_state.size = 0;

    // Attempt to parse up to the largest allowed indent, `nok` if there is more whitespace.
    tokenizer.attempt(
        StateName::ListWhitespace,
        State::Next(StateName::ListAfter),
        State::Next(StateName::ListPrefixOther),
    )
}

/// In whitespace after a marker.
///
/// ```markdown
/// > | * a
///      ^
/// ```
pub fn whitespace(tokenizer: &mut Tokenizer) -> State {
    let name = space_or_tab_min_max(tokenizer, 1, TAB_SIZE);
    tokenizer.attempt(
        name,
        State::Next(StateName::ListWhitespaceAfter),
        State::Nok,
    )
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

/// After a list item marker, followed by no indent or more indent that needed.
///
/// ```markdown
/// > | * a
///      ^
/// ```
pub fn prefix_other(tokenizer: &mut Tokenizer) -> State {
    match tokenizer.current {
        Some(b'\t' | b' ') => {
            tokenizer.enter(Token::SpaceOrTab);
            tokenizer.consume();
            tokenizer.exit(Token::SpaceOrTab);
            State::Next(StateName::ListAfter)
        }
        _ => State::Nok,
    }
}

/// After a list item prefix.
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
            &[Token::ListItem],
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

        let container = tokenizer.container.as_mut().unwrap();
        container.blank_initial = blank;
        container.size = prefix;

        tokenizer.exit(Token::ListItemPrefix);
        tokenizer.register_resolver_before("list_item".to_string(), Box::new(resolve_list_item));
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
        StateName::BlankLineStart,
        State::Next(StateName::ListContBlank),
        State::Next(StateName::ListContFilled),
    )
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
    let container = tokenizer.container.as_ref().unwrap();
    let size = container.size;

    if container.blank_initial {
        State::Nok
    } else {
        let name = space_or_tab_min_max(tokenizer, 0, size);
        // Consume, optionally, at most `size`.
        tokenizer.attempt(name, State::Next(StateName::ListOk), State::Nok)
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
    let container = tokenizer.container.as_mut().unwrap();
    let size = container.size;

    container.blank_initial = false;

    // Consume exactly `size`.
    let name = space_or_tab_min_max(tokenizer, size, size);
    tokenizer.attempt(name, State::Next(StateName::ListOk), State::Nok)
}

/// A state fn to yield [`State::Ok`].
pub fn ok(_tokenizer: &mut Tokenizer) -> State {
    State::Ok
}

/// A state fn to yield [`State::Nok`].
pub fn nok(_tokenizer: &mut Tokenizer) -> State {
    State::Nok
}

/// Find adjacent list items with the same marker.
pub fn resolve_list_item(tokenizer: &mut Tokenizer) {
    let mut lists_wip: Vec<(u8, usize, usize, usize)> = vec![];
    let mut lists: Vec<(u8, usize, usize, usize)> = vec![];
    let mut index = 0;
    let mut balance = 0;

    // Merge list items.
    while index < tokenizer.events.len() {
        let event = &tokenizer.events[index];

        if event.token_type == Token::ListItem {
            if event.event_type == EventType::Enter {
                let end = skip::opt(&tokenizer.events, index, &[Token::ListItem]) - 1;
                let marker = skip::to(&tokenizer.events, index, &[Token::ListItemMarker]);
                // Guaranteed to be a valid ASCII byte.
                let marker = Slice::from_index(
                    tokenizer.parse_state.bytes,
                    tokenizer.events[marker].point.index,
                )
                .head()
                .unwrap();
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
                            Token::SpaceOrTab,
                            Token::LineEnding,
                            Token::BlankLineEnding,
                            Token::BlockQuotePrefix,
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
        let token_type = match list_item.0 {
            b'.' | b')' => Token::ListOrdered,
            _ => Token::ListUnordered,
        };
        list_start.token_type = token_type.clone();
        list_end.token_type = token_type;

        tokenizer.map.add(list_item.2, 0, vec![list_start]);
        tokenizer.map.add(list_item.3 + 1, 0, vec![list_end]);

        index += 1;
    }
}
