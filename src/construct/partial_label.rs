//! Label occurs in [definition][] and label end.
//!
//! They’re formed with the following BNF:
//!
//! ```bnf
//! ; Restriction: maximum `999` codes allowed between brackets.
//! ; Restriction: no blank lines.
//! ; Restriction: at least 1 non-space and non-eol code must exist.
//! label ::= '[' *( label_text | label_escape ) ']'
//! label_text ::= code - '[' - '\\' - ']'
//! label_escape ::= '\\' [ '[' | '\\' | ']' ]
//! ```
//!
//! The maximum allowed size of the label, without the brackets, is `999`
//! (inclusive), which is defined in
//! [`LINK_REFERENCE_SIZE_MAX`][link_reference_size_max].
//!
//! Labels can contain line endings and whitespace, but they are not allowed to
//! contain blank lines, and they must not be blank themselves.
//!
//! The label is interpreted as the [string][] content type.
//! That means that [character escapes][character_escape] and
//! [character references][character_reference] are allowed.
//!
//! > 👉 **Note**: this label relates to, but is not, the initial “label” of
//! > what is know as a reference in markdown:
//! >
//! > | Kind      | Link     | Image     |
//! > | --------- | -------- | --------- |
//! > | Shortcut  | `[x]`    | `![x]`    |
//! > | Collapsed | `[x][]`  | `![x][]`  |
//! > | Full      | `[x][y]` | `![x][y]` |
//! >
//! > The 6 above things are references, in the three kinds they come in, as
//! > links and images.
//! > The label that this module focusses on is only the thing that contains
//! > `y`.
//! >
//! > The thing that contains `x` is not a single thing when parsing markdown,
//! > but instead constists of an opening
//! > ([label start (image)][label_start_image] or
//! > [label start (link)][label_start_link]) and a closing
//! > ([label end][label_end]), so as to allow further phrasing such as
//! > [code (text)][code_text] or [attention][].
//!
//! ## References
//!
//! *   [`micromark-factory-label/index.js` in `micromark`](https://github.com/micromark/micromark/blob/main/packages/micromark-factory-label/dev/index.js)
//!
//! [definition]: crate::construct::definition
//! [string]: crate::content::string
//! [attention]: crate::construct::attention
//! [character_escape]: crate::construct::character_escape
//! [character_reference]: crate::construct::character_reference
//! [label_start_image]: crate::construct::label_start_image
//! [label_start_link]: crate::construct::label_start_link
//! [label_end]: crate::construct::label_end
//! [code_text]: crate::construct::code_text
//! [link_reference_size_max]: crate::constant::LINK_REFERENCE_SIZE_MAX

use super::partial_space_or_tab::{space_or_tab_eol_with_options, EolOptions};
use crate::constant::LINK_REFERENCE_SIZE_MAX;
use crate::subtokenize::link;
use crate::token::Token;
use crate::tokenizer::{ContentType, State, Tokenizer};

/// Configuration.
///
/// You must pass the token types in that are used.
#[derive(Debug)]
pub struct Options {
    /// Token for the whole label.
    pub label: Token,
    /// Token for the markers.
    pub marker: Token,
    /// Token for the string (inside the markers).
    pub string: Token,
}

/// State needed to parse labels.
#[derive(Debug)]
struct Info {
    /// Whether we’ve seen our first `ChunkString`.
    connect: bool,
    /// Whether there are non-blank characters in the label.
    data: bool,
    /// Number of characters in the label.
    size: usize,
    /// Configuration.
    options: Options,
}

/// Before a label.
///
/// ```markdown
/// > | [a]
///     ^
/// ```
pub fn start(tokenizer: &mut Tokenizer, options: Options) -> State {
    match tokenizer.current {
        Some('[') => {
            let info = Info {
                connect: false,
                data: false,
                size: 0,
                options,
            };
            tokenizer.enter(info.options.label.clone());
            tokenizer.enter(info.options.marker.clone());
            tokenizer.consume();
            tokenizer.exit(info.options.marker.clone());
            tokenizer.enter(info.options.string.clone());
            State::Fn(Box::new(|t| at_break(t, info)))
        }
        _ => State::Nok,
    }
}

/// In a label, at something.
///
/// ```markdown
/// > | [a]
///      ^
/// ```
fn at_break(tokenizer: &mut Tokenizer, mut info: Info) -> State {
    match tokenizer.current {
        None | Some('[') => State::Nok,
        Some(']') if !info.data => State::Nok,
        _ if info.size > LINK_REFERENCE_SIZE_MAX => State::Nok,
        Some(']') => {
            tokenizer.exit(info.options.string.clone());
            tokenizer.enter(info.options.marker.clone());
            tokenizer.consume();
            tokenizer.exit(info.options.marker.clone());
            tokenizer.exit(info.options.label);
            State::Ok
        }
        Some('\n') => tokenizer.go(
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

            label(tokenizer, info)
        }
    }
}

/// In a label, in text.
///
/// ```markdown
/// > | [a]
///      ^
/// ```
fn label(tokenizer: &mut Tokenizer, mut info: Info) -> State {
    match tokenizer.current {
        None | Some('\n' | '[' | ']') => {
            tokenizer.exit(Token::Data);
            at_break(tokenizer, info)
        }
        _ if info.size > LINK_REFERENCE_SIZE_MAX => {
            tokenizer.exit(Token::Data);
            at_break(tokenizer, info)
        }
        Some('\t' | ' ') => {
            tokenizer.consume();
            info.size += 1;
            State::Fn(Box::new(|t| label(t, info)))
        }
        Some('\\') => {
            tokenizer.consume();
            info.size += 1;
            if !info.data {
                info.data = true;
            }
            State::Fn(Box::new(|t| escape(t, info)))
        }
        Some(_) => {
            tokenizer.consume();
            info.size += 1;
            if !info.data {
                info.data = true;
            }
            State::Fn(Box::new(|t| label(t, info)))
        }
    }
}

/// After `\` in a label.
///
/// ```markdown
/// > | [a\*a]
///        ^
/// ```
fn escape(tokenizer: &mut Tokenizer, mut info: Info) -> State {
    match tokenizer.current {
        Some('[' | '\\' | ']') => {
            tokenizer.consume();
            info.size += 1;
            State::Fn(Box::new(|t| label(t, info)))
        }
        _ => label(tokenizer, info),
    }
}
