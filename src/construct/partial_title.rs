//! Title occurs in [definition][] and label end.
//!
//! They’re formed with the following BNF:
//!
//! ```bnf
//! ; Restriction: no blank lines.
//! ; Restriction: markers must match (in case of `(` with `)`).
//! title ::= marker [  *( code - '\\' | '\\' [ marker ] ) ] marker
//! marker ::= '"' | '\'' | '('
//! ```
//!
//! Titles can be double quoted (`"a"`), single quoted (`'a'`), or
//! parenthesized (`(a)`).
//!
//! Titles can contain line endings and whitespace, but they are not allowed to
//! contain blank lines.
//! They are allowed to be blank themselves.
//!
//! The title is interpreted as the [string][] content type.
//! That means that [character escapes][character_escape] and
//! [character references][character_reference] are allowed.
//!
//! ## References
//!
//! *   [`micromark-factory-title/index.js` in `micromark`](https://github.com/micromark/micromark/blob/main/packages/micromark-factory-title/dev/index.js)
//!
//! [definition]: crate::construct::definition
//! [string]: crate::content::string
//! [character_escape]: crate::construct::character_escape
//! [character_reference]: crate::construct::character_reference
//!
//! <!-- To do: link label end. -->

// To do: pass token types in.

use crate::construct::partial_space_or_tab::space_or_tab_opt;
use crate::tokenizer::{Code, State, StateFnResult, TokenType, Tokenizer};
use crate::util::link::link;

/// Type of title.
#[derive(Debug, PartialEq)]
enum Kind {
    /// In a parenthesized (`(` and `)`) title.
    ///
    /// ## Example
    ///
    /// ```markdown
    /// [a] b (c)
    /// ```
    Paren,
    /// In a double quoted (`"`) title.
    ///
    /// ## Example
    ///
    /// ```markdown
    /// [a] b "c"
    /// ```
    Double,
    /// In a single quoted (`'`) title.
    ///
    /// ## Example
    ///
    /// ```markdown
    /// [a] b 'c'
    /// ```
    Single,
}

impl Kind {
    /// Turn the kind into a [char].
    ///
    /// > 👉 **Note**: a closing paren is used.
    fn as_char(&self) -> char {
        match self {
            Kind::Paren => ')',
            Kind::Double => '"',
            Kind::Single => '\'',
        }
    }
    /// Turn a [char] into a kind.
    ///
    /// > 👉 **Note**: an opening paren must be used.
    ///
    /// ## Panics
    ///
    /// Panics if `char` is not `(`, `"`, or `'`.
    fn from_char(char: char) -> Kind {
        match char {
            '(' => Kind::Paren,
            '"' => Kind::Double,
            '\'' => Kind::Single,
            _ => unreachable!("invalid char"),
        }
    }
}

/// Before a title.
///
/// ```markdown
/// |"a"
/// |'a'
/// |(a)
/// ```
pub fn start(tokenizer: &mut Tokenizer, code: Code) -> StateFnResult {
    match code {
        Code::Char(char) if char == '(' || char == '"' || char == '\'' => {
            let kind = Kind::from_char(char);
            tokenizer.enter(TokenType::DefinitionTitle);
            tokenizer.enter(TokenType::DefinitionTitleMarker);
            tokenizer.consume(code);
            tokenizer.exit(TokenType::DefinitionTitleMarker);
            (State::Fn(Box::new(|t, c| begin(t, c, kind))), None)
        }
        _ => (State::Nok, None),
    }
}

/// After the opening marker.
///
/// This is also used when at the closing marker.
///
/// ```markdown
/// "|a"
/// '|a'
/// (|a)
/// ```
fn begin(tokenizer: &mut Tokenizer, code: Code, kind: Kind) -> StateFnResult {
    match code {
        Code::Char(char) if char == kind.as_char() => {
            tokenizer.enter(TokenType::DefinitionTitleMarker);
            tokenizer.consume(code);
            tokenizer.exit(TokenType::DefinitionTitleMarker);
            tokenizer.exit(TokenType::DefinitionTitle);
            (State::Ok, None)
        }
        _ => {
            tokenizer.enter(TokenType::DefinitionTitleString);
            at_break(tokenizer, code, kind, false)
        }
    }
}

/// At something, before something else.
///
/// ```markdown
/// "|a"
/// 'a|'
/// (a|
/// b)
/// ```
fn at_break(tokenizer: &mut Tokenizer, code: Code, kind: Kind, connect: bool) -> StateFnResult {
    match code {
        Code::Char(char) if char == kind.as_char() => {
            tokenizer.exit(TokenType::DefinitionTitleString);
            begin(tokenizer, code, kind)
        }
        Code::None => (State::Nok, None),
        _ => {
            tokenizer.enter(TokenType::ChunkString);
            if connect {
                let index = tokenizer.events.len() - 1;
                link(&mut tokenizer.events, index);
            }
            title(tokenizer, code, kind)
        }
    }
}

/// After a line ending.
///
/// ```markdown
/// "a
/// |b"
/// ```
fn line_start(tokenizer: &mut Tokenizer, code: Code, kind: Kind) -> StateFnResult {
    tokenizer.go(space_or_tab_opt(), |t, c| line_begin(t, c, kind))(tokenizer, code)
}

/// After a line ending, after optional whitespace.
///
/// ```markdown
/// "a
/// |b"
/// ```
fn line_begin(tokenizer: &mut Tokenizer, code: Code, kind: Kind) -> StateFnResult {
    match code {
        // Blank line not allowed.
        Code::CarriageReturnLineFeed | Code::Char('\r' | '\n') => (State::Nok, None),
        _ => at_break(tokenizer, code, kind, true),
    }
}

/// In title text.
///
/// ```markdown
/// "a|b"
/// ```
fn title(tokenizer: &mut Tokenizer, code: Code, kind: Kind) -> StateFnResult {
    match code {
        Code::Char(char) if char == kind.as_char() => {
            tokenizer.exit(TokenType::ChunkString);
            at_break(tokenizer, code, kind, true)
        }
        Code::None => {
            tokenizer.exit(TokenType::ChunkString);
            at_break(tokenizer, code, kind, true)
        }
        Code::CarriageReturnLineFeed | Code::Char('\r' | '\n') => {
            tokenizer.consume(code);
            tokenizer.exit(TokenType::ChunkString);
            (State::Fn(Box::new(|t, c| line_start(t, c, kind))), None)
        }
        Code::Char('\\') => {
            tokenizer.consume(code);
            (State::Fn(Box::new(|t, c| escape(t, c, kind))), None)
        }
        _ => {
            tokenizer.consume(code);
            (State::Fn(Box::new(|t, c| title(t, c, kind))), None)
        }
    }
}

/// After `\`, in title text.
///
/// ```markdown
/// "a\|"b"
/// ```
fn escape(tokenizer: &mut Tokenizer, code: Code, kind: Kind) -> StateFnResult {
    match code {
        Code::Char(char) if char == kind.as_char() => {
            tokenizer.consume(code);
            (State::Fn(Box::new(|t, c| title(t, c, kind))), None)
        }
        _ => title(tokenizer, code, kind),
    }
}
