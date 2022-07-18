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
use crate::construct::{
    blank_line::start as blank_line, partial_space_or_tab::space_or_tab_min_max,
    thematic_break::start as thematic_break,
};
use crate::token::Token;
use crate::tokenizer::{Code, Event, EventType, State, StateFnResult, Tokenizer};
use crate::util::{
    edit_map::EditMap,
    skip,
    span::{codes as codes_from_span, from_exit_event},
};

/// Type of list.
#[derive(Debug, PartialEq)]
enum Kind {
    /// In a dot (`.`) list item.
    ///
    /// ## Example
    ///
    /// ```markdown
    /// 1. a
    /// ```
    Dot,
    /// In a paren (`)`) list item.
    ///
    /// ## Example
    ///
    /// ```markdown
    /// 1) a
    /// ```
    Paren,
    /// In an asterisk (`*`) list item.
    ///
    /// ## Example
    ///
    /// ```markdown
    /// * a
    /// ```
    Asterisk,
    /// In a plus (`+`) list item.
    ///
    /// ## Example
    ///
    /// ```markdown
    /// + a
    /// ```
    Plus,
    /// In a dash (`-`) list item.
    ///
    /// ## Example
    ///
    /// ```markdown
    /// - a
    /// ```
    Dash,
}

impl Kind {
    /// Turn a [char] into a kind.
    ///
    /// ## Panics
    ///
    /// Panics if `char` is not `.`, `)`, `*`, `+`, or `-`.
    fn from_char(char: char) -> Kind {
        match char {
            '.' => Kind::Dot,
            ')' => Kind::Paren,
            '*' => Kind::Asterisk,
            '+' => Kind::Plus,
            '-' => Kind::Dash,
            _ => unreachable!("invalid char"),
        }
    }
    /// Turn [Code] into a kind.
    ///
    /// ## Panics
    ///
    /// Panics if `code` is not `Code::Char('.' | ')' | '*' | '+' | '-')`.
    fn from_code(code: Code) -> Kind {
        match code {
            Code::Char(char) => Kind::from_char(char),
            _ => unreachable!("invalid code"),
        }
    }
}

/// Start of list item.
///
/// ```markdown
/// > | * a
///     ^
/// ```
pub fn start(tokenizer: &mut Tokenizer, code: Code) -> StateFnResult {
    tokenizer.enter(Token::ListItem);
    // To do: allow arbitrary when code (indented) is turned off.
    tokenizer.go(space_or_tab_min_max(0, TAB_SIZE - 1), before)(tokenizer, code)
}

/// Start of list item, after whitespace.
///
/// ```markdown
/// > | * a
///     ^
/// ```
fn before(tokenizer: &mut Tokenizer, code: Code) -> StateFnResult {
    match code {
        // Unordered.
        Code::Char('*' | '+' | '-') => tokenizer.check(thematic_break, |ok| {
            Box::new(if ok { nok } else { before_unordered })
        })(tokenizer, code),
        // Ordered.
        Code::Char(char) if char.is_ascii_digit() && (!tokenizer.interrupt || char == '1') => {
            tokenizer.enter(Token::ListItemPrefix);
            tokenizer.enter(Token::ListItemValue);
            inside(tokenizer, code, 0)
        }
        _ => (State::Nok, None),
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
fn before_unordered(tokenizer: &mut Tokenizer, code: Code) -> StateFnResult {
    tokenizer.enter(Token::ListItemPrefix);
    marker(tokenizer, code)
}

/// In an ordered list item value.
///
/// ```markdown
/// > | 1. a
///     ^
/// ```
fn inside(tokenizer: &mut Tokenizer, code: Code, size: usize) -> StateFnResult {
    match code {
        Code::Char(char) if char.is_ascii_digit() && size + 1 < LIST_ITEM_VALUE_SIZE_MAX => {
            tokenizer.consume(code);
            (
                State::Fn(Box::new(move |t, c| inside(t, c, size + 1))),
                None,
            )
        }
        Code::Char('.' | ')') if !tokenizer.interrupt || size < 2 => {
            tokenizer.exit(Token::ListItemValue);
            marker(tokenizer, code)
        }
        _ => (State::Nok, None),
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
fn marker(tokenizer: &mut Tokenizer, code: Code) -> StateFnResult {
    tokenizer.enter(Token::ListItemMarker);
    tokenizer.consume(code);
    tokenizer.exit(Token::ListItemMarker);
    (State::Fn(Box::new(marker_after)), None)
}

/// After a list item marker.
///
/// ```markdown
/// > | * a
///      ^
/// > | 1. b
///       ^
/// ```
fn marker_after(tokenizer: &mut Tokenizer, code: Code) -> StateFnResult {
    tokenizer.check(blank_line, move |ok| {
        if ok {
            Box::new(|t, c| after(t, c, true))
        } else {
            Box::new(marker_after_not_blank)
        }
    })(tokenizer, code)
}

/// After a list item marker, not followed by a blank line.
///
/// ```markdown
/// > | * a
///      ^
/// ```
fn marker_after_not_blank(tokenizer: &mut Tokenizer, code: Code) -> StateFnResult {
    // Attempt to parse up to the largest allowed indent, `nok` if there is more whitespace.
    tokenizer.attempt(whitespace, move |ok| {
        if ok {
            Box::new(|t, c| after(t, c, false))
        } else {
            Box::new(prefix_other)
        }
    })(tokenizer, code)
}

/// In whitespace after a marker.
///
/// ```markdown
/// > | * a
///      ^
/// ```
fn whitespace(tokenizer: &mut Tokenizer, code: Code) -> StateFnResult {
    tokenizer.go(space_or_tab_min_max(1, TAB_SIZE), whitespace_after)(tokenizer, code)
}

/// After acceptable whitespace.
///
/// ```markdown
/// > | * a
///      ^
/// ```
fn whitespace_after(_tokenizer: &mut Tokenizer, code: Code) -> StateFnResult {
    if matches!(code, Code::VirtualSpace | Code::Char('\t' | ' ')) {
        (State::Nok, None)
    } else {
        (State::Ok, Some(vec![code]))
    }
}

/// After a list item marker, followed by no indent or more indent that needed.
///
/// ```markdown
/// > | * a
///      ^
/// ```
fn prefix_other(tokenizer: &mut Tokenizer, code: Code) -> StateFnResult {
    match code {
        Code::VirtualSpace | Code::Char('\t' | ' ') => {
            tokenizer.enter(Token::SpaceOrTab);
            tokenizer.consume(code);
            tokenizer.exit(Token::SpaceOrTab);
            (State::Fn(Box::new(|t, c| after(t, c, false))), None)
        }
        _ => (State::Nok, None),
    }
}

/// After a list item prefix.
///
/// ```markdown
/// > | * a
///       ^
/// ```
fn after(tokenizer: &mut Tokenizer, code: Code, blank: bool) -> StateFnResult {
    if blank && tokenizer.interrupt {
        (State::Nok, None)
    } else {
        let start = skip::to_back(
            &tokenizer.events,
            tokenizer.events.len() - 1,
            &[Token::ListItem],
        );
        let prefix = tokenizer.index - tokenizer.events[start].index + (if blank { 1 } else { 0 });

        let container = tokenizer.container.as_mut().unwrap();
        container.blank_initial = blank;
        container.size = prefix;

        tokenizer.exit(Token::ListItemPrefix);
        tokenizer.register_resolver_before("list_item".to_string(), Box::new(resolve_list_item));
        (State::Ok, Some(vec![code]))
    }
}

/// Start of list item continuation.
///
/// ```markdown
///   | * a
/// > |   b
///     ^
/// ```
pub fn cont(tokenizer: &mut Tokenizer, code: Code) -> StateFnResult {
    tokenizer.check(blank_line, |ok| {
        Box::new(if ok { blank_cont } else { not_blank_cont })
    })(tokenizer, code)
}

/// Start of blank list item continuation.
///
/// ```markdown
///   | * a
/// > |
///     ^
///   |   b
/// ```
pub fn blank_cont(tokenizer: &mut Tokenizer, code: Code) -> StateFnResult {
    let container = tokenizer.container.as_ref().unwrap();
    let size = container.size;

    if container.blank_initial {
        (State::Nok, None)
    } else {
        // Consume, optionally, at most `size`.
        tokenizer.go(space_or_tab_min_max(0, size), ok)(tokenizer, code)
    }
}

/// Start of non-blank list item continuation.
///
/// ```markdown
///   | * a
/// > |   b
///     ^
/// ```
pub fn not_blank_cont(tokenizer: &mut Tokenizer, code: Code) -> StateFnResult {
    let container = tokenizer.container.as_mut().unwrap();
    let size = container.size;

    container.blank_initial = false;

    // Consume exactly `size`.
    tokenizer.go(space_or_tab_min_max(size, size), ok)(tokenizer, code)
}

/// A state fn to yield [`State::Ok`].
pub fn ok(_tokenizer: &mut Tokenizer, code: Code) -> StateFnResult {
    (State::Ok, Some(vec![code]))
}

/// A state fn to yield [`State::Nok`].
fn nok(_tokenizer: &mut Tokenizer, _code: Code) -> StateFnResult {
    (State::Nok, None)
}

/// Find adjacent list items with the same marker.
pub fn resolve_list_item(tokenizer: &mut Tokenizer) -> Vec<Event> {
    let mut edit_map = EditMap::new();
    let mut index = 0;
    let mut balance = 0;
    let mut lists_wip: Vec<(Kind, usize, usize, usize)> = vec![];
    let mut lists: Vec<(Kind, usize, usize, usize)> = vec![];

    // Merge list items.
    while index < tokenizer.events.len() {
        let event = &tokenizer.events[index];

        if event.token_type == Token::ListItem {
            if event.event_type == EventType::Enter {
                let end = skip::opt(&tokenizer.events, index, &[Token::ListItem]) - 1;
                let marker = skip::to(&tokenizer.events, index, &[Token::ListItemMarker]) + 1;
                let codes = codes_from_span(
                    &tokenizer.parse_state.codes,
                    &from_exit_event(&tokenizer.events, marker),
                );
                let kind = Kind::from_code(codes[0]);
                let current = (kind, balance, index, end);

                let mut list_index = lists_wip.len();
                let mut matched = false;

                while list_index > 0 {
                    list_index -= 1;
                    let previous = &lists_wip[list_index];
                    if previous.0 == current.0
                        && previous.1 == current.1
                        && skip::opt(
                            &tokenizer.events,
                            previous.3 + 1,
                            &[
                                Token::SpaceOrTab,
                                Token::LineEnding,
                                Token::BlankLineEnding,
                                Token::BlockQuotePrefix,
                            ],
                        ) == current.2
                    {
                        let previous_mut = &mut lists_wip[list_index];
                        previous_mut.3 = current.3;
                        let mut remainder = lists_wip.drain((list_index + 1)..).collect::<Vec<_>>();
                        lists.append(&mut remainder);
                        matched = true;
                        break;
                    }

                    // To do: move items that could never match anymore over to `lists`,
                    // This currently keeps on growing and growing!
                }

                if !matched {
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
            Kind::Paren | Kind::Dot => Token::ListOrdered,
            _ => Token::ListUnordered,
        };
        list_start.token_type = token_type.clone();
        list_end.token_type = token_type;

        edit_map.add(list_item.2, 0, vec![list_start]);
        edit_map.add(list_item.3 + 1, 0, vec![list_end]);

        index += 1;
    }

    edit_map.consume(&mut tokenizer.events)
}
