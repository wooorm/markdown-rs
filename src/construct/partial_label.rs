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
use crate::tokenizer::{ContentType, State, StateName, Tokenizer};

/// Before a label.
///
/// ```markdown
/// > | [a]
///     ^
/// ```
pub fn start(tokenizer: &mut Tokenizer) -> State {
    match tokenizer.current {
        Some(b'[') => {
            tokenizer.enter(tokenizer.tokenize_state.token_1.clone());
            tokenizer.enter(tokenizer.tokenize_state.token_2.clone());
            tokenizer.consume();
            tokenizer.exit(tokenizer.tokenize_state.token_2.clone());
            tokenizer.enter(tokenizer.tokenize_state.token_3.clone());
            State::Next(StateName::LabelAtBreak)
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
pub fn at_break(tokenizer: &mut Tokenizer) -> State {
    if tokenizer.tokenize_state.size > LINK_REFERENCE_SIZE_MAX
        || matches!(tokenizer.current, None | Some(b'['))
        || (matches!(tokenizer.current, Some(b']')) && !tokenizer.tokenize_state.seen)
    {
        tokenizer.tokenize_state.connect = false;
        tokenizer.tokenize_state.seen = false;
        tokenizer.tokenize_state.size = 0;
        State::Nok
    } else {
        match tokenizer.current {
            Some(b'\n') => {
                let name = space_or_tab_eol_with_options(
                    tokenizer,
                    EolOptions {
                        content_type: Some(ContentType::String),
                        connect: tokenizer.tokenize_state.connect,
                    },
                );
                tokenizer.attempt(
                    name,
                    State::Next(StateName::LabelEolAfter),
                    State::Next(StateName::LabelAtBlankLine),
                )
            }
            Some(b']') => {
                tokenizer.exit(tokenizer.tokenize_state.token_3.clone());
                tokenizer.enter(tokenizer.tokenize_state.token_2.clone());
                tokenizer.consume();
                tokenizer.exit(tokenizer.tokenize_state.token_2.clone());
                tokenizer.exit(tokenizer.tokenize_state.token_1.clone());
                tokenizer.tokenize_state.connect = false;
                tokenizer.tokenize_state.seen = false;
                tokenizer.tokenize_state.size = 0;
                State::Ok
            }
            _ => {
                tokenizer.enter_with_content(Token::Data, Some(ContentType::String));

                if tokenizer.tokenize_state.connect {
                    let index = tokenizer.events.len() - 1;
                    link(&mut tokenizer.events, index);
                } else {
                    tokenizer.tokenize_state.connect = true;
                }

                State::Retry(StateName::LabelInside)
            }
        }
    }
}

/// In a label, after whitespace.
///
/// ```markdown
///   | [a␊
/// > | b]
///     ^
/// ```
pub fn eol_after(tokenizer: &mut Tokenizer) -> State {
    tokenizer.tokenize_state.connect = true;
    State::Retry(StateName::LabelAtBreak)
}

/// In a label, at a blank line.
///
/// ```markdown
///   | [a␊
/// > | ␊
///     ^
///   | b]
/// ```
pub fn at_blank_line(tokenizer: &mut Tokenizer) -> State {
    tokenizer.tokenize_state.marker = 0;
    tokenizer.tokenize_state.connect = false;
    State::Nok
}

/// In a label, in text.
///
/// ```markdown
/// > | [a]
///      ^
/// ```
pub fn inside(tokenizer: &mut Tokenizer) -> State {
    match tokenizer.current {
        None | Some(b'\n' | b'[' | b']') => {
            tokenizer.exit(Token::Data);
            State::Retry(StateName::LabelAtBreak)
        }
        Some(byte) => {
            if tokenizer.tokenize_state.size > LINK_REFERENCE_SIZE_MAX {
                tokenizer.exit(Token::Data);
                State::Retry(StateName::LabelAtBreak)
            } else {
                tokenizer.consume();
                tokenizer.tokenize_state.size += 1;
                if !tokenizer.tokenize_state.seen && !matches!(byte, b'\t' | b' ') {
                    tokenizer.tokenize_state.seen = true;
                }
                State::Next(if matches!(byte, b'\\') {
                    StateName::LabelEscape
                } else {
                    StateName::LabelInside
                })
            }
        }
    }
}

/// After `\` in a label.
///
/// ```markdown
/// > | [a\*a]
///        ^
/// ```
pub fn escape(tokenizer: &mut Tokenizer) -> State {
    match tokenizer.current {
        Some(b'[' | b'\\' | b']') => {
            tokenizer.consume();
            tokenizer.tokenize_state.size += 1;
            State::Next(StateName::LabelInside)
        }
        _ => State::Retry(StateName::LabelInside),
    }
}
