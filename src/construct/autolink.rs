//! Autolinks are a construct that occurs in the [text][] content type.
//!
//! It forms with the following BNF:
//!
//! ```bnf
//! autolink ::= '<' ( url | email ) '>'
//!
//! url ::= ascii_alphabetic 0*31( '+' '-' '.' ascii_alphanumeric ) ':' *( code - ascii_control - '\r' - '\n' - ' ')
//! email ::= 1*ascii_atext '@' domain *('.' domain)
//! ; Restriction: up to (including) 63 character are allowed in each domain.
//! domain ::= ascii_alphanumeric *( ascii_alphanumeric | '-' ascii_alphanumeric )
//! ascii_atext ::= ascii_alphanumeric | '#' .. '\'' | '*' | '+' | '-' | '/' | '=' | '?' | '^' .. '`' | '{' .. '~'
//! ```
//!
//! Autolinks relate to the `<a>` element in HTML.
//! See [*§ 4.5.1 The `a` element*][html-a] in the HTML spec for more info.
//! When an email autolink is used (so, without a protocol), the string
//! `mailto:` is prepended before the email, when generating the `href`
//! attribute of the hyperlink.
//!
//! The maximum allowed size of a scheme is `31` (inclusive), which is defined
//! in [`AUTOLINK_SCHEME_SIZE_MAX`][autolink_scheme_size_max].
//! The maximum allowed size of a domain is `63` (inclusive), which is defined
//! in [`AUTOLINK_DOMAIN_SIZE_MAX`][autolink_domain_size_max].
//!
//! The grammar for autolinks is quite strict and prohibits the use of ASCII control
//! characters or spaces.
//! To use non-ascii characters and otherwise impossible characters, in URLs,
//! you can use percent encoding:
//!
//! ```markdown
//! <https://example.com/alpha%20bravo>
//! ```
//!
//! Yields:
//!
//! ```html
//! <p><a href="https://example.com/alpha%20bravo">https://example.com/alpha%20bravo</a></p>
//! ```
//!
//! There are several cases where incorrect encoding of URLs would, in other
//! languages, result in a parse error.
//! In markdown, there are no errors, and URLs are normalized.
//! In addition, unicode characters are percent encoded
//! ([`sanitize_uri`][sanitize_uri]).
//! For example:
//!
//! ```markdown
//! <https://a👍b%>
//! ```
//!
//! Yields:
//!
//! ```html
//! <p><a href="https://a%F0%9F%91%8Db%25">https://a👍b%</a></p>
//! ```
//!
//! Interestingly, there are a couple of things that are valid autolinks in
//! markdown but in HTML would be valid tags, such as `<svg:rect>` and
//! `<xml:lang/>`.
//! However, because `CommonMark` employs a naïve HTML parsing algorithm, those
//! are not considered HTML.
//!
//! While `CommonMark` restricts links from occurring in other links in the
//! case of bracketed links, this restriction is not in place for autolinks
//! inside autolinks:
//!
//! ```markdown
//! [<https://example.com>](#)
//! ```
//!
//! Yields:
//!
//! ```html
//! <p><a href="#"><a href="https://example.com">https://example.com</a></a></p>
//! ```
//!
//! The generated output, in this case, is invalid according to HTML.
//! When a browser sees that markup, it will instead parse it as:
//!
//! ```html
//! <p><a href="#"></a><a href="https://example.com">https://example.com</a></p>
//! ```
//!
//! ## Tokens
//!
//! *   [`Autolink`][TokenType::Autolink]
//! *   [`AutolinkEmail`][TokenType::AutolinkEmail]
//! *   [`AutolinkMarker`][TokenType::AutolinkMarker]
//! *   [`AutolinkProtocol`][TokenType::AutolinkProtocol]
//!
//! ## References
//!
//! *   [`autolink.js` in `micromark`](https://github.com/micromark/micromark/blob/main/packages/micromark-core-commonmark/dev/lib/autolink.js)
//! *   [*§ 6.4 Autolinks* in `CommonMark`](https://spec.commonmark.org/0.30/#autolinks)
//!
//! [text]: crate::content::text
//! [autolink_scheme_size_max]: crate::constant::AUTOLINK_SCHEME_SIZE_MAX
//! [autolink_domain_size_max]: crate::constant::AUTOLINK_DOMAIN_SIZE_MAX
//! [sanitize_uri]: crate::util::sanitize_uri
//! [html-a]: https://html.spec.whatwg.org/multipage/text-level-semantics.html#the-a-element

use crate::constant::{AUTOLINK_DOMAIN_SIZE_MAX, AUTOLINK_SCHEME_SIZE_MAX};
use crate::tokenizer::{Code, State, StateFnResult, TokenType, Tokenizer};

/// Start of an autolink.
///
/// ```markdown
/// a|<https://example.com>b
/// a|<user@example.com>b
/// ```
pub fn start(tokenizer: &mut Tokenizer, code: Code) -> StateFnResult {
    match code {
        Code::Char('<') => {
            tokenizer.enter(TokenType::Autolink);
            tokenizer.enter(TokenType::AutolinkMarker);
            tokenizer.consume(code);
            tokenizer.exit(TokenType::AutolinkMarker);
            tokenizer.enter(TokenType::AutolinkProtocol);
            (State::Fn(Box::new(open)), None)
        }
        _ => (State::Nok, None),
    }
}

/// After `<`, before the protocol.
///
/// ```markdown
/// a<|https://example.com>b
/// a<|user@example.com>b
/// ```
fn open(tokenizer: &mut Tokenizer, code: Code) -> StateFnResult {
    match code {
        Code::Char(char) if char.is_ascii_alphabetic() => {
            tokenizer.consume(code);
            (State::Fn(Box::new(scheme_or_email_atext)), None)
        }
        Code::Char(char) if is_ascii_atext(char) => email_atext(tokenizer, code),
        _ => (State::Nok, None),
    }
}

/// After the first character of the protocol or email name.
///
/// ```markdown
/// a<h|ttps://example.com>b
/// a<u|ser@example.com>b
/// ```
fn scheme_or_email_atext(tokenizer: &mut Tokenizer, code: Code) -> StateFnResult {
    // Whether this character can be both a protocol and email atext.
    let unknown = match code {
        Code::Char('+' | '-' | '.') => true,
        Code::Char(char) if char.is_ascii_alphanumeric() => true,
        _ => false,
    };

    if unknown {
        scheme_inside_or_email_atext(tokenizer, code, 1)
    } else {
        email_atext(tokenizer, code)
    }
}

/// Inside an ambiguous protocol or email name.
///
/// ```markdown
/// a<ht|tps://example.com>b
/// a<us|er@example.com>b
/// ```
fn scheme_inside_or_email_atext(
    tokenizer: &mut Tokenizer,
    code: Code,
    size: usize,
) -> StateFnResult {
    if let Code::Char(':') = code {
        tokenizer.consume(code);
        (State::Fn(Box::new(url_inside)), None)
    } else {
        // Whether this character can be both a protocol and email atext.
        let unknown = match code {
            Code::Char('+' | '-' | '.') if size < AUTOLINK_SCHEME_SIZE_MAX => true,
            Code::Char(char) if char.is_ascii_alphanumeric() && size < AUTOLINK_SCHEME_SIZE_MAX => {
                true
            }
            _ => false,
        };

        if unknown {
            tokenizer.consume(code);
            (
                State::Fn(Box::new(move |t, c| {
                    scheme_inside_or_email_atext(t, c, size + 1)
                })),
                None,
            )
        } else {
            email_atext(tokenizer, code)
        }
    }
}

/// Inside a URL, after the protocol.
///
/// ```markdown
/// a<https:|//example.com>b
/// ```
fn url_inside(tokenizer: &mut Tokenizer, code: Code) -> StateFnResult {
    match code {
        Code::Char('>') => {
            tokenizer.exit(TokenType::AutolinkProtocol);
            end(tokenizer, code)
        }
        Code::Char(char) if char.is_ascii_control() => (State::Nok, None),
        Code::None | Code::CarriageReturnLineFeed | Code::VirtualSpace | Code::Char(' ') => {
            (State::Nok, None)
        }
        Code::Char(_) => {
            tokenizer.consume(code);
            (State::Fn(Box::new(url_inside)), None)
        }
    }
}

/// Inside email atext.
///
/// ```markdown
/// a<user.na|me@example.com>b
/// ```
fn email_atext(tokenizer: &mut Tokenizer, code: Code) -> StateFnResult {
    match code {
        Code::Char('@') => {
            tokenizer.consume(code);
            (
                State::Fn(Box::new(|t, c| email_at_sign_or_dot(t, c, 0))),
                None,
            )
        }
        Code::Char(char) if is_ascii_atext(char) => {
            tokenizer.consume(code);
            (State::Fn(Box::new(email_atext)), None)
        }
        _ => (State::Nok, None),
    }
}

/// After an at-sign or a dot in the label.
///
/// ```markdown
/// a<user.name@|example.com>b
/// a<user.name@example.|com>b
/// ```
fn email_at_sign_or_dot(tokenizer: &mut Tokenizer, code: Code, size: usize) -> StateFnResult {
    match code {
        Code::Char(char) if char.is_ascii_alphanumeric() => email_value(tokenizer, code, size),
        _ => (State::Nok, None),
    }
}

/// In the label, where `.` and `>` are allowed.
///
/// ```markdown
/// a<user.name@ex|ample.com>b
/// ```
fn email_label(tokenizer: &mut Tokenizer, code: Code, size: usize) -> StateFnResult {
    match code {
        Code::Char('.') => {
            tokenizer.consume(code);
            (
                State::Fn(Box::new(|t, c| email_at_sign_or_dot(t, c, 0))),
                None,
            )
        }
        Code::Char('>') => {
            let index = tokenizer.events.len();
            tokenizer.exit(TokenType::AutolinkProtocol);
            // Change the token type.
            tokenizer.events[index - 1].token_type = TokenType::AutolinkEmail;
            tokenizer.events[index].token_type = TokenType::AutolinkEmail;
            end(tokenizer, code)
        }
        _ => email_value(tokenizer, code, size),
    }
}

/// In the label, where `.` and `>` are *not* allowed.
///
/// Though, this is also used in `email_label` to parse other values.
///
/// ```markdown
/// a<user.name@ex-|ample.com>b
/// ```
fn email_value(tokenizer: &mut Tokenizer, code: Code, size: usize) -> StateFnResult {
    let ok = match code {
        Code::Char('-') if size < AUTOLINK_DOMAIN_SIZE_MAX => true,
        Code::Char(char) if char.is_ascii_alphanumeric() && size < AUTOLINK_DOMAIN_SIZE_MAX => true,
        _ => false,
    };

    if ok {
        tokenizer.consume(code);
        let func = if let Code::Char('-') = code {
            email_value
        } else {
            email_label
        };
        (State::Fn(Box::new(move |t, c| func(t, c, size + 1))), None)
    } else {
        (State::Nok, None)
    }
}

/// At the `>`.
///
/// ```markdown
/// a<https://example.com|>b
/// a<user@example.com|>b
/// ```
fn end(tokenizer: &mut Tokenizer, code: Code) -> StateFnResult {
    match code {
        Code::Char('>') => {
            tokenizer.enter(TokenType::AutolinkMarker);
            tokenizer.consume(code);
            tokenizer.exit(TokenType::AutolinkMarker);
            tokenizer.exit(TokenType::Autolink);
            (State::Ok, None)
        }
        _ => unreachable!("expected `>` at `end`"),
    }
}

/// Check whether the character code represents an ASCII atext.
///
/// atext is an ASCII alphanumeric (see [`is_ascii_alphanumeric`][]), or a character in
/// the inclusive ranges U+0023 NUMBER SIGN (`#`) to U+0027 APOSTROPHE (`'`),
/// U+002A ASTERISK (`*`), U+002B PLUS SIGN (`+`), U+002D DASH (`-`), U+002F
/// SLASH (`/`), U+003D EQUALS TO (`=`), U+003F QUESTION MARK (`?`), U+005E
/// CARET (`^`) to U+0060 GRAVE ACCENT (`` ` ``), or U+007B LEFT CURLY BRACE
/// (`{`) to U+007E TILDE (`~`).
///
/// See:
/// **\[RFC5322]**:
/// [Internet Message Format](https://tools.ietf.org/html/rfc5322).
/// P. Resnick.
/// IETF.
///
/// [`is_ascii_alphanumeric`]: char::is_ascii_alphanumeric
fn is_ascii_atext(x: char) -> bool {
    matches!(x, '#'..='\'' | '*' | '+' | '-'..='9' | '=' | '?' | 'A'..='Z' | '^'..='~')
}
