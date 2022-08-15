//! Turn events into a string of HTML.
use crate::constant::{SAFE_PROTOCOL_HREF, SAFE_PROTOCOL_SRC};
use crate::event::{Event, Kind, Name};
use crate::util::{
    decode_character_reference::{decode_named, decode_numeric},
    encode::encode,
    normalize_identifier::normalize_identifier,
    sanitize_uri::sanitize_uri,
    skip,
    slice::{Position, Slice},
};
use crate::{LineEnding, Options};
use std::str;

/// Link or image, resource or reference.
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
    label_id: Option<(usize, usize)>,
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
    reference_id: Option<(usize, usize)>,
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

/// Context used to compile markdown.
#[allow(clippy::struct_excessive_bools)]
#[derive(Debug)]
struct CompileContext<'a> {
    /// Static info.
    pub events: &'a [Event],
    pub bytes: &'a [u8],
    /// Fields used by handlers to track the things they need to track to
    /// compile markdown.
    pub atx_opening_sequence_size: Option<usize>,
    pub heading_setext_buffer: Option<String>,
    pub code_flow_seen_data: Option<bool>,
    pub code_fenced_fences_count: Option<usize>,
    pub code_text_inside: bool,
    pub character_reference_marker: Option<u8>,
    pub expect_first_item: Option<bool>,
    pub media_stack: Vec<Media>,
    pub definitions: Vec<(String, Definition)>,
    pub tight_stack: Vec<bool>,
    /// Fields used to influance the current compilation.
    pub slurp_one_line_ending: bool,
    pub in_image_alt: bool,
    pub encode_html: bool,
    /// Configuration
    pub protocol_href: Option<Vec<&'static str>>,
    pub protocol_src: Option<Vec<&'static str>>,
    pub line_ending_default: LineEnding,
    pub allow_dangerous_html: bool,
    /// Intermediate results.
    pub buffers: Vec<String>,
    pub index: usize,
}

impl<'a> CompileContext<'a> {
    /// Create a new compile context.
    pub fn new(
        events: &'a [Event],
        bytes: &'a [u8],
        options: &Options,
        line_ending: LineEnding,
    ) -> CompileContext<'a> {
        CompileContext {
            events,
            bytes,
            atx_opening_sequence_size: None,
            heading_setext_buffer: None,
            code_flow_seen_data: None,
            code_fenced_fences_count: None,
            code_text_inside: false,
            character_reference_marker: None,
            expect_first_item: None,
            media_stack: vec![],
            definitions: vec![],
            tight_stack: vec![],
            slurp_one_line_ending: false,
            in_image_alt: false,
            encode_html: true,
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
            buffers: vec![String::new()],
            index: 0,
        }
    }

    /// Push a buffer.
    pub fn buffer(&mut self) {
        self.buffers.push(String::new());
    }

    /// Pop a buffer, returning its value.
    pub fn resume(&mut self) -> String {
        self.buffers.pop().expect("Cannot resume w/o buffer")
    }

    /// Push a str to the last buffer.
    pub fn push(&mut self, value: &str) {
        self.buffers
            .last_mut()
            .expect("Cannot push w/o buffer")
            .push_str(value);
    }

    /// Add a line ending.
    pub fn line_ending(&mut self) {
        let eol = self.line_ending_default.as_str().to_string();
        self.push(&eol);
    }

    /// Add a line ending if needed (as in, there’s no eol/eof already).
    pub fn line_ending_if_needed(&mut self) {
        let tail = self
            .buffers
            .last()
            .expect("at least one buffer should exist")
            .as_bytes()
            .last();

        if !matches!(tail, None | Some(b'\n' | b'\r')) {
            self.line_ending();
        }
    }
}

/// Turn events and codes into a string of HTML.
pub fn compile(events: &[Event], bytes: &[u8], options: &Options) -> String {
    let mut index = 0;
    let mut line_ending_inferred = None;

    // First, we figure out what the used line ending style is.
    // Stop when we find a line ending.
    while index < events.len() {
        let event = &events[index];

        if event.kind == Kind::Exit
            && (event.name == Name::BlankLineEnding || event.name == Name::LineEnding)
        {
            line_ending_inferred = Some(LineEnding::from_str(
                Slice::from_position(bytes, &Position::from_exit_event(events, index)).as_str(),
            ));
            break;
        }

        index += 1;
    }

    // Figure out which line ending style we’ll use.
    let line_ending_default = if let Some(value) = line_ending_inferred {
        value
    } else {
        options.default_line_ending.clone()
    };

    let mut context = CompileContext::new(events, bytes, options, line_ending_default);
    let mut definition_indices = vec![];
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

        if event.name == Name::Definition {
            if event.kind == Kind::Enter {
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
        .to_string()
}

// Handle the event at `index`.
fn handle(context: &mut CompileContext, index: usize) {
    context.index = index;

    if context.events[index].kind == Kind::Enter {
        enter(context);
    } else {
        exit(context);
    }
}

/// Handle [`Enter`][Kind::Enter].
fn enter(context: &mut CompileContext) {
    match context.events[context.index].name {
        Name::CodeFencedFenceInfo
        | Name::CodeFencedFenceMeta
        | Name::DefinitionLabelString
        | Name::DefinitionTitleString
        | Name::HeadingAtxText
        | Name::HeadingSetextText
        | Name::Label
        | Name::ReferenceString
        | Name::ResourceTitleString => on_enter_buffer(context),

        Name::BlockQuote => on_enter_block_quote(context),
        Name::CodeIndented => on_enter_code_indented(context),
        Name::CodeFenced => on_enter_code_fenced(context),
        Name::CodeText => on_enter_code_text(context),
        Name::Definition => on_enter_definition(context),
        Name::DefinitionDestinationString => on_enter_definition_destination_string(context),
        Name::Emphasis => on_enter_emphasis(context),
        Name::HtmlFlow => on_enter_html_flow(context),
        Name::HtmlText => on_enter_html_text(context),
        Name::Image => on_enter_image(context),
        Name::Link => on_enter_link(context),
        Name::ListItemMarker => on_enter_list_item_marker(context),
        Name::ListOrdered | Name::ListUnordered => on_enter_list(context),
        Name::Paragraph => on_enter_paragraph(context),
        Name::Resource => on_enter_resource(context),
        Name::ResourceDestinationString => on_enter_resource_destination_string(context),
        Name::Strong => on_enter_strong(context),
        _ => {}
    }
}

/// Handle [`Exit`][Kind::Exit].
fn exit(context: &mut CompileContext) {
    match context.events[context.index].name {
        Name::CodeFencedFenceMeta | Name::Resource => on_exit_drop(context),
        Name::CharacterEscapeValue | Name::CodeTextData | Name::Data => on_exit_data(context),

        Name::AutolinkEmail => on_exit_autolink_email(context),
        Name::AutolinkProtocol => on_exit_autolink_protocol(context),
        Name::BlankLineEnding => on_exit_blank_line_ending(context),
        Name::BlockQuote => on_exit_block_quote(context),
        Name::CharacterReferenceMarker => on_exit_character_reference_marker(context),
        Name::CharacterReferenceMarkerNumeric => {
            on_exit_character_reference_marker_numeric(context);
        }
        Name::CharacterReferenceMarkerHexadecimal => {
            on_exit_character_reference_marker_hexadecimal(context);
        }
        Name::CharacterReferenceValue => on_exit_character_reference_value(context),
        Name::CodeFenced | Name::CodeIndented => on_exit_code_flow(context),
        Name::CodeFencedFence => on_exit_code_fenced_fence(context),
        Name::CodeFencedFenceInfo => on_exit_code_fenced_fence_info(context),
        Name::CodeFlowChunk => on_exit_code_flow_chunk(context),
        Name::CodeText => on_exit_code_text(context),
        Name::Definition => on_exit_definition(context),
        Name::DefinitionDestinationString => on_exit_definition_destination_string(context),
        Name::DefinitionLabelString => on_exit_definition_label_string(context),
        Name::DefinitionTitleString => on_exit_definition_title_string(context),
        Name::Emphasis => on_exit_emphasis(context),
        Name::HardBreakEscape | Name::HardBreakTrailing => on_exit_break(context),
        Name::HeadingAtx => on_exit_heading_atx(context),
        Name::HeadingAtxSequence => on_exit_heading_atx_sequence(context),
        Name::HeadingAtxText => on_exit_heading_atx_text(context),
        Name::HeadingSetextText => on_exit_heading_setext_text(context),
        Name::HeadingSetextUnderline => on_exit_heading_setext_underline(context),
        Name::HtmlFlow | Name::HtmlText => on_exit_html(context),
        Name::HtmlFlowData | Name::HtmlTextData => on_exit_html_data(context),
        Name::Image | Name::Link => on_exit_media(context),
        Name::Label => on_exit_label(context),
        Name::LabelText => on_exit_label_text(context),
        Name::LineEnding => on_exit_line_ending(context),
        Name::ListOrdered | Name::ListUnordered => on_exit_list(context),
        Name::ListItem => on_exit_list_item(context),
        Name::ListItemValue => on_exit_list_item_value(context),
        Name::Paragraph => on_exit_paragraph(context),
        Name::ReferenceString => on_exit_reference_string(context),
        Name::ResourceDestinationString => on_exit_resource_destination_string(context),
        Name::ResourceTitleString => on_exit_resource_title_string(context),
        Name::Strong => on_exit_strong(context),
        Name::ThematicBreak => on_exit_thematic_break(context),
        _ => {}
    }
}

/// Handle [`Enter`][Kind::Enter]:`*`.
///
/// Buffers data.
fn on_enter_buffer(context: &mut CompileContext) {
    context.buffer();
}

/// Handle [`Enter`][Kind::Enter]:[`BlockQuote`][Name::BlockQuote].
fn on_enter_block_quote(context: &mut CompileContext) {
    context.tight_stack.push(false);
    context.line_ending_if_needed();
    context.push("<blockquote>");
}

/// Handle [`Enter`][Kind::Enter]:[`CodeIndented`][Name::CodeIndented].
fn on_enter_code_indented(context: &mut CompileContext) {
    context.code_flow_seen_data = Some(false);
    context.line_ending_if_needed();
    context.push("<pre><code>");
}

/// Handle [`Enter`][Kind::Enter]:[`CodeFenced`][Name::CodeFenced].
fn on_enter_code_fenced(context: &mut CompileContext) {
    context.code_flow_seen_data = Some(false);
    context.line_ending_if_needed();
    // Note that no `>` is used, which is added later.
    context.push("<pre><code");
    context.code_fenced_fences_count = Some(0);
}

/// Handle [`Enter`][Kind::Enter]:[`CodeText`][Name::CodeText].
fn on_enter_code_text(context: &mut CompileContext) {
    context.code_text_inside = true;
    if !context.in_image_alt {
        context.push("<code>");
    }
    context.buffer();
}

/// Handle [`Enter`][Kind::Enter]:[`Definition`][Name::Definition].
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

/// Handle [`Enter`][Kind::Enter]:[`DefinitionDestinationString`][Name::DefinitionDestinationString].
fn on_enter_definition_destination_string(context: &mut CompileContext) {
    context.buffer();
    context.encode_html = false;
}

/// Handle [`Enter`][Kind::Enter]:[`Emphasis`][Name::Emphasis].
fn on_enter_emphasis(context: &mut CompileContext) {
    if !context.in_image_alt {
        context.push("<em>");
    }
}

/// Handle [`Enter`][Kind::Enter]:[`HtmlFlow`][Name::HtmlFlow].
fn on_enter_html_flow(context: &mut CompileContext) {
    context.line_ending_if_needed();
    if context.allow_dangerous_html {
        context.encode_html = false;
    }
}

/// Handle [`Enter`][Kind::Enter]:[`HtmlText`][Name::HtmlText].
fn on_enter_html_text(context: &mut CompileContext) {
    if context.allow_dangerous_html {
        context.encode_html = false;
    }
}

/// Handle [`Enter`][Kind::Enter]:[`Image`][Name::Image].
fn on_enter_image(context: &mut CompileContext) {
    context.media_stack.push(Media {
        image: true,
        label_id: None,
        label: None,
        reference_id: None,
        destination: None,
        title: None,
    });
    context.in_image_alt = true; // Disallow tags.
}

/// Handle [`Enter`][Kind::Enter]:[`Link`][Name::Link].
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

/// Handle [`Enter`][Kind::Enter]:{[`ListOrdered`][Name::ListOrdered],[`ListUnordered`][Name::ListUnordered]}.
fn on_enter_list(context: &mut CompileContext) {
    let events = &context.events;
    let mut index = context.index;
    let mut balance = 0;
    let mut loose = false;
    let name = &events[index].name;

    while index < events.len() {
        let event = &events[index];

        if event.kind == Kind::Enter {
            balance += 1;
        } else {
            balance -= 1;

            if balance < 3 && event.name == Name::BlankLineEnding {
                // Blank line directly after a prefix:
                //
                // ```markdown
                // > | -␊
                //      ^
                //   |   a
                // ```
                let mut at_prefix = false;
                // Blank line directly after item, which is just a prefix.
                //
                // ```markdown
                // > | -␊
                //      ^
                //   | - a
                // ```
                let mut at_empty_list_item = false;
                // Blank line at block quote prefix:
                //
                // ```markdown
                // > | * >␊
                //        ^
                //   | * a
                // ```
                let mut at_empty_block_quote = false;

                if balance == 1 {
                    let mut before = index - 2;

                    if events[before].name == Name::ListItem {
                        before -= 1;

                        if events[before].name == Name::SpaceOrTab {
                            before -= 2;
                        }

                        if events[before].name == Name::BlockQuote
                            && events[before - 1].name == Name::BlockQuotePrefix
                        {
                            at_empty_block_quote = true;
                        } else if events[before].name == Name::ListItemPrefix {
                            at_empty_list_item = true;
                        }
                    }
                } else {
                    let mut before = index - 2;

                    if events[before].name == Name::SpaceOrTab {
                        before -= 2;
                    }

                    if events[before].name == Name::ListItemPrefix {
                        at_prefix = true;
                    }
                }

                if !at_prefix && !at_empty_list_item && !at_empty_block_quote {
                    loose = true;
                    break;
                }
            }

            // Done.
            if balance == 0 && event.name == *name {
                break;
            }
        }

        index += 1;
    }

    context.tight_stack.push(!loose);
    context.line_ending_if_needed();
    // Note: no `>`.
    context.push(if *name == Name::ListOrdered {
        "<ol"
    } else {
        "<ul"
    });
    context.expect_first_item = Some(true);
}

/// Handle [`Enter`][Kind::Enter]:[`ListItemMarker`][Name::ListItemMarker].
fn on_enter_list_item_marker(context: &mut CompileContext) {
    let expect_first_item = context.expect_first_item.take().unwrap();

    if expect_first_item {
        context.push(">");
    }

    context.line_ending_if_needed();

    context.push("<li>");
    context.expect_first_item = Some(false);
}

/// Handle [`Enter`][Kind::Enter]:[`Paragraph`][Name::Paragraph].
fn on_enter_paragraph(context: &mut CompileContext) {
    let tight = context.tight_stack.last().unwrap_or(&false);

    if !tight {
        context.line_ending_if_needed();
        context.push("<p>");
    }
}

/// Handle [`Enter`][Kind::Enter]:[`Resource`][Name::Resource].
fn on_enter_resource(context: &mut CompileContext) {
    context.buffer(); // We can have line endings in the resource, ignore them.
    context.media_stack.last_mut().unwrap().destination = Some("".to_string());
}

/// Handle [`Enter`][Kind::Enter]:[`ResourceDestinationString`][Name::ResourceDestinationString].
fn on_enter_resource_destination_string(context: &mut CompileContext) {
    context.buffer();
    // Ignore encoding the result, as we’ll first percent encode the url and
    // encode manually after.
    context.encode_html = false;
}

/// Handle [`Enter`][Kind::Enter]:[`Strong`][Name::Strong].
fn on_enter_strong(context: &mut CompileContext) {
    if !context.in_image_alt {
        context.push("<strong>");
    }
}

/// Handle [`Exit`][Kind::Exit]:[`AutolinkEmail`][Name::AutolinkEmail].
fn on_exit_autolink_email(context: &mut CompileContext) {
    let slice = Slice::from_position(
        context.bytes,
        &Position::from_exit_event(context.events, context.index),
    );
    let value = slice.as_str();

    if !context.in_image_alt {
        context.push("<a href=\"");
        context.push(&sanitize_uri(
            &format!("mailto:{}", value),
            &context.protocol_href,
        ));
        context.push("\">");
    }

    context.push(&encode(value, context.encode_html));

    if !context.in_image_alt {
        context.push("</a>");
    }
}

/// Handle [`Exit`][Kind::Exit]:[`AutolinkProtocol`][Name::AutolinkProtocol].
fn on_exit_autolink_protocol(context: &mut CompileContext) {
    let slice = Slice::from_position(
        context.bytes,
        &Position::from_exit_event(context.events, context.index),
    );
    let value = slice.as_str();

    if !context.in_image_alt {
        context.push("<a href=\"");
        context.push(&sanitize_uri(value, &context.protocol_href));
        context.push("\">");
    }

    context.push(&encode(value, context.encode_html));

    if !context.in_image_alt {
        context.push("</a>");
    }
}

/// Handle [`Exit`][Kind::Exit]:{[`HardBreakEscape`][Name::HardBreakEscape],[`HardBreakTrailing`][Name::HardBreakTrailing]}.
fn on_exit_break(context: &mut CompileContext) {
    if !context.in_image_alt {
        context.push("<br />");
    }
}

/// Handle [`Exit`][Kind::Exit]:[`BlankLineEnding`][Name::BlankLineEnding].
fn on_exit_blank_line_ending(context: &mut CompileContext) {
    if context.index == context.events.len() - 1 {
        context.line_ending_if_needed();
    }
}

/// Handle [`Exit`][Kind::Exit]:[`BlockQuote`][Name::BlockQuote].
fn on_exit_block_quote(context: &mut CompileContext) {
    context.tight_stack.pop();
    context.line_ending_if_needed();
    context.slurp_one_line_ending = false;
    context.push("</blockquote>");
}

/// Handle [`Exit`][Kind::Exit]:[`CharacterReferenceMarker`][Name::CharacterReferenceMarker].
fn on_exit_character_reference_marker(context: &mut CompileContext) {
    context.character_reference_marker = Some(b'&');
}

/// Handle [`Exit`][Kind::Exit]:[`CharacterReferenceMarkerHexadecimal`][Name::CharacterReferenceMarkerHexadecimal].
fn on_exit_character_reference_marker_hexadecimal(context: &mut CompileContext) {
    context.character_reference_marker = Some(b'x');
}

/// Handle [`Exit`][Kind::Exit]:[`CharacterReferenceMarkerNumeric`][Name::CharacterReferenceMarkerNumeric].
fn on_exit_character_reference_marker_numeric(context: &mut CompileContext) {
    context.character_reference_marker = Some(b'#');
}

/// Handle [`Exit`][Kind::Exit]:[`CharacterReferenceValue`][Name::CharacterReferenceValue].
fn on_exit_character_reference_value(context: &mut CompileContext) {
    let marker = context
        .character_reference_marker
        .take()
        .expect("expected `character_reference_kind` to be set");
    let slice = Slice::from_position(
        context.bytes,
        &Position::from_exit_event(context.events, context.index),
    );
    let value = slice.as_str();

    let value = match marker {
        b'#' => decode_numeric(value, 10),
        b'x' => decode_numeric(value, 16),
        b'&' => decode_named(value),
        _ => panic!("impossible"),
    };

    context.push(&encode(&value, context.encode_html));
}

/// Handle [`Exit`][Kind::Exit]:[`CodeFlowChunk`][Name::CodeFlowChunk].
fn on_exit_code_flow_chunk(context: &mut CompileContext) {
    context.code_flow_seen_data = Some(true);
    context.push(&encode(
        &Slice::from_position(
            context.bytes,
            &Position::from_exit_event(context.events, context.index),
        )
        // Must serialize to get virtual spaces.
        .serialize(),
        context.encode_html,
    ));
}

/// Handle [`Exit`][Kind::Exit]:[`CodeFencedFence`][Name::CodeFencedFence].
fn on_exit_code_fenced_fence(context: &mut CompileContext) {
    let count = if let Some(count) = context.code_fenced_fences_count {
        count
    } else {
        0
    };

    if count == 0 {
        context.push(">");
        context.slurp_one_line_ending = true;
    }

    context.code_fenced_fences_count = Some(count + 1);
}

/// Handle [`Exit`][Kind::Exit]:[`CodeFencedFenceInfo`][Name::CodeFencedFenceInfo].
fn on_exit_code_fenced_fence_info(context: &mut CompileContext) {
    let value = context.resume();
    context.push(" class=\"language-");
    context.push(&value);
    context.push("\"");
}

/// Handle [`Exit`][Kind::Exit]:{[`CodeFenced`][Name::CodeFenced],[`CodeIndented`][Name::CodeIndented]}.
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
        // No closing fence.
        if count == 1
            // In a container.
            && !context.tight_stack.is_empty()
            // Empty (as the closing is right at the opening fence)
            && context.events[context.index - 1].name != Name::CodeFencedFence
        {
            context.line_ending();
        }
    }

    // But in most cases, it’s simpler: when we’ve seen some data, emit an extra
    // line ending when needed.
    if seen_data {
        context.line_ending_if_needed();
    }

    context.push("</code></pre>");

    if let Some(count) = context.code_fenced_fences_count.take() {
        if count < 2 {
            context.line_ending_if_needed();
        }
    }

    context.slurp_one_line_ending = false;
}

/// Handle [`Exit`][Kind::Exit]:[`CodeText`][Name::CodeText].
fn on_exit_code_text(context: &mut CompileContext) {
    let result = context.resume();
    let mut bytes = result.as_bytes();
    let mut trim = false;
    let mut index = 0;
    let mut end = bytes.len();

    if end > 2 && bytes[index] == b' ' && bytes[end - 1] == b' ' {
        index += 1;
        end -= 1;
        while index < end && !trim {
            if bytes[index] != b' ' {
                trim = true;
                break;
            }
            index += 1;
        }
    }

    if trim {
        bytes = &bytes[1..end];
    }

    context.code_text_inside = false;
    context.push(str::from_utf8(bytes).unwrap());

    if !context.in_image_alt {
        context.push("</code>");
    }
}

/// Handle [`Exit`][Kind::Exit]:*.
///
/// Resumes, and ignores what was resumed.
fn on_exit_drop(context: &mut CompileContext) {
    context.resume();
}

/// Handle [`Exit`][Kind::Exit]:{[`CodeTextData`][Name::CodeTextData],[`Data`][Name::Data],[`CharacterEscapeValue`][Name::CharacterEscapeValue]}.
fn on_exit_data(context: &mut CompileContext) {
    context.push(&encode(
        Slice::from_position(
            context.bytes,
            &Position::from_exit_event(context.events, context.index),
        )
        .as_str(),
        context.encode_html,
    ));
}

/// Handle [`Exit`][Kind::Exit]:[`Definition`][Name::Definition].
fn on_exit_definition(context: &mut CompileContext) {
    context.resume();
    let media = context.media_stack.pop().unwrap();
    let indices = media.reference_id.unwrap();
    let id =
        normalize_identifier(Slice::from_indices(context.bytes, indices.0, indices.1).as_str());

    context.definitions.push((
        id,
        Definition {
            destination: media.destination,
            title: media.title,
        },
    ));
}

/// Handle [`Exit`][Kind::Exit]:[`DefinitionDestinationString`][Name::DefinitionDestinationString].
fn on_exit_definition_destination_string(context: &mut CompileContext) {
    let buf = context.resume();
    context.media_stack.last_mut().unwrap().destination = Some(buf);
    context.encode_html = true;
}

/// Handle [`Exit`][Kind::Exit]:[`DefinitionLabelString`][Name::DefinitionLabelString].
fn on_exit_definition_label_string(context: &mut CompileContext) {
    // Discard label, use the source content instead.
    context.resume();
    context.media_stack.last_mut().unwrap().reference_id =
        Some(Position::from_exit_event(context.events, context.index).to_indices());
}

/// Handle [`Exit`][Kind::Exit]:[`DefinitionTitleString`][Name::DefinitionTitleString].
fn on_exit_definition_title_string(context: &mut CompileContext) {
    let buf = context.resume();
    context.media_stack.last_mut().unwrap().title = Some(buf);
}

/// Handle [`Exit`][Kind::Exit]:[`Strong`][Name::Emphasis].
fn on_exit_emphasis(context: &mut CompileContext) {
    if !context.in_image_alt {
        context.push("</em>");
    }
}

/// Handle [`Exit`][Kind::Exit]:[`HeadingAtx`][Name::HeadingAtx].
fn on_exit_heading_atx(context: &mut CompileContext) {
    let rank = context
        .atx_opening_sequence_size
        .take()
        .expect("`atx_opening_sequence_size` must be set in headings");

    context.push("</h");
    context.push(&rank.to_string());
    context.push(">");
}

/// Handle [`Exit`][Kind::Exit]:[`HeadingAtxSequence`][Name::HeadingAtxSequence].
fn on_exit_heading_atx_sequence(context: &mut CompileContext) {
    // First fence we see.
    if context.atx_opening_sequence_size.is_none() {
        let rank = Slice::from_position(
            context.bytes,
            &Position::from_exit_event(context.events, context.index),
        )
        .len();
        context.line_ending_if_needed();
        context.atx_opening_sequence_size = Some(rank);
        context.push("<h");
        context.push(&rank.to_string());
        context.push(">");
    }
}

/// Handle [`Exit`][Kind::Exit]:[`HeadingAtxText`][Name::HeadingAtxText].
fn on_exit_heading_atx_text(context: &mut CompileContext) {
    let value = context.resume();
    context.push(&value);
}

/// Handle [`Exit`][Kind::Exit]:[`HeadingSetextText`][Name::HeadingSetextText].
fn on_exit_heading_setext_text(context: &mut CompileContext) {
    let buf = context.resume();
    context.heading_setext_buffer = Some(buf);
    context.slurp_one_line_ending = true;
}

/// Handle [`Exit`][Kind::Exit]:[`HeadingSetextUnderline`][Name::HeadingSetextUnderline].
fn on_exit_heading_setext_underline(context: &mut CompileContext) {
    let text = context
        .heading_setext_buffer
        .take()
        .expect("`atx_opening_sequence_size` must be set in headings");
    let head = Slice::from_position(
        context.bytes,
        &Position::from_exit_event(context.events, context.index),
    )
    .head();
    let rank = if head == Some(b'-') { "2" } else { "1" };

    context.line_ending_if_needed();
    context.push("<h");
    context.push(rank);
    context.push(">");
    context.push(&text);
    context.push("</h");
    context.push(rank);
    context.push(">");
}

/// Handle [`Exit`][Kind::Exit]:{[`HtmlFlow`][Name::HtmlFlow],[`HtmlText`][Name::HtmlText]}.
fn on_exit_html(context: &mut CompileContext) {
    context.encode_html = true;
}

/// Handle [`Exit`][Kind::Exit]:{[`HtmlFlowData`][Name::HtmlFlowData],[`HtmlTextData`][Name::HtmlTextData]}.
fn on_exit_html_data(context: &mut CompileContext) {
    context.push(&encode(
        Slice::from_position(
            context.bytes,
            &Position::from_exit_event(context.events, context.index),
        )
        .as_str(),
        context.encode_html,
    ));
}

/// Handle [`Exit`][Kind::Exit]:[`Label`][Name::Label].
fn on_exit_label(context: &mut CompileContext) {
    let buf = context.resume();
    context.media_stack.last_mut().unwrap().label = Some(buf);
}

/// Handle [`Exit`][Kind::Exit]:[`LabelText`][Name::LabelText].
fn on_exit_label_text(context: &mut CompileContext) {
    context.media_stack.last_mut().unwrap().label_id =
        Some(Position::from_exit_event(context.events, context.index).to_indices());
}

/// Handle [`Exit`][Kind::Exit]:[`LineEnding`][Name::LineEnding].
fn on_exit_line_ending(context: &mut CompileContext) {
    if context.code_text_inside {
        context.push(" ");
    } else if context.slurp_one_line_ending {
        context.slurp_one_line_ending = false;
    } else {
        context.push(&encode(
            Slice::from_position(
                context.bytes,
                &Position::from_exit_event(context.events, context.index),
            )
            .as_str(),
            context.encode_html,
        ));
    }
}

/// Handle [`Exit`][Kind::Exit]:{[`ListOrdered`][Name::ListOrdered],[`ListUnordered`][Name::ListUnordered]}.
fn on_exit_list(context: &mut CompileContext) {
    context.tight_stack.pop();
    context.line_ending();
    context.push(if context.events[context.index].name == Name::ListOrdered {
        "</ol>"
    } else {
        "</ul>"
    });
}

/// Handle [`Exit`][Kind::Exit]:[`ListItem`][Name::ListItem].
fn on_exit_list_item(context: &mut CompileContext) {
    let tight = context.tight_stack.last().unwrap_or(&false);
    let before_item = skip::opt_back(
        context.events,
        context.index - 1,
        &[
            Name::BlankLineEnding,
            Name::LineEnding,
            Name::SpaceOrTab,
            Name::BlockQuotePrefix,
        ],
    );
    let previous = &context.events[before_item];
    let tight_paragraph = *tight && previous.name == Name::Paragraph;
    let empty_item = previous.name == Name::ListItemPrefix;

    context.slurp_one_line_ending = false;

    if !tight_paragraph && !empty_item {
        context.line_ending_if_needed();
    }

    context.push("</li>");
}

/// Handle [`Exit`][Kind::Exit]:[`ListItemValue`][Name::ListItemValue].
fn on_exit_list_item_value(context: &mut CompileContext) {
    let expect_first_item = context.expect_first_item.unwrap();

    if expect_first_item {
        let slice = Slice::from_position(
            context.bytes,
            &Position::from_exit_event(context.events, context.index),
        );
        let value = slice.as_str().parse::<u32>().ok().unwrap();

        if value != 1 {
            context.push(" start=\"");
            context.push(&value.to_string());
            context.push("\"");
        }
    }
}

/// Handle [`Exit`][Kind::Exit]:{[`Image`][Name::Image],[`Link`][Name::Link]}.
fn on_exit_media(context: &mut CompileContext) {
    let mut is_in_image = false;
    let mut index = 0;

    // Skip current.
    let end = context.media_stack.len() - 1;
    while index < end {
        if context.media_stack[index].image {
            is_in_image = true;
            break;
        }
        index += 1;
    }

    context.in_image_alt = is_in_image;

    let media = context.media_stack.pop().unwrap();
    let label = media.label.unwrap();
    let in_image_alt = context.in_image_alt;
    let id = media.reference_id.or(media.label_id).map(|indices| {
        normalize_identifier(Slice::from_indices(context.bytes, indices.0, indices.1).as_str())
    });

    let definition_index = if media.destination.is_none() {
        id.and_then(|id| {
            let mut index = 0;

            while index < context.definitions.len() {
                if context.definitions[index].0 == id {
                    return Some(index);
                }

                index += 1;
            }

            None
        })
    } else {
        None
    };

    if !in_image_alt {
        if media.image {
            context.push("<img src=\"");
        } else {
            context.push("<a href=\"");
        };

        let destination = if let Some(index) = definition_index {
            context.definitions[index].1.destination.as_ref()
        } else {
            media.destination.as_ref()
        };

        if let Some(destination) = destination {
            context.push(&sanitize_uri(
                destination,
                if media.image {
                    &context.protocol_src
                } else {
                    &context.protocol_href
                },
            ));
        }

        if media.image {
            context.push("\" alt=\"");
        };
    }

    if media.image {
        context.push(&label);
    }

    if !in_image_alt {
        context.push("\"");

        let title = if let Some(index) = definition_index {
            context.definitions[index].1.title.clone()
        } else {
            media.title
        };

        if let Some(title) = title {
            context.push(" title=\"");
            context.push(&title);
            context.push("\"");
        };

        if media.image {
            context.push(" /");
        }

        context.push(">");
    }

    if !media.image {
        context.push(&label);

        if !in_image_alt {
            context.push("</a>");
        }
    }
}

/// Handle [`Exit`][Kind::Exit]:[`Paragraph`][Name::Paragraph].
fn on_exit_paragraph(context: &mut CompileContext) {
    let tight = context.tight_stack.last().unwrap_or(&false);

    if *tight {
        context.slurp_one_line_ending = true;
    } else {
        context.push("</p>");
    }
}

/// Handle [`Exit`][Kind::Exit]:[`ReferenceString`][Name::ReferenceString].
fn on_exit_reference_string(context: &mut CompileContext) {
    // Drop stuff.
    context.resume();

    context.media_stack.last_mut().unwrap().reference_id =
        Some(Position::from_exit_event(context.events, context.index).to_indices());
}

/// Handle [`Exit`][Kind::Exit]:[`ResourceDestinationString`][Name::ResourceDestinationString].
fn on_exit_resource_destination_string(context: &mut CompileContext) {
    let buf = context.resume();
    context.media_stack.last_mut().unwrap().destination = Some(buf);
    context.encode_html = true;
}

/// Handle [`Exit`][Kind::Exit]:[`ResourceTitleString`][Name::ResourceTitleString].
fn on_exit_resource_title_string(context: &mut CompileContext) {
    let buf = context.resume();
    context.media_stack.last_mut().unwrap().title = Some(buf);
}

/// Handle [`Exit`][Kind::Exit]:[`Strong`][Name::Strong].
fn on_exit_strong(context: &mut CompileContext) {
    if !context.in_image_alt {
        context.push("</strong>");
    }
}

/// Handle [`Exit`][Kind::Exit]:[`ThematicBreak`][Name::ThematicBreak].
fn on_exit_thematic_break(context: &mut CompileContext) {
    context.line_ending_if_needed();
    context.push("<hr />");
}
