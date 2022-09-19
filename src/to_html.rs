//! Turn events into a string of HTML.
use crate::event::{Event, Kind, Name};
use crate::util::{
    constant::{SAFE_PROTOCOL_HREF, SAFE_PROTOCOL_SRC},
    decode_character_reference::{decode_named, decode_numeric},
    encode::encode,
    gfm_tagfilter::gfm_tagfilter,
    normalize_identifier::normalize_identifier,
    sanitize_uri::{sanitize, sanitize_with_protocols},
    skip,
    slice::{Position, Slice},
};
use crate::{LineEnding, Options};
use alloc::{
    format,
    string::{String, ToString},
    vec,
    vec::Vec,
};
use core::str;

/// Link, image, or footnote call.
/// Resource or reference.
/// Reused for temporary definitions as well, in the first pass.
#[derive(Debug)]
struct Media {
    /// Whether this represents an image (`true`) or a link or definition
    /// (`false`).
    image: bool,
    /// The text between the brackets (`x` in `![x]()` and `[x]()`).
    ///
    /// Not interpreted.
    label_id: Option<(usize, usize)>,
    /// The result of interpreting the text between the brackets
    /// (`x` in `![x]()` and `[x]()`).
    ///
    /// When this is a link, it contains further text content and thus HTML
    /// tags.
    /// Otherwise, when an image, text content is also allowed, but resulting
    /// tags are ignored.
    label: Option<String>,
    /// The string between the explicit brackets of the reference (`y` in
    /// `[x][y]`), as content.
    ///
    /// Not interpreted.
    reference_id: Option<(usize, usize)>,
    /// The destination (url).
    ///
    /// Interpreted string content.
    destination: Option<String>,
    /// The destination (url).
    ///
    /// Interpreted string content.
    title: Option<String>,
}

/// Representation of a definition.
#[derive(Debug)]
struct Definition {
    /// Identifier.
    id: String,
    /// The destination (url).
    ///
    /// Interpreted string content.
    destination: Option<String>,
    /// The title.
    ///
    /// Interpreted string content.
    title: Option<String>,
}

/// GFM table: column alignment.
// To do: share with `mdast`.
#[derive(Debug, PartialEq, Eq, Copy, Clone)]
enum GfmTableAlign {
    /// No alignment.
    ///
    /// ```markdown
    ///   | | aaa |
    /// > | | --- |
    ///       ^^^
    /// ```
    None,
    /// Left alignment.
    ///
    /// ```markdown
    ///   | | aaa |
    /// > | | :-- |
    ///       ^^^
    /// ```
    Left,
    /// Center alignment.
    ///
    /// ```markdown
    ///   | | aaa |
    /// > | | :-: |
    ///       ^^^
    /// ```
    Center,
    /// Right alignment.
    ///
    /// ```markdown
    ///   | | aaa |
    /// > | | --: |
    ///       ^^^
    /// ```
    Right,
}

/// Context used to compile markdown.
#[allow(clippy::struct_excessive_bools)]
#[derive(Debug)]
struct CompileContext<'a> {
    // Static info.
    /// List of events.
    pub events: &'a [Event],
    /// List of bytes.
    pub bytes: &'a [u8],
    /// Configuration.
    pub options: &'a Options,
    // Fields used by handlers to track the things they need to track to
    // compile markdown.
    /// Rank of heading (atx).
    pub heading_atx_rank: Option<usize>,
    /// Buffer of heading (setext) text.
    pub heading_setext_buffer: Option<String>,
    /// Whether raw (flow) (code (fenced), math (flow)) or code (indented) contains data.
    pub raw_flow_seen_data: Option<bool>,
    /// Number of raw (flow) fences.
    pub raw_flow_fences_count: Option<usize>,
    /// Whether we are in code (text).
    pub raw_text_inside: bool,
    /// Whether we are in image text.
    pub image_alt_inside: bool,
    /// Marker of character reference.
    pub character_reference_marker: Option<u8>,
    /// Whether we are expecting the first list item marker.
    pub list_expect_first_marker: Option<bool>,
    /// Stack of media (link, image).
    pub media_stack: Vec<Media>,
    /// Stack of containers.
    pub tight_stack: Vec<bool>,
    /// List of definitions.
    pub definitions: Vec<Definition>,
    /// List of definitions.
    pub gfm_footnote_definitions: Vec<(String, String)>,
    pub gfm_footnote_definition_calls: Vec<(String, usize)>,
    pub gfm_footnote_definition_stack: Vec<(usize, usize)>,
    /// Whether we are in a GFM table head.
    pub gfm_table_in_head: bool,
    /// Current GFM table alignment.
    pub gfm_table_align: Option<Vec<GfmTableAlign>>,
    /// Current GFM table column.
    pub gfm_table_column: usize,
    // Fields used to influance the current compilation.
    /// Ignore the next line ending.
    pub slurp_one_line_ending: bool,
    /// Whether to encode HTML.
    pub encode_html: bool,
    // Configuration
    /// Line ending to use.
    pub line_ending_default: LineEnding,
    // Intermediate results.
    /// Stack of buffers.
    pub buffers: Vec<String>,
    /// Current event index.
    pub index: usize,
}

impl<'a> CompileContext<'a> {
    /// Create a new compile context.
    pub fn new(
        events: &'a [Event],
        bytes: &'a [u8],
        options: &'a Options,
        line_ending: LineEnding,
    ) -> CompileContext<'a> {
        CompileContext {
            events,
            bytes,
            heading_atx_rank: None,
            heading_setext_buffer: None,
            raw_flow_seen_data: None,
            raw_flow_fences_count: None,
            raw_text_inside: false,
            character_reference_marker: None,
            list_expect_first_marker: None,
            media_stack: vec![],
            definitions: vec![],
            gfm_footnote_definitions: vec![],
            gfm_footnote_definition_calls: vec![],
            gfm_footnote_definition_stack: vec![],
            gfm_table_in_head: false,
            gfm_table_align: None,
            gfm_table_column: 0,
            tight_stack: vec![],
            slurp_one_line_ending: false,
            image_alt_inside: false,
            encode_html: true,
            line_ending_default: line_ending,
            buffers: vec![String::new()],
            index: 0,
            options,
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

/// Turn events and bytes into a string of HTML.
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
    let line_ending_default =
        line_ending_inferred.unwrap_or_else(|| options.default_line_ending.clone());

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
    //
    // We don’t need to handle GFM footnote definitions like this, because
    // unlike normal definitions, what they produce is not used in calls.
    // It would also get very complex, because footnote definitions can be
    // nested.
    while index < events.len() {
        let event = &events[index];

        if definition_inside {
            handle(&mut context, index);
        }

        if event.kind == Kind::Enter {
            if event.name == Name::Definition {
                handle(&mut context, index); // Also handle start.
                definition_inside = true;
                definition_indices.push((index, index));
            }
        } else if event.name == Name::Definition {
            definition_inside = false;
            definition_indices.last_mut().unwrap().1 = index;
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
        } else {
            handle(&mut context, index);
            index += 1;
        }
    }

    // No section to generate.
    if !context.gfm_footnote_definition_calls.is_empty() {
        generate_footnote_section(&mut context);
    }

    debug_assert_eq!(context.buffers.len(), 1, "expected 1 final buffer");
    context
        .buffers
        .get(0)
        .expect("expected 1 final buffer")
        .to_string()
}

/// Handle the event at `index`.
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
        | Name::MathFlowFenceMeta
        | Name::DefinitionLabelString
        | Name::DefinitionTitleString
        | Name::GfmFootnoteDefinitionPrefix
        | Name::HeadingAtxText
        | Name::HeadingSetextText
        | Name::Label
        | Name::MdxEsm
        | Name::MdxFlowExpression
        | Name::MdxTextExpression
        | Name::MdxJsxFlowTag
        | Name::MdxJsxTextTag
        | Name::ReferenceString
        | Name::ResourceTitleString => on_enter_buffer(context),

        Name::BlockQuote => on_enter_block_quote(context),
        Name::CodeIndented => on_enter_code_indented(context),
        Name::CodeFenced | Name::MathFlow => on_enter_raw_flow(context),
        Name::CodeText | Name::MathText => on_enter_raw_text(context),
        Name::Definition => on_enter_definition(context),
        Name::DefinitionDestinationString => on_enter_definition_destination_string(context),
        Name::Emphasis => on_enter_emphasis(context),
        Name::Frontmatter => on_enter_frontmatter(context),
        Name::GfmFootnoteDefinition => on_enter_gfm_footnote_definition(context),
        Name::GfmFootnoteCall => on_enter_gfm_footnote_call(context),
        Name::GfmStrikethrough => on_enter_gfm_strikethrough(context),
        Name::GfmTable => on_enter_gfm_table(context),
        Name::GfmTableBody => on_enter_gfm_table_body(context),
        Name::GfmTableCell => on_enter_gfm_table_cell(context),
        Name::GfmTableHead => on_enter_gfm_table_head(context),
        Name::GfmTableRow => on_enter_gfm_table_row(context),
        Name::GfmTaskListItemCheck => on_enter_gfm_task_list_item_check(context),
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
        Name::CodeFencedFenceMeta
        | Name::MathFlowFenceMeta
        | Name::MdxJsxTextTag
        | Name::MdxTextExpression
        | Name::Resource => {
            on_exit_drop(context);
        }
        Name::MdxEsm | Name::MdxFlowExpression | Name::MdxJsxFlowTag => on_exit_drop_slurp(context),
        Name::CharacterEscapeValue | Name::CodeTextData | Name::Data | Name::MathTextData => {
            on_exit_data(context);
        }
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
        Name::CodeFenced | Name::CodeIndented | Name::MathFlow => on_exit_raw_flow(context),
        Name::CodeFencedFence | Name::MathFlowFence => on_exit_raw_flow_fence(context),
        Name::CodeFencedFenceInfo => on_exit_raw_flow_fence_info(context),
        Name::CodeFlowChunk | Name::MathFlowChunk => on_exit_raw_flow_chunk(context),
        Name::CodeText | Name::MathText => on_exit_raw_text(context),
        Name::Definition => on_exit_definition(context),
        Name::DefinitionDestinationString => on_exit_definition_destination_string(context),
        Name::DefinitionLabelString => on_exit_definition_label_string(context),
        Name::DefinitionTitleString => on_exit_definition_title_string(context),
        Name::Emphasis => on_exit_emphasis(context),
        Name::Frontmatter => on_exit_frontmatter(context),
        Name::GfmAutolinkLiteralEmail => on_exit_gfm_autolink_literal_email(context),
        Name::GfmAutolinkLiteralMailto => on_exit_gfm_autolink_literal_mailto(context),
        Name::GfmAutolinkLiteralProtocol => on_exit_gfm_autolink_literal_protocol(context),
        Name::GfmAutolinkLiteralWww => on_exit_gfm_autolink_literal_www(context),
        Name::GfmAutolinkLiteralXmpp => on_exit_gfm_autolink_literal_xmpp(context),
        Name::GfmFootnoteCall => on_exit_gfm_footnote_call(context),
        Name::GfmFootnoteDefinitionLabelString => {
            on_exit_gfm_footnote_definition_label_string(context);
        }
        Name::GfmFootnoteDefinitionPrefix => on_exit_gfm_footnote_definition_prefix(context),
        Name::GfmFootnoteDefinition => on_exit_gfm_footnote_definition(context),
        Name::GfmStrikethrough => on_exit_gfm_strikethrough(context),
        Name::GfmTable => on_exit_gfm_table(context),
        Name::GfmTableBody => on_exit_gfm_table_body(context),
        Name::GfmTableCell => on_exit_gfm_table_cell(context),
        Name::GfmTableHead => on_exit_gfm_table_head(context),
        Name::GfmTableRow => on_exit_gfm_table_row(context),
        Name::GfmTaskListItemCheck => on_exit_gfm_task_list_item_check(context),
        Name::GfmTaskListItemValueChecked => on_exit_gfm_task_list_item_value_checked(context),
        Name::HardBreakEscape | Name::HardBreakTrailing => on_exit_break(context),
        Name::HeadingAtx => on_exit_heading_atx(context),
        Name::HeadingAtxSequence => on_exit_heading_atx_sequence(context),
        Name::HeadingAtxText => on_exit_heading_atx_text(context),
        Name::HeadingSetextText => on_exit_heading_setext_text(context),
        Name::HeadingSetextUnderlineSequence => on_exit_heading_setext_underline_sequence(context),
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
    context.raw_flow_seen_data = Some(false);
    context.line_ending_if_needed();
    context.push("<pre><code>");
}

/// Handle [`Enter`][Kind::Enter]:{[`CodeFenced`][Name::CodeFenced],[`MathFlow`][Name::MathFlow]}.
fn on_enter_raw_flow(context: &mut CompileContext) {
    context.raw_flow_seen_data = Some(false);
    context.line_ending_if_needed();
    // Note that no `>` is used, which is added later (due to info)
    context.push("<pre><code");
    context.raw_flow_fences_count = Some(0);

    if context.events[context.index].name == Name::MathFlow {
        context.push(" class=\"language-math math-display\"");
    }
}

/// Handle [`Enter`][Kind::Enter]:{[`CodeText`][Name::CodeText],[`MathText`][Name::MathText]}.
fn on_enter_raw_text(context: &mut CompileContext) {
    context.raw_text_inside = true;
    if !context.image_alt_inside {
        context.push("<code");
        if context.events[context.index].name == Name::MathText {
            context.push(" class=\"language-math math-inline\"");
        }
        context.push(">");
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
    if !context.image_alt_inside {
        context.push("<em>");
    }
}

/// Handle [`Enter`][Kind::Enter]:[`Frontmatter`][Name::Frontmatter].
fn on_enter_frontmatter(context: &mut CompileContext) {
    context.buffer();
}

/// Handle [`Enter`][Kind::Enter]:[`GfmFootnoteDefinition`][Name::GfmFootnoteDefinition].
fn on_enter_gfm_footnote_definition(context: &mut CompileContext) {
    context.tight_stack.push(false);
}

/// Handle [`Enter`][Kind::Enter]:[`GfmFootnoteCall`][Name::GfmFootnoteCall].
fn on_enter_gfm_footnote_call(context: &mut CompileContext) {
    context.media_stack.push(Media {
        image: false,
        label_id: None,
        label: None,
        reference_id: None,
        destination: None,
        title: None,
    });
}

/// Handle [`Enter`][Kind::Enter]:[`GfmStrikethrough`][Name::GfmStrikethrough].
fn on_enter_gfm_strikethrough(context: &mut CompileContext) {
    if !context.image_alt_inside {
        context.push("<del>");
    }
}

/// Handle [`Enter`][Kind::Enter]:[`GfmTable`][Name::GfmTable].
fn on_enter_gfm_table(context: &mut CompileContext) {
    // Find the alignment.
    let mut index = context.index;
    let mut in_delimiter_row = false;
    let mut align = vec![];

    while index < context.events.len() {
        let event = &context.events[index];

        if in_delimiter_row {
            if event.kind == Kind::Enter {
                // Start of alignment value: set a new column.
                if event.name == Name::GfmTableDelimiterCellValue {
                    align.push(
                        if context.events[index + 1].name == Name::GfmTableDelimiterMarker {
                            GfmTableAlign::Left
                        } else {
                            GfmTableAlign::None
                        },
                    );
                }
            } else {
                // End of alignment value: change the column.
                if event.name == Name::GfmTableDelimiterCellValue {
                    if context.events[index - 1].name == Name::GfmTableDelimiterMarker {
                        let align_index = align.len() - 1;
                        align[align_index] = if align[align_index] == GfmTableAlign::Left {
                            GfmTableAlign::Center
                        } else {
                            GfmTableAlign::Right
                        }
                    }
                }
                // Done!
                else if event.name == Name::GfmTableDelimiterRow {
                    break;
                }
            }
        } else if event.kind == Kind::Enter && event.name == Name::GfmTableDelimiterRow {
            in_delimiter_row = true;
        }

        index += 1;
    }

    // Generate.
    context.gfm_table_align = Some(align);
    context.line_ending_if_needed();
    context.push("<table>");
}

/// Handle [`Enter`][Kind::Enter]:[`GfmTableBody`][Name::GfmTableBody].
fn on_enter_gfm_table_body(context: &mut CompileContext) {
    context.push("<tbody>");
}

/// Handle [`Enter`][Kind::Enter]:[`GfmTableCell`][Name::GfmTableCell].
fn on_enter_gfm_table_cell(context: &mut CompileContext) {
    let column = context.gfm_table_column;
    let align = context.gfm_table_align.as_ref().unwrap();

    if column >= align.len() {
        // Capture cell to ignore it.
        context.buffer();
    } else {
        let value = align[column];
        context.line_ending_if_needed();

        if context.gfm_table_in_head {
            context.push("<th");
        } else {
            context.push("<td");
        }

        match value {
            GfmTableAlign::Left => context.push(" align=\"left\""),
            GfmTableAlign::Right => context.push(" align=\"right\""),
            GfmTableAlign::Center => context.push(" align=\"center\""),
            GfmTableAlign::None => {}
        }

        context.push(">");
    }
}

/// Handle [`Enter`][Kind::Enter]:[`GfmTableHead`][Name::GfmTableHead].
fn on_enter_gfm_table_head(context: &mut CompileContext) {
    context.line_ending_if_needed();
    context.push("<thead>");
    context.gfm_table_in_head = true;
}

/// Handle [`Enter`][Kind::Enter]:[`GfmTableRow`][Name::GfmTableRow].
fn on_enter_gfm_table_row(context: &mut CompileContext) {
    context.line_ending_if_needed();
    context.push("<tr>");
}

/// Handle [`Enter`][Kind::Enter]:[`GfmTaskListItemCheck`][Name::GfmTaskListItemCheck].
fn on_enter_gfm_task_list_item_check(context: &mut CompileContext) {
    if !context.image_alt_inside {
        context.push("<input type=\"checkbox\" disabled=\"\" ");
    }
}

/// Handle [`Enter`][Kind::Enter]:[`HtmlFlow`][Name::HtmlFlow].
fn on_enter_html_flow(context: &mut CompileContext) {
    context.line_ending_if_needed();
    if context.options.allow_dangerous_html {
        context.encode_html = false;
    }
}

/// Handle [`Enter`][Kind::Enter]:[`HtmlText`][Name::HtmlText].
fn on_enter_html_text(context: &mut CompileContext) {
    if context.options.allow_dangerous_html {
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
    context.image_alt_inside = true; // Disallow tags.
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
    context.list_expect_first_marker = Some(true);
}

/// Handle [`Enter`][Kind::Enter]:[`ListItemMarker`][Name::ListItemMarker].
fn on_enter_list_item_marker(context: &mut CompileContext) {
    if context.list_expect_first_marker.take().unwrap() {
        context.push(">");
    }

    context.line_ending_if_needed();

    context.push("<li>");
    context.list_expect_first_marker = Some(false);
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
    if !context.image_alt_inside {
        context.push("<strong>");
    }
}

/// Handle [`Exit`][Kind::Exit]:[`AutolinkEmail`][Name::AutolinkEmail].
fn on_exit_autolink_email(context: &mut CompileContext) {
    generate_autolink(
        context,
        Some("mailto:"),
        Slice::from_position(
            context.bytes,
            &Position::from_exit_event(context.events, context.index),
        )
        .as_str(),
        false,
    );
}

/// Handle [`Exit`][Kind::Exit]:[`AutolinkProtocol`][Name::AutolinkProtocol].
fn on_exit_autolink_protocol(context: &mut CompileContext) {
    generate_autolink(
        context,
        None,
        Slice::from_position(
            context.bytes,
            &Position::from_exit_event(context.events, context.index),
        )
        .as_str(),
        false,
    );
}

/// Handle [`Exit`][Kind::Exit]:{[`HardBreakEscape`][Name::HardBreakEscape],[`HardBreakTrailing`][Name::HardBreakTrailing]}.
fn on_exit_break(context: &mut CompileContext) {
    if !context.image_alt_inside {
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

/// Handle [`Exit`][Kind::Exit]:{[`CodeFlowChunk`][Name::CodeFlowChunk],[`MathFlowChunk`][Name::MathFlowChunk]}.
fn on_exit_raw_flow_chunk(context: &mut CompileContext) {
    context.raw_flow_seen_data = Some(true);
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

/// Handle [`Exit`][Kind::Exit]:{[`CodeFencedFence`][Name::CodeFencedFence],[`MathFlowFence`][Name::MathFlowFence]}.
fn on_exit_raw_flow_fence(context: &mut CompileContext) {
    let count = if let Some(count) = context.raw_flow_fences_count {
        count
    } else {
        0
    };

    if count == 0 {
        context.push(">");
        context.slurp_one_line_ending = true;
    }

    context.raw_flow_fences_count = Some(count + 1);
}

/// Handle [`Exit`][Kind::Exit]:[`CodeFencedFenceInfo`][Name::CodeFencedFenceInfo].
///
/// Note: math (flow) does not support `info`.
fn on_exit_raw_flow_fence_info(context: &mut CompileContext) {
    let value = context.resume();
    context.push(" class=\"language-");
    context.push(&value);
    context.push("\"");
}

/// Handle [`Exit`][Kind::Exit]:{[`CodeFenced`][Name::CodeFenced],[`CodeIndented`][Name::CodeIndented],[`MathFlow`][Name::MathFlow]}.
fn on_exit_raw_flow(context: &mut CompileContext) {
    // One special case is if we are inside a container, and the raw (flow) was
    // not closed (meaning it runs to the end).
    // In that case, the following line ending, is considered *outside* the
    // fenced code and block quote by micromark, but CM wants to treat that
    // ending as part of the code.
    if let Some(count) = context.raw_flow_fences_count {
        // No closing fence.
        if count == 1
            // In a container.
            && !context.tight_stack.is_empty()
            // Empty (as the closing is right at the opening fence)
            && !matches!(context.events[context.index - 1].name, Name::CodeFencedFence | Name::MathFlowFence)
        {
            context.line_ending();
        }
    }

    // But in most cases, it’s simpler: when we’ve seen some data, emit an extra
    // line ending when needed.
    if context
        .raw_flow_seen_data
        .take()
        .expect("`raw_flow_seen_data` must be defined")
    {
        context.line_ending_if_needed();
    }

    context.push("</code></pre>");

    if let Some(count) = context.raw_flow_fences_count.take() {
        if count < 2 {
            context.line_ending_if_needed();
        }
    }

    context.slurp_one_line_ending = false;
}

/// Handle [`Exit`][Kind::Exit]:{[`CodeText`][Name::CodeText],[`MathText`][Name::MathText]}.
fn on_exit_raw_text(context: &mut CompileContext) {
    let result = context.resume();
    let mut bytes = result.as_bytes().to_vec();

    // If we are in a GFM table, we need to decode escaped pipes.
    // This is a rather weird GFM feature.
    if context.gfm_table_align.is_some() {
        let mut index = 0;
        let mut len = bytes.len();

        while index < len {
            if index + 1 < len && bytes[index] == b'\\' && bytes[index + 1] == b'|' {
                bytes.remove(index);
                len -= 1;
            }

            index += 1;
        }
    }

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
        bytes.remove(0);
        bytes.pop();
    }

    context.raw_text_inside = false;
    context.push(str::from_utf8(&bytes).unwrap());

    if !context.image_alt_inside {
        context.push("</code>");
    }
}

/// Handle [`Exit`][Kind::Exit]:*.
///
/// Resumes, and ignores what was resumed.
fn on_exit_drop(context: &mut CompileContext) {
    context.resume();
}

/// Handle [`Exit`][Kind::Exit]:*.
///
/// Resumes, ignores what was resumed, and slurps the following line ending.
fn on_exit_drop_slurp(context: &mut CompileContext) {
    context.resume();
    context.slurp_one_line_ending = true;
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

    context.definitions.push(Definition {
        id,
        destination: media.destination,
        title: media.title,
    });
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

/// Handle [`Exit`][Kind::Exit]:[`Emphasis`][Name::Emphasis].
fn on_exit_emphasis(context: &mut CompileContext) {
    if !context.image_alt_inside {
        context.push("</em>");
    }
}

/// Handle [`Exit`][Kind::Exit]:[`Frontmatter`][Name::Frontmatter].
fn on_exit_frontmatter(context: &mut CompileContext) {
    context.resume();
    context.slurp_one_line_ending = true;
}

/// Handle [`Exit`][Kind::Exit]:[`GfmAutolinkLiteralEmail`][Name::GfmAutolinkLiteralEmail].
fn on_exit_gfm_autolink_literal_email(context: &mut CompileContext) {
    generate_autolink(
        context,
        Some("mailto:"),
        Slice::from_position(
            context.bytes,
            &Position::from_exit_event(context.events, context.index),
        )
        .as_str(),
        true,
    );
}

/// Handle [`Exit`][Kind::Exit]:[`GfmAutolinkLiteralMailto`][Name::GfmAutolinkLiteralMailto].
fn on_exit_gfm_autolink_literal_mailto(context: &mut CompileContext) {
    generate_autolink(
        context,
        None,
        Slice::from_position(
            context.bytes,
            &Position::from_exit_event(context.events, context.index),
        )
        .as_str(),
        true,
    );
}

/// Handle [`Exit`][Kind::Exit]:[`GfmAutolinkLiteralProtocol`][Name::GfmAutolinkLiteralProtocol].
fn on_exit_gfm_autolink_literal_protocol(context: &mut CompileContext) {
    generate_autolink(
        context,
        None,
        Slice::from_position(
            context.bytes,
            &Position::from_exit_event(context.events, context.index),
        )
        .as_str(),
        true,
    );
}

/// Handle [`Exit`][Kind::Exit]:[`GfmAutolinkLiteralWww`][Name::GfmAutolinkLiteralWww].
fn on_exit_gfm_autolink_literal_www(context: &mut CompileContext) {
    generate_autolink(
        context,
        Some("http://"),
        Slice::from_position(
            context.bytes,
            &Position::from_exit_event(context.events, context.index),
        )
        .as_str(),
        true,
    );
}

/// Handle [`Exit`][Kind::Exit]:[`GfmAutolinkLiteralXmpp`][Name::GfmAutolinkLiteralXmpp].
fn on_exit_gfm_autolink_literal_xmpp(context: &mut CompileContext) {
    generate_autolink(
        context,
        None,
        Slice::from_position(
            context.bytes,
            &Position::from_exit_event(context.events, context.index),
        )
        .as_str(),
        true,
    );
}

/// Handle [`Exit`][Kind::Exit]:[`GfmFootnoteCall`][Name::GfmFootnoteCall].
fn on_exit_gfm_footnote_call(context: &mut CompileContext) {
    let indices = context.media_stack.pop().unwrap().label_id.unwrap();
    let id =
        normalize_identifier(Slice::from_indices(context.bytes, indices.0, indices.1).as_str());
    let safe_id = sanitize(&id.to_lowercase());
    let mut call_index = 0;

    // See if this has been called before.
    while call_index < context.gfm_footnote_definition_calls.len() {
        if context.gfm_footnote_definition_calls[call_index].0 == id {
            break;
        }
        call_index += 1;
    }

    // New.
    if call_index == context.gfm_footnote_definition_calls.len() {
        context.gfm_footnote_definition_calls.push((id, 0));
    }

    // Increment.
    context.gfm_footnote_definition_calls[call_index].1 += 1;

    // No call is output in an image alt, though the definition and
    // backreferences are generated as if it was the case.
    if context.image_alt_inside {
        return;
    }

    context.push("<sup><a href=\"#");
    if let Some(ref value) = context.options.gfm_footnote_clobber_prefix {
        context.push(&encode(value, context.encode_html));
    } else {
        context.push("user-content-");
    }
    context.push("fn-");
    context.push(&safe_id);
    context.push("\" id=\"");
    if let Some(ref value) = context.options.gfm_footnote_clobber_prefix {
        context.push(&encode(value, context.encode_html));
    } else {
        context.push("user-content-");
    }
    context.push("fnref-");
    context.push(&safe_id);
    if context.gfm_footnote_definition_calls[call_index].1 > 1 {
        context.push("-");
        context.push(
            &context.gfm_footnote_definition_calls[call_index]
                .1
                .to_string(),
        );
    }
    context.push("\" data-footnote-ref=\"\" aria-describedby=\"footnote-label\">");

    context.push(&(call_index + 1).to_string());
    context.push("</a></sup>");
}

/// Handle [`Exit`][Kind::Exit]:[`GfmFootnoteDefinitionLabelString`][Name::GfmFootnoteDefinitionLabelString].
fn on_exit_gfm_footnote_definition_label_string(context: &mut CompileContext) {
    context
        .gfm_footnote_definition_stack
        .push(Position::from_exit_event(context.events, context.index).to_indices());
}

/// Handle [`Exit`][Kind::Exit]:[`GfmFootnoteDefinitionPrefix`][Name::GfmFootnoteDefinitionPrefix].
fn on_exit_gfm_footnote_definition_prefix(context: &mut CompileContext) {
    // Drop the prefix.
    context.resume();
    // Capture everything until end of definition.
    context.buffer();
}

/// Handle [`Exit`][Kind::Exit]:[`GfmFootnoteDefinition`][Name::GfmFootnoteDefinition].
fn on_exit_gfm_footnote_definition(context: &mut CompileContext) {
    let value = context.resume();
    let indices = context.gfm_footnote_definition_stack.pop().unwrap();
    context.tight_stack.pop();
    context.gfm_footnote_definitions.push((
        normalize_identifier(Slice::from_indices(context.bytes, indices.0, indices.1).as_str()),
        value,
    ));
}

/// Handle [`Exit`][Kind::Exit]:[`GfmStrikethrough`][Name::GfmStrikethrough].
fn on_exit_gfm_strikethrough(context: &mut CompileContext) {
    if !context.image_alt_inside {
        context.push("</del>");
    }
}

/// Handle [`Exit`][Kind::Exit]:[`GfmTable`][Name::GfmTable].
fn on_exit_gfm_table(context: &mut CompileContext) {
    context.gfm_table_align = None;
    context.line_ending_if_needed();
    context.push("</table>");
}

/// Handle [`Exit`][Kind::Exit]:[`GfmTableBody`][Name::GfmTableBody].
fn on_exit_gfm_table_body(context: &mut CompileContext) {
    context.line_ending_if_needed();
    context.push("</tbody>");
}

/// Handle [`Exit`][Kind::Exit]:[`GfmTableCell`][Name::GfmTableCell].
fn on_exit_gfm_table_cell(context: &mut CompileContext) {
    let align = context.gfm_table_align.as_ref().unwrap();

    if context.gfm_table_column < align.len() {
        if context.gfm_table_in_head {
            context.push("</th>");
        } else {
            context.push("</td>");
        }
    } else {
        // Stop capturing.
        context.resume();
    }

    context.gfm_table_column += 1;
}

/// Handle [`Exit`][Kind::Exit]:[`GfmTableHead`][Name::GfmTableHead].
fn on_exit_gfm_table_head(context: &mut CompileContext) {
    context.gfm_table_in_head = false;
    context.line_ending_if_needed();
    context.push("</thead>");
}

/// Handle [`Exit`][Kind::Exit]:[`GfmTableRow`][Name::GfmTableRow].
fn on_exit_gfm_table_row(context: &mut CompileContext) {
    let mut column = context.gfm_table_column;
    let len = context.gfm_table_align.as_ref().unwrap().len();

    // Add “phantom” cells, for body rows that are shorter than the delimiter
    // row (which is equal to the head row).
    while column < len {
        on_enter_gfm_table_cell(context);
        on_exit_gfm_table_cell(context);
        column += 1;
    }

    context.gfm_table_column = 0;
    context.line_ending_if_needed();
    context.push("</tr>");
}

/// Handle [`Exit`][Kind::Exit]:[`GfmTaskListItemCheck`][Name::GfmTaskListItemCheck].
fn on_exit_gfm_task_list_item_check(context: &mut CompileContext) {
    if !context.image_alt_inside {
        context.push("/>");
    }
}

/// Handle [`Exit`][Kind::Exit]:[`GfmTaskListItemValueChecked`][Name::GfmTaskListItemValueChecked].
fn on_exit_gfm_task_list_item_value_checked(context: &mut CompileContext) {
    if !context.image_alt_inside {
        context.push("checked=\"\" ");
    }
}

/// Handle [`Exit`][Kind::Exit]:[`HeadingAtx`][Name::HeadingAtx].
fn on_exit_heading_atx(context: &mut CompileContext) {
    let rank = context
        .heading_atx_rank
        .take()
        .expect("`heading_atx_rank` must be set in headings");

    context.push("</h");
    context.push(&rank.to_string());
    context.push(">");
}

/// Handle [`Exit`][Kind::Exit]:[`HeadingAtxSequence`][Name::HeadingAtxSequence].
fn on_exit_heading_atx_sequence(context: &mut CompileContext) {
    // First fence we see.
    if context.heading_atx_rank.is_none() {
        let rank = Slice::from_position(
            context.bytes,
            &Position::from_exit_event(context.events, context.index),
        )
        .len();
        context.line_ending_if_needed();
        context.heading_atx_rank = Some(rank);
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

/// Handle [`Exit`][Kind::Exit]:[`HeadingSetextUnderlineSequence`][Name::HeadingSetextUnderlineSequence].
fn on_exit_heading_setext_underline_sequence(context: &mut CompileContext) {
    let text = context
        .heading_setext_buffer
        .take()
        .expect("`heading_atx_rank` must be set in headings");
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
    let slice = Slice::from_position(
        context.bytes,
        &Position::from_exit_event(context.events, context.index),
    );
    let value = slice.as_str();

    let encoded = if context.options.gfm_tagfilter && context.options.allow_dangerous_html {
        encode(&gfm_tagfilter(value), context.encode_html)
    } else {
        encode(value, context.encode_html)
    };

    context.push(&encoded);
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
    if context.raw_text_inside {
        context.push(" ");
    } else if context.slurp_one_line_ending
        // Ignore line endings after definitions.
        || (context.index > 1
            && (context.events[context.index - 2].name == Name::Definition
                || context.events[context.index - 2].name == Name::GfmFootnoteDefinition))
    {
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
            Name::BlockQuotePrefix,
            Name::LineEnding,
            Name::SpaceOrTab,
            // Also ignore things that don’t contribute to the document.
            Name::Definition,
            Name::GfmFootnoteDefinition,
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
    if context.list_expect_first_marker.unwrap() {
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

    context.image_alt_inside = is_in_image;

    let media = context.media_stack.pop().unwrap();
    let label = media.label.unwrap();
    let id = media.reference_id.or(media.label_id).map(|indices| {
        normalize_identifier(Slice::from_indices(context.bytes, indices.0, indices.1).as_str())
    });

    let definition_index = if media.destination.is_none() {
        id.and_then(|id| {
            let mut index = 0;

            while index < context.definitions.len() {
                if context.definitions[index].id == id {
                    return Some(index);
                }

                index += 1;
            }

            None
        })
    } else {
        None
    };

    if !is_in_image {
        if media.image {
            context.push("<img src=\"");
        } else {
            context.push("<a href=\"");
        };

        let destination = if let Some(index) = definition_index {
            context.definitions[index].destination.as_ref()
        } else {
            media.destination.as_ref()
        };

        if let Some(destination) = destination {
            let url = if context.options.allow_dangerous_protocol {
                sanitize(destination)
            } else {
                sanitize_with_protocols(
                    destination,
                    if media.image {
                        &SAFE_PROTOCOL_SRC
                    } else {
                        &SAFE_PROTOCOL_HREF
                    },
                )
            };
            context.push(&url);
        }

        if media.image {
            context.push("\" alt=\"");
        };
    }

    if media.image {
        context.push(&label);
    }

    if !is_in_image {
        context.push("\"");

        let title = if let Some(index) = definition_index {
            context.definitions[index].title.clone()
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

        if !is_in_image {
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
    if !context.image_alt_inside {
        context.push("</strong>");
    }
}

/// Handle [`Exit`][Kind::Exit]:[`ThematicBreak`][Name::ThematicBreak].
fn on_exit_thematic_break(context: &mut CompileContext) {
    context.line_ending_if_needed();
    context.push("<hr />");
}

/// Generate a footnote section.
fn generate_footnote_section(context: &mut CompileContext) {
    context.line_ending_if_needed();
    context.push("<section data-footnotes=\"\" class=\"footnotes\"><");
    if let Some(ref value) = context.options.gfm_footnote_label_tag_name {
        context.push(&encode(value, context.encode_html));
    } else {
        context.push("h2");
    }
    context.push(" id=\"footnote-label\" ");
    if let Some(ref value) = context.options.gfm_footnote_label_attributes {
        context.push(value);
    } else {
        context.push("class=\"sr-only\"");
    }
    context.push(">");
    if let Some(ref value) = context.options.gfm_footnote_label {
        context.push(&encode(value, context.encode_html));
    } else {
        context.push("Footnotes");
    }
    context.push("</");
    if let Some(ref value) = context.options.gfm_footnote_label_tag_name {
        context.push(&encode(value, context.encode_html));
    } else {
        context.push("h2");
    }
    context.push(">");
    context.line_ending();
    context.push("<ol>");

    let mut index = 0;
    while index < context.gfm_footnote_definition_calls.len() {
        generate_footnote_item(context, index);
        index += 1;
    }

    context.line_ending();
    context.push("</ol>");
    context.line_ending();
    context.push("</section>");
    context.line_ending();
}

/// Generate a footnote item from a call.
fn generate_footnote_item(context: &mut CompileContext, index: usize) {
    let id = &context.gfm_footnote_definition_calls[index].0;
    let safe_id = sanitize(&id.to_lowercase());

    // Find definition: we’ll always find it.
    let mut definition_index = 0;
    while definition_index < context.gfm_footnote_definitions.len() {
        if &context.gfm_footnote_definitions[definition_index].0 == id {
            break;
        }
        definition_index += 1;
    }

    debug_assert_ne!(
        definition_index,
        context.gfm_footnote_definitions.len(),
        "expected definition"
    );

    context.line_ending();
    context.push("<li id=\"");
    if let Some(ref value) = context.options.gfm_footnote_clobber_prefix {
        context.push(&encode(value, context.encode_html));
    } else {
        context.push("user-content-");
    }
    context.push("fn-");
    context.push(&safe_id);
    context.push("\">");
    context.line_ending();

    // Create one or more backreferences.
    let mut reference_index = 0;
    let mut backreferences = String::new();
    while reference_index < context.gfm_footnote_definition_calls[index].1 {
        if reference_index != 0 {
            backreferences.push(' ');
        }
        backreferences.push_str("<a href=\"#");
        if let Some(ref value) = context.options.gfm_footnote_clobber_prefix {
            backreferences.push_str(&encode(value, context.encode_html));
        } else {
            backreferences.push_str("user-content-");
        }
        backreferences.push_str("fnref-");
        backreferences.push_str(&safe_id);
        if reference_index != 0 {
            backreferences.push('-');
            backreferences.push_str(&(reference_index + 1).to_string());
        }
        backreferences.push_str("\" data-footnote-backref=\"\" aria-label=\"");
        if let Some(ref value) = context.options.gfm_footnote_back_label {
            backreferences.push_str(&encode(value, context.encode_html));
        } else {
            backreferences.push_str("Back to content");
        }
        backreferences.push_str("\" class=\"data-footnote-backref\">↩");
        if reference_index != 0 {
            backreferences.push_str("<sup>");
            backreferences.push_str(&(reference_index + 1).to_string());
            backreferences.push_str("</sup>");
        }
        backreferences.push_str("</a>");

        reference_index += 1;
    }

    let value = context.gfm_footnote_definitions[definition_index].1.clone();
    let bytes = value.as_bytes();
    let mut byte_index = bytes.len();
    // Move back past EOL.
    while byte_index > 0 && matches!(bytes[byte_index - 1], b'\n' | b'\r') {
        byte_index -= 1;
    }
    // Check if it ends in `</p>`.
    // This is a bit funky if someone wrote a safe paragraph by hand in
    // there.
    // But in all other cases, `<` and `>` would be encoded, so we can be
    // sure that this is generated by our compiler.
    if byte_index > 3
        && bytes[byte_index - 4] == b'<'
        && bytes[byte_index - 3] == b'/'
        && bytes[byte_index - 2] == b'p'
        && bytes[byte_index - 1] == b'>'
    {
        let (before, after) = bytes.split_at(byte_index - 4);
        let mut result = String::new();
        result.push_str(str::from_utf8(before).unwrap());
        result.push(' ');
        result.push_str(&backreferences);
        result.push_str(str::from_utf8(after).unwrap());
        context.push(&result);
    } else {
        context.push(&value);
        context.line_ending_if_needed();
        context.push(&backreferences);
    }
    context.line_ending_if_needed();
    context.push("</li>");
}

/// Generate an autolink (used by unicode autolinks and GFM autolink literals).
fn generate_autolink(
    context: &mut CompileContext,
    protocol: Option<&str>,
    value: &str,
    is_gfm_literal: bool,
) {
    let mut is_in_link = false;
    let mut index = 0;

    while index < context.media_stack.len() {
        if !context.media_stack[index].image {
            is_in_link = true;
            break;
        }
        index += 1;
    }

    if !context.image_alt_inside && (!is_in_link || !is_gfm_literal) {
        context.push("<a href=\"");
        let url = if let Some(protocol) = protocol {
            format!("{}{}", protocol, value)
        } else {
            value.to_string()
        };

        let url = if context.options.allow_dangerous_protocol {
            sanitize(&url)
        } else {
            sanitize_with_protocols(&url, &SAFE_PROTOCOL_HREF)
        };

        context.push(&url);
        context.push("\">");
    }

    context.push(&encode(value, context.encode_html));

    if !context.image_alt_inside && (!is_in_link || !is_gfm_literal) {
        context.push("</a>");
    }
}
