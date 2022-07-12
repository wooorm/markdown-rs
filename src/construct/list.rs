//! To do.

use crate::constant::{LIST_ITEM_VALUE_SIZE_MAX, TAB_SIZE};
use crate::construct::{
    blank_line::start as blank_line, partial_space_or_tab::space_or_tab_min_max,
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
    // To do: allow arbitrary when code (indented) is turned off.
    tokenizer.go(space_or_tab_min_max(0, TAB_SIZE - 1), before)(tokenizer, code)
}

/// To do.
fn before(tokenizer: &mut Tokenizer, code: Code) -> StateFnResult {
    match code {
        // Unordered.
        Code::Char('*' | '+' | '-') => {
            // To do: check if this is a thematic break?
            tokenizer.enter(Token::ListItem);
            tokenizer.enter(Token::ListItemPrefix);
            marker(tokenizer, code)
        }
        // Ordered.
        Code::Char(char) if char.is_ascii_digit() => {
            tokenizer.enter(Token::ListItem);
            tokenizer.enter(Token::ListItemPrefix);
            tokenizer.enter(Token::ListItemValue);
            // To do: `interrupt || !1`?
            inside(tokenizer, code, 0)
        }
        _ => (State::Nok, None),
    }
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
    // To do: check blank line, if true `State::Nok` else `on_blank`.
    (State::Fn(Box::new(marker_after)), None)
}

/// To do.
fn marker_after(tokenizer: &mut Tokenizer, code: Code) -> StateFnResult {
    tokenizer.attempt(list_item_prefix_whitespace, |ok| {
        let func = if ok { prefix_end } else { prefix_other };
        Box::new(func)
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
            (State::Fn(Box::new(prefix_end)), None)
        }
        _ => (State::Nok, None),
    }
}

/// To do.
fn prefix_end(tokenizer: &mut Tokenizer, code: Code) -> StateFnResult {
    // To do: calculate size.
    tokenizer.exit(Token::ListItemPrefix);
    tokenizer.register_resolver_before("list_item".to_string(), Box::new(resolve));
    (State::Ok, Some(vec![code]))
}

/// To do.
fn list_item_prefix_whitespace(tokenizer: &mut Tokenizer, code: Code) -> StateFnResult {
    // To do: check how big this should be?
    tokenizer.go(
        space_or_tab_min_max(1, TAB_SIZE - 1),
        list_item_prefix_whitespace_after,
    )(tokenizer, code)
}

fn list_item_prefix_whitespace_after(_tokenizer: &mut Tokenizer, code: Code) -> StateFnResult {
    // To do: check some stuff?
    (State::Ok, Some(vec![code]))
}

/// To do.
pub fn cont(tokenizer: &mut Tokenizer, code: Code) -> StateFnResult {
    tokenizer.check(blank_line, |ok| {
        let func = if ok { blank_cont } else { not_blank_cont };
        Box::new(func)
    })(tokenizer, code)
}

pub fn blank_cont(tokenizer: &mut Tokenizer, code: Code) -> StateFnResult {
    println!("cont: blank");
    // self.containerState.furtherBlankLines =
    //   self.containerState.furtherBlankLines ||
    //   self.containerState.initialBlankLine

    // We have a blank line.
    // Still, try to consume at most the items size.
    // To do: eat at most `size` whitespace.
    tokenizer.go(space_or_tab_min_max(0, TAB_SIZE), blank_cont_after)(tokenizer, code)
}

pub fn blank_cont_after(_tokenizer: &mut Tokenizer, code: Code) -> StateFnResult {
    println!("cont: blank: after");
    (State::Ok, Some(vec![code]))
}

pub fn not_blank_cont(tokenizer: &mut Tokenizer, code: Code) -> StateFnResult {
    println!("cont: not blank");
    // if (self.containerState.furtherBlankLines || !markdownSpace(code)) nok
    // To do: eat exactly `size` whitespace.
    tokenizer.go(space_or_tab_min_max(TAB_SIZE, TAB_SIZE), blank_cont_after)(tokenizer, code)
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
    let mut list_items: Vec<(Kind, usize, usize, usize)> = vec![];
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

                let previous = list_items.last();
                let mut matched = false;

                // There’s a previous list item.
                if let Some(previous) = previous {
                    // …with the same marker and depth, and with only (blank) line endings between them.
                    if previous.0 == current.0
                        && previous.1 == current.1
                        && skip::opt(
                            &tokenizer.events,
                            previous.3 + 1,
                            &[Token::LineEnding, Token::BlankLineEnding],
                        ) == current.2
                    {
                        matched = true;
                    }
                }

                if matched {
                    let previous = list_items.last_mut().unwrap();
                    previous.3 = current.3;
                } else {
                    // let previous = list_items.pop();
                    // if let Some(previous) = previous {
                    //     lists.push(previous);
                    // }

                    println!("prev:!match {:?} {:?}", previous, current);
                    list_items.push(current);
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

    let mut index = 0;
    while index < list_items.len() {
        let list_item = &list_items[index];
        let mut list_start = tokenizer.events[list_item.2].clone();
        let token_type = if matches!(list_item.0, Kind::Paren | Kind::Dot) {
            Token::ListOrdered
        } else {
            Token::ListUnordered
        };
        list_start.token_type = token_type.clone();
        let mut list_end = tokenizer.events[list_item.3].clone();
        list_end.token_type = token_type;
        println!("inject: {:?} {:?}", list_start, list_end);

        edit_map.add(list_item.2, 0, vec![list_start]);
        edit_map.add(list_item.3 + 1, 0, vec![list_end]);

        index += 1;
    }

    println!("list items: {:#?}", list_items);

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
