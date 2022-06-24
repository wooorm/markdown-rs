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

use crate::construct::partial_space_or_tab::space_or_tab;
use crate::subtokenize::link_to;
use crate::tokenizer::{Code, State, StateFnResult, TokenType, Tokenizer};

/// Configuration.
///
/// You must pass the token types in that are used.
#[derive(Debug)]
pub struct Options {
    /// Token for the whole title.
    pub title: TokenType,
    /// Token for the marker.
    pub marker: TokenType,
    /// Token for the string inside the quotes.
    pub string: TokenType,
}

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

/// State needed to parse titles.
#[derive(Debug)]
struct Info {
    /// Whether we’ve seen our first `ChunkString`.
    connect_index: Option<usize>,
    /// Kind of title.
    kind: Kind,
    /// Configuration.
    options: Options,
}

/// Before a title.
///
/// ```markdown
/// |"a"
/// |'a'
/// |(a)
/// ```
pub fn start(tokenizer: &mut Tokenizer, code: Code, options: Options) -> StateFnResult {
    match code {
        Code::Char(char) if char == '"' || char == '\'' || char == '(' => {
            let info = Info {
                connect_index: None,
                kind: Kind::from_char(char),
                options,
            };
            tokenizer.enter(info.options.title.clone());
            tokenizer.enter(info.options.marker.clone());
            tokenizer.consume(code);
            tokenizer.exit(info.options.marker.clone());
            (State::Fn(Box::new(|t, c| begin(t, c, info))), None)
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
fn begin(tokenizer: &mut Tokenizer, code: Code, info: Info) -> StateFnResult {
    match code {
        Code::Char(char) if char == info.kind.as_char() => {
            tokenizer.enter(info.options.marker.clone());
            tokenizer.consume(code);
            tokenizer.exit(info.options.marker.clone());
            tokenizer.exit(info.options.title);
            (State::Ok, None)
        }
        _ => {
            tokenizer.enter(info.options.string.clone());
            at_break(tokenizer, code, info)
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
fn at_break(tokenizer: &mut Tokenizer, code: Code, mut info: Info) -> StateFnResult {
    match code {
        Code::Char(char) if char == info.kind.as_char() => {
            tokenizer.exit(info.options.string.clone());
            begin(tokenizer, code, info)
        }
        Code::None => (State::Nok, None),
        _ => {
            tokenizer.enter(TokenType::ChunkString);

            if let Some(connect_index) = info.connect_index {
                let index = tokenizer.events.len() - 1;
                link_to(&mut tokenizer.events, connect_index, index);
            } else {
                info.connect_index = Some(tokenizer.events.len() - 1);
            }

            title(tokenizer, code, info)
        }
    }
}

/// After a line ending.
///
/// ```markdown
/// "a
/// |b"
/// ```
fn line_start(tokenizer: &mut Tokenizer, code: Code, info: Info) -> StateFnResult {
    tokenizer.attempt_opt(space_or_tab(), |t, c| line_begin(t, c, info))(tokenizer, code)
}

/// After a line ending, after optional whitespace.
///
/// ```markdown
/// "a
/// |b"
/// ```
fn line_begin(tokenizer: &mut Tokenizer, code: Code, info: Info) -> StateFnResult {
    match code {
        // Blank line not allowed.
        Code::CarriageReturnLineFeed | Code::Char('\r' | '\n') => (State::Nok, None),
        _ => at_break(tokenizer, code, info),
    }
}

/// In title text.
///
/// ```markdown
/// "a|b"
/// ```
fn title(tokenizer: &mut Tokenizer, code: Code, info: Info) -> StateFnResult {
    match code {
        Code::Char(char) if char == info.kind.as_char() => {
            tokenizer.exit(TokenType::ChunkString);
            at_break(tokenizer, code, info)
        }
        Code::None => {
            tokenizer.exit(TokenType::ChunkString);
            at_break(tokenizer, code, info)
        }
        Code::CarriageReturnLineFeed | Code::Char('\r' | '\n') => {
            tokenizer.consume(code);
            tokenizer.exit(TokenType::ChunkString);
            (State::Fn(Box::new(|t, c| line_start(t, c, info))), None)
        }
        Code::Char('\\') => {
            tokenizer.consume(code);
            (State::Fn(Box::new(|t, c| escape(t, c, info))), None)
        }
        _ => {
            tokenizer.consume(code);
            (State::Fn(Box::new(|t, c| title(t, c, info))), None)
        }
    }
}

/// After `\`, in title text.
///
/// ```markdown
/// "a\|"b"
/// ```
fn escape(tokenizer: &mut Tokenizer, code: Code, info: Info) -> StateFnResult {
    match code {
        Code::Char(char) if char == info.kind.as_char() => {
            tokenizer.consume(code);
            (State::Fn(Box::new(|t, c| title(t, c, info))), None)
        }
        _ => title(tokenizer, code, info),
    }
}
