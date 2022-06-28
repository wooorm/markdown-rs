//! Turn events into a string of HTML.
use crate::constant::{SAFE_PROTOCOL_HREF, SAFE_PROTOCOL_SRC};
use crate::construct::character_reference::Kind as CharacterReferenceKind;
use crate::tokenizer::{Code, Event, EventType, TokenType};
use crate::util::{
    decode_character_reference::{decode_named, decode_numeric},
    encode::encode,
    sanitize_uri::sanitize_uri,
    span::{codes as codes_from_span, from_exit_event, serialize},
};

/// To do.
#[derive(Debug, Clone, PartialEq)]
pub enum LineEnding {
    CarriageReturnLineFeed,
    CarriageReturn,
    LineFeed,
}

/// To do.
#[derive(Debug)]
struct Media {
    /// To do.
    image: bool,
    /// To do.
    label_id: String,
    /// To do.
    label: String,
    /// To do.
    // reference_id: String,
    /// To do.
    destination: Option<String>,
    /// To do.
    title: Option<String>,
}

impl LineEnding {
    /// Turn the line ending into a [str].
    fn as_str(&self) -> &str {
        match self {
            LineEnding::CarriageReturnLineFeed => "\r\n",
            LineEnding::CarriageReturn => "\r",
            LineEnding::LineFeed => "\n",
        }
    }
    /// Turn a [Code] into a line ending.
    ///
    /// ## Panics
    ///
    /// Panics if `code` is not `\r\n`, `\r`, or `\n`.
    fn from_code(code: Code) -> LineEnding {
        match code {
            Code::CarriageReturnLineFeed => LineEnding::CarriageReturnLineFeed,
            Code::Char('\r') => LineEnding::CarriageReturn,
            Code::Char('\n') => LineEnding::LineFeed,
            _ => unreachable!("invalid code"),
        }
    }
}

/// Configuration (optional).
#[derive(Default, Debug)]
pub struct Options {
    /// Whether to allow (dangerous) HTML.
    /// The default is `false`, you can turn it on to `true` for trusted
    /// content.
    ///
    /// ## Examples
    ///
    /// ```rust
    /// use micromark::{micromark, micromark_with_options, Options};
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
    ///         &Options {
    ///             allow_dangerous_html: true,
    ///             allow_dangerous_protocol: false,
    ///             default_line_ending: None,
    ///
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
    /// use micromark::{micromark, micromark_with_options, Options};
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
    ///         &Options {
    ///             allow_dangerous_html: false,
    ///             allow_dangerous_protocol: true,
    ///             default_line_ending: None,
    ///         }
    ///     ),
    ///     "<p><a href=\"javascript:alert(1)\">javascript:alert(1)</a></p>"
    /// );
    /// ```
    pub allow_dangerous_protocol: bool,

    /// Default line ending to use, for line endings not in `value`.
    ///
    /// Generally, micromark copies line endings (`\r`, `\n`, `\r\n`) in the
    /// markdown document over to the compiled HTML.
    /// In some cases, such as `> a`, CommonMark requires that extra line
    /// endings are added: `<blockquote>\n<p>a</p>\n</blockquote>`.
    ///
    /// To create that line ending, the document is checked for the first line
    /// ending that is used.
    /// If there is no line ending, `default_line_ending` is used.
    /// If that isn’t configured, `\n` is used.
    ///
    /// ## Examples
    ///
    /// ```rust
    /// use micromark::{micromark, micromark_with_options, Options, LineEnding};
    ///
    /// // micromark is safe by default:
    /// assert_eq!(
    ///     micromark("> a"),
    ///     // To do: block quote
    ///     // "<blockquote>\n<p>a</p>\n</blockquote>"
    ///     "<p>&gt; a</p>"
    /// );
    ///
    /// // Define `default_line_ending` to configure the default:
    /// assert_eq!(
    ///     micromark_with_options(
    ///         "> a",
    ///         &Options {
    ///             allow_dangerous_html: false,
    ///             allow_dangerous_protocol: false,
    ///             default_line_ending: Some(LineEnding::CarriageReturnLineFeed),
    ///
    ///         }
    ///     ),
    ///     // To do: block quote
    ///     // "<blockquote>\r\n<p>a</p>\r\n</blockquote>"
    ///     "<p>&gt; a</p>"
    /// );
    /// ```
    pub default_line_ending: Option<LineEnding>,
}

/// Turn events and codes into a string of HTML.
#[allow(clippy::too_many_lines)]
pub fn compile(events: &[Event], codes: &[Code], options: &Options) -> String {
    let mut index = 0;
    // let mut last_was_tag = false;
    let buffers: &mut Vec<Vec<String>> = &mut vec![vec![]];
    let mut atx_opening_sequence_size: Option<usize> = None;
    let mut heading_setext_buffer: Option<String> = None;
    let mut code_flow_seen_data: Option<bool> = None;
    let mut code_fenced_fences_count: Option<usize> = None;
    let mut slurp_one_line_ending = false;
    let mut ignore_encode = false;
    let mut character_reference_kind: Option<CharacterReferenceKind> = None;
    let protocol_href = if options.allow_dangerous_protocol {
        None
    } else {
        Some(SAFE_PROTOCOL_HREF.to_vec())
    };
    let protocol_src = if options.allow_dangerous_protocol {
        None
    } else {
        Some(SAFE_PROTOCOL_SRC.to_vec())
    };
    let mut line_ending_inferred: Option<LineEnding> = None;
    let mut media_stack: Vec<Media> = vec![];

    // let mut slurp_all_line_endings = false;
    while index < events.len() {
        let event = &events[index];

        if event.event_type == EventType::Exit
            && (event.token_type == TokenType::BlankLineEnding
                || event.token_type == TokenType::CodeTextLineEnding
                || event.token_type == TokenType::LineEnding)
        {
            let codes = codes_from_span(codes, &from_exit_event(events, index));
            line_ending_inferred = Some(LineEnding::from_code(*codes.first().unwrap()));
            break;
        }

        index += 1;
    }

    let line_ending_default = if let Some(value) = line_ending_inferred {
        value
    } else if let Some(value) = &options.default_line_ending {
        value.clone()
    } else {
        LineEnding::LineFeed
    };

    index = 0;

    while index < events.len() {
        let event = &events[index];
        let token_type = &event.token_type;

        match event.event_type {
            EventType::Enter => match token_type {
                TokenType::Autolink
                | TokenType::AutolinkEmail
                | TokenType::AutolinkMarker
                | TokenType::AutolinkProtocol
                | TokenType::BlankLineEnding
                | TokenType::CharacterEscape
                | TokenType::CharacterEscapeMarker
                | TokenType::CharacterEscapeValue
                | TokenType::CharacterReference
                | TokenType::CharacterReferenceMarker
                | TokenType::CharacterReferenceMarkerHexadecimal
                | TokenType::CharacterReferenceMarkerNumeric
                | TokenType::CharacterReferenceMarkerSemi
                | TokenType::CharacterReferenceValue
                | TokenType::CodeFencedFence
                | TokenType::CodeFencedFenceSequence
                | TokenType::CodeFlowChunk
                | TokenType::CodeTextData
                | TokenType::CodeTextLineEnding
                | TokenType::CodeTextSequence
                | TokenType::Data
                | TokenType::DefinitionLabel
                | TokenType::DefinitionLabelMarker
                | TokenType::DefinitionLabelString
                | TokenType::DefinitionMarker
                | TokenType::DefinitionDestination
                | TokenType::DefinitionDestinationLiteral
                | TokenType::DefinitionDestinationLiteralMarker
                | TokenType::DefinitionDestinationRaw
                | TokenType::DefinitionDestinationString
                | TokenType::DefinitionTitle
                | TokenType::DefinitionTitleMarker
                | TokenType::DefinitionTitleString
                | TokenType::HardBreakEscape
                | TokenType::HardBreakEscapeMarker
                | TokenType::HardBreakTrailing
                | TokenType::HardBreakTrailingSpace
                | TokenType::HeadingAtx
                | TokenType::HeadingAtxSequence
                | TokenType::HeadingSetext
                | TokenType::HeadingSetextUnderline
                | TokenType::HtmlFlowData
                | TokenType::HtmlTextData
                | TokenType::LineEnding
                | TokenType::ThematicBreak
                | TokenType::ThematicBreakSequence
                | TokenType::SpaceOrTab => {
                    // Ignore.
                }
                TokenType::CodeFencedFenceInfo
                | TokenType::CodeFencedFenceMeta
                | TokenType::Definition
                | TokenType::HeadingAtxText
                | TokenType::HeadingSetextText
                | TokenType::Label
                | TokenType::ResourceTitleString => {
                    buffer(buffers);
                }
                TokenType::CodeIndented => {
                    code_flow_seen_data = Some(false);
                    line_ending_if_needed(buffers, &line_ending_default);
                    buf_tail_mut(buffers).push("<pre><code>".to_string());
                }
                TokenType::CodeFenced => {
                    code_flow_seen_data = Some(false);
                    line_ending_if_needed(buffers, &line_ending_default);
                    // Note that no `>` is used, which is added later.
                    buf_tail_mut(buffers).push("<pre><code".to_string());
                    code_fenced_fences_count = Some(0);
                }
                TokenType::CodeText => {
                    buf_tail_mut(buffers).push("<code>".to_string());
                    buffer(buffers);
                }
                TokenType::HtmlFlow => {
                    line_ending_if_needed(buffers, &line_ending_default);
                    if options.allow_dangerous_html {
                        ignore_encode = true;
                    }
                }
                TokenType::HtmlText => {
                    if options.allow_dangerous_html {
                        ignore_encode = true;
                    }
                }
                TokenType::Image => {
                    media_stack.push(Media {
                        image: true,
                        label_id: "".to_string(),
                        label: "".to_string(),
                        // reference_id: "".to_string(),
                        destination: None,
                        title: None,
                    });
                    // tags = undefined // Disallow tags.
                }
                TokenType::Link => {
                    media_stack.push(Media {
                        image: false,
                        label_id: "".to_string(),
                        label: "".to_string(),
                        // reference_id: "".to_string(),
                        destination: None,
                        title: None,
                    });
                }
                TokenType::Resource => {
                    buffer(buffers); // We can have line endings in the resource, ignore them.
                    let media = media_stack.last_mut().unwrap();
                    media.destination = Some("".to_string());
                }
                TokenType::ResourceDestinationString => {
                    buffer(buffers);
                    // Ignore encoding the result, as we’ll first percent encode the url and
                    // encode manually after.
                    ignore_encode = true;
                }
                TokenType::LabelImage
                | TokenType::LabelImageMarker
                | TokenType::LabelLink
                | TokenType::LabelMarker
                | TokenType::LabelEnd
                | TokenType::ResourceMarker
                | TokenType::ResourceDestination
                | TokenType::ResourceDestinationLiteral
                | TokenType::ResourceDestinationLiteralMarker
                | TokenType::ResourceDestinationRaw
                | TokenType::ResourceTitle
                | TokenType::ResourceTitleMarker
                | TokenType::Reference
                | TokenType::ReferenceMarker
                | TokenType::ReferenceString
                | TokenType::LabelText => {
                    println!("ignore labels for now");
                }
                TokenType::Paragraph => {
                    buf_tail_mut(buffers).push("<p>".to_string());
                }
                #[allow(unreachable_patterns)]
                _ => {
                    unreachable!("unhandled `enter` of TokenType {:?}", token_type)
                }
            },
            EventType::Exit => match token_type {
                TokenType::Autolink
                | TokenType::AutolinkMarker
                | TokenType::BlankLineEnding
                | TokenType::CharacterEscape
                | TokenType::CharacterEscapeMarker
                | TokenType::CharacterReference
                | TokenType::CharacterReferenceMarkerSemi
                | TokenType::CodeFencedFenceSequence
                | TokenType::CodeTextSequence
                | TokenType::DefinitionLabel
                | TokenType::DefinitionLabelMarker
                | TokenType::DefinitionLabelString
                | TokenType::DefinitionMarker
                | TokenType::DefinitionDestination
                | TokenType::DefinitionDestinationLiteral
                | TokenType::DefinitionDestinationLiteralMarker
                | TokenType::DefinitionDestinationRaw
                | TokenType::DefinitionDestinationString
                | TokenType::DefinitionTitle
                | TokenType::DefinitionTitleMarker
                | TokenType::DefinitionTitleString
                | TokenType::HardBreakEscapeMarker
                | TokenType::HardBreakTrailingSpace
                | TokenType::HeadingSetext
                | TokenType::ThematicBreakSequence
                | TokenType::SpaceOrTab => {
                    // Ignore.
                }
                TokenType::LabelImage
                | TokenType::LabelImageMarker
                | TokenType::LabelLink
                | TokenType::LabelMarker
                | TokenType::LabelEnd
                | TokenType::ResourceMarker
                | TokenType::ResourceDestination
                | TokenType::ResourceDestinationLiteral
                | TokenType::ResourceDestinationLiteralMarker
                | TokenType::ResourceDestinationRaw
                | TokenType::ResourceTitle
                | TokenType::ResourceTitleMarker
                | TokenType::Reference
                | TokenType::ReferenceMarker
                | TokenType::ReferenceString => {
                    println!("ignore labels for now");
                }
                TokenType::Label => {
                    let media = media_stack.last_mut().unwrap();
                    media.label = resume(buffers);
                }
                TokenType::LabelText => {
                    let media = media_stack.last_mut().unwrap();
                    media.label_id = serialize(codes, &from_exit_event(events, index), false);
                }
                TokenType::ResourceDestinationString => {
                    let media = media_stack.last_mut().unwrap();
                    media.destination = Some(resume(buffers));
                    ignore_encode = false;
                }
                TokenType::ResourceTitleString => {
                    let media = media_stack.last_mut().unwrap();
                    media.title = Some(resume(buffers));
                }
                TokenType::Image | TokenType::Link => {
                    // let mut is_in_image = false;
                    // let mut index = 0;
                    // Skip current.
                    // while index < (media_stack.len() - 1) {
                    //     if media_stack[index].image {
                    //         is_in_image = true;
                    //         break;
                    //     }
                    //     index += 1;
                    // }

                    // tags = is_in_image;

                    let media = media_stack.pop().unwrap();
                    println!("media: {:?}", media);
                    let buf = buf_tail_mut(buffers);
                    // To do: get from definition.
                    let destination = media.destination.unwrap();
                    let title = if let Some(title) = media.title {
                        format!(" title=\"{}\"", title)
                    } else {
                        "".to_string()
                    };

                    if media.image {
                        buf.push(format!(
                            "<img src=\"{}\" alt=\"{}\"{} />",
                            sanitize_uri(&destination, &protocol_src),
                            media.label,
                            title
                        ));
                    } else {
                        buf.push(format!(
                            "<a href=\"{}\"{}>{}</a>",
                            sanitize_uri(&destination, &protocol_href),
                            title,
                            media.label
                        ));
                    }
                }
                // Just output it.
                TokenType::CodeTextData | TokenType::Data | TokenType::CharacterEscapeValue => {
                    // last_was_tag = false;
                    buf_tail_mut(buffers).push(encode_opt(
                        &serialize(codes, &from_exit_event(events, index), false),
                        ignore_encode,
                    ));
                }
                TokenType::AutolinkEmail => {
                    let slice = serialize(codes, &from_exit_event(events, index), false);
                    let buf = buf_tail_mut(buffers);
                    buf.push(format!(
                        "<a href=\"mailto:{}\">",
                        sanitize_uri(slice.as_str(), &protocol_href)
                    ));
                    buf.push(encode_opt(&slice, ignore_encode));
                    buf.push("</a>".to_string());
                }
                TokenType::AutolinkProtocol => {
                    let slice = serialize(codes, &from_exit_event(events, index), false);
                    let buf = buf_tail_mut(buffers);
                    buf.push(format!(
                        "<a href=\"{}\">",
                        sanitize_uri(slice.as_str(), &protocol_href)
                    ));
                    buf.push(encode_opt(&slice, ignore_encode));
                    buf.push("</a>".to_string());
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

                    buf_tail_mut(buffers).push(encode_opt(&value, ignore_encode));
                    character_reference_kind = None;
                }
                TokenType::CodeFenced | TokenType::CodeIndented => {
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
                        line_ending_if_needed(buffers, &line_ending_default);
                    }

                    buf_tail_mut(buffers).push("</code></pre>".to_string());

                    if let Some(count) = code_fenced_fences_count {
                        if count < 2 {
                            line_ending_if_needed(buffers, &line_ending_default);
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
                TokenType::CodeFencedFenceMeta | TokenType::Resource => {
                    resume(buffers);
                }
                TokenType::CodeFlowChunk => {
                    code_flow_seen_data = Some(true);
                    buf_tail_mut(buffers).push(encode_opt(
                        &serialize(codes, &from_exit_event(events, index), false),
                        ignore_encode,
                    ));
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
                TokenType::Definition => {
                    resume(buffers);
                    slurp_one_line_ending = true;
                }
                TokenType::HardBreakEscape | TokenType::HardBreakTrailing => {
                    buf_tail_mut(buffers).push("<br />".to_string());
                }
                TokenType::HeadingAtx => {
                    let rank = atx_opening_sequence_size
                        .expect("`atx_opening_sequence_size` must be set in headings");
                    buf_tail_mut(buffers).push(format!("</h{}>", rank));
                    atx_opening_sequence_size = None;
                }
                TokenType::HeadingAtxSequence => {
                    // First fence we see.
                    if None == atx_opening_sequence_size {
                        let rank = serialize(codes, &from_exit_event(events, index), false).len();
                        atx_opening_sequence_size = Some(rank);
                        buf_tail_mut(buffers).push(format!("<h{}>", rank));
                    }
                }
                TokenType::HeadingAtxText => {
                    let value = resume(buffers);
                    buf_tail_mut(buffers).push(value);
                }
                TokenType::HeadingSetextText => {
                    heading_setext_buffer = Some(resume(buffers));
                    slurp_one_line_ending = true;
                }
                TokenType::HeadingSetextUnderline => {
                    let text = heading_setext_buffer
                        .expect("`atx_opening_sequence_size` must be set in headings");
                    let head = codes_from_span(codes, &from_exit_event(events, index))[0];
                    let level: usize = if head == Code::Char('-') { 2 } else { 1 };

                    heading_setext_buffer = None;
                    buf_tail_mut(buffers).push(format!("<h{}>{}</h{}>", level, text, level));
                }
                TokenType::HtmlFlow | TokenType::HtmlText => {
                    ignore_encode = false;
                }
                TokenType::HtmlFlowData | TokenType::HtmlTextData => {
                    let slice = serialize(codes, &from_exit_event(events, index), false);
                    // last_was_tag = false;
                    buf_tail_mut(buffers).push(encode_opt(&slice, ignore_encode));
                }
                TokenType::LineEnding => {
                    // if slurp_all_line_endings {
                    //     // Empty.
                    // } else
                    if slurp_one_line_ending {
                        slurp_one_line_ending = false;
                    } else {
                        buf_tail_mut(buffers).push(encode_opt(
                            &serialize(codes, &from_exit_event(events, index), false),
                            ignore_encode,
                        ));
                    }
                }
                TokenType::Paragraph => {
                    buf_tail_mut(buffers).push("</p>".to_string());
                }
                TokenType::ThematicBreak => {
                    buf_tail_mut(buffers).push("<hr />".to_string());
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

/// To do.
fn encode_opt(value: &str, ignore_encode: bool) -> String {
    if ignore_encode {
        value.to_string()
    } else {
        encode(value)
    }
}

/// Add a line ending.
fn line_ending(buffers: &mut [Vec<String>], default: &LineEnding) {
    let tail = buf_tail_mut(buffers);
    // lastWasTag = false
    tail.push(default.as_str().to_string());
}

/// Add a line ending if needed (as in, there’s no eol/eof already).
fn line_ending_if_needed(buffers: &mut [Vec<String>], default: &LineEnding) {
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
        line_ending(buffers, default);
    }
}
