//! To do.

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
    // /// Turn the kind into a [char].
    // fn as_char(&self) -> char {
    //     match self {
    //         Kind::Dot => '.',
    //         Kind::Paren => ')',
    //         Kind::Asterisk => '*',
    //         Kind::Plus => '+',
    //         Kind::Dash => '-',
    //     }
    // }
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

/// To do.
pub fn start(tokenizer: &mut Tokenizer, code: Code) -> StateFnResult {
    tokenizer.enter(Token::ListItem);
    // To do: allow arbitrary when code (indented) is turned off.
    tokenizer.go(space_or_tab_min_max(0, TAB_SIZE - 1), before)(tokenizer, code)
}

/// To do.
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

/// To do.
fn before_unordered(tokenizer: &mut Tokenizer, code: Code) -> StateFnResult {
    tokenizer.enter(Token::ListItemPrefix);
    marker(tokenizer, code)
}

/// To do.
fn inside(tokenizer: &mut Tokenizer, code: Code, mut size: usize) -> StateFnResult {
    size += 1;
    match code {
        Code::Char(char) if char.is_ascii_digit() && size < LIST_ITEM_VALUE_SIZE_MAX => {
            tokenizer.consume(code);
            (State::Fn(Box::new(move |t, c| inside(t, c, size))), None)
        }
        // To do: `(!self.interrupt || size < 2)`
        Code::Char('.' | ')') => {
            tokenizer.exit(Token::ListItemValue);
            marker(tokenizer, code)
        }
        _ => (State::Nok, None),
    }
}

/// To do.
fn marker(tokenizer: &mut Tokenizer, code: Code) -> StateFnResult {
    let kind = Kind::from_code(code);
    println!("list item kind: {:?}", kind);
    tokenizer.enter(Token::ListItemMarker);
    tokenizer.consume(code);
    tokenizer.exit(Token::ListItemMarker);
    (State::Fn(Box::new(marker_after)), None)
}

/// To do.
fn marker_after(tokenizer: &mut Tokenizer, code: Code) -> StateFnResult {
    let interrupt = tokenizer.interrupt;

    tokenizer.check(blank_line, move |ok| {
        let func = if ok {
            if interrupt {
                nok
            } else {
                on_blank
            }
        } else {
            marker_after_after
        };
        Box::new(func)
    })(tokenizer, code)
}

/// To do.
fn on_blank(tokenizer: &mut Tokenizer, code: Code) -> StateFnResult {
    if let Some(container) = tokenizer.container.as_mut() {
        container.blank_initial = true;
    }

    // self.containerState.initialBlankLine = true
    prefix_end(tokenizer, code, true)
}

/// To do.
fn marker_after_after(tokenizer: &mut Tokenizer, code: Code) -> StateFnResult {
    let interrupt = tokenizer.interrupt;
    tokenizer.attempt(list_item_prefix_whitespace, move |ok| {
        println!("marker:after:after: {:?} {:?}", ok, interrupt);
        if ok {
            Box::new(|t, c| prefix_end(t, c, false))
        } else {
            Box::new(prefix_other)
        }
    })(tokenizer, code)
}

// To do: `on_blank`.

/// To do.
fn prefix_other(tokenizer: &mut Tokenizer, code: Code) -> StateFnResult {
    match code {
        Code::VirtualSpace | Code::Char('\t' | ' ') => {
            tokenizer.enter(Token::SpaceOrTab);
            tokenizer.consume(code);
            tokenizer.exit(Token::SpaceOrTab);
            (State::Fn(Box::new(|t, c| prefix_end(t, c, false))), None)
        }
        _ => (State::Nok, None),
    }
}

/// To do.
fn prefix_end(tokenizer: &mut Tokenizer, code: Code, blank: bool) -> StateFnResult {
    let start = skip::to_back(
        &tokenizer.events,
        tokenizer.events.len() - 1,
        &[Token::ListItem],
    );
    let prefix = tokenizer.index - tokenizer.events[start].index + (if blank { 1 } else { 0 });

    if let Some(container) = tokenizer.container.as_mut() {
        container.size = prefix;
    }

    tokenizer.exit(Token::ListItemPrefix);
    tokenizer.register_resolver_before("list_item".to_string(), Box::new(resolve));
    (State::Ok, Some(vec![code]))
}

/// To do.
fn list_item_prefix_whitespace(tokenizer: &mut Tokenizer, code: Code) -> StateFnResult {
    // To do: check how big this should be?
    tokenizer.go(
        space_or_tab_min_max(1, TAB_SIZE),
        list_item_prefix_whitespace_after,
    )(tokenizer, code)
}

fn list_item_prefix_whitespace_after(_tokenizer: &mut Tokenizer, code: Code) -> StateFnResult {
    if matches!(code, Code::VirtualSpace | Code::Char('\t' | ' ')) {
        (State::Nok, None)
    } else {
        (State::Ok, Some(vec![code]))
    }
}

/// To do.
fn nok(_tokenizer: &mut Tokenizer, _code: Code) -> StateFnResult {
    (State::Nok, None)
}

/// To do.
pub fn cont(tokenizer: &mut Tokenizer, code: Code) -> StateFnResult {
    tokenizer.check(blank_line, |ok| {
        println!("cont:check:blank:after: {:?}", ok);
        Box::new(if ok { blank_cont } else { not_blank_cont })
    })(tokenizer, code)
}

pub fn blank_cont(tokenizer: &mut Tokenizer, code: Code) -> StateFnResult {
    let mut size = 0;
    if let Some(container) = tokenizer.container.as_ref() {
        size = container.size;

        if container.blank_initial {
            return (State::Nok, None);
        }
    }

    // We have a blank line.
    // Still, try to consume at most the items size.
    tokenizer.go(space_or_tab_min_max(0, size), cont_after)(tokenizer, code)
}

pub fn not_blank_cont(tokenizer: &mut Tokenizer, code: Code) -> StateFnResult {
    let mut size = 0;

    if let Some(container) = tokenizer.container.as_mut() {
        container.blank_initial = false;
        size = container.size;
    }

    tokenizer.go(space_or_tab_min_max(size, size), cont_after)(tokenizer, code)
}

pub fn cont_after(_tokenizer: &mut Tokenizer, code: Code) -> StateFnResult {
    println!("cont: blank: after");
    (State::Ok, Some(vec![code]))
}

/// To do.
pub fn end() -> Vec<Token> {
    vec![Token::ListItem]
}

/// To do.
pub fn resolve(tokenizer: &mut Tokenizer) -> Vec<Event> {
    let mut edit_map = EditMap::new();

    let mut index = 0;
    println!("list item:before: {:?}", tokenizer.events.len());
    while index < tokenizer.events.len() {
        let event = &tokenizer.events[index];
        println!(
            "ev: {:?} {:?} {:?} {:?} {:?} {:?}",
            index,
            event.event_type,
            event.token_type,
            event.content_type,
            event.previous,
            event.next
        );
        index += 1;
    }

    let mut index = 0;
    let mut balance = 0;
    let mut lists_wip: Vec<(Kind, usize, usize, usize)> = vec![];
    let mut lists: Vec<(Kind, usize, usize, usize)> = vec![];
    // To do: track balance? Or, check what’s between them?

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
                        println!("prev:match {:?} {:?}", previous, current);
                        let previous_mut = &mut lists_wip[list_index];
                        previous_mut.3 = current.3;
                        let mut remainder = lists_wip.drain((list_index + 1)..).collect::<Vec<_>>();
                        lists.append(&mut remainder);
                        matched = true;
                        break;
                    }

                    println!(
                        "todo: move them over to `lists` at some point? {:?}",
                        previous
                    );
                }

                if !matched {
                    println!("prev:!match {:?} {:?}", lists_wip, current);
                    lists_wip.push(current);
                }

                println!("enter: {:?}", event.token_type);
                balance += 1;
            } else {
                println!("exit: {:?}", event.token_type);
                balance -= 1;
            }
        }

        index += 1;
    }

    lists.append(&mut lists_wip);

    let mut index = 0;
    while index < lists.len() {
        let list_item = &lists[index];
        let mut list_start = tokenizer.events[list_item.2].clone();
        let token_type = if matches!(list_item.0, Kind::Paren | Kind::Dot) {
            Token::ListOrdered
        } else {
            Token::ListUnordered
        };
        list_start.token_type = token_type.clone();
        let mut list_end = tokenizer.events[list_item.3].clone();
        list_end.token_type = token_type;
        println!("inject:list: {:?} {:?}", list_start, list_end);

        edit_map.add(list_item.2, 0, vec![list_start]);
        edit_map.add(list_item.3 + 1, 0, vec![list_end]);

        index += 1;
    }

    println!("list items: {:#?}", lists);

    let events = edit_map.consume(&mut tokenizer.events);

    let mut index = 0;
    println!("list item:after: {:?}", events.len());
    while index < events.len() {
        let event = &events[index];
        println!(
            "ev: {:?} {:?} {:?} {:?} {:?} {:?}",
            index,
            event.event_type,
            event.token_type,
            event.content_type,
            event.previous,
            event.next
        );
        index += 1;
    }

    events
}
