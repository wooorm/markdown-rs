//! Title occurs in [definition][] and [label end][label_end].
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
//! [label_end]: crate::construct::label_end

use super::partial_space_or_tab::{space_or_tab_eol_with_options, EolOptions};
use crate::subtokenize::link;
use crate::token::Token;
use crate::tokenizer::{Code, ContentType, State, Tokenizer};

/// Configuration.
///
/// You must pass the token types in that are used.
#[derive(Debug)]
pub struct Options {
    /// Token for the whole title.
    pub title: Token,
    /// Token for the marker.
    pub marker: Token,
    /// Token for the string inside the quotes.
    pub string: Token,
}

/// Type of title.
#[derive(Debug, PartialEq)]
enum Kind {
    /// In a parenthesized (`(` and `)`) title.
    ///
    /// ## Example
    ///
    /// ```markdown
    /// (a)
    /// ```
    Paren,
    /// In a double quoted (`"`) title.
    ///
    /// ## Example
    ///
    /// ```markdown
    /// "a"
    /// ```
    Double,
    /// In a single quoted (`'`) title.
    ///
    /// ## Example
    ///
    /// ```markdown
    /// 'a'
    /// ```
    Single,
}

impl Kind {
    /// Turn the kind into a [char].
    ///
    /// > 👉 **Note**: a closing paren is used for `Kind::Paren`.
    fn as_char(&self) -> char {
        match self {
            Kind::Paren => ')',
            Kind::Double => '"',
            Kind::Single => '\'',
        }
    }
    /// Turn a [char] into a kind.
    ///
    /// > 👉 **Note**: an opening paren must be used for `Kind::Paren`.
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
    /// Turn [Code] into a kind.
    ///
    /// > 👉 **Note**: an opening paren must be used for `Kind::Paren`.
    ///
    /// ## Panics
    ///
    /// Panics if `code` is not `Code::Char('(' | '"' | '\'')`.
    fn from_code(code: Code) -> Kind {
        match code {
            Code::Char(char) => Kind::from_char(char),
            _ => unreachable!("invalid code"),
        }
    }
}

/// State needed to parse titles.
#[derive(Debug)]
struct Info {
    /// Whether we’ve seen data.
    connect: bool,
    /// Kind of title.
    kind: Kind,
    /// Configuration.
    options: Options,
}

/// Before a title.
///
/// ```markdown
/// > | "a"
///     ^
/// ```
pub fn start(tokenizer: &mut Tokenizer, options: Options) -> State {
    match tokenizer.current {
        Code::Char('"' | '\'' | '(') => {
            let info = Info {
                connect: false,
                kind: Kind::from_code(tokenizer.current),
                options,
            };
            tokenizer.enter(info.options.title.clone());
            tokenizer.enter(info.options.marker.clone());
            tokenizer.consume();
            tokenizer.exit(info.options.marker.clone());
            State::Fn(Box::new(|t| begin(t, info)))
        }
        _ => State::Nok,
    }
}

/// After the opening marker.
///
/// This is also used when at the closing marker.
///
/// ```markdown
/// > | "a"
///      ^
/// ```
fn begin(tokenizer: &mut Tokenizer, info: Info) -> State {
    match tokenizer.current {
        Code::Char(char) if char == info.kind.as_char() => {
            tokenizer.enter(info.options.marker.clone());
            tokenizer.consume();
            tokenizer.exit(info.options.marker.clone());
            tokenizer.exit(info.options.title);
            State::Ok
        }
        _ => {
            tokenizer.enter(info.options.string.clone());
            at_break(tokenizer, info)
        }
    }
}

/// At something, before something else.
///
/// ```markdown
/// > | "a"
///      ^
/// ```
fn at_break(tokenizer: &mut Tokenizer, mut info: Info) -> State {
    match tokenizer.current {
        Code::Char(char) if char == info.kind.as_char() => {
            tokenizer.exit(info.options.string.clone());
            begin(tokenizer, info)
        }
        Code::None => State::Nok,
        Code::CarriageReturnLineFeed | Code::Char('\n' | '\r') => tokenizer.go(
            space_or_tab_eol_with_options(EolOptions {
                content_type: Some(ContentType::String),
                connect: info.connect,
            }),
            |t| {
                info.connect = true;
                at_break(t, info)
            },
        )(tokenizer),
        _ => {
            tokenizer.enter_with_content(Token::Data, Some(ContentType::String));

            if info.connect {
                let index = tokenizer.events.len() - 1;
                link(&mut tokenizer.events, index);
            } else {
                info.connect = true;
            }

            title(tokenizer, info)
        }
    }
}

/// In title text.
///
/// ```markdown
/// > | "a"
///      ^
/// ```
fn title(tokenizer: &mut Tokenizer, info: Info) -> State {
    match tokenizer.current {
        Code::Char(char) if char == info.kind.as_char() => {
            tokenizer.exit(Token::Data);
            at_break(tokenizer, info)
        }
        Code::None | Code::CarriageReturnLineFeed | Code::Char('\n' | '\r') => {
            tokenizer.exit(Token::Data);
            at_break(tokenizer, info)
        }
        Code::Char('\\') => {
            tokenizer.consume();
            State::Fn(Box::new(|t| escape(t, info)))
        }
        _ => {
            tokenizer.consume();
            State::Fn(Box::new(|t| title(t, info)))
        }
    }
}

/// After `\`, in title text.
///
/// ```markdown
/// > | "a\*b"
///      ^
/// ```
fn escape(tokenizer: &mut Tokenizer, info: Info) -> State {
    match tokenizer.current {
        Code::Char(char) if char == info.kind.as_char() => {
            tokenizer.consume();
            State::Fn(Box::new(|t| title(t, info)))
        }
        _ => title(tokenizer, info),
    }
}
