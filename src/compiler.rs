//! Turn events into a string of HTML.
use crate::constant::{SAFE_PROTOCOL_HREF, SAFE_PROTOCOL_SRC};
use crate::construct::character_reference::Kind as CharacterReferenceKind;
use crate::token::Token;
use crate::tokenizer::{Code, Event, EventType};
use crate::util::normalize_identifier::normalize_identifier;
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
/// Reused for temporary definitions as well, in the first pass.
#[derive(Debug)]
struct Media {
    /// Whether this represents an image (`true`) or a link or definition
    /// (`false`).
    image: bool,
    /// The text between the brackets (`x` in `![x]()` and `[x]()`), as an
    /// identifier, meaning that the original source characters are used
    /// instead of interpreting them.
    /// Not interpreted.
    label_id: Option<String>,
    /// The text between the brackets (`x` in `![x]()` and `[x]()`), as
    /// interpreted content.
    /// When this is a link, it can contain further text content and thus HTML
    /// tags.
    /// Otherwise, when an image, text content is also allowed, but resulting
    /// tags are ignored.
    label: Option<String>,
    /// The text between the explicit brackets of the reference (`y` in
    /// `[x][y]`), as content.
    /// Not interpreted.
    reference_id: Option<String>,
    /// The destination (url).
    /// Interpreted string content.
    destination: Option<String>,
    /// The destination (url).
    /// Interpreted string content.
    title: Option<String>,
}

/// Representation of a definition.
#[derive(Debug)]
struct Definition {
    /// The destination (url).
    /// Interpreted string content.
    destination: Option<String>,
    /// The title.
    /// Interpreted string content.
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
    ///     "<blockquote>\n<p>a</p>\n</blockquote>"
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
    ///     "<blockquote>\r\n<p>a</p>\r\n</blockquote>"
    /// );
    /// ```
    pub default_line_ending: Option<LineEnding>,
}

/// Handle an event.
///
/// The current event is available at `context.events[context.index]`.
type Handle = fn(&mut CompileContext);

/// Map of [`Token`][] to [`Handle`][].
type Map = HashMap<Token, Handle>;

/// Context used to compile markdown.
#[allow(clippy::struct_excessive_bools)]
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
    pub expect_first_item: Option<bool>,
    pub media_stack: Vec<Media>,
    pub definitions: HashMap<String, Definition>,
    pub tight_stack: Vec<bool>,
    /// Fields used to influance the current compilation.
    pub slurp_one_line_ending: bool,
    pub tags: bool,
    pub ignore_encode: bool,
    pub last_was_tag: bool,
    /// Configuration
    pub protocol_href: Option<Vec<&'static str>>,
    pub protocol_src: Option<Vec<&'static str>>,
    pub line_ending_default: LineEnding,
    pub allow_dangerous_html: bool,
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
            expect_first_item: None,
            media_stack: vec![],
            definitions: HashMap::new(),
            tight_stack: vec![],
            slurp_one_line_ending: false,
            tags: true,
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
        self.last_was_tag = false;
    }

    pub fn tag(&mut self, value: String) {
        if self.tags {
            self.buffers
                .last_mut()
                .expect("Cannot push w/o buffer")
                .push(value);
            self.last_was_tag = true;
        }
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
    // let slurp_all_line_endings = false;
    let mut index = 0;
    let mut line_ending_inferred: Option<LineEnding> = None;

    // First, we figure out what the used line ending style is.
    // Stop when we find a line ending.
    while index < events.len() {
        let event = &events[index];

        if event.event_type == EventType::Exit
            && (event.token_type == Token::BlankLineEnding
                || event.token_type == Token::CodeTextLineEnding
                || event.token_type == Token::LineEnding)
        {
            let codes = codes_from_span(codes, &from_exit_event(events, index));
            line_ending_inferred = Some(LineEnding::from_code(*codes.first().unwrap()));
            break;
        }

        index += 1;
    }

    // Figure out which line ending style we’ll use.
    let line_ending_default = if let Some(value) = line_ending_inferred {
        value
    } else if let Some(value) = &options.default_line_ending {
        value.clone()
    } else {
        LineEnding::LineFeed
    };

    let mut enter_map: Map = HashMap::new();

    enter_map.insert(Token::BlockQuote, on_enter_block_quote);
    enter_map.insert(Token::CodeIndented, on_enter_code_indented);
    enter_map.insert(Token::CodeFenced, on_enter_code_fenced);
    enter_map.insert(Token::CodeFencedFenceInfo, on_enter_buffer);
    enter_map.insert(Token::CodeFencedFenceMeta, on_enter_buffer);
    enter_map.insert(Token::CodeText, on_enter_code_text);
    enter_map.insert(Token::Definition, on_enter_definition);
    enter_map.insert(
        Token::DefinitionDestinationString,
        on_enter_definition_destination_string,
    );
    enter_map.insert(Token::DefinitionLabelString, on_enter_buffer);
    enter_map.insert(Token::DefinitionTitleString, on_enter_buffer);
    enter_map.insert(Token::Emphasis, on_enter_emphasis);
    enter_map.insert(Token::HeadingAtxText, on_enter_buffer);
    enter_map.insert(Token::HeadingSetextText, on_enter_buffer);
    enter_map.insert(Token::HtmlFlow, on_enter_html_flow);
    enter_map.insert(Token::HtmlText, on_enter_html_text);
    enter_map.insert(Token::Image, on_enter_image);
    enter_map.insert(Token::Label, on_enter_buffer);
    enter_map.insert(Token::Link, on_enter_link);
    enter_map.insert(Token::Paragraph, on_enter_paragraph);
    enter_map.insert(Token::ReferenceString, on_enter_buffer);
    enter_map.insert(Token::Resource, on_enter_resource);
    enter_map.insert(
        Token::ResourceDestinationString,
        on_enter_resource_destination_string,
    );
    enter_map.insert(Token::ResourceTitleString, on_enter_buffer);
    enter_map.insert(Token::Strong, on_enter_strong);

    // To do: sort.
    enter_map.insert(Token::ListItemMarker, on_enter_list_item_marker);
    enter_map.insert(Token::ListOrdered, on_enter_list);
    enter_map.insert(Token::ListUnordered, on_enter_list);

    let mut exit_map: Map = HashMap::new();
    exit_map.insert(Token::AutolinkEmail, on_exit_autolink_email);
    exit_map.insert(Token::AutolinkProtocol, on_exit_autolink_protocol);
    exit_map.insert(Token::BlockQuote, on_exit_block_quote);
    exit_map.insert(Token::CharacterEscapeValue, on_exit_data);
    exit_map.insert(
        Token::CharacterReferenceMarker,
        on_exit_character_reference_marker,
    );
    exit_map.insert(
        Token::CharacterReferenceMarkerNumeric,
        on_exit_character_reference_marker_numeric,
    );
    exit_map.insert(
        Token::CharacterReferenceMarkerHexadecimal,
        on_exit_character_reference_marker_hexadecimal,
    );
    exit_map.insert(
        Token::CharacterReferenceValue,
        on_exit_character_reference_value,
    );
    exit_map.insert(Token::CodeFenced, on_exit_code_flow);
    exit_map.insert(Token::CodeFencedFence, on_exit_code_fenced_fence);
    exit_map.insert(Token::CodeFencedFenceInfo, on_exit_code_fenced_fence_info);
    exit_map.insert(Token::CodeFencedFenceMeta, on_exit_drop);
    exit_map.insert(Token::CodeFlowChunk, on_exit_code_flow_chunk);
    exit_map.insert(Token::CodeIndented, on_exit_code_flow);
    exit_map.insert(Token::CodeText, on_exit_code_text);
    exit_map.insert(Token::CodeTextData, on_exit_data);
    exit_map.insert(Token::CodeTextLineEnding, on_exit_code_text_line_ending);
    exit_map.insert(Token::Data, on_exit_data);
    exit_map.insert(Token::Definition, on_exit_definition);
    exit_map.insert(
        Token::DefinitionDestinationString,
        on_exit_definition_destination_string,
    );
    exit_map.insert(
        Token::DefinitionLabelString,
        on_exit_definition_label_string,
    );
    exit_map.insert(
        Token::DefinitionTitleString,
        on_exit_definition_title_string,
    );
    exit_map.insert(Token::Emphasis, on_exit_emphasis);
    exit_map.insert(Token::HardBreakEscape, on_exit_break);
    exit_map.insert(Token::HardBreakTrailing, on_exit_break);
    exit_map.insert(Token::HeadingAtx, on_exit_heading_atx);
    exit_map.insert(Token::HeadingAtxSequence, on_exit_heading_atx_sequence);
    exit_map.insert(Token::HeadingAtxText, on_exit_heading_atx_text);
    exit_map.insert(Token::HeadingSetextText, on_exit_heading_setext_text);
    exit_map.insert(
        Token::HeadingSetextUnderline,
        on_exit_heading_setext_underline,
    );
    exit_map.insert(Token::HtmlFlow, on_exit_html);
    exit_map.insert(Token::HtmlText, on_exit_html);
    exit_map.insert(Token::HtmlFlowData, on_exit_html_data);
    exit_map.insert(Token::HtmlTextData, on_exit_html_data);
    exit_map.insert(Token::Image, on_exit_media);
    exit_map.insert(Token::Label, on_exit_label);
    exit_map.insert(Token::LabelText, on_exit_label_text);
    exit_map.insert(Token::LineEnding, on_exit_line_ending);
    exit_map.insert(Token::Link, on_exit_media);
    exit_map.insert(Token::Paragraph, on_exit_paragraph);
    exit_map.insert(Token::ReferenceString, on_exit_reference_string);
    exit_map.insert(Token::Resource, on_exit_drop);
    exit_map.insert(
        Token::ResourceDestinationString,
        on_exit_resource_destination_string,
    );
    exit_map.insert(Token::ResourceTitleString, on_exit_resource_title_string);
    exit_map.insert(Token::Strong, on_exit_strong);
    exit_map.insert(Token::ThematicBreak, on_exit_thematic_break);

    // To do: sort.
    exit_map.insert(Token::ListItemValue, on_exit_list_item_value);
    exit_map.insert(Token::ListItem, on_exit_list_item);
    exit_map.insert(Token::ListOrdered, on_exit_list);
    exit_map.insert(Token::ListUnordered, on_exit_list);

    // Handle one event.
    let handle = |context: &mut CompileContext, index: usize| {
        let event = &events[index];

        let map = if event.event_type == EventType::Enter {
            &enter_map
        } else {
            &exit_map
        };

        if let Some(func) = map.get(&event.token_type) {
            context.index = index;
            func(context);
        }
    };

    let mut context = CompileContext::new(events, codes, options, line_ending_default);
    let mut definition_indices: Vec<(usize, usize)> = vec![];
    let mut index = 0;
    let mut definition_inside = false;

    // Handle all definitions first.
    // We must do two passes because we need to compile the events in
    // definitions which come after references already.
    //
    // To speed things up, we collect the places we can jump over for the
    // second pass.
    while index < events.len() {
        let event = &events[index];

        if definition_inside {
            handle(&mut context, index);
        }

        if event.token_type == Token::Definition {
            if event.event_type == EventType::Enter {
                handle(&mut context, index); // Also handle start.
                definition_inside = true;
                definition_indices.push((index, index));
            } else {
                definition_inside = false;
                definition_indices.last_mut().unwrap().1 = index;
            }
        }

        index += 1;
    }

    index = 0;
    let jump_default = (events.len(), events.len());
    let mut definition_index = 0;
    let mut jump = definition_indices
        .get(definition_index)
        .unwrap_or(&jump_default);

    while index < events.len() {
        if index == jump.0 {
            index = jump.1 + 1;
            definition_index += 1;
            jump = definition_indices
                .get(definition_index)
                .unwrap_or(&jump_default);
            // Ignore line endings after definitions.
            context.slurp_one_line_ending = true;
        } else {
            handle(&mut context, index);
            index += 1;
        }
    }

    assert_eq!(context.buffers.len(), 1, "expected 1 final buffer");
    context
        .buffers
        .get(0)
        .expect("expected 1 final buffer")
        .concat()
}

/// Handle [`Enter`][EventType::Enter]:`*`.
///
/// Buffers data.
fn on_enter_buffer(context: &mut CompileContext) {
    context.buffer();
}

/// Handle [`Enter`][EventType::Enter]:[`BlockQuote`][Token::BlockQuote].
fn on_enter_block_quote(context: &mut CompileContext) {
    context.tight_stack.push(false);
    context.line_ending_if_needed();
    context.tag("<blockquote>".to_string());
}

/// Handle [`Enter`][EventType::Enter]:[`CodeIndented`][Token::CodeIndented].
fn on_enter_code_indented(context: &mut CompileContext) {
    context.code_flow_seen_data = Some(false);
    context.line_ending_if_needed();
    context.tag("<pre><code>".to_string());
}

/// Handle [`Enter`][EventType::Enter]:[`CodeFenced`][Token::CodeFenced].
fn on_enter_code_fenced(context: &mut CompileContext) {
    context.code_flow_seen_data = Some(false);
    context.line_ending_if_needed();
    // Note that no `>` is used, which is added later.
    context.tag("<pre><code".to_string());
    context.code_fenced_fences_count = Some(0);
}

/// Handle [`Enter`][EventType::Enter]:[`CodeText`][Token::CodeText].
fn on_enter_code_text(context: &mut CompileContext) {
    context.tag("<code>".to_string());
    context.buffer();
}

/// Handle [`Enter`][EventType::Enter]:[`Definition`][Token::Definition].
fn on_enter_definition(context: &mut CompileContext) {
    context.buffer();
    context.media_stack.push(Media {
        image: false,
        label: None,
        label_id: None,
        reference_id: None,
        destination: None,
        title: None,
    });
}

/// Handle [`Enter`][EventType::Enter]:[`DefinitionDestinationString`][Token::DefinitionDestinationString].
fn on_enter_definition_destination_string(context: &mut CompileContext) {
    context.buffer();
    context.ignore_encode = true;
}

/// Handle [`Enter`][EventType::Enter]:[`Emphasis`][Token::Emphasis].
fn on_enter_emphasis(context: &mut CompileContext) {
    context.tag("<em>".to_string());
}

/// Handle [`Enter`][EventType::Enter]:[`HtmlFlow`][Token::HtmlFlow].
fn on_enter_html_flow(context: &mut CompileContext) {
    context.line_ending_if_needed();
    if context.allow_dangerous_html {
        context.ignore_encode = true;
    }
}

/// Handle [`Enter`][EventType::Enter]:[`HtmlText`][Token::HtmlText].
fn on_enter_html_text(context: &mut CompileContext) {
    if context.allow_dangerous_html {
        context.ignore_encode = true;
    }
}

/// Handle [`Enter`][EventType::Enter]:[`Image`][Token::Image].
fn on_enter_image(context: &mut CompileContext) {
    context.media_stack.push(Media {
        image: true,
        label_id: None,
        label: None,
        reference_id: None,
        destination: None,
        title: None,
    });
    context.tags = false; // Disallow tags.
}

/// Handle [`Enter`][EventType::Enter]:[`Link`][Token::Link].
fn on_enter_link(context: &mut CompileContext) {
    context.media_stack.push(Media {
        image: false,
        label_id: None,
        label: None,
        reference_id: None,
        destination: None,
        title: None,
    });
}

/// Handle [`Enter`][EventType::Enter]:[`Paragraph`][Token::Paragraph].
fn on_enter_paragraph(context: &mut CompileContext) {
    let tight = context.tight_stack.last().unwrap_or(&false);

    if !tight {
        context.line_ending_if_needed();
        context.tag("<p>".to_string());
    }

    // context.slurp_all_line_endings = false;
}

/// Handle [`Enter`][EventType::Enter]:[`Resource`][Token::Resource].
fn on_enter_resource(context: &mut CompileContext) {
    context.buffer(); // We can have line endings in the resource, ignore them.
    let media = context.media_stack.last_mut().unwrap();
    media.destination = Some("".to_string());
}

/// Handle [`Enter`][EventType::Enter]:[`ResourceDestinationString`][Token::ResourceDestinationString].
fn on_enter_resource_destination_string(context: &mut CompileContext) {
    context.buffer();
    // Ignore encoding the result, as we’ll first percent encode the url and
    // encode manually after.
    context.ignore_encode = true;
}

/// Handle [`Enter`][EventType::Enter]:[`Strong`][Token::Strong].
fn on_enter_strong(context: &mut CompileContext) {
    context.tag("<strong>".to_string());
}

/// Handle [`Exit`][EventType::Exit]:[`AutolinkEmail`][Token::AutolinkEmail].
fn on_exit_autolink_email(context: &mut CompileContext) {
    let slice = serialize(
        context.codes,
        &from_exit_event(context.events, context.index),
        false,
    );
    context.tag(format!(
        "<a href=\"{}\">",
        sanitize_uri(
            format!("mailto:{}", slice.as_str()).as_str(),
            &context.protocol_href
        )
    ));
    context.push(context.encode_opt(&slice));
    context.tag("</a>".to_string());
}

/// Handle [`Exit`][EventType::Exit]:[`AutolinkProtocol`][Token::AutolinkProtocol].
fn on_exit_autolink_protocol(context: &mut CompileContext) {
    let slice = serialize(
        context.codes,
        &from_exit_event(context.events, context.index),
        false,
    );
    context.tag(format!(
        "<a href=\"{}\">",
        sanitize_uri(slice.as_str(), &context.protocol_href)
    ));
    context.push(context.encode_opt(&slice));
    context.tag("</a>".to_string());
}

/// Handle [`Exit`][EventType::Exit]:{[`HardBreakEscape`][Token::HardBreakEscape],[`HardBreakTrailing`][Token::HardBreakTrailing]}.
fn on_exit_break(context: &mut CompileContext) {
    context.tag("<br />".to_string());
}

/// Handle [`Exit`][EventType::Exit]:[`BlockQuote`][Token::BlockQuote].
fn on_exit_block_quote(context: &mut CompileContext) {
    context.tight_stack.pop();
    context.line_ending_if_needed();
    context.tag("</blockquote>".to_string());
    // context.slurp_all_line_endings = false;
}

/// Handle [`Exit`][EventType::Exit]:[`CharacterReferenceMarker`][Token::CharacterReferenceMarker].
fn on_exit_character_reference_marker(context: &mut CompileContext) {
    context.character_reference_kind = Some(CharacterReferenceKind::Named);
}

/// Handle [`Exit`][EventType::Exit]:[`CharacterReferenceMarkerHexadecimal`][Token::CharacterReferenceMarkerHexadecimal].
fn on_exit_character_reference_marker_hexadecimal(context: &mut CompileContext) {
    context.character_reference_kind = Some(CharacterReferenceKind::Hexadecimal);
}

/// Handle [`Exit`][EventType::Exit]:[`CharacterReferenceMarkerNumeric`][Token::CharacterReferenceMarkerNumeric].
fn on_exit_character_reference_marker_numeric(context: &mut CompileContext) {
    context.character_reference_kind = Some(CharacterReferenceKind::Decimal);
}

/// Handle [`Exit`][EventType::Exit]:[`CharacterReferenceValue`][Token::CharacterReferenceValue].
fn on_exit_character_reference_value(context: &mut CompileContext) {
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

/// Handle [`Exit`][EventType::Exit]:[`CodeFlowChunk`][Token::CodeFlowChunk].
fn on_exit_code_flow_chunk(context: &mut CompileContext) {
    context.code_flow_seen_data = Some(true);
    context.push(context.encode_opt(&serialize(
        context.codes,
        &from_exit_event(context.events, context.index),
        false,
    )));
}

/// Handle [`Exit`][EventType::Exit]:[`CodeFencedFence`][Token::CodeFencedFence].
fn on_exit_code_fenced_fence(context: &mut CompileContext) {
    let count = if let Some(count) = context.code_fenced_fences_count {
        count
    } else {
        0
    };

    if count == 0 {
        context.tag(">".to_string());
        context.slurp_one_line_ending = true;
    }

    context.code_fenced_fences_count = Some(count + 1);
}

/// Handle [`Exit`][EventType::Exit]:[`CodeFencedFenceInfo`][Token::CodeFencedFenceInfo].
fn on_exit_code_fenced_fence_info(context: &mut CompileContext) {
    let value = context.resume();
    context.tag(format!(" class=\"language-{}\"", value));
}

/// Handle [`Exit`][EventType::Exit]:{[`CodeFenced`][Token::CodeFenced],[`CodeIndented`][Token::CodeIndented]}.
fn on_exit_code_flow(context: &mut CompileContext) {
    let seen_data = context
        .code_flow_seen_data
        .take()
        .expect("`code_flow_seen_data` must be defined");

    // One special case is if we are inside a container, and the fenced code was
    // not closed (meaning it runs to the end).
    // In that case, the following line ending, is considered *outside* the
    // fenced code and block quote by micromark, but CM wants to treat that
    // ending as part of the code.
    if let Some(count) = context.code_fenced_fences_count {
        if count == 1 && !context.tight_stack.is_empty() && !context.last_was_tag {
            context.line_ending();
        }
    }

    // But in most cases, it’s simpler: when we’ve seen some data, emit an extra
    // line ending when needed.
    if seen_data {
        context.line_ending_if_needed();
    }

    context.tag("</code></pre>".to_string());

    if let Some(count) = context.code_fenced_fences_count.take() {
        if count < 2 {
            context.line_ending_if_needed();
        }
    }

    context.slurp_one_line_ending = false;
}

/// Handle [`Exit`][EventType::Exit]:[`CodeText`][Token::CodeText].
fn on_exit_code_text(context: &mut CompileContext) {
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
    context.tag("</code>".to_string());
}

/// Handle [`Exit`][EventType::Exit]:[`CodeTextLineEnding`][Token::CodeTextLineEnding].
fn on_exit_code_text_line_ending(context: &mut CompileContext) {
    context.push(" ".to_string());
}

/// Handle [`Exit`][EventType::Exit]:*.
///
/// Resumes, and ignores what was resumed.
fn on_exit_drop(context: &mut CompileContext) {
    context.resume();
}

/// Handle [`Exit`][EventType::Exit]:{[`CodeTextData`][Token::CodeTextData],[`Data`][Token::Data],[`CharacterEscapeValue`][Token::CharacterEscapeValue]}.
fn on_exit_data(context: &mut CompileContext) {
    // Just output it.
    context.push(context.encode_opt(&serialize(
        context.codes,
        &from_exit_event(context.events, context.index),
        false,
    )));
}

/// Handle [`Exit`][EventType::Exit]:[`Definition`][Token::Definition].
fn on_exit_definition(context: &mut CompileContext) {
    let definition = context.media_stack.pop().unwrap();
    let reference_id = normalize_identifier(&definition.reference_id.unwrap());
    let destination = definition.destination;
    let title = definition.title;

    context.resume();

    context
        .definitions
        .entry(reference_id)
        .or_insert(Definition { destination, title });
}

/// Handle [`Exit`][EventType::Exit]:[`DefinitionDestinationString`][Token::DefinitionDestinationString].
fn on_exit_definition_destination_string(context: &mut CompileContext) {
    let buf = context.resume();
    let definition = context.media_stack.last_mut().unwrap();
    definition.destination = Some(buf);
    context.ignore_encode = false;
}

/// Handle [`Exit`][EventType::Exit]:[`DefinitionLabelString`][Token::DefinitionLabelString].
fn on_exit_definition_label_string(context: &mut CompileContext) {
    // Discard label, use the source content instead.
    context.resume();
    let definition = context.media_stack.last_mut().unwrap();
    definition.reference_id = Some(serialize(
        context.codes,
        &from_exit_event(context.events, context.index),
        false,
    ));
}

/// Handle [`Exit`][EventType::Exit]:[`DefinitionTitleString`][Token::DefinitionTitleString].
fn on_exit_definition_title_string(context: &mut CompileContext) {
    let buf = context.resume();
    let definition = context.media_stack.last_mut().unwrap();
    definition.title = Some(buf);
}

/// Handle [`Exit`][EventType::Exit]:[`Strong`][Token::Emphasis].
fn on_exit_emphasis(context: &mut CompileContext) {
    context.tag("</em>".to_string());
}

/// Handle [`Exit`][EventType::Exit]:[`HeadingAtx`][Token::HeadingAtx].
fn on_exit_heading_atx(context: &mut CompileContext) {
    let rank = context
        .atx_opening_sequence_size
        .take()
        .expect("`atx_opening_sequence_size` must be set in headings");

    context.tag(format!("</h{}>", rank));
}

/// Handle [`Exit`][EventType::Exit]:[`HeadingAtxSequence`][Token::HeadingAtxSequence].
fn on_exit_heading_atx_sequence(context: &mut CompileContext) {
    // First fence we see.
    if context.atx_opening_sequence_size.is_none() {
        let rank = serialize(
            context.codes,
            &from_exit_event(context.events, context.index),
            false,
        )
        .len();
        context.line_ending_if_needed();
        context.atx_opening_sequence_size = Some(rank);
        context.tag(format!("<h{}>", rank));
    }
}

/// Handle [`Exit`][EventType::Exit]:[`HeadingAtxText`][Token::HeadingAtxText].
fn on_exit_heading_atx_text(context: &mut CompileContext) {
    let value = context.resume();
    context.push(value);
}

/// Handle [`Exit`][EventType::Exit]:[`HeadingSetextText`][Token::HeadingSetextText].
fn on_exit_heading_setext_text(context: &mut CompileContext) {
    let buf = context.resume();
    context.heading_setext_buffer = Some(buf);
    context.slurp_one_line_ending = true;
}

/// Handle [`Exit`][EventType::Exit]:[`HeadingSetextUnderline`][Token::HeadingSetextUnderline].
fn on_exit_heading_setext_underline(context: &mut CompileContext) {
    let text = context
        .heading_setext_buffer
        .take()
        .expect("`atx_opening_sequence_size` must be set in headings");
    let head = codes_from_span(
        context.codes,
        &from_exit_event(context.events, context.index),
    )[0];
    let level: usize = if head == Code::Char('-') { 2 } else { 1 };

    context.line_ending_if_needed();
    context.tag(format!("<h{}>", level));
    context.push(text);
    context.tag(format!("</h{}>", level));
}

/// Handle [`Exit`][EventType::Exit]:{[`HtmlFlow`][Token::HtmlFlow],[`HtmlText`][Token::HtmlText]}.
fn on_exit_html(context: &mut CompileContext) {
    context.ignore_encode = false;
}

/// Handle [`Exit`][EventType::Exit]:{[`HtmlFlowData`][Token::HtmlFlowData],[`HtmlTextData`][Token::HtmlTextData]}.
fn on_exit_html_data(context: &mut CompileContext) {
    let slice = serialize(
        context.codes,
        &from_exit_event(context.events, context.index),
        false,
    );
    context.push(context.encode_opt(&slice));
}

/// Handle [`Exit`][EventType::Exit]:[`Label`][Token::Label].
fn on_exit_label(context: &mut CompileContext) {
    let buf = context.resume();
    let media = context.media_stack.last_mut().unwrap();
    media.label = Some(buf);
}

/// Handle [`Exit`][EventType::Exit]:[`LabelText`][Token::LabelText].
fn on_exit_label_text(context: &mut CompileContext) {
    let media = context.media_stack.last_mut().unwrap();
    media.label_id = Some(serialize(
        context.codes,
        &from_exit_event(context.events, context.index),
        false,
    ));
}

/// Handle [`Exit`][EventType::Exit]:[`LineEnding`][Token::LineEnding].
fn on_exit_line_ending(context: &mut CompileContext) {
    // if context.slurp_all_line_endings {
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

/// Handle [`Exit`][EventType::Exit]:{[`Image`][Token::Image],[`Link`][Token::Link]}.
fn on_exit_media(context: &mut CompileContext) {
    let mut is_in_image = false;
    let mut index = 0;

    // Skip current.
    while index < (context.media_stack.len() - 1) {
        if context.media_stack[index].image {
            is_in_image = true;
            break;
        }
        index += 1;
    }

    context.tags = !is_in_image;

    let media = context.media_stack.pop().unwrap();
    let id = media
        .reference_id
        .or(media.label_id)
        .map(|id| normalize_identifier(&id));
    let label = media.label.unwrap();
    let definition = id.and_then(|id| context.definitions.get(&id));
    let destination = if media.destination.is_some() {
        &media.destination
    } else {
        &definition.unwrap().destination
    };
    let title = if media.destination.is_some() {
        &media.title
    } else {
        &definition.unwrap().title
    };

    let destination = if let Some(destination) = destination {
        destination.clone()
    } else {
        "".to_string()
    };

    let title = if let Some(title) = title {
        format!(" title=\"{}\"", title)
    } else {
        "".to_string()
    };

    if media.image {
        context.tag(format!(
            "<img src=\"{}\" alt=\"",
            sanitize_uri(&destination, &context.protocol_src),
        ));
        context.push(label);
        context.tag(format!("\"{} />", title));
    } else {
        context.tag(format!(
            "<a href=\"{}\"{}>",
            sanitize_uri(&destination, &context.protocol_href),
            title,
        ));
        context.push(label);
        context.tag("</a>".to_string());
    };
}

/// Handle [`Exit`][EventType::Exit]:[`Paragraph`][Token::Paragraph].
fn on_exit_paragraph(context: &mut CompileContext) {
    let tight = context.tight_stack.last().unwrap_or(&false);

    if !tight {
        context.tag("</p>".to_string());
    }
}

/// Handle [`Exit`][EventType::Exit]:[`ReferenceString`][Token::ReferenceString].
fn on_exit_reference_string(context: &mut CompileContext) {
    // Drop stuff.
    context.resume();
    let media = context.media_stack.last_mut().unwrap();
    media.reference_id = Some(serialize(
        context.codes,
        &from_exit_event(context.events, context.index),
        false,
    ));
}

/// Handle [`Exit`][EventType::Exit]:[`ResourceDestinationString`][Token::ResourceDestinationString].
fn on_exit_resource_destination_string(context: &mut CompileContext) {
    let buf = context.resume();
    let media = context.media_stack.last_mut().unwrap();
    media.destination = Some(buf);
    context.ignore_encode = false;
}

/// Handle [`Exit`][EventType::Exit]:[`ResourceTitleString`][Token::ResourceTitleString].
fn on_exit_resource_title_string(context: &mut CompileContext) {
    let buf = context.resume();
    let media = context.media_stack.last_mut().unwrap();
    media.title = Some(buf);
}

/// Handle [`Exit`][EventType::Exit]:[`Strong`][Token::Strong].
fn on_exit_strong(context: &mut CompileContext) {
    context.tag("</strong>".to_string());
}

/// Handle [`Exit`][EventType::Exit]:[`ThematicBreak`][Token::ThematicBreak].
fn on_exit_thematic_break(context: &mut CompileContext) {
    context.line_ending_if_needed();
    context.tag("<hr />".to_string());
}

// To do: sort.
/// To do (onenterlist{un,}ordered)
fn on_enter_list(context: &mut CompileContext) {
    let events = &context.events;
    let mut index = context.index;
    let mut balance = 0;
    let mut loose = false;
    let token_type = &events[index].token_type;

    while index < events.len() {
        let event = &events[index];

        if event.event_type == EventType::Enter {
            balance += 1;
        } else {
            balance -= 1;

            // Blank line directly in list or directly in list item.
            if balance < 3 && event.token_type == Token::BlankLineEnding {
                loose = true;
                break;
            }

            // Done.
            if balance == 0 && event.token_type == *token_type {
                break;
            }
        }

        index += 1;
    }

    println!("list: {:?} {:?}", token_type, loose);
    context.tight_stack.push(!loose);
    context.line_ending_if_needed();
    // Note: no `>`.
    context.tag(format!(
        "<{}",
        if *token_type == Token::ListOrdered {
            "ol"
        } else {
            "ul"
        }
    ));
    context.expect_first_item = Some(true);
}

/// To do
fn on_enter_list_item_marker(context: &mut CompileContext) {
    let expect_first_item = context.expect_first_item.take().unwrap();

    if expect_first_item {
        context.tag(">".to_string());
    }

    context.line_ending_if_needed();
    context.tag("<li>".to_string());
    context.expect_first_item = Some(false);
    // “Hack” to prevent a line ending from showing up if the item is empty.
    context.last_was_tag = false;
}

/// To do
fn on_exit_list_item_value(context: &mut CompileContext) {
    let expect_first_item = context.expect_first_item.unwrap();

    if expect_first_item {
        let slice = serialize(
            context.codes,
            &from_exit_event(context.events, context.index),
            false,
        );
        let value = slice.parse::<u32>().ok().unwrap();

        if value != 1 {
            context.tag(format!(" start=\"{}\"", encode(&value.to_string())));
        }
    }
}

/// To do.
fn on_exit_list_item(context: &mut CompileContext) {
    //  && !context.slurp_all_line_endings
    if context.last_was_tag {
        context.line_ending_if_needed();
    }

    context.tag("</li>".to_string());
    // context.slurp_all_line_endings = false;
}

/// To do.
fn on_exit_list(context: &mut CompileContext) {
    let tag_name = if context.events[context.index].token_type == Token::ListOrdered {
        "ol"
    } else {
        "ul"
    };
    context.tight_stack.pop();
    context.line_ending();
    context.tag(format!("</{}>", tag_name));
}
