//! The tokenizer glues states from the state machine together.
//!
//! It facilitates everything needed to turn codes into tokens and events with
//! a state machine.
//! It also enables logic needed for parsing markdown, such as an [`attempt`][]
//! to parse something, which can succeed or, when unsuccessful, revert the
//! attempt.
//! Similarly, a [`check`][] exists, which does the same as an `attempt` but
//! reverts even if successful.
//!
//! [`attempt`]: Tokenizer::attempt
//! [`check`]: Tokenizer::check

use crate::constant::TAB_SIZE;
use crate::construct;
use crate::content;
use crate::parser::ParseState;
use crate::token::{Token, VOID_TOKENS};
use crate::util::edit_map::EditMap;

/// Embedded content type.
#[derive(Debug, Clone, PartialEq)]
pub enum ContentType {
    /// Represents [flow content][crate::content::flow].
    Flow,
    /// Represents [string content][crate::content::string].
    String,
    /// Represents [text content][crate::content::text].
    Text,
}

/// To do.
#[derive(Debug, PartialEq)]
pub enum ByteAction {
    Normal(u8),
    Insert(u8),
    Ignore,
}

/// A location in the document (`line`/`column`/`offset`).
///
/// The interface for the location in the document comes from unist `Point`:
/// <https://github.com/syntax-tree/unist#point>.
#[derive(Debug, Clone)]
pub struct Point {
    /// 1-indexed line number.
    pub line: usize,
    /// 1-indexed column number.
    /// This is increases up to a tab stop for tabs.
    /// Some editors count tabs as 1 character, so this position is not the
    /// same as editors.
    pub column: usize,
    /// 0-indexed position in the document.
    ///
    /// Also an `index` into `bytes`.
    pub index: usize,
    /// Virtual step on the same `index`.
    pub vs: usize,
}

/// Possible event types.
#[derive(Debug, PartialEq, Clone)]
pub enum EventType {
    /// The start of something.
    Enter,
    /// The end of something.
    Exit,
}

/// A link to another event.
#[derive(Debug, Clone)]
pub struct Link {
    pub previous: Option<usize>,
    pub next: Option<usize>,
    pub content_type: ContentType,
}

/// Something semantic happening somewhere.
#[derive(Debug, Clone)]
pub struct Event {
    pub event_type: EventType,
    pub token_type: Token,
    pub point: Point,
    pub link: Option<Link>,
}

#[derive(Debug, PartialEq)]
enum AttemptKind {
    Attempt,
    Check,
}

/// To do.
#[derive(Debug)]
struct Attempt {
    /// To do.
    ok: State,
    nok: State,
    kind: AttemptKind,
    state: Option<InternalState>,
}

/// Callback that can be registered and is called when the tokenizer is done.
///
/// Resolvers are supposed to change the list of events, because parsing is
/// sometimes messy, and they help expose a cleaner interface of events to
/// the compiler and other users.
pub type Resolver = dyn FnOnce(&mut Tokenizer);

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum StateName {
    AttentionStart,
    AttentionInside,

    AutolinkStart,
    AutolinkOpen,
    AutolinkSchemeOrEmailAtext,
    AutolinkSchemeInsideOrEmailAtext,
    AutolinkUrlInside,
    AutolinkEmailAtSignOrDot,
    AutolinkEmailAtext,
    AutolinkEmailValue,
    AutolinkEmailLabel,

    BlankLineStart,
    BlankLineAfter,

    BlockQuoteStart,
    BlockQuoteBefore,
    BlockQuoteContStart,
    BlockQuoteContBefore,
    BlockQuoteContAfter,

    BomStart,
    BomInside,

    CharacterEscapeStart,
    CharacterEscapeInside,

    CharacterReferenceStart,
    CharacterReferenceOpen,
    CharacterReferenceNumeric,
    CharacterReferenceValue,

    CodeFencedStart,
    CodeFencedBeforeSequenceOpen,
    CodeFencedSequenceOpen,
    CodeFencedInfoBefore,
    CodeFencedInfo,
    CodeFencedMetaBefore,
    CodeFencedMeta,
    CodeFencedAtNonLazyBreak,
    CodeFencedCloseBefore,
    CodeFencedCloseStart,
    CodeFencedBeforeSequenceClose,
    CodeFencedSequenceClose,
    CodeFencedAfterSequenceClose,
    CodeFencedContentBefore,
    CodeFencedContentStart,
    CodeFencedBeforeContentChunk,
    CodeFencedContentChunk,
    CodeFencedAfter,

    CodeIndentedStart,
    CodeIndentedAtBreak,
    CodeIndentedAfter,
    CodeIndentedFurtherStart,
    CodeIndentedInside,
    CodeIndentedFurtherEnd,
    CodeIndentedFurtherBegin,
    CodeIndentedFurtherAfter,

    CodeTextStart,
    CodeTextSequenceOpen,
    CodeTextBetween,
    CodeTextData,
    CodeTextSequenceClose,

    DataStart,
    DataInside,
    DataAtBreak,

    DefinitionStart,
    DefinitionBefore,
    DefinitionLabelAfter,
    DefinitionMarkerAfter,
    DefinitionDestinationBefore,
    DefinitionDestinationAfter,
    DefinitionDestinationMissing,
    DefinitionTitleBefore,
    DefinitionAfter,
    DefinitionAfterWhitespace,
    DefinitionTitleBeforeMarker,
    DefinitionTitleAfter,
    DefinitionTitleAfterOptionalWhitespace,

    DestinationStart,
    DestinationEnclosedBefore,
    DestinationEnclosed,
    DestinationEnclosedEscape,
    DestinationRaw,
    DestinationRawEscape,

    DocumentStart,
    DocumentLineStart,
    // DocumentContainerExistingBefore,
    DocumentContainerExistingAfter,
    DocumentContainerExistingMissing,
    // DocumentContainerNewBefore,
    DocumentContainerNewBeforeNotBlockQuote,
    DocumentContainerNewAfter,
    DocumentContainersAfter,
    DocumentFlowInside,
    DocumentFlowEnd,

    FlowStart,
    FlowBeforeCodeIndented,
    FlowBeforeCodeFenced,
    FlowBeforeHtml,
    FlowBeforeHeadingAtx,
    FlowBeforeHeadingSetext,
    FlowBeforeThematicBreak,
    FlowBeforeDefinition,
    FlowAfter,
    FlowBlankLineAfter,
    FlowBeforeParagraph,

    HardBreakEscapeStart,
    HardBreakEscapeAfter,

    HeadingAtxStart,
    HeadingAtxBefore,
    HeadingAtxSequenceOpen,
    HeadingAtxAtBreak,
    HeadingAtxSequenceFurther,
    HeadingAtxData,

    HeadingSetextStart,
    HeadingSetextBefore,
    HeadingSetextInside,
    HeadingSetextAfter,

    HtmlFlowStart,
    HtmlFlowBefore,
    HtmlFlowOpen,
    HtmlFlowDeclarationOpen,
    HtmlFlowCommentOpenInside,
    HtmlFlowCdataOpenInside,
    HtmlFlowTagCloseStart,
    HtmlFlowTagName,
    HtmlFlowBasicSelfClosing,
    HtmlFlowCompleteClosingTagAfter,
    HtmlFlowCompleteEnd,
    HtmlFlowCompleteAttributeNameBefore,
    HtmlFlowCompleteAttributeName,
    HtmlFlowCompleteAttributeNameAfter,
    HtmlFlowCompleteAttributeValueBefore,
    HtmlFlowCompleteAttributeValueQuoted,
    HtmlFlowCompleteAttributeValueQuotedAfter,
    HtmlFlowCompleteAttributeValueUnquoted,
    HtmlFlowCompleteAfter,
    HtmlFlowBlankLineBefore,
    HtmlFlowContinuation,
    HtmlFlowContinuationDeclarationInside,
    HtmlFlowContinuationAfter,
    HtmlFlowContinuationStart,
    HtmlFlowContinuationBefore,
    HtmlFlowContinuationCommentInside,
    HtmlFlowContinuationRawTagOpen,
    HtmlFlowContinuationRawEndTag,
    HtmlFlowContinuationClose,
    HtmlFlowContinuationCdataInside,
    HtmlFlowContinuationStartNonLazy,

    HtmlTextStart,
    HtmlTextOpen,
    HtmlTextDeclarationOpen,
    HtmlTextTagCloseStart,
    HtmlTextTagClose,
    HtmlTextTagCloseBetween,
    HtmlTextTagOpen,
    HtmlTextTagOpenBetween,
    HtmlTextTagOpenAttributeName,
    HtmlTextTagOpenAttributeNameAfter,
    HtmlTextTagOpenAttributeValueBefore,
    HtmlTextTagOpenAttributeValueQuoted,
    HtmlTextTagOpenAttributeValueQuotedAfter,
    HtmlTextTagOpenAttributeValueUnquoted,
    HtmlTextCdata,
    HtmlTextCdataOpenInside,
    HtmlTextCdataClose,
    HtmlTextCdataEnd,
    HtmlTextCommentOpenInside,
    HtmlTextCommentStart,
    HtmlTextCommentStartDash,
    HtmlTextComment,
    HtmlTextCommentClose,
    HtmlTextDeclaration,
    HtmlTextEnd,
    HtmlTextInstruction,
    HtmlTextInstructionClose,
    HtmlTextLineEndingAfter,
    HtmlTextLineEndingAfterPrefix,

    LabelStart,
    LabelAtBreak,
    LabelEolAfter,
    LabelAtBlankLine,
    LabelEscape,
    LabelInside,

    LabelEndStart,
    LabelEndAfter,
    LabelEndResourceStart,
    LabelEndResourceBefore,
    LabelEndResourceOpen,
    LabelEndResourceDestinationAfter,
    LabelEndResourceDestinationMissing,
    LabelEndResourceBetween,
    LabelEndResourceTitleAfter,
    LabelEndResourceEnd,
    LabelEndOk,
    LabelEndNok,
    LabelEndReferenceFull,
    LabelEndReferenceFullAfter,
    LabelEndReferenceNotFull,
    LabelEndReferenceCollapsed,
    LabelEndReferenceCollapsedOpen,

    LabelStartImageStart,
    LabelStartImageOpen,

    LabelStartLinkStart,

    ListStart,
    ListBefore,
    ListNok,
    ListBeforeUnordered,
    ListValue,
    ListMarkerAfter,
    ListAfter,
    ListMarkerAfterFilled,
    ListWhitespace,
    ListPrefixOther,
    ListWhitespaceAfter,
    ListContStart,
    ListContBlank,
    ListContFilled,
    ListOk,

    NonLazyContinuationStart,
    NonLazyContinuationAfter,

    ParagraphStart,
    ParagraphInside,

    SpaceOrTabStart,
    SpaceOrTabInside,

    SpaceOrTabEolStart,
    SpaceOrTabEolAfterFirst,
    SpaceOrTabEolAfterEol,
    SpaceOrTabEolAtEol,
    SpaceOrTabEolAfterMore,

    StringStart,
    StringBefore,
    StringBeforeData,

    TextStart,
    TextBefore,
    TextBeforeHtml,
    TextBeforeHardBreakEscape,
    TextBeforeData,

    ThematicBreakStart,
    ThematicBreakBefore,
    ThematicBreakSequence,
    ThematicBreakAtBreak,

    TitleStart,
    TitleBegin,
    TitleAfterEol,
    TitleAtBlankLine,
    TitleEscape,
    TitleInside,
}

impl StateName {
    /// Create a new tokenizer.
    #[allow(clippy::too_many_lines)]
    pub fn to_func(self) -> Box<dyn FnOnce(&mut Tokenizer) -> State + 'static> {
        let func = match self {
            StateName::AttentionStart => construct::attention::start,
            StateName::AttentionInside => construct::attention::inside,

            StateName::AutolinkStart => construct::autolink::start,
            StateName::AutolinkOpen => construct::autolink::open,
            StateName::AutolinkSchemeOrEmailAtext => construct::autolink::scheme_or_email_atext,
            StateName::AutolinkSchemeInsideOrEmailAtext => {
                construct::autolink::scheme_inside_or_email_atext
            }
            StateName::AutolinkUrlInside => construct::autolink::url_inside,
            StateName::AutolinkEmailAtSignOrDot => construct::autolink::email_at_sign_or_dot,
            StateName::AutolinkEmailAtext => construct::autolink::email_atext,
            StateName::AutolinkEmailValue => construct::autolink::email_value,
            StateName::AutolinkEmailLabel => construct::autolink::email_label,

            StateName::BlankLineStart => construct::blank_line::start,
            StateName::BlankLineAfter => construct::blank_line::after,

            StateName::BlockQuoteStart => construct::block_quote::start,
            StateName::BlockQuoteBefore => construct::block_quote::before,
            StateName::BlockQuoteContStart => construct::block_quote::cont_start,
            StateName::BlockQuoteContBefore => construct::block_quote::cont_before,
            StateName::BlockQuoteContAfter => construct::block_quote::cont_after,

            StateName::BomStart => construct::partial_bom::start,
            StateName::BomInside => construct::partial_bom::inside,

            StateName::CharacterEscapeStart => construct::character_escape::start,
            StateName::CharacterEscapeInside => construct::character_escape::inside,

            StateName::CharacterReferenceStart => construct::character_reference::start,
            StateName::CharacterReferenceOpen => construct::character_reference::open,
            StateName::CharacterReferenceNumeric => construct::character_reference::numeric,
            StateName::CharacterReferenceValue => construct::character_reference::value,

            StateName::CodeFencedStart => construct::code_fenced::start,
            StateName::CodeFencedBeforeSequenceOpen => construct::code_fenced::before_sequence_open,
            StateName::CodeFencedSequenceOpen => construct::code_fenced::sequence_open,
            StateName::CodeFencedInfoBefore => construct::code_fenced::info_before,
            StateName::CodeFencedInfo => construct::code_fenced::info,
            StateName::CodeFencedMetaBefore => construct::code_fenced::meta_before,
            StateName::CodeFencedMeta => construct::code_fenced::meta,
            StateName::CodeFencedAtNonLazyBreak => construct::code_fenced::at_non_lazy_break,
            StateName::CodeFencedCloseBefore => construct::code_fenced::close_before,
            StateName::CodeFencedCloseStart => construct::code_fenced::close_start,
            StateName::CodeFencedBeforeSequenceClose => {
                construct::code_fenced::before_sequence_close
            }
            StateName::CodeFencedSequenceClose => construct::code_fenced::sequence_close,
            StateName::CodeFencedAfterSequenceClose => construct::code_fenced::sequence_close_after,
            StateName::CodeFencedContentBefore => construct::code_fenced::content_before,
            StateName::CodeFencedContentStart => construct::code_fenced::content_start,
            StateName::CodeFencedBeforeContentChunk => construct::code_fenced::before_content_chunk,
            StateName::CodeFencedContentChunk => construct::code_fenced::content_chunk,
            StateName::CodeFencedAfter => construct::code_fenced::after,

            StateName::CodeIndentedStart => construct::code_indented::start,
            StateName::CodeIndentedAtBreak => construct::code_indented::at_break,
            StateName::CodeIndentedAfter => construct::code_indented::after,
            StateName::CodeIndentedFurtherStart => construct::code_indented::further_start,
            StateName::CodeIndentedInside => construct::code_indented::inside,
            StateName::CodeIndentedFurtherEnd => construct::code_indented::further_end,
            StateName::CodeIndentedFurtherBegin => construct::code_indented::further_begin,
            StateName::CodeIndentedFurtherAfter => construct::code_indented::further_after,

            StateName::CodeTextStart => construct::code_text::start,
            StateName::CodeTextSequenceOpen => construct::code_text::sequence_open,
            StateName::CodeTextBetween => construct::code_text::between,
            StateName::CodeTextData => construct::code_text::data,
            StateName::CodeTextSequenceClose => construct::code_text::sequence_close,

            StateName::DataStart => construct::partial_data::start,
            StateName::DataInside => construct::partial_data::inside,
            StateName::DataAtBreak => construct::partial_data::at_break,

            StateName::DefinitionStart => construct::definition::start,
            StateName::DefinitionBefore => construct::definition::before,
            StateName::DefinitionLabelAfter => construct::definition::label_after,
            StateName::DefinitionMarkerAfter => construct::definition::marker_after,
            StateName::DefinitionDestinationBefore => construct::definition::destination_before,
            StateName::DefinitionDestinationAfter => construct::definition::destination_after,
            StateName::DefinitionDestinationMissing => construct::definition::destination_missing,
            StateName::DefinitionTitleBefore => construct::definition::title_before,
            StateName::DefinitionAfter => construct::definition::after,
            StateName::DefinitionAfterWhitespace => construct::definition::after_whitespace,
            StateName::DefinitionTitleBeforeMarker => construct::definition::title_before_marker,
            StateName::DefinitionTitleAfter => construct::definition::title_after,
            StateName::DefinitionTitleAfterOptionalWhitespace => {
                construct::definition::title_after_optional_whitespace
            }

            StateName::DestinationStart => construct::partial_destination::start,
            StateName::DestinationEnclosedBefore => construct::partial_destination::enclosed_before,
            StateName::DestinationEnclosed => construct::partial_destination::enclosed,
            StateName::DestinationEnclosedEscape => construct::partial_destination::enclosed_escape,
            StateName::DestinationRaw => construct::partial_destination::raw,
            StateName::DestinationRawEscape => construct::partial_destination::raw_escape,

            StateName::DocumentStart => content::document::start,
            StateName::DocumentLineStart => content::document::line_start,
            // StateName::DocumentContainerExistingBefore => content::document::container_existing_before,
            StateName::DocumentContainerExistingAfter => {
                content::document::container_existing_after
            }
            StateName::DocumentContainerExistingMissing => {
                content::document::container_existing_missing
            }
            // StateName::DocumentContainerNewBefore => content::document::container_new_before,
            StateName::DocumentContainerNewBeforeNotBlockQuote => {
                content::document::container_new_before_not_block_quote
            }
            StateName::DocumentContainerNewAfter => content::document::container_new_after,
            StateName::DocumentContainersAfter => content::document::containers_after,
            StateName::DocumentFlowEnd => content::document::flow_end,
            StateName::DocumentFlowInside => content::document::flow_inside,

            StateName::FlowStart => content::flow::start,
            StateName::FlowBeforeCodeIndented => content::flow::before_code_indented,
            StateName::FlowBeforeCodeFenced => content::flow::before_code_fenced,
            StateName::FlowBeforeHtml => content::flow::before_html,
            StateName::FlowBeforeHeadingAtx => content::flow::before_heading_atx,
            StateName::FlowBeforeHeadingSetext => content::flow::before_heading_setext,
            StateName::FlowBeforeThematicBreak => content::flow::before_thematic_break,
            StateName::FlowBeforeDefinition => content::flow::before_definition,
            StateName::FlowAfter => content::flow::after,
            StateName::FlowBlankLineAfter => content::flow::blank_line_after,
            StateName::FlowBeforeParagraph => content::flow::before_paragraph,

            StateName::HardBreakEscapeStart => construct::hard_break_escape::start,
            StateName::HardBreakEscapeAfter => construct::hard_break_escape::after,

            StateName::HeadingAtxStart => construct::heading_atx::start,
            StateName::HeadingAtxBefore => construct::heading_atx::before,
            StateName::HeadingAtxSequenceOpen => construct::heading_atx::sequence_open,
            StateName::HeadingAtxAtBreak => construct::heading_atx::at_break,
            StateName::HeadingAtxSequenceFurther => construct::heading_atx::sequence_further,
            StateName::HeadingAtxData => construct::heading_atx::data,

            StateName::HeadingSetextStart => construct::heading_setext::start,
            StateName::HeadingSetextBefore => construct::heading_setext::before,
            StateName::HeadingSetextInside => construct::heading_setext::inside,
            StateName::HeadingSetextAfter => construct::heading_setext::after,

            StateName::HtmlFlowStart => construct::html_flow::start,
            StateName::HtmlFlowBefore => construct::html_flow::before,
            StateName::HtmlFlowOpen => construct::html_flow::open,
            StateName::HtmlFlowDeclarationOpen => construct::html_flow::declaration_open,
            StateName::HtmlFlowCommentOpenInside => construct::html_flow::comment_open_inside,
            StateName::HtmlFlowCdataOpenInside => construct::html_flow::cdata_open_inside,
            StateName::HtmlFlowTagCloseStart => construct::html_flow::tag_close_start,
            StateName::HtmlFlowTagName => construct::html_flow::tag_name,
            StateName::HtmlFlowBasicSelfClosing => construct::html_flow::basic_self_closing,
            StateName::HtmlFlowCompleteClosingTagAfter => {
                construct::html_flow::complete_closing_tag_after
            }
            StateName::HtmlFlowCompleteEnd => construct::html_flow::complete_end,
            StateName::HtmlFlowCompleteAttributeNameBefore => {
                construct::html_flow::complete_attribute_name_before
            }
            StateName::HtmlFlowCompleteAttributeName => {
                construct::html_flow::complete_attribute_name
            }
            StateName::HtmlFlowCompleteAttributeNameAfter => {
                construct::html_flow::complete_attribute_name_after
            }
            StateName::HtmlFlowCompleteAttributeValueBefore => {
                construct::html_flow::complete_attribute_value_before
            }
            StateName::HtmlFlowCompleteAttributeValueQuoted => {
                construct::html_flow::complete_attribute_value_quoted
            }
            StateName::HtmlFlowCompleteAttributeValueQuotedAfter => {
                construct::html_flow::complete_attribute_value_quoted_after
            }
            StateName::HtmlFlowCompleteAttributeValueUnquoted => {
                construct::html_flow::complete_attribute_value_unquoted
            }
            StateName::HtmlFlowCompleteAfter => construct::html_flow::complete_after,
            StateName::HtmlFlowBlankLineBefore => construct::html_flow::blank_line_before,
            StateName::HtmlFlowContinuation => construct::html_flow::continuation,
            StateName::HtmlFlowContinuationDeclarationInside => {
                construct::html_flow::continuation_declaration_inside
            }
            StateName::HtmlFlowContinuationAfter => construct::html_flow::continuation_after,
            StateName::HtmlFlowContinuationStart => construct::html_flow::continuation_start,
            StateName::HtmlFlowContinuationBefore => construct::html_flow::continuation_before,
            StateName::HtmlFlowContinuationCommentInside => {
                construct::html_flow::continuation_comment_inside
            }
            StateName::HtmlFlowContinuationRawTagOpen => {
                construct::html_flow::continuation_raw_tag_open
            }
            StateName::HtmlFlowContinuationRawEndTag => {
                construct::html_flow::continuation_raw_end_tag
            }
            StateName::HtmlFlowContinuationClose => construct::html_flow::continuation_close,
            StateName::HtmlFlowContinuationCdataInside => {
                construct::html_flow::continuation_cdata_inside
            }
            StateName::HtmlFlowContinuationStartNonLazy => {
                construct::html_flow::continuation_start_non_lazy
            }

            StateName::HtmlTextStart => construct::html_text::start,
            StateName::HtmlTextOpen => construct::html_text::open,
            StateName::HtmlTextDeclarationOpen => construct::html_text::declaration_open,
            StateName::HtmlTextTagCloseStart => construct::html_text::tag_close_start,
            StateName::HtmlTextTagClose => construct::html_text::tag_close,
            StateName::HtmlTextTagCloseBetween => construct::html_text::tag_close_between,
            StateName::HtmlTextTagOpen => construct::html_text::tag_open,
            StateName::HtmlTextTagOpenBetween => construct::html_text::tag_open_between,
            StateName::HtmlTextTagOpenAttributeName => {
                construct::html_text::tag_open_attribute_name
            }
            StateName::HtmlTextTagOpenAttributeNameAfter => {
                construct::html_text::tag_open_attribute_name_after
            }
            StateName::HtmlTextTagOpenAttributeValueBefore => {
                construct::html_text::tag_open_attribute_value_before
            }
            StateName::HtmlTextTagOpenAttributeValueQuoted => {
                construct::html_text::tag_open_attribute_value_quoted
            }
            StateName::HtmlTextTagOpenAttributeValueQuotedAfter => {
                construct::html_text::tag_open_attribute_value_quoted_after
            }
            StateName::HtmlTextTagOpenAttributeValueUnquoted => {
                construct::html_text::tag_open_attribute_value_unquoted
            }
            StateName::HtmlTextCdata => construct::html_text::cdata,
            StateName::HtmlTextCdataOpenInside => construct::html_text::cdata_open_inside,
            StateName::HtmlTextCdataClose => construct::html_text::cdata_close,
            StateName::HtmlTextCdataEnd => construct::html_text::cdata_end,
            StateName::HtmlTextCommentOpenInside => construct::html_text::comment_open_inside,
            StateName::HtmlTextCommentStart => construct::html_text::comment_start,
            StateName::HtmlTextCommentStartDash => construct::html_text::comment_start_dash,
            StateName::HtmlTextComment => construct::html_text::comment,
            StateName::HtmlTextCommentClose => construct::html_text::comment_close,
            StateName::HtmlTextDeclaration => construct::html_text::declaration,
            StateName::HtmlTextEnd => construct::html_text::end,
            StateName::HtmlTextInstruction => construct::html_text::instruction,
            StateName::HtmlTextInstructionClose => construct::html_text::instruction_close,
            StateName::HtmlTextLineEndingAfter => construct::html_text::line_ending_after,
            StateName::HtmlTextLineEndingAfterPrefix => {
                construct::html_text::line_ending_after_prefix
            }

            StateName::LabelStart => construct::partial_label::start,
            StateName::LabelAtBreak => construct::partial_label::at_break,
            StateName::LabelEolAfter => construct::partial_label::eol_after,
            StateName::LabelAtBlankLine => construct::partial_label::at_blank_line,
            StateName::LabelEscape => construct::partial_label::escape,
            StateName::LabelInside => construct::partial_label::inside,

            StateName::LabelEndStart => construct::label_end::start,
            StateName::LabelEndAfter => construct::label_end::after,
            StateName::LabelEndResourceStart => construct::label_end::resource_start,
            StateName::LabelEndResourceBefore => construct::label_end::resource_before,
            StateName::LabelEndResourceOpen => construct::label_end::resource_open,
            StateName::LabelEndResourceDestinationAfter => {
                construct::label_end::resource_destination_after
            }
            StateName::LabelEndResourceDestinationMissing => {
                construct::label_end::resource_destination_missing
            }
            StateName::LabelEndResourceBetween => construct::label_end::resource_between,
            StateName::LabelEndResourceTitleAfter => construct::label_end::resource_title_after,
            StateName::LabelEndResourceEnd => construct::label_end::resource_end,
            StateName::LabelEndOk => construct::label_end::ok,
            StateName::LabelEndNok => construct::label_end::nok,
            StateName::LabelEndReferenceFull => construct::label_end::reference_full,
            StateName::LabelEndReferenceFullAfter => construct::label_end::reference_full_after,
            StateName::LabelEndReferenceNotFull => construct::label_end::reference_not_full,
            StateName::LabelEndReferenceCollapsed => construct::label_end::reference_collapsed,
            StateName::LabelEndReferenceCollapsedOpen => {
                construct::label_end::reference_collapsed_open
            }

            StateName::LabelStartImageStart => construct::label_start_image::start,
            StateName::LabelStartImageOpen => construct::label_start_image::open,
            StateName::LabelStartLinkStart => construct::label_start_link::start,

            StateName::ListStart => construct::list::start,
            StateName::ListBefore => construct::list::before,
            StateName::ListNok => construct::list::nok,
            StateName::ListBeforeUnordered => construct::list::before_unordered,
            StateName::ListValue => construct::list::value,
            StateName::ListMarkerAfter => construct::list::marker_after,
            StateName::ListAfter => construct::list::after,
            StateName::ListMarkerAfterFilled => construct::list::marker_after_filled,
            StateName::ListWhitespace => construct::list::whitespace,
            StateName::ListWhitespaceAfter => construct::list::whitespace_after,
            StateName::ListPrefixOther => construct::list::prefix_other,
            StateName::ListContStart => construct::list::cont_start,
            StateName::ListContBlank => construct::list::cont_blank,
            StateName::ListContFilled => construct::list::cont_filled,
            StateName::ListOk => construct::list::ok,

            StateName::NonLazyContinuationStart => construct::partial_non_lazy_continuation::start,
            StateName::NonLazyContinuationAfter => construct::partial_non_lazy_continuation::after,

            StateName::ParagraphStart => construct::paragraph::start,
            StateName::ParagraphInside => construct::paragraph::inside,

            StateName::SpaceOrTabStart => construct::partial_space_or_tab::start,
            StateName::SpaceOrTabInside => construct::partial_space_or_tab::inside,

            StateName::SpaceOrTabEolStart => construct::partial_space_or_tab::eol_start,
            StateName::SpaceOrTabEolAfterFirst => construct::partial_space_or_tab::eol_after_first,
            StateName::SpaceOrTabEolAfterEol => construct::partial_space_or_tab::eol_after_eol,
            StateName::SpaceOrTabEolAtEol => construct::partial_space_or_tab::eol_at_eol,
            StateName::SpaceOrTabEolAfterMore => construct::partial_space_or_tab::eol_after_more,

            StateName::StringStart => content::string::start,
            StateName::StringBefore => content::string::before,
            StateName::StringBeforeData => content::string::before_data,

            StateName::TextStart => content::text::start,
            StateName::TextBefore => content::text::before,
            StateName::TextBeforeHtml => content::text::before_html,
            StateName::TextBeforeHardBreakEscape => content::text::before_hard_break_escape,
            StateName::TextBeforeData => content::text::before_data,

            StateName::ThematicBreakStart => construct::thematic_break::start,
            StateName::ThematicBreakBefore => construct::thematic_break::before,
            StateName::ThematicBreakSequence => construct::thematic_break::sequence,
            StateName::ThematicBreakAtBreak => construct::thematic_break::at_break,

            StateName::TitleStart => construct::partial_title::start,
            StateName::TitleBegin => construct::partial_title::begin,
            StateName::TitleAfterEol => construct::partial_title::after_eol,
            StateName::TitleAtBlankLine => construct::partial_title::at_blank_line,
            StateName::TitleEscape => construct::partial_title::escape,
            StateName::TitleInside => construct::partial_title::inside,
        };

        Box::new(func)
    }
}

/// The result of a state.
#[derive(Debug, PartialEq, Copy, Clone)]
pub enum State {
    /// There is a future state: a [`StateName`][] to pass the next code to.
    Next(StateName),
    /// The state is successful.
    Ok,
    /// The state is not successful.
    Nok,
}

/// Loose label starts we found.
#[derive(Debug)]
pub struct LabelStart {
    /// Indices of where the label starts and ends in `events`.
    pub start: (usize, usize),
    /// A boolean used internally to figure out if a label start link can’t be
    /// used (because links in links are incorrect).
    pub inactive: bool,
    /// A boolean used internally to figure out if a label is balanced: they’re
    /// not media, it’s just balanced braces.
    pub balanced: bool,
}

/// Media we found.
#[derive(Debug)]
pub struct Media {
    /// Indices of where the media’s label start starts and ends in `events`.
    pub start: (usize, usize),
    /// Indices of where the media’s label end starts and ends in `events`.
    pub end: (usize, usize),
}

/// Supported containers.
#[derive(Debug, PartialEq)]
pub enum Container {
    BlockQuote,
    ListItem,
}

/// Info used to tokenize the current container.
///
/// This info is shared between the initial construct and its continuation.
/// It’s only used for list items.
#[derive(Debug)]
pub struct ContainerState {
    /// Kind.
    pub kind: Container,
    /// Whether the first line was blank.
    pub blank_initial: bool,
    /// The size of the initial construct.
    pub size: usize,
}

/// The internal state of a tokenizer, not to be confused with states from the
/// state machine, this instead is all the information about where we currently
/// are and what’s going on.
#[derive(Debug, Clone)]
struct InternalState {
    /// Length of `events`. We only add to events, so reverting will just pop stuff off.
    events_len: usize,
    /// Length of the stack. It’s not allowed to decrease the stack in a check or an attempt.
    stack_len: usize,
    /// Previous code.
    previous: Option<u8>,
    /// Current code.
    current: Option<u8>,
    /// Current relative and absolute position in the file.
    point: Point,
}

/// To do
#[allow(clippy::struct_excessive_bools)]
pub struct TokenizeState<'a> {
    /// To do.
    pub connect: bool,
    /// To do.
    pub document_container_stack: Vec<ContainerState>,
    /// To do.
    pub document_continued: usize,
    /// To do.
    pub document_interrupt_before: bool,
    /// To do.
    pub document_paragraph_before: bool,
    /// To do.
    pub document_data_index: Option<usize>,
    /// To do.
    pub document_child_state: Option<State>,
    /// To do.
    pub child_tokenizer: Option<Box<Tokenizer<'a>>>,
    /// To do.
    pub marker: u8,
    /// To do.
    pub marker_other: u8,
    /// To do.
    pub prefix: usize,
    /// To do.
    pub return_state: Option<StateName>,
    /// To do.
    pub seen: bool,
    /// To do.
    pub size: usize,
    /// To do.
    pub size_other: usize,
    /// To do.
    pub start: usize,
    /// To do.
    pub end: usize,
    /// To do.
    pub stop: &'static [u8],
    pub space_or_tab_eol_content_type: Option<ContentType>,
    pub space_or_tab_eol_connect: bool,
    pub space_or_tab_eol_ok: bool,
    pub space_or_tab_connect: bool,
    pub space_or_tab_content_type: Option<ContentType>,
    pub space_or_tab_min: usize,
    pub space_or_tab_max: usize,
    pub space_or_tab_size: usize,
    pub space_or_tab_token: Token,
    /// To do.
    pub token_1: Token,
    pub token_2: Token,
    pub token_3: Token,
    pub token_4: Token,
    pub token_5: Token,
}

/// A tokenizer itself.
#[allow(clippy::struct_excessive_bools)]
pub struct Tokenizer<'a> {
    /// Jump between line endings.
    column_start: Vec<(usize, usize)>,
    // First line.
    first_line: usize,
    /// First point after the last line ending.
    line_start: Point,
    /// Track whether the current byte is already consumed (`true`) or expected
    /// to be consumed (`false`).
    ///
    /// Tracked to make sure everything’s valid.
    consumed: bool,
    /// Track whether this tokenizer is done.
    resolved: bool,
    /// To do.
    attempts: Vec<Attempt>,
    /// Current byte.
    pub current: Option<u8>,
    /// Previous byte.
    pub previous: Option<u8>,
    /// Current relative and absolute place in the file.
    pub point: Point,
    /// Semantic labels of one or more codes in `codes`.
    pub events: Vec<Event>,
    /// Hierarchy of semantic labels.
    ///
    /// Tracked to make sure everything’s valid.
    pub stack: Vec<Token>,
    /// Edit map, to batch changes.
    pub map: EditMap,
    /// List of attached resolvers, which will be called when done feeding,
    /// to clean events.
    pub resolvers: Vec<Box<Resolver>>,
    /// List of names associated with attached resolvers.
    pub resolver_ids: Vec<String>,
    /// Shared parsing state across tokenizers.
    pub parse_state: &'a ParseState<'a>,
    /// To do.
    pub tokenize_state: TokenizeState<'a>,
    /// Stack of label (start) that could form images and links.
    ///
    /// Used when tokenizing [text content][crate::content::text].
    pub label_start_stack: Vec<LabelStart>,
    /// Stack of label (start) that cannot form images and links.
    ///
    /// Used when tokenizing [text content][crate::content::text].
    pub label_start_list_loose: Vec<LabelStart>,
    /// Stack of images and links.
    ///
    /// Used when tokenizing [text content][crate::content::text].
    pub media_list: Vec<Media>,
    /// Current container state.
    pub container: Option<ContainerState>,
    /// Whether we would be interrupting something.
    ///
    /// Used when tokenizing [flow content][crate::content::flow].
    pub interrupt: bool,
    /// Whether containers cannot “pierce” into the current construct.
    ///
    /// Used when tokenizing [document content][crate::content::document].
    pub concrete: bool,
    /// Whether this line is lazy.
    ///
    /// The previous line was a paragraph, and this line’s containers did not
    /// match.
    pub lazy: bool,
}

impl<'a> Tokenizer<'a> {
    /// Create a new tokenizer.
    pub fn new(point: Point, parse_state: &'a ParseState) -> Tokenizer<'a> {
        Tokenizer {
            previous: None,
            current: None,
            // To do: reserve size when feeding?
            column_start: vec![],
            first_line: point.line,
            line_start: point.clone(),
            consumed: true,
            resolved: false,
            attempts: vec![],
            point,
            stack: vec![],
            events: vec![],
            parse_state,
            tokenize_state: TokenizeState {
                connect: false,
                document_container_stack: vec![],
                document_continued: 0,
                document_interrupt_before: false,
                document_paragraph_before: false,
                document_data_index: None,
                document_child_state: None,
                child_tokenizer: None,
                marker: 0,
                marker_other: 0,
                prefix: 0,
                seen: false,
                size: 0,
                size_other: 0,
                start: 0,
                end: 0,
                stop: &[],
                return_state: None,
                space_or_tab_eol_content_type: None,
                space_or_tab_eol_connect: false,
                space_or_tab_eol_ok: false,
                space_or_tab_connect: false,
                space_or_tab_content_type: None,
                space_or_tab_min: 0,
                space_or_tab_max: 0,
                space_or_tab_size: 0,
                space_or_tab_token: Token::SpaceOrTab,
                token_1: Token::Data,
                token_2: Token::Data,
                token_3: Token::Data,
                token_4: Token::Data,
                token_5: Token::Data,
            },
            map: EditMap::new(),
            label_start_stack: vec![],
            label_start_list_loose: vec![],
            media_list: vec![],
            container: None,
            interrupt: false,
            concrete: false,
            lazy: false,
            // Assume about 10 resolvers.
            resolvers: Vec::with_capacity(10),
            resolver_ids: Vec::with_capacity(10),
        }
    }

    /// Register a resolver.
    pub fn register_resolver(&mut self, id: String, resolver: Box<Resolver>) {
        if !self.resolver_ids.contains(&id) {
            self.resolver_ids.push(id);
            self.resolvers.push(resolver);
        }
    }

    /// Register a resolver, before others.
    pub fn register_resolver_before(&mut self, id: String, resolver: Box<Resolver>) {
        if !self.resolver_ids.contains(&id) {
            self.resolver_ids.push(id);
            self.resolvers.insert(0, resolver);
        }
    }

    /// Define a jump between two places.
    ///
    /// This defines to which future index we move after a line ending.
    pub fn define_skip(&mut self, mut point: Point) {
        move_point_back(self, &mut point);

        let info = (point.index, point.vs);
        log::debug!("position: define skip: {:?} -> ({:?})", point.line, info);
        let at = point.line - self.first_line;

        if at >= self.column_start.len() {
            self.column_start.push(info);
        } else {
            self.column_start[at] = info;
        }

        self.account_for_potential_skip();
    }

    /// Increment the current positional info if we’re right after a line
    /// ending, which has a skip defined.
    fn account_for_potential_skip(&mut self) {
        let at = self.point.line - self.first_line;

        if self.point.column == 1 && at != self.column_start.len() {
            self.move_to(self.column_start[at]);
        }
    }

    /// Prepare for a next code to get consumed.
    pub fn expect(&mut self, byte: Option<u8>) {
        debug_assert!(self.consumed, "expected previous byte to be consumed");
        self.consumed = false;
        self.current = byte;
    }

    /// Consume the current byte.
    /// Each state function is expected to call this to signal that this code is
    /// used, or call a next function.
    pub fn consume(&mut self) {
        log::debug!("consume: `{:?}` ({:?})", self.current, self.point);
        debug_assert!(!self.consumed, "expected code to not have been consumed: this might be because `x(code)` instead of `x` was returned");

        self.move_one();

        self.previous = self.current;
        // While we’re not at the eof, it is at least better to not have the
        // same current code as `previous` *and* `current`.
        self.current = None;
        // Mark as consumed.
        self.consumed = true;
    }

    /// Move to the next (virtual) byte.
    pub fn move_one(&mut self) {
        match byte_action(self.parse_state.bytes, &self.point) {
            ByteAction::Ignore => {
                self.point.index += 1;
            }
            ByteAction::Insert(byte) => {
                self.previous = Some(byte);
                self.point.column += 1;
                self.point.vs += 1;
            }
            ByteAction::Normal(byte) => {
                self.previous = Some(byte);
                self.point.vs = 0;
                self.point.index += 1;

                if byte == b'\n' {
                    self.point.line += 1;
                    self.point.column = 1;

                    if self.point.line - self.first_line + 1 > self.column_start.len() {
                        self.column_start.push((self.point.index, self.point.vs));
                    }

                    self.line_start = self.point.clone();

                    self.account_for_potential_skip();
                    log::debug!("position: after eol: `{:?}`", self.point);
                } else {
                    self.point.column += 1;
                }
            }
        }
    }

    /// Move (virtual) bytes.
    pub fn move_to(&mut self, to: (usize, usize)) {
        let (to_index, to_vs) = to;
        while self.point.index < to_index || self.point.index == to_index && self.point.vs < to_vs {
            self.move_one();
        }
    }

    /// Mark the start of a semantic label.
    pub fn enter(&mut self, token_type: Token) {
        self.enter_with_link(token_type, None);
    }

    pub fn enter_with_content(&mut self, token_type: Token, content_type_opt: Option<ContentType>) {
        self.enter_with_link(
            token_type,
            content_type_opt.map(|content_type| Link {
                content_type,
                previous: None,
                next: None,
            }),
        );
    }

    pub fn enter_with_link(&mut self, token_type: Token, link: Option<Link>) {
        let mut point = self.point.clone();
        move_point_back(self, &mut point);

        log::debug!("enter: `{:?}` ({:?})", token_type, point);
        self.events.push(Event {
            event_type: EventType::Enter,
            token_type: token_type.clone(),
            point,
            link,
        });
        self.stack.push(token_type);
    }

    /// Mark the end of a semantic label.
    pub fn exit(&mut self, token_type: Token) {
        let current_token = self.stack.pop().expect("cannot close w/o open tokens");

        debug_assert_eq!(
            current_token, token_type,
            "expected exit token to match current token"
        );

        let previous = self.events.last().expect("cannot close w/o open event");
        let mut point = self.point.clone();

        debug_assert!(
            current_token != previous.token_type
                || previous.point.index != point.index
                || previous.point.vs != point.vs,
            "expected non-empty token"
        );

        if VOID_TOKENS.iter().any(|d| d == &token_type) {
            debug_assert!(
                current_token == previous.token_type,
                "expected token to be void (`{:?}`), instead of including `{:?}`",
                current_token,
                previous.token_type
            );
        }

        // A bit weird, but if we exit right after a line ending, we *don’t* want to consider
        // potential skips.
        if matches!(self.previous, Some(b'\n')) {
            point = self.line_start.clone();
        } else {
            move_point_back(self, &mut point);
        }

        log::debug!("exit: `{:?}` ({:?})", token_type, point);
        self.events.push(Event {
            event_type: EventType::Exit,
            token_type,
            point,
            link: None,
        });
    }

    /// Capture the internal state.
    fn capture(&mut self) -> InternalState {
        InternalState {
            previous: self.previous,
            current: self.current,
            point: self.point.clone(),
            events_len: self.events.len(),
            stack_len: self.stack.len(),
        }
    }

    /// Apply the internal state.
    fn free(&mut self, previous: InternalState) {
        self.previous = previous.previous;
        self.current = previous.current;
        self.point = previous.point;
        debug_assert!(
            self.events.len() >= previous.events_len,
            "expected to restore less events than before"
        );
        self.events.truncate(previous.events_len);
        debug_assert!(
            self.stack.len() >= previous.stack_len,
            "expected to restore less stack items than before"
        );
        self.stack.truncate(previous.stack_len);
    }

    /// Parse with `name` and its future states, to check if it result in
    /// [`State::Ok`][] or [`State::Nok`][], revert on both cases, and then
    /// call `done` with whether it was successful or not.
    ///
    /// This captures the current state of the tokenizer, returns a wrapped
    /// state that captures all codes and feeds them to `name` and its
    /// future states until it yields `State::Ok` or `State::Nok`.
    /// It then applies the captured state, calls `done`, and feeds all
    /// captured codes to its future states.
    pub fn check(&mut self, name: StateName, ok: State, nok: State) -> State {
        attempt_impl(self, name, ok, nok, AttemptKind::Check)
    }

    /// Parse with `name` and its future states, to check if it results in
    /// [`State::Ok`][] or [`State::Nok`][], revert on the case of
    /// `State::Nok`, and then call `done` with whether it was successful or
    /// not.
    ///
    /// This captures the current state of the tokenizer, returns a wrapped
    /// state that captures all codes and feeds them to `name` and its
    /// future states until it yields `State::Ok`, at which point it calls
    /// `done` and yields its result.
    /// If instead `State::Nok` was yielded, the captured state is applied,
    /// `done` is called, and all captured codes are fed to its future states.
    pub fn attempt(&mut self, name: StateName, ok: State, nok: State) -> State {
        attempt_impl(self, name, ok, nok, AttemptKind::Attempt)
    }

    /// Feed a list of `codes` into `start`.
    ///
    /// This is set up to support repeatedly calling `feed`, and thus streaming
    /// markdown into the state machine, and normally pauses after feeding.
    // Note: if needed: accept `vs`?
    pub fn push(&mut self, min: usize, max: usize, name: StateName) -> State {
        debug_assert!(!self.resolved, "cannot feed after drain");
        // debug_assert!(min >= self.point.index, "cannot move backwards");
        if min > self.point.index {
            self.move_to((min, 0));
        }

        let mut state = State::Next(name);

        while self.point.index < max {
            match state {
                State::Ok | State::Nok => {
                    if let Some(attempt) = self.attempts.pop() {
                        state = attempt_done_impl(self, attempt, state);
                    } else {
                        break;
                    }
                }
                State::Next(name) => {
                    let action = byte_action(self.parse_state.bytes, &self.point);
                    state = feed_action_impl(self, &Some(action), name);
                }
            }
        }

        state
    }

    /// Flush the tokenizer.
    pub fn flush(&mut self, mut state: State, resolve: bool) {
        let max = self.point.index;

        self.consumed = true;

        loop {
            match state {
                State::Ok | State::Nok => {
                    if let Some(attempt) = self.attempts.pop() {
                        state = attempt_done_impl(self, attempt, state);
                    } else {
                        break;
                    }
                }
                State::Next(name) => {
                    // We sometimes move back when flushing, so then we use those codes.
                    state = feed_action_impl(
                        self,
                        &if self.point.index == max {
                            None
                        } else {
                            Some(byte_action(self.parse_state.bytes, &self.point))
                        },
                        name,
                    );
                }
            }
        }

        self.consumed = true;
        debug_assert!(matches!(state, State::Ok), "must be ok");

        if resolve {
            self.resolved = true;

            while !self.resolvers.is_empty() {
                let resolver = self.resolvers.remove(0);
                resolver(self);
            }

            self.map.consume(&mut self.events);
        }
    }
}

fn byte_action(bytes: &[u8], point: &Point) -> ByteAction {
    if point.index < bytes.len() {
        let byte = bytes[point.index];

        if byte == b'\r' {
            // CRLF.
            if point.index < bytes.len() - 1 && bytes[point.index + 1] == b'\n' {
                ByteAction::Ignore
            }
            // CR.
            else {
                ByteAction::Normal(b'\n')
            }
        } else if byte == b'\t' {
            let remainder = point.column % TAB_SIZE;
            let vs = if remainder == 0 {
                0
            } else {
                TAB_SIZE - remainder
            };

            // On the tab itself, first send it.
            if point.vs == 0 {
                if vs == 0 {
                    ByteAction::Normal(byte)
                } else {
                    ByteAction::Insert(byte)
                }
            } else if vs == 0 {
                ByteAction::Normal(b' ')
            } else {
                ByteAction::Insert(b' ')
            }
        } else {
            ByteAction::Normal(byte)
        }
    } else {
        unreachable!("out of bounds")
    }
}

/// Internal utility to wrap states to also capture codes.
///
/// Recurses into itself.
/// Used in [`Tokenizer::attempt`][Tokenizer::attempt] and  [`Tokenizer::check`][Tokenizer::check].
fn attempt_impl(
    tokenizer: &mut Tokenizer,
    name: StateName,
    ok: State,
    nok: State,
    kind: AttemptKind,
) -> State {
    // Always capture (and restore) when checking.
    // No need to capture (and restore) when `nok` is `State::Nok`, because the
    // parent attempt will do it.
    let state = if kind == AttemptKind::Check || nok != State::Nok {
        Some(tokenizer.capture())
    } else {
        None
    };

    tokenizer.attempts.push(Attempt {
        ok,
        nok,
        kind,
        state,
    });

    call_impl(tokenizer, name)
}

fn attempt_done_impl(tokenizer: &mut Tokenizer, attempt: Attempt, state: State) -> State {
    if attempt.kind == AttemptKind::Check || state == State::Nok {
        if let Some(state) = attempt.state {
            tokenizer.free(state);
        }
    }

    tokenizer.consumed = true;
    if state == State::Ok {
        attempt.ok
    } else {
        attempt.nok
    }
}

fn feed_action_impl(
    tokenizer: &mut Tokenizer,
    action: &Option<ByteAction>,
    name: StateName,
) -> State {
    if let Some(ByteAction::Ignore) = action {
        tokenizer.move_one();
        State::Next(name)
    } else {
        let byte = if let Some(ByteAction::Insert(byte) | ByteAction::Normal(byte)) = action {
            Some(*byte)
        } else {
            None
        };

        log::debug!(
            "main: flushing: `{:?}` ({:?}) to {:?}",
            byte,
            tokenizer.point,
            name
        );
        tokenizer.expect(byte);
        call_impl(tokenizer, name)
    }
}

#[allow(clippy::too_many_lines)]
fn call_impl(tokenizer: &mut Tokenizer, name: StateName) -> State {
    let func = name.to_func();

    func(tokenizer)
}

fn move_point_back(tokenizer: &mut Tokenizer, point: &mut Point) {
    // Move back past ignored bytes.
    while point.index > 0 {
        point.index -= 1;
        let action = byte_action(tokenizer.parse_state.bytes, point);
        if !matches!(action, ByteAction::Ignore) {
            point.index += 1;
            break;
        }
    }
}
