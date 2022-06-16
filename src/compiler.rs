//! Turn events into a string of HTML.
use crate::construct::character_reference::Kind as CharacterReferenceKind;
use crate::tokenizer::{Code, Event, EventType, TokenType};
use crate::util::{
    decode_character_reference::{decode_named, decode_numeric},
    encode::encode,
    sanitize_uri::sanitize_uri,
    span::{from_exit_event, serialize},
};

/// Configuration (optional).
#[derive(Default, Debug)]
pub struct CompileOptions {
    /// Whether to allow (dangerous) HTML.
    /// The default is `false`, you can turn it on to `true` for trusted
    /// content.
    ///
    /// ## Examples
    ///
    /// ```rust
    /// use micromark::{micromark, micromark_with_options, CompileOptions};
    ///
    /// // micromark is safe by default:
    /// assert_eq!(
    ///     micromark("Hi, <i>venus</i>!"),
    ///     "<p>Hi, &lt;i&gt;venus&lt;/i&gt;!</p>"
    /// );
    ///
    /// // Turn `allow_dangerous_html` on to allow potentially dangerous HTML:
    /// assert_eq!(
    ///     micromark_with_options(
    ///         "Hi, <i>venus</i>!",
    ///         &CompileOptions {
    ///             allow_dangerous_html: true,
    ///             allow_dangerous_protocol: false,
    ///         }
    ///     ),
    ///     "<p>Hi, <i>venus</i>!</p>"
    /// );
    /// ```
    pub allow_dangerous_html: bool,

    /// Whether to allow (dangerous) protocols in links and images.
    /// The default is `false`, you can turn it on to `true` for trusted
    /// content.
    ///
    /// ## Examples
    ///
    /// ```rust
    /// use micromark::{micromark, micromark_with_options, CompileOptions};
    ///
    /// // micromark is safe by default:
    /// assert_eq!(
    ///     micromark("<javascript:alert(1)>"),
    ///     "<p><a href=\"\">javascript:alert(1)</a></p>"
    /// );
    ///
    /// // Turn `allow_dangerous_protocol` on to allow potentially dangerous protocols:
    /// assert_eq!(
    ///     micromark_with_options(
    ///         "<javascript:alert(1)>",
    ///         &CompileOptions {
    ///             allow_dangerous_html: false,
    ///             allow_dangerous_protocol: true,
    ///         }
    ///     ),
    ///     "<p><a href=\"javascript:alert(1)\">javascript:alert(1)</a></p>"
    /// );
    /// ```
    pub allow_dangerous_protocol: bool,
}

/// Turn events and codes into a string of HTML.
#[allow(clippy::too_many_lines)]
pub fn compile(events: &[Event], codes: &[Code], options: &CompileOptions) -> String {
    let mut index = 0;
    // let mut last_was_tag = false;
    let buffers: &mut Vec<Vec<String>> = &mut vec![vec![]];
    let mut atx_opening_sequence_size: Option<usize> = None;
    let mut atx_heading_buffer: Option<String> = None;
    let mut code_flow_seen_data: Option<bool> = None;
    let mut code_fenced_fences_count: Option<usize> = None;
    let mut slurp_one_line_ending = false;
    let mut ignore_encode = false;
    let mut character_reference_kind: Option<CharacterReferenceKind> = None;
    let protocol_href = if options.allow_dangerous_protocol {
        None
    } else {
        Some(vec!["http", "https", "irc", "ircs", "mailto", "xmpp"])
    };
    // let protocol_src = if options.allow_dangerous_protocol {
    //     None
    // } else {
    //     Some(vec!["http", "https"])
    // };

    // let mut slurp_all_line_endings = false;

    while index < events.len() {
        let event = &events[index];
        let token_type = &event.token_type;

        match event.event_type {
            EventType::Enter => match token_type {
                TokenType::Paragraph => {
                    buf_tail_mut(buffers).push("<p>".to_string());
                }
                TokenType::CodeIndented => {
                    code_flow_seen_data = Some(false);
                    line_ending_if_needed(buffers);
                    buf_tail_mut(buffers).push("<pre><code>".to_string());
                }
                TokenType::CodeFenced => {
                    code_flow_seen_data = Some(false);
                    line_ending_if_needed(buffers);
                    // Note: no `>`, which is added later.
                    buf_tail_mut(buffers).push("<pre><code".to_string());
                    code_fenced_fences_count = Some(0);
                }
                TokenType::AtxHeadingText
                | TokenType::CodeFencedFenceInfo
                | TokenType::CodeFencedFenceMeta => {
                    buffer(buffers);
                }
                TokenType::HtmlFlow => {
                    line_ending_if_needed(buffers);
                    if options.allow_dangerous_html {
                        ignore_encode = true;
                    }
                }
                TokenType::HtmlText => {
                    if options.allow_dangerous_html {
                        ignore_encode = true;
                    }
                }
                TokenType::CodeText => {
                    buf_tail_mut(buffers).push("<code>".to_string());
                    buffer(buffers);
                }
                TokenType::Content
                | TokenType::AtxHeading
                | TokenType::AtxHeadingSequence
                | TokenType::AtxHeadingWhitespace
                | TokenType::LineEnding
                | TokenType::ThematicBreak
                | TokenType::ThematicBreakSequence
                | TokenType::ThematicBreakWhitespace
                | TokenType::CodeIndentedPrefixWhitespace
                | TokenType::CodeFlowChunk
                | TokenType::BlankLineEnding
                | TokenType::BlankLineWhitespace
                | TokenType::Whitespace
                | TokenType::HardBreakEscape
                | TokenType::HardBreakEscapeMarker
                | TokenType::HardBreakTrailing
                | TokenType::HardBreakTrailingSpace
                | TokenType::HtmlFlowData
                | TokenType::HtmlTextData
                | TokenType::CodeFencedFence
                | TokenType::CodeFencedFenceSequence
                | TokenType::CodeFencedFenceWhitespace
                | TokenType::CodeTextSequence
                | TokenType::CodeTextData
                | TokenType::CodeTextLineEnding
                | TokenType::Data
                | TokenType::CharacterEscape
                | TokenType::CharacterEscapeMarker
                | TokenType::CharacterEscapeValue
                | TokenType::CharacterReference
                | TokenType::CharacterReferenceMarker
                | TokenType::CharacterReferenceMarkerNumeric
                | TokenType::CharacterReferenceMarkerHexadecimal
                | TokenType::CharacterReferenceMarkerSemi
                | TokenType::CharacterReferenceValue
                | TokenType::Autolink
                | TokenType::AutolinkMarker
                | TokenType::AutolinkProtocol
                | TokenType::AutolinkEmail => {}
                #[allow(unreachable_patterns)]
                _ => {
                    unreachable!("unhandled `enter` of TokenType {:?}", token_type)
                }
            },
            EventType::Exit => match token_type {
                TokenType::Content
                | TokenType::ThematicBreakSequence
                | TokenType::ThematicBreakWhitespace
                | TokenType::CodeIndentedPrefixWhitespace
                | TokenType::BlankLineEnding
                | TokenType::BlankLineWhitespace
                | TokenType::Whitespace
                | TokenType::CodeFencedFenceSequence
                | TokenType::CodeFencedFenceWhitespace
                | TokenType::CodeTextSequence
                | TokenType::CharacterEscape
                | TokenType::CharacterEscapeMarker
                | TokenType::CharacterReference
                | TokenType::CharacterReferenceMarkerSemi
                | TokenType::HardBreakEscapeMarker
                | TokenType::HardBreakTrailingSpace
                | TokenType::Autolink
                | TokenType::AutolinkMarker => {}
                TokenType::HtmlFlow | TokenType::HtmlText => {
                    ignore_encode = false;
                }
                TokenType::HtmlFlowData | TokenType::HtmlTextData => {
                    let slice = serialize(codes, &from_exit_event(events, index), false);

                    let res = if ignore_encode { slice } else { encode(&slice) };

                    // last_was_tag = false;
                    buf_tail_mut(buffers).push(res);
                }
                TokenType::Paragraph => {
                    buf_tail_mut(buffers).push("</p>".to_string());
                }
                TokenType::HardBreakEscape | TokenType::HardBreakTrailing => {
                    buf_tail_mut(buffers).push("<br />".to_string());
                }
                TokenType::CodeIndented | TokenType::CodeFenced => {
                    let seen_data =
                        code_flow_seen_data.expect("`code_flow_seen_data` must be defined");

                    // To do: containers.
                    // One special case is if we are inside a container, and the fenced code was
                    // not closed (meaning it runs to the end).
                    // In that case, the following line ending, is considered *outside* the
                    // fenced code and block quote by micromark, but CM wants to treat that
                    // ending as part of the code.
                    // if fenced_count != None && fenced_count < 2 && tightStack.length > 0 && !last_was_tag {
                    //     line_ending();
                    // }

                    // But in most cases, it’s simpler: when we’ve seen some data, emit an extra
                    // line ending when needed.
                    if seen_data {
                        line_ending_if_needed(buffers);
                    }

                    buf_tail_mut(buffers).push("</code></pre>".to_string());

                    if let Some(count) = code_fenced_fences_count {
                        if count < 2 {
                            line_ending_if_needed(buffers);
                        }
                    }

                    code_flow_seen_data = None;
                    code_fenced_fences_count = None;
                    slurp_one_line_ending = false;
                }
                TokenType::CodeFencedFence => {
                    let count = if let Some(count) = code_fenced_fences_count {
                        count
                    } else {
                        0
                    };

                    if count == 0 {
                        buf_tail_mut(buffers).push(">".to_string());
                        // tag = true;
                        slurp_one_line_ending = true;
                    }

                    code_fenced_fences_count = Some(count + 1);
                }
                TokenType::CodeFencedFenceInfo => {
                    let value = resume(buffers);
                    buf_tail_mut(buffers).push(format!(" class=\"language-{}\"", value));
                    // tag = true;
                }
                TokenType::CodeFencedFenceMeta => {
                    resume(buffers);
                }
                TokenType::CodeFlowChunk => {
                    code_flow_seen_data = Some(true);
                    buf_tail_mut(buffers).push(encode(&serialize(
                        codes,
                        &from_exit_event(events, index),
                        false,
                    )));
                }

                // `AtxHeadingWhitespace` is ignored after the opening sequence,
                // before the closing sequence, and after the closing sequence.
                // But it is used around intermediate sequences.
                // `atx_heading_buffer` is set to `Some` by the first `AtxHeadingText`.
                // `AtxHeadingSequence` is ignored as the opening and closing sequence,
                // but not when intermediate.
                TokenType::AtxHeadingWhitespace | TokenType::AtxHeadingSequence => {
                    if let Some(buf) = atx_heading_buffer {
                        atx_heading_buffer = Some(
                            buf.to_string()
                                + &encode(&serialize(
                                    codes,
                                    &from_exit_event(events, index),
                                    false,
                                )),
                        );
                    }

                    // First fence we see.
                    if None == atx_opening_sequence_size {
                        let rank = serialize(codes, &from_exit_event(events, index), false).len();
                        atx_opening_sequence_size = Some(rank);
                        buf_tail_mut(buffers).push(format!("<h{}>", rank));
                    }
                }
                TokenType::AtxHeadingText => {
                    let result = resume(buffers);

                    if let Some(ref buf) = atx_heading_buffer {
                        if !buf.is_empty() {
                            buf_tail_mut(buffers).push(encode(buf));
                            atx_heading_buffer = Some("".to_string());
                        }
                    } else {
                        atx_heading_buffer = Some("".to_string());
                    }

                    buf_tail_mut(buffers).push(encode(&result));
                }
                TokenType::AtxHeading => {
                    let rank = atx_opening_sequence_size
                        .expect("`atx_opening_sequence_size` must be set in headings");
                    buf_tail_mut(buffers).push(format!("</h{}>", rank));
                    atx_opening_sequence_size = None;
                    atx_heading_buffer = None;
                }
                TokenType::AutolinkProtocol => {
                    let slice = serialize(codes, &from_exit_event(events, index), false);
                    let buf = buf_tail_mut(buffers);
                    buf.push(format!(
                        "<a href=\"{}\">",
                        sanitize_uri(slice.as_str(), &protocol_href)
                    ));
                    buf.push(encode(&slice));
                    buf.push("</a>".to_string());
                }
                TokenType::AutolinkEmail => {
                    let slice = serialize(codes, &from_exit_event(events, index), false);
                    let buf = buf_tail_mut(buffers);
                    buf.push(format!(
                        "<a href=\"mailto:{}\">",
                        sanitize_uri(slice.as_str(), &protocol_href)
                    ));
                    buf.push(encode(&slice));
                    buf.push("</a>".to_string());
                }
                TokenType::ThematicBreak => {
                    buf_tail_mut(buffers).push("<hr />".to_string());
                }
                TokenType::LineEnding => {
                    // if slurp_all_line_endings {
                    //     // Empty.
                    // } else
                    if slurp_one_line_ending {
                        slurp_one_line_ending = false;
                    } else {
                        buf_tail_mut(buffers).push(encode(&serialize(
                            codes,
                            &from_exit_event(events, index),
                            false,
                        )));
                    }
                }
                TokenType::CharacterReferenceMarker => {
                    character_reference_kind = Some(CharacterReferenceKind::Named);
                }
                TokenType::CharacterReferenceMarkerNumeric => {
                    character_reference_kind = Some(CharacterReferenceKind::Decimal);
                }
                TokenType::CharacterReferenceMarkerHexadecimal => {
                    character_reference_kind = Some(CharacterReferenceKind::Hexadecimal);
                }
                TokenType::CharacterReferenceValue => {
                    let kind = character_reference_kind
                        .expect("expected `character_reference_kind` to be set");
                    let reference = serialize(codes, &from_exit_event(events, index), false);
                    let ref_string = reference.as_str();
                    let value = match kind {
                        CharacterReferenceKind::Decimal => {
                            decode_numeric(ref_string, 10).to_string()
                        }
                        CharacterReferenceKind::Hexadecimal => {
                            decode_numeric(ref_string, 16).to_string()
                        }
                        CharacterReferenceKind::Named => decode_named(ref_string),
                    };

                    buf_tail_mut(buffers).push(encode(&value));

                    character_reference_kind = None;
                }
                TokenType::CodeText => {
                    let result = resume(buffers);
                    let mut chars = result.chars();
                    let mut trim = false;

                    if Some(' ') == chars.next() && Some(' ') == chars.next_back() {
                        let mut next = chars.next();
                        while next != None && !trim {
                            if Some(' ') != next {
                                trim = true;
                            }
                            next = chars.next();
                        }
                    }

                    buf_tail_mut(buffers).push(if trim {
                        result[1..(result.len() - 1)].to_string()
                    } else {
                        result
                    });
                    buf_tail_mut(buffers).push("</code>".to_string());
                }
                TokenType::CodeTextLineEnding => {
                    buf_tail_mut(buffers).push(" ".to_string());
                }
                // This branch below currently acts as the resulting `data` tokens.
                TokenType::CodeTextData | TokenType::Data | TokenType::CharacterEscapeValue => {
                    // last_was_tag = false;
                    buf_tail_mut(buffers).push(encode(&serialize(
                        codes,
                        &from_exit_event(events, index),
                        false,
                    )));
                }
                #[allow(unreachable_patterns)]
                _ => {
                    unreachable!("unhandled `exit` of TokenType {:?}", token_type)
                }
            },
        }

        index += 1;
    }

    assert!(buffers.len() == 1, "expected 1 final buffer");
    buffers.get(0).expect("expected 1 final buffer").concat()
}

/// Push a buffer.
fn buffer(buffers: &mut Vec<Vec<String>>) {
    buffers.push(vec![]);
}

/// Pop a buffer, returning its value.
fn resume(buffers: &mut Vec<Vec<String>>) -> String {
    let buf = buffers.pop().expect("Cannot resume w/o buffer");
    buf.concat()
}

/// Get the last chunk of current buffer.
fn buf_tail_slice(buffers: &mut [Vec<String>]) -> Option<&String> {
    let tail = buf_tail(buffers);
    tail.last()
}

/// Get the mutable last chunk of current buffer.
fn buf_tail_mut(buffers: &mut [Vec<String>]) -> &mut Vec<String> {
    buffers
        .last_mut()
        .expect("at least one buffer should exist")
}

/// Get the current buffer.
fn buf_tail(buffers: &mut [Vec<String>]) -> &Vec<String> {
    buffers.last().expect("at least one buffer should exist")
}

/// Add a line ending.
fn line_ending(buffers: &mut [Vec<String>]) {
    let tail = buf_tail_mut(buffers);
    // To do: use inferred line ending style.
    // lastWasTag = false
    tail.push("\n".to_string());
}

/// Add a line ending if needed (as in, there’s no eol/eof already).
fn line_ending_if_needed(buffers: &mut [Vec<String>]) {
    let slice = buf_tail_slice(buffers);
    let last_char = if let Some(x) = slice {
        x.chars().last()
    } else {
        None
    };
    let mut add = true;

    if let Some(x) = last_char {
        if x == '\n' || x == '\r' {
            add = false;
        }
    } else {
        add = false;
    }

    if add {
        line_ending(buffers);
    }
}
