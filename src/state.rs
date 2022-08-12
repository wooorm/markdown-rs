use crate::construct;
use crate::content;
use crate::tokenizer::Tokenizer;

/// The result of a state.
#[derive(Debug, PartialEq, Copy, Clone)]
pub enum State {
    /// Move to [`Name`][] next.
    Next(Name),
    /// Retry in [`Name`][].
    Retry(Name),
    /// The state is successful.
    Ok,
    /// The state is not successful.
    Nok,
}

/// Names of functions to move to.
#[derive(Debug, Clone, Copy, PartialEq)]
#[allow(clippy::enum_variant_names)]
pub enum Name {
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
    DocumentContainerExistingBefore,
    DocumentContainerExistingAfter,
    DocumentContainerNewBefore,
    DocumentContainerNewBeforeNotBlockQuote,
    DocumentContainerNewBeforeNotList,
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
    FlowBlankLineBefore,
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
    HtmlTextLineEndingBefore,
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
    ListBeforeOrdered,
    ListBeforeUnordered,
    ListValue,
    ListMarker,
    ListMarkerAfter,
    ListAfter,
    ListMarkerAfterFilled,
    ListWhitespace,
    ListPrefixOther,
    ListWhitespaceAfter,
    ListContStart,
    ListContBlank,
    ListContFilled,

    NonLazyContinuationStart,
    NonLazyContinuationAfter,

    ParagraphStart,
    ParagraphInside,

    SpaceOrTabStart,
    SpaceOrTabInside,
    SpaceOrTabAfter,

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
    TitleAtBreak,
    TitleAtBlankLine,
    TitleEscape,
    TitleInside,
}

#[allow(clippy::too_many_lines)]
/// Call the corresponding function for a state name.
pub fn call(tokenizer: &mut Tokenizer, name: Name) -> State {
    let func = match name {
        Name::AttentionStart => construct::attention::start,
        Name::AttentionInside => construct::attention::inside,

        Name::AutolinkStart => construct::autolink::start,
        Name::AutolinkOpen => construct::autolink::open,
        Name::AutolinkSchemeOrEmailAtext => construct::autolink::scheme_or_email_atext,
        Name::AutolinkSchemeInsideOrEmailAtext => construct::autolink::scheme_inside_or_email_atext,
        Name::AutolinkUrlInside => construct::autolink::url_inside,
        Name::AutolinkEmailAtSignOrDot => construct::autolink::email_at_sign_or_dot,
        Name::AutolinkEmailAtext => construct::autolink::email_atext,
        Name::AutolinkEmailValue => construct::autolink::email_value,
        Name::AutolinkEmailLabel => construct::autolink::email_label,

        Name::BlankLineStart => construct::blank_line::start,
        Name::BlankLineAfter => construct::blank_line::after,

        Name::BlockQuoteStart => construct::block_quote::start,
        Name::BlockQuoteContStart => construct::block_quote::cont_start,
        Name::BlockQuoteContBefore => construct::block_quote::cont_before,
        Name::BlockQuoteContAfter => construct::block_quote::cont_after,

        Name::BomStart => construct::partial_bom::start,
        Name::BomInside => construct::partial_bom::inside,

        Name::CharacterEscapeStart => construct::character_escape::start,
        Name::CharacterEscapeInside => construct::character_escape::inside,

        Name::CharacterReferenceStart => construct::character_reference::start,
        Name::CharacterReferenceOpen => construct::character_reference::open,
        Name::CharacterReferenceNumeric => construct::character_reference::numeric,
        Name::CharacterReferenceValue => construct::character_reference::value,

        Name::CodeFencedStart => construct::code_fenced::start,
        Name::CodeFencedBeforeSequenceOpen => construct::code_fenced::before_sequence_open,
        Name::CodeFencedSequenceOpen => construct::code_fenced::sequence_open,
        Name::CodeFencedInfoBefore => construct::code_fenced::info_before,
        Name::CodeFencedInfo => construct::code_fenced::info,
        Name::CodeFencedMetaBefore => construct::code_fenced::meta_before,
        Name::CodeFencedMeta => construct::code_fenced::meta,
        Name::CodeFencedAtNonLazyBreak => construct::code_fenced::at_non_lazy_break,
        Name::CodeFencedCloseBefore => construct::code_fenced::close_before,
        Name::CodeFencedCloseStart => construct::code_fenced::close_start,
        Name::CodeFencedBeforeSequenceClose => construct::code_fenced::before_sequence_close,
        Name::CodeFencedSequenceClose => construct::code_fenced::sequence_close,
        Name::CodeFencedAfterSequenceClose => construct::code_fenced::sequence_close_after,
        Name::CodeFencedContentBefore => construct::code_fenced::content_before,
        Name::CodeFencedContentStart => construct::code_fenced::content_start,
        Name::CodeFencedBeforeContentChunk => construct::code_fenced::before_content_chunk,
        Name::CodeFencedContentChunk => construct::code_fenced::content_chunk,
        Name::CodeFencedAfter => construct::code_fenced::after,

        Name::CodeIndentedStart => construct::code_indented::start,
        Name::CodeIndentedAtBreak => construct::code_indented::at_break,
        Name::CodeIndentedAfter => construct::code_indented::after,
        Name::CodeIndentedFurtherStart => construct::code_indented::further_start,
        Name::CodeIndentedInside => construct::code_indented::inside,
        Name::CodeIndentedFurtherBegin => construct::code_indented::further_begin,
        Name::CodeIndentedFurtherAfter => construct::code_indented::further_after,

        Name::CodeTextStart => construct::code_text::start,
        Name::CodeTextSequenceOpen => construct::code_text::sequence_open,
        Name::CodeTextBetween => construct::code_text::between,
        Name::CodeTextData => construct::code_text::data,
        Name::CodeTextSequenceClose => construct::code_text::sequence_close,

        Name::DataStart => construct::partial_data::start,
        Name::DataInside => construct::partial_data::inside,
        Name::DataAtBreak => construct::partial_data::at_break,

        Name::DefinitionStart => construct::definition::start,
        Name::DefinitionBefore => construct::definition::before,
        Name::DefinitionLabelAfter => construct::definition::label_after,
        Name::DefinitionMarkerAfter => construct::definition::marker_after,
        Name::DefinitionDestinationBefore => construct::definition::destination_before,
        Name::DefinitionDestinationAfter => construct::definition::destination_after,
        Name::DefinitionDestinationMissing => construct::definition::destination_missing,
        Name::DefinitionTitleBefore => construct::definition::title_before,
        Name::DefinitionAfter => construct::definition::after,
        Name::DefinitionAfterWhitespace => construct::definition::after_whitespace,
        Name::DefinitionTitleBeforeMarker => construct::definition::title_before_marker,
        Name::DefinitionTitleAfter => construct::definition::title_after,
        Name::DefinitionTitleAfterOptionalWhitespace => {
            construct::definition::title_after_optional_whitespace
        }

        Name::DestinationStart => construct::partial_destination::start,
        Name::DestinationEnclosedBefore => construct::partial_destination::enclosed_before,
        Name::DestinationEnclosed => construct::partial_destination::enclosed,
        Name::DestinationEnclosedEscape => construct::partial_destination::enclosed_escape,
        Name::DestinationRaw => construct::partial_destination::raw,
        Name::DestinationRawEscape => construct::partial_destination::raw_escape,

        Name::DocumentStart => content::document::start,
        Name::DocumentContainerExistingBefore => content::document::container_existing_before,
        Name::DocumentContainerExistingAfter => content::document::container_existing_after,
        Name::DocumentContainerNewBefore => content::document::container_new_before,
        Name::DocumentContainerNewBeforeNotBlockQuote => {
            content::document::container_new_before_not_block_quote
        }
        Name::DocumentContainerNewBeforeNotList => content::document::container_new_before_not_list,
        Name::DocumentContainerNewAfter => content::document::container_new_after,
        Name::DocumentContainersAfter => content::document::containers_after,
        Name::DocumentFlowEnd => content::document::flow_end,
        Name::DocumentFlowInside => content::document::flow_inside,

        Name::FlowStart => content::flow::start,
        Name::FlowBeforeCodeIndented => content::flow::before_code_indented,
        Name::FlowBeforeCodeFenced => content::flow::before_code_fenced,
        Name::FlowBeforeHtml => content::flow::before_html,
        Name::FlowBeforeHeadingAtx => content::flow::before_heading_atx,
        Name::FlowBeforeHeadingSetext => content::flow::before_heading_setext,
        Name::FlowBeforeThematicBreak => content::flow::before_thematic_break,
        Name::FlowBeforeDefinition => content::flow::before_definition,
        Name::FlowAfter => content::flow::after,
        Name::FlowBlankLineBefore => content::flow::blank_line_before,
        Name::FlowBlankLineAfter => content::flow::blank_line_after,
        Name::FlowBeforeParagraph => content::flow::before_paragraph,

        Name::HardBreakEscapeStart => construct::hard_break_escape::start,
        Name::HardBreakEscapeAfter => construct::hard_break_escape::after,

        Name::HeadingAtxStart => construct::heading_atx::start,
        Name::HeadingAtxBefore => construct::heading_atx::before,
        Name::HeadingAtxSequenceOpen => construct::heading_atx::sequence_open,
        Name::HeadingAtxAtBreak => construct::heading_atx::at_break,
        Name::HeadingAtxSequenceFurther => construct::heading_atx::sequence_further,
        Name::HeadingAtxData => construct::heading_atx::data,

        Name::HeadingSetextStart => construct::heading_setext::start,
        Name::HeadingSetextBefore => construct::heading_setext::before,
        Name::HeadingSetextInside => construct::heading_setext::inside,
        Name::HeadingSetextAfter => construct::heading_setext::after,

        Name::HtmlFlowStart => construct::html_flow::start,
        Name::HtmlFlowBefore => construct::html_flow::before,
        Name::HtmlFlowOpen => construct::html_flow::open,
        Name::HtmlFlowDeclarationOpen => construct::html_flow::declaration_open,
        Name::HtmlFlowCommentOpenInside => construct::html_flow::comment_open_inside,
        Name::HtmlFlowCdataOpenInside => construct::html_flow::cdata_open_inside,
        Name::HtmlFlowTagCloseStart => construct::html_flow::tag_close_start,
        Name::HtmlFlowTagName => construct::html_flow::tag_name,
        Name::HtmlFlowBasicSelfClosing => construct::html_flow::basic_self_closing,
        Name::HtmlFlowCompleteClosingTagAfter => construct::html_flow::complete_closing_tag_after,
        Name::HtmlFlowCompleteEnd => construct::html_flow::complete_end,
        Name::HtmlFlowCompleteAttributeNameBefore => {
            construct::html_flow::complete_attribute_name_before
        }
        Name::HtmlFlowCompleteAttributeName => construct::html_flow::complete_attribute_name,
        Name::HtmlFlowCompleteAttributeNameAfter => {
            construct::html_flow::complete_attribute_name_after
        }
        Name::HtmlFlowCompleteAttributeValueBefore => {
            construct::html_flow::complete_attribute_value_before
        }
        Name::HtmlFlowCompleteAttributeValueQuoted => {
            construct::html_flow::complete_attribute_value_quoted
        }
        Name::HtmlFlowCompleteAttributeValueQuotedAfter => {
            construct::html_flow::complete_attribute_value_quoted_after
        }
        Name::HtmlFlowCompleteAttributeValueUnquoted => {
            construct::html_flow::complete_attribute_value_unquoted
        }
        Name::HtmlFlowCompleteAfter => construct::html_flow::complete_after,
        Name::HtmlFlowBlankLineBefore => construct::html_flow::blank_line_before,
        Name::HtmlFlowContinuation => construct::html_flow::continuation,
        Name::HtmlFlowContinuationDeclarationInside => {
            construct::html_flow::continuation_declaration_inside
        }
        Name::HtmlFlowContinuationAfter => construct::html_flow::continuation_after,
        Name::HtmlFlowContinuationStart => construct::html_flow::continuation_start,
        Name::HtmlFlowContinuationBefore => construct::html_flow::continuation_before,
        Name::HtmlFlowContinuationCommentInside => {
            construct::html_flow::continuation_comment_inside
        }
        Name::HtmlFlowContinuationRawTagOpen => construct::html_flow::continuation_raw_tag_open,
        Name::HtmlFlowContinuationRawEndTag => construct::html_flow::continuation_raw_end_tag,
        Name::HtmlFlowContinuationClose => construct::html_flow::continuation_close,
        Name::HtmlFlowContinuationCdataInside => construct::html_flow::continuation_cdata_inside,
        Name::HtmlFlowContinuationStartNonLazy => construct::html_flow::continuation_start_non_lazy,

        Name::HtmlTextStart => construct::html_text::start,
        Name::HtmlTextOpen => construct::html_text::open,
        Name::HtmlTextDeclarationOpen => construct::html_text::declaration_open,
        Name::HtmlTextTagCloseStart => construct::html_text::tag_close_start,
        Name::HtmlTextTagClose => construct::html_text::tag_close,
        Name::HtmlTextTagCloseBetween => construct::html_text::tag_close_between,
        Name::HtmlTextTagOpen => construct::html_text::tag_open,
        Name::HtmlTextTagOpenBetween => construct::html_text::tag_open_between,
        Name::HtmlTextTagOpenAttributeName => construct::html_text::tag_open_attribute_name,
        Name::HtmlTextTagOpenAttributeNameAfter => {
            construct::html_text::tag_open_attribute_name_after
        }
        Name::HtmlTextTagOpenAttributeValueBefore => {
            construct::html_text::tag_open_attribute_value_before
        }
        Name::HtmlTextTagOpenAttributeValueQuoted => {
            construct::html_text::tag_open_attribute_value_quoted
        }
        Name::HtmlTextTagOpenAttributeValueQuotedAfter => {
            construct::html_text::tag_open_attribute_value_quoted_after
        }
        Name::HtmlTextTagOpenAttributeValueUnquoted => {
            construct::html_text::tag_open_attribute_value_unquoted
        }
        Name::HtmlTextCdata => construct::html_text::cdata,
        Name::HtmlTextCdataOpenInside => construct::html_text::cdata_open_inside,
        Name::HtmlTextCdataClose => construct::html_text::cdata_close,
        Name::HtmlTextCdataEnd => construct::html_text::cdata_end,
        Name::HtmlTextCommentOpenInside => construct::html_text::comment_open_inside,
        Name::HtmlTextCommentStart => construct::html_text::comment_start,
        Name::HtmlTextCommentStartDash => construct::html_text::comment_start_dash,
        Name::HtmlTextComment => construct::html_text::comment,
        Name::HtmlTextCommentClose => construct::html_text::comment_close,
        Name::HtmlTextDeclaration => construct::html_text::declaration,
        Name::HtmlTextEnd => construct::html_text::end,
        Name::HtmlTextInstruction => construct::html_text::instruction,
        Name::HtmlTextInstructionClose => construct::html_text::instruction_close,
        Name::HtmlTextLineEndingBefore => construct::html_text::line_ending_before,
        Name::HtmlTextLineEndingAfter => construct::html_text::line_ending_after,
        Name::HtmlTextLineEndingAfterPrefix => construct::html_text::line_ending_after_prefix,

        Name::LabelStart => construct::partial_label::start,
        Name::LabelAtBreak => construct::partial_label::at_break,
        Name::LabelEolAfter => construct::partial_label::eol_after,
        Name::LabelAtBlankLine => construct::partial_label::at_blank_line,
        Name::LabelEscape => construct::partial_label::escape,
        Name::LabelInside => construct::partial_label::inside,

        Name::LabelEndStart => construct::label_end::start,
        Name::LabelEndAfter => construct::label_end::after,
        Name::LabelEndResourceStart => construct::label_end::resource_start,
        Name::LabelEndResourceBefore => construct::label_end::resource_before,
        Name::LabelEndResourceOpen => construct::label_end::resource_open,
        Name::LabelEndResourceDestinationAfter => construct::label_end::resource_destination_after,
        Name::LabelEndResourceDestinationMissing => {
            construct::label_end::resource_destination_missing
        }
        Name::LabelEndResourceBetween => construct::label_end::resource_between,
        Name::LabelEndResourceTitleAfter => construct::label_end::resource_title_after,
        Name::LabelEndResourceEnd => construct::label_end::resource_end,
        Name::LabelEndOk => construct::label_end::ok,
        Name::LabelEndNok => construct::label_end::nok,
        Name::LabelEndReferenceFull => construct::label_end::reference_full,
        Name::LabelEndReferenceFullAfter => construct::label_end::reference_full_after,
        Name::LabelEndReferenceNotFull => construct::label_end::reference_not_full,
        Name::LabelEndReferenceCollapsed => construct::label_end::reference_collapsed,
        Name::LabelEndReferenceCollapsedOpen => construct::label_end::reference_collapsed_open,

        Name::LabelStartImageStart => construct::label_start_image::start,
        Name::LabelStartImageOpen => construct::label_start_image::open,
        Name::LabelStartLinkStart => construct::label_start_link::start,

        Name::ListStart => construct::list::start,
        Name::ListBefore => construct::list::before,
        Name::ListBeforeOrdered => construct::list::before_ordered,
        Name::ListBeforeUnordered => construct::list::before_unordered,
        Name::ListValue => construct::list::value,
        Name::ListMarker => construct::list::marker,
        Name::ListMarkerAfter => construct::list::marker_after,
        Name::ListAfter => construct::list::after,
        Name::ListMarkerAfterFilled => construct::list::marker_after_filled,
        Name::ListWhitespace => construct::list::whitespace,
        Name::ListWhitespaceAfter => construct::list::whitespace_after,
        Name::ListPrefixOther => construct::list::prefix_other,
        Name::ListContStart => construct::list::cont_start,
        Name::ListContBlank => construct::list::cont_blank,
        Name::ListContFilled => construct::list::cont_filled,

        Name::NonLazyContinuationStart => construct::partial_non_lazy_continuation::start,
        Name::NonLazyContinuationAfter => construct::partial_non_lazy_continuation::after,

        Name::ParagraphStart => construct::paragraph::start,
        Name::ParagraphInside => construct::paragraph::inside,

        Name::SpaceOrTabStart => construct::partial_space_or_tab::start,
        Name::SpaceOrTabInside => construct::partial_space_or_tab::inside,
        Name::SpaceOrTabAfter => construct::partial_space_or_tab::after,

        Name::SpaceOrTabEolStart => construct::partial_space_or_tab_eol::start,
        Name::SpaceOrTabEolAfterFirst => construct::partial_space_or_tab_eol::after_first,
        Name::SpaceOrTabEolAfterEol => construct::partial_space_or_tab_eol::after_eol,
        Name::SpaceOrTabEolAtEol => construct::partial_space_or_tab_eol::at_eol,
        Name::SpaceOrTabEolAfterMore => construct::partial_space_or_tab_eol::after_more,

        Name::StringStart => content::string::start,
        Name::StringBefore => content::string::before,
        Name::StringBeforeData => content::string::before_data,

        Name::TextStart => content::text::start,
        Name::TextBefore => content::text::before,
        Name::TextBeforeHtml => content::text::before_html,
        Name::TextBeforeHardBreakEscape => content::text::before_hard_break_escape,
        Name::TextBeforeData => content::text::before_data,

        Name::ThematicBreakStart => construct::thematic_break::start,
        Name::ThematicBreakBefore => construct::thematic_break::before,
        Name::ThematicBreakSequence => construct::thematic_break::sequence,
        Name::ThematicBreakAtBreak => construct::thematic_break::at_break,

        Name::TitleStart => construct::partial_title::start,
        Name::TitleBegin => construct::partial_title::begin,
        Name::TitleAfterEol => construct::partial_title::after_eol,
        Name::TitleAtBreak => construct::partial_title::at_break,
        Name::TitleAtBlankLine => construct::partial_title::at_blank_line,
        Name::TitleEscape => construct::partial_title::escape,
        Name::TitleInside => construct::partial_title::inside,
    };

    func(tokenizer)
}
