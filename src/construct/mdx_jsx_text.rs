//! To do.

use crate::construct::partial_space_or_tab_eol::space_or_tab_eol;
use crate::event::Name;
use crate::state::{Name as StateName, State};
use crate::tokenizer::Tokenizer;
use crate::util::{
    classify_character::Kind as CharacterKind,
    slice::{byte_to_kind, char_after_index},
};
use alloc::{
    format,
    string::{String, ToString},
};
use core::str;
use unicode_id::UnicodeID;

/// Start of MDX: JSX (text).
///
/// ```markdown
/// > | a <B /> c
///       ^
/// ```
pub fn start(tokenizer: &mut Tokenizer) -> State {
    if Some(b'<') == tokenizer.current && tokenizer.parse_state.options.constructs.mdx_jsx_text {
        tokenizer.enter(Name::MdxJsxTextTag);
        tokenizer.enter(Name::MdxJsxTextTagMarker);
        tokenizer.consume();
        tokenizer.exit(Name::MdxJsxTextTagMarker);
        State::Next(StateName::MdxJsxTextStartAfter)
    } else {
        State::Nok
    }
}

/// After `<`.
///
/// ```markdown
/// > | a <B /> c
///        ^
/// ```
pub fn start_after(tokenizer: &mut Tokenizer) -> State {
    // Deviate from JSX, which allows arbitrary whitespace.
    // See: <https://github.com/micromark/micromark-extension-mdx-jsx/issues/7>.
    if let Some(b'\t' | b'\n' | b' ') = tokenizer.current {
        State::Nok
    } else {
        tokenizer.attempt(State::Next(StateName::MdxJsxTextNameBefore), State::Nok);
        State::Retry(StateName::MdxJsxTextEsWhitespaceStart)
    }
}

/// Before name, self slash, or end of tag for fragments.
///
/// ```markdown
/// > | a <B> c
///        ^
/// > | a </B> c
///        ^
/// > | a <> b
///        ^
/// ```
pub fn name_before(tokenizer: &mut Tokenizer) -> State {
    match tokenizer.current {
        // Closing tag.
        Some(b'/') => {
            tokenizer.enter(Name::MdxJsxTextTagClosingMarker);
            tokenizer.consume();
            tokenizer.exit(Name::MdxJsxTextTagClosingMarker);
            tokenizer.attempt(
                State::Next(StateName::MdxJsxTextClosingTagNameBefore),
                State::Nok,
            );
            State::Next(StateName::MdxJsxTextEsWhitespaceStart)
        }
        // Fragment opening tag.
        Some(b'>') => State::Retry(StateName::MdxJsxTextTagEnd),
        _ => {
            if id_start(char_after_index(
                tokenizer.parse_state.bytes,
                tokenizer.point.index,
            )) {
                tokenizer.enter(Name::MdxJsxTextTagName);
                tokenizer.enter(Name::MdxJsxTextTagNamePrimary);
                tokenizer.consume();
                State::Next(StateName::MdxJsxTextPrimaryName)
            } else {
                crash(
                    tokenizer,
                    "before name",
                    &format!(
                        "a character that can start a name, such as a letter, `$`, or `_`{}",
                        if tokenizer.current == Some(b'!') {
                            " (note: to create a comment in MDX, use `{/* text */}`)"
                        } else {
                            ""
                        }
                    ),
                )
            }
        }
    }
}

/// Before name of closing tag or end of closing fragment tag.
///
/// ```markdown
/// > | a </> b
///         ^
/// > | a </B> c
///         ^
/// ```
pub fn closing_tag_name_before(tokenizer: &mut Tokenizer) -> State {
    // Fragment closing tag.
    if let Some(b'>') = tokenizer.current {
        State::Retry(StateName::MdxJsxTextTagEnd)
    }
    // Start of a closing tag name.
    else if id_start(char_after_index(
        tokenizer.parse_state.bytes,
        tokenizer.point.index,
    )) {
        tokenizer.enter(Name::MdxJsxTextTagName);
        tokenizer.enter(Name::MdxJsxTextTagNamePrimary);
        tokenizer.consume();
        State::Next(StateName::MdxJsxTextPrimaryName)
    } else {
        crash(
            tokenizer,
            "before name",
            &format!(
                "a character that can start a name, such as a letter, `$`, or `_`{}",
                if tokenizer.current == Some(b'*' | b'/') {
                    " (note: JS comments in JSX tags are not supported in MDX)"
                } else {
                    ""
                }
            ),
        )
    }
}

/// In primary name.
///
/// ```markdown
/// > | a <Bc> d
///         ^
/// ```
pub fn primary_name(tokenizer: &mut Tokenizer) -> State {
    // End of name.
    if byte_to_kind(tokenizer.parse_state.bytes, tokenizer.point.index) == CharacterKind::Whitespace
        || matches!(tokenizer.current, Some(b'.' | b'/' | b':' | b'>' | b'{'))
    {
        tokenizer.exit(Name::MdxJsxTextTagNamePrimary);
        tokenizer.attempt(
            State::Next(StateName::MdxJsxTextPrimaryNameAfter),
            State::Nok,
        );
        State::Retry(StateName::MdxJsxTextEsWhitespaceStart)
    }
    // Continuation of name: remain.
    // Allow continuation bytes.
    else if matches!(tokenizer.current, Some(0x80..=0xBF))
        || id_cont(char_after_index(
            tokenizer.parse_state.bytes,
            tokenizer.point.index,
        ))
    {
        tokenizer.consume();
        State::Next(StateName::MdxJsxTextPrimaryName)
    } else {
        crash(
            tokenizer,
            "in name",
            &format!(
                "a name character such as letters, digits, `$`, or `_`; whitespace before attributes; or the end of the tag{}",
                if tokenizer.current == Some(b'@') {
                    " (note: to create a link in MDX, use `[text](url)`)"
                } else {
                    ""
                }
            ),
        )
    }
}

/// After primary name.
///
/// ```markdown
/// > | a <b.c> d
///         ^
/// > | a <b:c> d
///         ^
/// ```
pub fn primary_name_after(tokenizer: &mut Tokenizer) -> State {
    match tokenizer.current {
        // Start of a member name.
        Some(b'.') => {
            tokenizer.enter(Name::MdxJsxTextTagNameMemberMarker);
            tokenizer.consume();
            tokenizer.exit(Name::MdxJsxTextTagNameMemberMarker);
            tokenizer.attempt(
                State::Next(StateName::MdxJsxTextMemberNameBefore),
                State::Nok,
            );
            State::Next(StateName::MdxJsxTextEsWhitespaceStart)
        }
        // Start of a local name.
        Some(b':') => {
            tokenizer.enter(Name::MdxJsxTextTagNamePrefixMarker);
            tokenizer.consume();
            tokenizer.exit(Name::MdxJsxTextTagNamePrefixMarker);
            tokenizer.attempt(
                State::Next(StateName::MdxJsxTextLocalNameBefore),
                State::Nok,
            );
            State::Next(StateName::MdxJsxTextEsWhitespaceStart)
        }
        // End of name.
        _ => {
            if matches!(tokenizer.current, Some(b'/' | b'>' | b'{'))
                || id_start(char_after_index(
                    tokenizer.parse_state.bytes,
                    tokenizer.point.index,
                ))
            {
                tokenizer.exit(Name::MdxJsxTextTagName);
                State::Retry(StateName::MdxJsxTextAttributeBefore)
            } else {
                crash(
                    tokenizer,
                    "after name",
                    "a character that can start an attribute name, such as a letter, `$`, or `_`; whitespace before attributes; or the end of the tag"
                )
            }
        }
    }
}

/// Before member name.
///
/// ```markdown
/// > | a <b.c> d
///          ^
/// ```
pub fn member_name_before(tokenizer: &mut Tokenizer) -> State {
    // Start of a member name.
    if id_start(char_after_index(
        tokenizer.parse_state.bytes,
        tokenizer.point.index,
    )) {
        tokenizer.enter(Name::MdxJsxTextTagNameMember);
        tokenizer.consume();
        State::Next(StateName::MdxJsxTextMemberName)
    } else {
        crash(
            tokenizer,
            "before member name",
            "a character that can start an attribute name, such as a letter, `$`, or `_`; whitespace before attributes; or the end of the tag"
        )
    }
}

/// In member name.
///
/// ```markdown
/// > | a <b.cd> e
///           ^
/// ```
pub fn member_name(tokenizer: &mut Tokenizer) -> State {
    // End of name.
    // Note: no `:` allowed here.
    if byte_to_kind(tokenizer.parse_state.bytes, tokenizer.point.index) == CharacterKind::Whitespace
        || matches!(tokenizer.current, Some(b'.' | b'/' | b'>' | b'{'))
    {
        tokenizer.exit(Name::MdxJsxTextTagNameMember);
        tokenizer.attempt(
            State::Next(StateName::MdxJsxTextMemberNameAfter),
            State::Nok,
        );
        State::Retry(StateName::MdxJsxTextEsWhitespaceStart)
    }
    // Continuation of name: remain.
    // Allow continuation bytes.
    else if matches!(tokenizer.current, Some(0x80..=0xBF))
        || id_cont(char_after_index(
            tokenizer.parse_state.bytes,
            tokenizer.point.index,
        ))
    {
        tokenizer.consume();
        State::Next(StateName::MdxJsxTextMemberName)
    } else {
        crash(
            tokenizer,
            "in member name",
            &format!(
                "a name character such as letters, digits, `$`, or `_`; whitespace before attributes; or the end of the tag{}",
                if tokenizer.current == Some(b'@') {
                    " (note: to create a link in MDX, use `[text](url)`)"
                } else {
                    ""
                }
            ),
        )
    }
}

/// After member name.
///
/// ```markdown
/// > | a <b.c> d
///           ^
/// > | a <b.c.d> e
///           ^
/// ```
pub fn member_name_after(tokenizer: &mut Tokenizer) -> State {
    match tokenizer.current {
        // Start of another member name.
        Some(b'.') => {
            tokenizer.enter(Name::MdxJsxTextTagNameMemberMarker);
            tokenizer.consume();
            tokenizer.exit(Name::MdxJsxTextTagNameMemberMarker);
            tokenizer.attempt(
                State::Next(StateName::MdxJsxTextMemberNameBefore),
                State::Nok,
            );
            State::Next(StateName::MdxJsxTextEsWhitespaceStart)
        }
        // End of name.
        _ => {
            if matches!(tokenizer.current, Some(b'/' | b'>' | b'{'))
                || id_start(char_after_index(
                    tokenizer.parse_state.bytes,
                    tokenizer.point.index,
                ))
            {
                tokenizer.exit(Name::MdxJsxTextTagName);
                State::Retry(StateName::MdxJsxTextAttributeBefore)
            } else {
                crash(
                    tokenizer,
                    "after member name",
                    "a character that can start an attribute name, such as a letter, `$`, or `_`; whitespace before attributes; or the end of the tag"
                )
            }
        }
    }
}

/// Local member name.
///
/// ```markdown
/// > | a <b:c> d
///          ^
/// ```
pub fn local_name_before(tokenizer: &mut Tokenizer) -> State {
    // Start of a local name.
    if id_start(char_after_index(
        tokenizer.parse_state.bytes,
        tokenizer.point.index,
    )) {
        tokenizer.enter(Name::MdxJsxTextTagNameLocal);
        tokenizer.consume();
        State::Next(StateName::MdxJsxTextLocalName)
    } else {
        crash(
            tokenizer,
            "before local name",
            &format!(
                "a character that can start a name, such as a letter, `$`, or `_`{}",
                if matches!(tokenizer.current, Some(b'+' | b'/'..=b'9')) {
                    " (note: to create a link in MDX, use `[text](url)`)"
                } else {
                    ""
                }
            ),
        )
    }
}

/// In local name.
///
/// ```markdown
/// > | a <b:cd> e
///           ^
/// ```
pub fn local_name(tokenizer: &mut Tokenizer) -> State {
    // End of local name (note that we don’t expect another colon, or a member).
    if byte_to_kind(tokenizer.parse_state.bytes, tokenizer.point.index) == CharacterKind::Whitespace
        || matches!(tokenizer.current, Some(b'/' | b'>' | b'{'))
    {
        tokenizer.exit(Name::MdxJsxTextTagNameLocal);
        tokenizer.attempt(State::Next(StateName::MdxJsxTextLocalNameAfter), State::Nok);
        State::Retry(StateName::MdxJsxTextEsWhitespaceStart)
    }
    // Continuation of name: remain.
    // Allow continuation bytes.
    else if matches!(tokenizer.current, Some(0x80..=0xBF))
        || id_cont(char_after_index(
            tokenizer.parse_state.bytes,
            tokenizer.point.index,
        ))
    {
        tokenizer.consume();
        State::Next(StateName::MdxJsxTextLocalName)
    } else {
        crash(
            tokenizer,
            "in local name",
            "a name character such as letters, digits, `$`, or `_`; whitespace before attributes; or the end of the tag"
        )
    }
}

/// After local name.
///
/// This is like as `primary_name_after`, but we don’t expect colons or
/// periods.
///
/// ```markdown
/// > | a <b.c> d
///           ^
/// > | a <b.c.d> e
///           ^
/// ```
pub fn local_name_after(tokenizer: &mut Tokenizer) -> State {
    // End of name.
    if matches!(tokenizer.current, Some(b'/' | b'>' | b'{'))
        || id_start(char_after_index(
            tokenizer.parse_state.bytes,
            tokenizer.point.index,
        ))
    {
        tokenizer.exit(Name::MdxJsxTextTagName);
        State::Retry(StateName::MdxJsxTextAttributeBefore)
    } else {
        crash(
            tokenizer,
            "after local name",
            "a character that can start an attribute name, such as a letter, `$`, or `_`; whitespace before attributes; or the end of the tag"
        )
    }
}

/// Before attribute.
///
/// ```markdown
/// > | a <b /> c
///          ^
/// > | a <b > c
///          ^
/// > | a <b {...c}> d
///          ^
/// > | a <b c> d
///          ^
/// ```
pub fn attribute_before(tokenizer: &mut Tokenizer) -> State {
    match tokenizer.current {
        // Self-closing.
        Some(b'/') => {
            tokenizer.enter(Name::MdxJsxTextTagSelfClosingMarker);
            tokenizer.consume();
            tokenizer.exit(Name::MdxJsxTextTagSelfClosingMarker);
            tokenizer.attempt(State::Next(StateName::MdxJsxTextSelfClosing), State::Nok);
            State::Next(StateName::MdxJsxTextEsWhitespaceStart)
        }
        // End of tag.
        Some(b'>') => State::Retry(StateName::MdxJsxTextTagEnd),
        // Attribute expression.
        Some(b'{') => unreachable!("to do: attribute expression"),
        _ => {
            // Start of an attribute name.
            if id_start(char_after_index(
                tokenizer.parse_state.bytes,
                tokenizer.point.index,
            )) {
                tokenizer.enter(Name::MdxJsxTextTagAttribute);
                tokenizer.enter(Name::MdxJsxTextTagAttributeName);
                tokenizer.enter(Name::MdxJsxTextTagAttributePrimaryName);
                tokenizer.consume();
                State::Next(StateName::MdxJsxTextAttributePrimaryName)
            } else {
                crash(
                    tokenizer,
                    "before attribute name",
                    "a character that can start an attribute name, such as a letter, `$`, or `_`; whitespace before attributes; or the end of the tag"
                )
            }
        }
    }
}

/// In primary attribute name.
///
/// ```markdown
/// > | a <b cd/> e
///           ^
/// > | a <b c:d> e
///           ^
/// > | a <b c=d> e
///           ^
/// ```
pub fn attribute_primary_name(tokenizer: &mut Tokenizer) -> State {
    // End of attribute name or tag.
    if byte_to_kind(tokenizer.parse_state.bytes, tokenizer.point.index) == CharacterKind::Whitespace
        || matches!(tokenizer.current, Some(b'/' | b':' | b'=' | b'>' | b'{'))
    {
        tokenizer.exit(Name::MdxJsxTextTagAttributePrimaryName);
        tokenizer.attempt(
            State::Next(StateName::MdxJsxTextAttributePrimaryNameAfter),
            State::Nok,
        );
        State::Retry(StateName::MdxJsxTextEsWhitespaceStart)
    }
    // Continuation of name: remain.
    // Allow continuation bytes.
    else if matches!(tokenizer.current, Some(0x80..=0xBF))
        || id_cont(char_after_index(
            tokenizer.parse_state.bytes,
            tokenizer.point.index,
        ))
    {
        tokenizer.consume();
        State::Next(StateName::MdxJsxTextAttributePrimaryName)
    } else {
        crash(
            tokenizer,
            "in attribute name",
            "an attribute name character such as letters, digits, `$`, or `_`; `=` to initialize a value; whitespace before attributes; or the end of the tag"
        )
    }
}

/// After primary attribute name.
///
/// ```markdown
/// > | a <b c/> d
///           ^
/// > | a <b c:d> e
///           ^
/// > | a <b c=d> e
///           ^
/// ```
pub fn attribute_primary_name_after(tokenizer: &mut Tokenizer) -> State {
    match tokenizer.current {
        // Start of a local name.
        Some(b':') => {
            tokenizer.enter(Name::MdxJsxTextTagAttributeNamePrefixMarker);
            tokenizer.consume();
            tokenizer.exit(Name::MdxJsxTextTagAttributeNamePrefixMarker);
            tokenizer.attempt(
                State::Next(StateName::MdxJsxTextAttributeLocalNameBefore),
                State::Nok,
            );
            State::Next(StateName::MdxJsxTextEsWhitespaceStart)
        }
        // Initializer: start of an attribute value.
        Some(b'=') => {
            tokenizer.exit(Name::MdxJsxTextTagAttributeName);
            tokenizer.enter(Name::MdxJsxTextTagAttributeInitializerMarker);
            tokenizer.consume();
            tokenizer.exit(Name::MdxJsxTextTagAttributeInitializerMarker);
            tokenizer.attempt(
                State::Next(StateName::MdxJsxTextAttributeValueBefore),
                State::Nok,
            );
            State::Retry(StateName::MdxJsxTextEsWhitespaceStart)
        }
        _ => {
            // End of tag / new attribute.
            if byte_to_kind(tokenizer.parse_state.bytes, tokenizer.point.index)
                == CharacterKind::Whitespace
                || matches!(tokenizer.current, Some(b'/' | b'>' | b'{'))
                || id_start(char_after_index(
                    tokenizer.parse_state.bytes,
                    tokenizer.point.index,
                ))
            {
                tokenizer.exit(Name::MdxJsxTextTagAttributeName);
                tokenizer.exit(Name::MdxJsxTextTagAttribute);
                tokenizer.attempt(
                    State::Next(StateName::MdxJsxTextAttributeBefore),
                    State::Nok,
                );
                State::Retry(StateName::MdxJsxTextEsWhitespaceStart)
            } else {
                crash(
                    tokenizer,
                    "after attribute name",
                    "a character that can start an attribute name, such as a letter, `$`, or `_`; `=` to initialize a value; or the end of the tag"
                )
            }
        }
    }
}

/// Before local attribute name.
///
/// ```markdown
/// > | a <b c:d/> e
///            ^
/// ```
pub fn attribute_local_name_before(tokenizer: &mut Tokenizer) -> State {
    // Start of a local name.
    if id_start(char_after_index(
        tokenizer.parse_state.bytes,
        tokenizer.point.index,
    )) {
        tokenizer.enter(Name::MdxJsxTextTagAttributeNameLocal);
        tokenizer.consume();
        State::Next(StateName::MdxJsxTextAttributeLocalName)
    } else {
        crash(
            tokenizer,
            "before local attribute name",
            "a character that can start an attribute name, such as a letter, `$`, or `_`; `=` to initialize a value; or the end of the tag"
        )
    }
}

/// In local attribute name.
///
/// ```markdown
/// > | a <b c:de/> f
///             ^
/// > | a <b c:d=e/> f
///             ^
/// ```
pub fn attribute_local_name(tokenizer: &mut Tokenizer) -> State {
    // End of local name (note that we don’t expect another colon).
    if byte_to_kind(tokenizer.parse_state.bytes, tokenizer.point.index) == CharacterKind::Whitespace
        || matches!(tokenizer.current, Some(b'/' | b'=' | b'>' | b'{'))
    {
        tokenizer.exit(Name::MdxJsxTextTagAttributeNameLocal);
        tokenizer.exit(Name::MdxJsxTextTagAttributeName);
        tokenizer.attempt(
            State::Next(StateName::MdxJsxTextAttributeLocalNameAfter),
            State::Nok,
        );
        State::Retry(StateName::MdxJsxTextEsWhitespaceStart)
    }
    // Continuation of name: remain.
    // Allow continuation bytes.
    else if matches!(tokenizer.current, Some(0x80..=0xBF))
        || id_cont(char_after_index(
            tokenizer.parse_state.bytes,
            tokenizer.point.index,
        ))
    {
        tokenizer.consume();
        State::Next(StateName::MdxJsxTextAttributeLocalName)
    } else {
        crash(
            tokenizer,
            "in local attribute name",
            "an attribute name character such as letters, digits, `$`, or `_`; `=` to initialize a value; whitespace before attributes; or the end of the tag"
        )
    }
}

/// After local attribute name.
///
/// ```markdown
/// > | a <b c:d/> f
///             ^
/// > | a <b c:d=e/> f
///             ^
/// ```
pub fn attribute_local_name_after(tokenizer: &mut Tokenizer) -> State {
    match tokenizer.current {
        // Start of an attribute value.
        Some(b'=') => {
            tokenizer.enter(Name::MdxJsxTextTagAttributeInitializerMarker);
            tokenizer.consume();
            tokenizer.exit(Name::MdxJsxTextTagAttributeInitializerMarker);
            tokenizer.attempt(
                State::Next(StateName::MdxJsxTextAttributeValueBefore),
                State::Nok,
            );
            State::Next(StateName::MdxJsxTextEsWhitespaceStart)
        }
        _ => {
            // End of name.
            if matches!(tokenizer.current, Some(b'/' | b'>' | b'{'))
                || id_start(char_after_index(
                    tokenizer.parse_state.bytes,
                    tokenizer.point.index,
                ))
            {
                tokenizer.exit(Name::MdxJsxTextTagAttribute);
                State::Retry(StateName::MdxJsxTextAttributeBefore)
            } else {
                crash(
                    tokenizer,
                    "after local attribute name",
                    "a character that can start an attribute name, such as a letter, `$`, or `_`; `=` to initialize a value; or the end of the tag"
                )
            }
        }
    }
}

/// After `=`, before value.
///
/// ```markdown
/// > | a <b c="d"/> e
///            ^
/// > | a <b c={d}/> e
///            ^
/// ```
pub fn attribute_value_before(tokenizer: &mut Tokenizer) -> State {
    match tokenizer.current {
        // Start of double- or single quoted value.
        Some(b'"' | b'\'') => {
            tokenizer.tokenize_state.marker = tokenizer.current.unwrap();
            tokenizer.enter(Name::MdxJsxTextTagAttributeValueLiteral);
            tokenizer.enter(Name::MdxJsxTextTagAttributeValueLiteralMarker);
            tokenizer.consume();
            tokenizer.exit(Name::MdxJsxTextTagAttributeValueLiteralMarker);
            State::Next(StateName::MdxJsxTextAttributeValueQuotedStart)
        }
        // Attribute value expression.
        Some(b'{') => unreachable!("to do: attribute value expression"),
        _ => crash(
            tokenizer,
            "before attribute value",
            &format!(
                "a character that can start an attribute value, such as `\"`, `'`, or `{{`{}",
                if tokenizer.current == Some(b'<') {
                    " (note: to use an element or fragment as a prop value in MDX, use `{<element />}`)"
                } else {
                    ""
                }
            ),
        ),
    }
}

/// Before quoted literal attribute value.
///
/// ```markdown
/// > | a <b c="d"/> e
///            ^
/// ```
pub fn attribute_value_quoted_start(tokenizer: &mut Tokenizer) -> State {
    if let Some(byte) = tokenizer.current {
        if byte == tokenizer.tokenize_state.marker {
            tokenizer.tokenize_state.marker = 0;
            tokenizer.enter(Name::MdxJsxTextTagAttributeValueLiteralMarker);
            tokenizer.consume();
            tokenizer.exit(Name::MdxJsxTextTagAttributeValueLiteralMarker);
            tokenizer.exit(Name::MdxJsxTextTagAttributeValueLiteral);
            tokenizer.exit(Name::MdxJsxTextTagAttribute);
            tokenizer.attempt(
                State::Next(StateName::MdxJsxTextAttributeBefore),
                State::Nok,
            );
            State::Next(StateName::MdxJsxTextEsWhitespaceStart)
        } else if byte == b'\n' {
            tokenizer.attempt(
                State::Next(StateName::MdxJsxTextAttributeValueQuotedStart),
                State::Nok,
            );
            State::Retry(StateName::MdxJsxTextEsWhitespaceStart)
        } else {
            tokenizer.enter(Name::MdxJsxTextTagAttributeValueLiteralValue);
            State::Retry(StateName::MdxJsxTextAttributeValueQuoted)
        }
    } else {
        crash(
            tokenizer,
            "in attribute value",
            &format!(
                "a corresponding closing quote {}",
                format_byte(tokenizer.tokenize_state.marker)
            ),
        )
    }
}

/// In quoted literal attribute value.
///
/// ```markdown
/// > | a <b c="d"/> e
///             ^
/// ```
pub fn attribute_value_quoted(tokenizer: &mut Tokenizer) -> State {
    // To do: doesn’t this break for:
    // ```markdown
    // a <b c="d"
    // "f">
    if tokenizer.current == Some(tokenizer.tokenize_state.marker)
        || matches!(tokenizer.current, None | Some(b'\n'))
    {
        tokenizer.exit(Name::MdxJsxTextTagAttributeValueLiteralValue);
        State::Retry(StateName::MdxJsxTextAttributeValueQuotedStart)
    } else {
        tokenizer.consume();
        State::Next(StateName::MdxJsxTextAttributeValueQuoted)
    }
}

/// After self-closing slash.
///
/// ```markdown
/// > | a <b/> c
///          ^
/// ```
pub fn self_closing(tokenizer: &mut Tokenizer) -> State {
    match tokenizer.current {
        Some(b'>') => State::Retry(StateName::MdxJsxTextTagEnd),
        _ => crash(
            tokenizer,
            "after self-closing slash",
            &format!(
                "`>` to end the tag{}",
                if tokenizer.current == Some(b'*' | b'/') {
                    " (note: JS comments in JSX tags are not supported in MDX)"
                } else {
                    ""
                }
            ),
        ),
    }
}

/// At final `>`.
///
/// ```markdown
/// > | a <b> c
///         ^
/// ```
pub fn tag_end(tokenizer: &mut Tokenizer) -> State {
    match tokenizer.current {
        Some(b'>') => {
            tokenizer.enter(Name::MdxJsxTextTagMarker);
            tokenizer.consume();
            tokenizer.exit(Name::MdxJsxTextTagMarker);
            tokenizer.exit(Name::MdxJsxTextTag);
            State::Ok
        }
        _ => unreachable!("expected `>`"),
    }
}

/// Before optional ECMAScript whitespace.
///
/// ```markdown
/// > | a <a b> c
///         ^
/// ```
pub fn es_whitespace_start(tokenizer: &mut Tokenizer) -> State {
    match tokenizer.current {
        Some(b'\n') => {
            // To do: check if this works for blank lines?
            tokenizer.attempt(
                State::Next(StateName::MdxJsxTextEsWhitespaceStart),
                State::Nok,
            );
            State::Retry(space_or_tab_eol(tokenizer))
        }
        _ => {
            if byte_to_kind(tokenizer.parse_state.bytes, tokenizer.point.index)
                == CharacterKind::Whitespace
            {
                tokenizer.enter(Name::MdxJsxTextEsWhitespace);
                State::Retry(StateName::MdxJsxTextEsWhitespaceInside)
            } else {
                State::Ok
            }
        }
    }
}

/// In ECMAScript whitespace.
///
/// ```markdown
/// > | a <a  b> c
///          ^
/// ```
pub fn es_whitespace_inside(tokenizer: &mut Tokenizer) -> State {
    match tokenizer.current {
        Some(b'\n') => {
            tokenizer.exit(Name::MdxJsxTextEsWhitespace);
            State::Retry(StateName::MdxJsxTextEsWhitespaceStart)
        }
        // Allow continuation bytes.
        Some(0x80..=0xBF) => {
            tokenizer.consume();
            State::Next(StateName::MdxJsxTextEsWhitespaceInside)
        }
        _ => {
            if byte_to_kind(tokenizer.parse_state.bytes, tokenizer.point.index)
                == CharacterKind::Whitespace
            {
                tokenizer.consume();
                State::Next(StateName::MdxJsxTextEsWhitespaceInside)
            } else {
                tokenizer.exit(Name::MdxJsxTextEsWhitespace);
                State::Ok
            }
        }
    }
}

fn id_start(code: Option<char>) -> bool {
    if let Some(char) = code {
        UnicodeID::is_id_start(char) || matches!(char, '$' | '_')
    } else {
        false
    }
}

fn id_cont(code: Option<char>) -> bool {
    if let Some(char) = code {
        UnicodeID::is_id_continue(char) || matches!(char, '-' | '\u{200c}' | '\u{200d}')
    } else {
        false
    }
}

fn crash(tokenizer: &Tokenizer, at: &str, expect: &str) -> State {
    let char = char_after_index(tokenizer.parse_state.bytes, tokenizer.point.index);

    // To do: externalize this, and the print mechanism in the tokenizer,
    // to one proper formatter.
    let actual = match char {
        None => "end of file".to_string(),
        Some(char) => format!("character {}", format_char(char)),
    };

    State::Error(format!(
        "{}:{}: Unexpected {} {}, expected {}",
        tokenizer.point.line, tokenizer.point.column, actual, at, expect
    ))
}

fn format_char(char: char) -> String {
    let unicode = format!("U+{:>04X}", char as u32);
    let printable = match char {
        '`' => Some("`` ` ``".to_string()),
        ' '..='~' => Some(format!("`{}`", char)),
        _ => None,
    };

    if let Some(char) = printable {
        format!("{} ({})", char, unicode)
    } else {
        unicode
    }
}

fn format_byte(byte: u8) -> String {
    let unicode = format!("U+{:>04X}", byte);
    let printable = match byte {
        b'`' => Some("`` ` ``".to_string()),
        b' '..=b'~' => Some(format!("`{}`", str::from_utf8(&[byte]).unwrap())),
        _ => None,
    };

    if let Some(char) = printable {
        format!("{} ({})", char, unicode)
    } else {
        unicode
    }
}
