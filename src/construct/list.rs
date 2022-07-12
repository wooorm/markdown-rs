//! To do.

use crate::constant::{LIST_ITEM_VALUE_SIZE_MAX, TAB_SIZE};
use crate::construct::partial_space_or_tab::space_or_tab_min_max;
use crate::token::Token;
use crate::tokenizer::{Code, State, StateFnResult, Tokenizer};

/// Type of title.
#[derive(Debug, PartialEq)]
enum Kind {
    /// In a dot (`.`) list.
    ///
    /// ## Example
    ///
    /// ```markdown
    /// 1. a
    /// ```
    Dot,
    /// In a paren (`)`) list.
    ///
    /// ## Example
    ///
    /// ```markdown
    /// 1) a
    /// ```
    Paren,
    /// In an asterisk (`*`) list.
    ///
    /// ## Example
    ///
    /// ```markdown
    /// * a
    /// ```
    Asterisk,
    /// In a plus (`+`) list.
    ///
    /// ## Example
    ///
    /// ```markdown
    /// + a
    /// ```
    Plus,
    /// In a dash (`-`) list.
    ///
    /// ## Example
    ///
    /// ```markdown
    /// - a
    /// ```
    Dash,
}

impl Kind {
    /// Turn the kind into a [char].
    fn as_char(&self) -> char {
        match self {
            Kind::Dot => '.',
            Kind::Paren => ')',
            Kind::Asterisk => '*',
            Kind::Plus => '+',
            Kind::Dash => '-',
        }
    }
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
            tokenizer.enter(Token::List);
            tokenizer.enter(Token::ListItemPrefix);
            marker(tokenizer, code)
        }
        // Ordered.
        Code::Char(char) if char.is_ascii_digit() => {
            tokenizer.enter(Token::List);
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
    match code {
        Code::Char(char) if char.is_ascii_digit() && size < LIST_ITEM_VALUE_SIZE_MAX => {
            tokenizer.consume(code);
            size += 1;
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

/// End of a block quote.
pub fn end() -> Vec<Token> {
    vec![Token::List]
}
