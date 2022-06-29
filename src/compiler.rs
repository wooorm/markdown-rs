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
use std::collections::HashMap;

/// Type of line endings in markdown.
#[derive(Debug, Clone, PartialEq)]
pub enum LineEnding {
    /// Both a carriage return (`\r`) and a line feed (`\n`).
    ///
    /// ## Example
    ///
    /// ```markdown
    /// a␍␊
    /// b
    /// ```
    CarriageReturnLineFeed,
    /// Sole carriage return (`\r`).
    ///
    /// ## Example
    ///
    /// ```markdown
    /// a␍
    /// b
    /// ```
    CarriageReturn,
    /// Sole line feed (`\n`).
    ///
    /// ## Example
    ///
    /// ```markdown
    /// a␊
    /// b
    /// ```
    LineFeed,
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

/// Representation of a link or image, resource or reference.
#[derive(Debug)]
struct Media {
    /// Whether this represents an image (`true`) or a link (`false`).
    image: bool,
    /// The text between the brackets (`x` in `![x]()` and `[x]()`), as an
    /// identifier, meaning that the original source characters are used
    /// instead of interpreting them.
    label_id: Option<String>,
    /// The text between the brackets (`x` in `![x]()` and `[x]()`), as
    /// content.
    /// When this is a link, it can contain further text content and thus HTML
    /// tags.
    /// Otherwise, when an image, text content is also allowed, but resulting
    /// tags are ignored.
    label: Option<String>,
    /// To do.
    // reference_id: String,
    /// The destination (url).
    /// Interpreted string content.
    destination: Option<String>,
    /// The destination (url).
    /// Interpreted string content.
    title: Option<String>,
}

/// To do.
#[derive(Debug, Clone, PartialEq)]
struct DefinitionInfo {
    id: Option<String>,
    destination: Option<String>,
    title: Option<String>,
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

/// To do.
type Handler = fn(&mut CompileContext, &Event);

/// To do.
type Map = HashMap<TokenType, Handler>;

/// To do.
struct CompileContext<'a> {
    /// Static info.
    pub events: &'a [Event],
    pub codes: &'a [Code],
    /// Fields used by handlers to track the things they need to track to
    /// compile markdown.
    pub atx_opening_sequence_size: Option<usize>,
    pub heading_setext_buffer: Option<String>,
    pub code_flow_seen_data: Option<bool>,
    pub code_fenced_fences_count: Option<usize>,
    pub character_reference_kind: Option<CharacterReferenceKind>,
    pub media_stack: Vec<Media>,
    /// Fields used to influance the current compilation.
    pub slurp_one_line_ending: bool,
    pub ignore_encode: bool,
    pub last_was_tag: bool,
    /// Configuration
    pub protocol_href: Option<Vec<&'static str>>,
    pub protocol_src: Option<Vec<&'static str>>,
    pub line_ending_default: LineEnding,
    pub allow_dangerous_html: bool,
    /// Data inferred about the document.
    // To do: definitions.
    /// Intermediate results.
    pub buffers: Vec<Vec<String>>,
    pub index: usize,
}

impl<'a> CompileContext<'a> {
    /// Create a new compile context.
    pub fn new(
        events: &'a [Event],
        codes: &'a [Code],
        options: &Options,
        line_ending: LineEnding,
    ) -> CompileContext<'a> {
        CompileContext {
            events,
            codes,
            atx_opening_sequence_size: None,
            heading_setext_buffer: None,
            code_flow_seen_data: None,
            code_fenced_fences_count: None,
            character_reference_kind: None,
            media_stack: vec![],
            slurp_one_line_ending: false,
            ignore_encode: false,
            last_was_tag: false,
            protocol_href: if options.allow_dangerous_protocol {
                None
            } else {
                Some(SAFE_PROTOCOL_HREF.to_vec())
            },
            protocol_src: if options.allow_dangerous_protocol {
                None
            } else {
                Some(SAFE_PROTOCOL_SRC.to_vec())
            },
            line_ending_default: line_ending,
            allow_dangerous_html: options.allow_dangerous_html,
            buffers: vec![vec![]],
            index: 0,
        }
    }
    /// Push a buffer.
    pub fn buffer(&mut self) {
        self.buffers.push(vec![]);
    }

    /// Pop a buffer, returning its value.
    pub fn resume(&mut self) -> String {
        self.buffers
            .pop()
            .expect("Cannot resume w/o buffer")
            .concat()
    }

    pub fn push(&mut self, value: String) {
        self.buffers
            .last_mut()
            .expect("Cannot push w/o buffer")
            .push(value);
    }

    /// Get the last chunk of current buffer.
    pub fn buf_tail_slice(&self) -> Option<&String> {
        self.buf_tail().last()
    }

    /// Get the current buffer.
    pub fn buf_tail(&self) -> &Vec<String> {
        self.buffers
            .last()
            .expect("at least one buffer should exist")
    }

    /// Get the mutable last chunk of current buffer.
    pub fn buf_tail_mut(&mut self) -> &mut Vec<String> {
        self.buffers
            .last_mut()
            .expect("at least one buffer should exist")
    }

    /// Optionally encode.
    pub fn encode_opt(&self, value: &str) -> String {
        if self.ignore_encode {
            value.to_string()
        } else {
            encode(value)
        }
    }

    /// Add a line ending.
    pub fn line_ending(&mut self) {
        let line_ending = self.line_ending_default.as_str().to_string();
        // lastWasTag = false
        self.push(line_ending);
    }

    /// Add a line ending if needed (as in, there’s no eol/eof already).
    pub fn line_ending_if_needed(&mut self) {
        let slice = self.buf_tail_slice();
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
            self.line_ending();
        }
    }
}

/// Turn events and codes into a string of HTML.
#[allow(clippy::too_many_lines)]
pub fn compile(events: &[Event], codes: &[Code], options: &Options) -> String {
    // let mut slurp_all_line_endings = false;
    let mut definition: Option<DefinitionInfo> = None;
    let mut index = 0;
    let mut line_ending_inferred: Option<LineEnding> = None;
    // To do: actually do a compile pass, so that `buffer`, `resume`, etc can be used.
    while index < events.len() {
        let event = &events[index];

        // Find the used line ending style.
        if line_ending_inferred.is_none()
            && event.event_type == EventType::Exit
            && (event.token_type == TokenType::BlankLineEnding
                || event.token_type == TokenType::CodeTextLineEnding
                || event.token_type == TokenType::LineEnding)
        {
            let codes = codes_from_span(codes, &from_exit_event(events, index));
            line_ending_inferred = Some(LineEnding::from_code(*codes.first().unwrap()));
        }

        if event.event_type == EventType::Enter {
            if event.token_type == TokenType::Definition {
                definition = Some(DefinitionInfo {
                    id: None,
                    destination: None,
                    title: None,
                });
            }
        } else if event.token_type == TokenType::Definition {
            definition = None;
        } else if event.token_type == TokenType::DefinitionLabelString
            || event.token_type == TokenType::DefinitionDestinationString
            || event.token_type == TokenType::DefinitionTitleString
        {
            let slice = serialize(codes, &from_exit_event(events, index), false);
            println!("set: {:?} {:?}", slice, definition);
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

    let mut enter_map: Map = HashMap::new();
    enter_map.insert(TokenType::CodeFencedFenceInfo, on_enter_buffer);
    enter_map.insert(TokenType::CodeFencedFenceMeta, on_enter_buffer);
    enter_map.insert(TokenType::Definition, on_enter_buffer);
    enter_map.insert(TokenType::HeadingAtxText, on_enter_buffer);
    enter_map.insert(TokenType::HeadingSetextText, on_enter_buffer);
    enter_map.insert(TokenType::Label, on_enter_buffer);
    enter_map.insert(TokenType::ResourceTitleString, on_enter_buffer);
    enter_map.insert(TokenType::CodeIndented, on_enter_code_indented);
    enter_map.insert(TokenType::CodeFenced, on_enter_code_fenced);
    enter_map.insert(TokenType::CodeText, on_enter_code_text);
    enter_map.insert(TokenType::HtmlFlow, on_enter_html_flow);
    enter_map.insert(TokenType::HtmlText, on_enter_html_text);
    enter_map.insert(TokenType::Image, on_enter_image);
    enter_map.insert(TokenType::Link, on_enter_link);
    enter_map.insert(TokenType::Resource, on_enter_resource);
    enter_map.insert(
        TokenType::ResourceDestinationString,
        on_enter_destination_string,
    );
    enter_map.insert(TokenType::Paragraph, on_enter_paragraph);

    let mut exit_map: Map = HashMap::new();
    exit_map.insert(TokenType::Label, on_exit_label);
    exit_map.insert(TokenType::LabelText, on_exit_label_text);
    exit_map.insert(
        TokenType::ResourceDestinationString,
        on_exit_resource_destination_string,
    );
    exit_map.insert(
        TokenType::ResourceTitleString,
        on_exit_resource_title_string,
    );
    exit_map.insert(TokenType::Image, on_exit_media);
    exit_map.insert(TokenType::Link, on_exit_media);
    exit_map.insert(TokenType::CodeTextData, on_exit_push);
    exit_map.insert(TokenType::Data, on_exit_push);
    exit_map.insert(TokenType::CharacterEscapeValue, on_exit_push);
    exit_map.insert(TokenType::AutolinkEmail, on_exit_autolink_email);
    exit_map.insert(TokenType::AutolinkProtocol, on_exit_autolink_protocol);
    exit_map.insert(
        TokenType::CharacterReferenceMarker,
        on_exit_character_reference_marker,
    );
    exit_map.insert(
        TokenType::CharacterReferenceMarkerNumeric,
        on_exit_character_reference_marker_numeric,
    );
    exit_map.insert(
        TokenType::CharacterReferenceMarkerHexadecimal,
        on_exit_character_reference_marker_hexadecimal,
    );
    exit_map.insert(
        TokenType::CharacterReferenceValue,
        on_exit_character_reference_value,
    );
    exit_map.insert(TokenType::CodeFenced, on_exit_code_flow);
    exit_map.insert(TokenType::CodeIndented, on_exit_code_flow);
    exit_map.insert(TokenType::CodeFencedFence, on_exit_code_fenced_fence);
    exit_map.insert(
        TokenType::CodeFencedFenceInfo,
        on_exit_code_fenced_fence_info,
    );
    exit_map.insert(TokenType::CodeFencedFenceMeta, on_exit_resume);
    exit_map.insert(TokenType::Resource, on_exit_resume);
    exit_map.insert(TokenType::CodeFlowChunk, on_exit_code_flow_chunk);
    exit_map.insert(TokenType::CodeText, on_exit_code_text);
    exit_map.insert(TokenType::CodeTextLineEnding, on_exit_code_text_line_ending);
    exit_map.insert(TokenType::Definition, on_exit_definition);
    exit_map.insert(TokenType::HardBreakEscape, on_exit_break);
    exit_map.insert(TokenType::HardBreakTrailing, on_exit_break);
    exit_map.insert(TokenType::HeadingAtx, on_exit_heading_atx);
    exit_map.insert(TokenType::HeadingAtxSequence, on_exit_heading_atx_sequence);
    exit_map.insert(TokenType::HeadingAtxText, on_exit_heading_atx_text);
    exit_map.insert(TokenType::HeadingSetextText, on_exit_heading_setext_text);
    exit_map.insert(
        TokenType::HeadingSetextUnderline,
        on_exit_heading_setext_underline,
    );
    exit_map.insert(TokenType::HtmlFlow, on_exit_html);
    exit_map.insert(TokenType::HtmlText, on_exit_html);
    exit_map.insert(TokenType::HtmlFlowData, on_exit_html_data);
    exit_map.insert(TokenType::HtmlTextData, on_exit_html_data);
    exit_map.insert(TokenType::LineEnding, on_exit_line_ending);
    exit_map.insert(TokenType::Paragraph, on_exit_paragraph);
    exit_map.insert(TokenType::ThematicBreak, on_exit_thematic_break);

    let mut index = 0;
    let mut context = CompileContext::new(events, codes, options, line_ending_default);

    while index < events.len() {
        let event = &events[index];
        context.index = index;

        let map = if event.event_type == EventType::Enter {
            &enter_map
        } else {
            &exit_map
        };
        if let Some(func) = map.get(&event.token_type) {
            func(&mut context, event);
        }

        index += 1;
    }

    assert!(context.buffers.len() == 1, "expected 1 final buffer");
    context
        .buffers
        .get(0)
        .expect("expected 1 final buffer")
        .concat()
}

fn on_enter_buffer(context: &mut CompileContext, _event: &Event) {
    context.buffer();
}

fn on_enter_code_indented(context: &mut CompileContext, _event: &Event) {
    context.code_flow_seen_data = Some(false);
    context.line_ending_if_needed();
    context.push("<pre><code>".to_string());
}

fn on_enter_code_fenced(context: &mut CompileContext, _event: &Event) {
    context.code_flow_seen_data = Some(false);
    context.line_ending_if_needed();
    // Note that no `>` is used, which is added later.
    context.push("<pre><code".to_string());
    context.code_fenced_fences_count = Some(0);
}

fn on_enter_code_text(context: &mut CompileContext, _event: &Event) {
    context.push("<code>".to_string());
    context.buffer();
}

fn on_enter_html_flow(context: &mut CompileContext, _event: &Event) {
    context.line_ending_if_needed();
    if context.allow_dangerous_html {
        context.ignore_encode = true;
    }
}

fn on_enter_html_text(context: &mut CompileContext, _event: &Event) {
    if context.allow_dangerous_html {
        context.ignore_encode = true;
    }
}

fn on_enter_image(context: &mut CompileContext, _event: &Event) {
    context.media_stack.push(Media {
        image: true,
        label_id: None,
        label: None,
        // reference_id: "".to_string(),
        destination: None,
        title: None,
    });
    // tags = undefined // Disallow tags.
}

fn on_enter_link(context: &mut CompileContext, _event: &Event) {
    context.media_stack.push(Media {
        image: false,
        label_id: None,
        label: None,
        // reference_id: "".to_string(),
        destination: None,
        title: None,
    });
}

fn on_enter_resource(context: &mut CompileContext, _event: &Event) {
    context.buffer(); // We can have line endings in the resource, ignore them.
    let media = context.media_stack.last_mut().unwrap();
    media.destination = Some("".to_string());
}

fn on_enter_destination_string(context: &mut CompileContext, _event: &Event) {
    context.buffer();
    // Ignore encoding the result, as we’ll first percent encode the url and
    // encode manually after.
    context.ignore_encode = true;
}

fn on_enter_paragraph(context: &mut CompileContext, _event: &Event) {
    context.buf_tail_mut().push("<p>".to_string());
}

fn on_exit_label(context: &mut CompileContext, _event: &Event) {
    let buf = context.resume();
    let media = context.media_stack.last_mut().unwrap();
    media.label = Some(buf);
}

fn on_exit_label_text(context: &mut CompileContext, _event: &Event) {
    let media = context.media_stack.last_mut().unwrap();
    media.label_id = Some(serialize(
        context.codes,
        &from_exit_event(context.events, context.index),
        false,
    ));
}

fn on_exit_resource_destination_string(context: &mut CompileContext, _event: &Event) {
    let buf = context.resume();
    let media = context.media_stack.last_mut().unwrap();
    media.destination = Some(buf);
    context.ignore_encode = false;
}

fn on_exit_resource_title_string(context: &mut CompileContext, _event: &Event) {
    let buf = context.resume();
    let media = context.media_stack.last_mut().unwrap();
    media.title = Some(buf);
}

fn on_exit_media(context: &mut CompileContext, _event: &Event) {
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

    let media = context.media_stack.pop().unwrap();
    println!("media: {:?}", media);
    let label = media.label.unwrap();
    // To do: get from definition.
    let destination = media.destination.unwrap_or_else(|| "".to_string());
    let title = if let Some(title) = media.title {
        format!(" title=\"{}\"", title)
    } else {
        "".to_string()
    };

    let result = if media.image {
        format!(
            "<img src=\"{}\" alt=\"{}\"{} />",
            sanitize_uri(&destination, &context.protocol_src),
            label,
            title
        )
    } else {
        format!(
            "<a href=\"{}\"{}>{}</a>",
            sanitize_uri(&destination, &context.protocol_href),
            title,
            label
        )
    };

    context.push(result);
}

fn on_exit_push(context: &mut CompileContext, _event: &Event) {
    // Just output it.
    // last_was_tag = false;
    context.push(context.encode_opt(&serialize(
        context.codes,
        &from_exit_event(context.events, context.index),
        false,
    )));
}

fn on_exit_autolink_email(context: &mut CompileContext, _event: &Event) {
    let slice = serialize(
        context.codes,
        &from_exit_event(context.events, context.index),
        false,
    );
    context.push(format!(
        "<a href=\"mailto:{}\">{}</a>",
        sanitize_uri(slice.as_str(), &context.protocol_href),
        context.encode_opt(&slice)
    ));
}

fn on_exit_autolink_protocol(context: &mut CompileContext, _event: &Event) {
    let slice = serialize(
        context.codes,
        &from_exit_event(context.events, context.index),
        false,
    );
    let href = sanitize_uri(slice.as_str(), &context.protocol_href);
    println!("xxx: {:?} {:?}", href, &context.protocol_href);
    context.push(format!(
        "<a href=\"{}\">{}</a>",
        href,
        context.encode_opt(&slice)
    ));
}

fn on_exit_character_reference_marker(context: &mut CompileContext, _event: &Event) {
    context.character_reference_kind = Some(CharacterReferenceKind::Named);
}

fn on_exit_character_reference_marker_numeric(context: &mut CompileContext, _event: &Event) {
    context.character_reference_kind = Some(CharacterReferenceKind::Decimal);
}

fn on_exit_character_reference_marker_hexadecimal(context: &mut CompileContext, _event: &Event) {
    context.character_reference_kind = Some(CharacterReferenceKind::Hexadecimal);
}

fn on_exit_character_reference_value(context: &mut CompileContext, _event: &Event) {
    let kind = context
        .character_reference_kind
        .take()
        .expect("expected `character_reference_kind` to be set");
    let reference = serialize(
        context.codes,
        &from_exit_event(context.events, context.index),
        false,
    );
    let ref_string = reference.as_str();
    let value = match kind {
        CharacterReferenceKind::Decimal => decode_numeric(ref_string, 10).to_string(),
        CharacterReferenceKind::Hexadecimal => decode_numeric(ref_string, 16).to_string(),
        CharacterReferenceKind::Named => decode_named(ref_string),
    };

    context.push(context.encode_opt(&value));
}

fn on_exit_code_flow(context: &mut CompileContext, _event: &Event) {
    let seen_data = context
        .code_flow_seen_data
        .take()
        .expect("`code_flow_seen_data` must be defined");

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
        context.line_ending_if_needed();
    }

    context.push("</code></pre>".to_string());

    if let Some(count) = context.code_fenced_fences_count.take() {
        if count < 2 {
            context.line_ending_if_needed();
        }
    }

    context.slurp_one_line_ending = false;
}

fn on_exit_code_fenced_fence(context: &mut CompileContext, _event: &Event) {
    let count = if let Some(count) = context.code_fenced_fences_count {
        count
    } else {
        0
    };

    if count == 0 {
        context.push(">".to_string());
        // tag = true;
        context.slurp_one_line_ending = true;
    }

    context.code_fenced_fences_count = Some(count + 1);
}

fn on_exit_code_fenced_fence_info(context: &mut CompileContext, _event: &Event) {
    let value = context.resume();
    context.push(format!(" class=\"language-{}\"", value));
    // tag = true;
}

fn on_exit_resume(context: &mut CompileContext, _event: &Event) {
    context.resume();
}

fn on_exit_code_flow_chunk(context: &mut CompileContext, _event: &Event) {
    context.code_flow_seen_data = Some(true);
    context.push(context.encode_opt(&serialize(
        context.codes,
        &from_exit_event(context.events, context.index),
        false,
    )));
}

fn on_exit_code_text(context: &mut CompileContext, _event: &Event) {
    let result = context.resume();
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

    context.push(if trim {
        result[1..(result.len() - 1)].to_string()
    } else {
        result
    });
    context.push("</code>".to_string());
}

fn on_exit_code_text_line_ending(context: &mut CompileContext, _event: &Event) {
    context.push(" ".to_string());
}

fn on_exit_definition(context: &mut CompileContext, _event: &Event) {
    context.resume();
    context.slurp_one_line_ending = true;
}

fn on_exit_break(context: &mut CompileContext, _event: &Event) {
    context.push("<br />".to_string());
}

fn on_exit_heading_atx(context: &mut CompileContext, _event: &Event) {
    let rank = context
        .atx_opening_sequence_size
        .take()
        .expect("`atx_opening_sequence_size` must be set in headings");

    context.push(format!("</h{}>", rank));
}

fn on_exit_heading_atx_sequence(context: &mut CompileContext, _event: &Event) {
    // First fence we see.
    if context.atx_opening_sequence_size.is_none() {
        let rank = serialize(
            context.codes,
            &from_exit_event(context.events, context.index),
            false,
        )
        .len();
        context.atx_opening_sequence_size = Some(rank);
        context.push(format!("<h{}>", rank));
    }
}

fn on_exit_heading_atx_text(context: &mut CompileContext, _event: &Event) {
    let value = context.resume();
    context.push(value);
}

fn on_exit_heading_setext_text(context: &mut CompileContext, _event: &Event) {
    let buf = context.resume();
    context.heading_setext_buffer = Some(buf);
    context.slurp_one_line_ending = true;
}

fn on_exit_heading_setext_underline(context: &mut CompileContext, _event: &Event) {
    let text = context
        .heading_setext_buffer
        .take()
        .expect("`atx_opening_sequence_size` must be set in headings");
    let head = codes_from_span(
        context.codes,
        &from_exit_event(context.events, context.index),
    )[0];
    let level: usize = if head == Code::Char('-') { 2 } else { 1 };

    context.push(format!("<h{}>{}</h{}>", level, text, level));
}

fn on_exit_html(context: &mut CompileContext, _event: &Event) {
    context.ignore_encode = false;
}

fn on_exit_html_data(context: &mut CompileContext, _event: &Event) {
    let slice = serialize(
        context.codes,
        &from_exit_event(context.events, context.index),
        false,
    );
    // last_was_tag = false;
    context.push(context.encode_opt(&slice));
}

fn on_exit_line_ending(context: &mut CompileContext, _event: &Event) {
    // if slurp_all_line_endings {
    //     // Empty.
    // } else
    if context.slurp_one_line_ending {
        context.slurp_one_line_ending = false;
    } else {
        context.push(context.encode_opt(&serialize(
            context.codes,
            &from_exit_event(context.events, context.index),
            false,
        )));
    }
}

fn on_exit_paragraph(context: &mut CompileContext, _event: &Event) {
    context.push("</p>".to_string());
}

fn on_exit_thematic_break(context: &mut CompileContext, _event: &Event) {
    context.push("<hr />".to_string());
}
