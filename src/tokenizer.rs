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

/// To do: could we do without `HashMap`, so we don’t need `std`?
use std::collections::HashMap;

use crate::constant::TAB_SIZE;
use crate::parser::ParseState;

/// Semantic label of a span.
// To do: figure out how to share this so extensions can add their own stuff,
// though perhaps that’s impossible and we should inline all extensions?
#[derive(Debug, Clone, PartialEq)]
pub enum TokenType {
    /// Whole autolink.
    ///
    /// ## Info
    ///
    /// *   **Context**:
    ///     [text content][crate::content::text]
    /// *   **Content model**:
    ///     [`AutolinkEmail`][TokenType::AutolinkEmail],
    ///     [`AutolinkMarker`][TokenType::AutolinkMarker],
    ///     [`AutolinkProtocol`][TokenType::AutolinkProtocol]
    /// *   **Construct**:
    ///     [`autolink`][crate::construct::autolink]
    ///
    /// ## Example
    ///
    /// ```markdown
    /// > | <https://example.com> and <admin@example.com>
    ///     ^^^^^^^^^^^^^^^^^^^^^     ^^^^^^^^^^^^^^^^^^^
    /// ```
    Autolink,
    /// Email autolink w/o markers.
    ///
    /// ## Info
    ///
    /// *   **Context**:
    ///     [`Autolink`][TokenType::Autolink]
    /// *   **Content model**:
    ///     void
    /// *   **Construct**:
    ///     [`autolink`][crate::construct::autolink]
    ///
    /// ## Example
    ///
    /// ```markdown
    /// > | <admin@example.com>
    ///      ^^^^^^^^^^^^^^^^^
    /// ```
    AutolinkEmail,
    /// Marker of an autolink.
    ///
    /// ## Info
    ///
    /// *   **Context**:
    ///     [`Autolink`][TokenType::Autolink]
    /// *   **Content model**:
    ///     void
    /// *   **Construct**:
    ///     [`autolink`][crate::construct::autolink]
    ///
    /// ## Example
    ///
    /// ```markdown
    /// > | <https://example.com>
    ///     ^                   ^
    /// ```
    AutolinkMarker,
    /// Protocol autolink w/o markers.
    ///
    /// ## Info
    ///
    /// *   **Context**:
    ///     [`Autolink`][TokenType::Autolink]
    /// *   **Content model**:
    ///     void
    /// *   **Construct**:
    ///     [`autolink`][crate::construct::autolink]
    ///
    /// ## Example
    ///
    /// ```markdown
    /// > | <https://example.com>
    ///      ^^^^^^^^^^^^^^^^^^^
    /// ```
    AutolinkProtocol,
    /// Line ending preceded only by whitespace or nothing at all.
    ///
    /// ## Info
    ///
    /// *   **Context**:
    ///     [flow content][crate::content::flow]
    /// *   **Content model**:
    ///     void
    /// *   **Construct**:
    ///     [`blank_line`][crate::construct::blank_line]
    ///
    /// ## Example
    ///
    /// ```markdown
    /// > | ␠␠␊
    ///       ^
    /// ```
    BlankLineEnding,
    /// Whole character escape.
    ///
    /// ## Info
    ///
    /// *   **Context**:
    ///     [string content][crate::content::string] or
    ///     [text content][crate::content::text]
    /// *   **Content model**:
    ///     [`CharacterEscapeMarker`][TokenType::CharacterEscapeMarker],
    ///     [`CharacterEscapeValue`][TokenType::CharacterEscapeValue]
    /// *   **Construct**:
    ///     [`character_escape`][crate::construct::character_escape]
    ///
    /// ## Example
    ///
    /// ```markdown
    /// > | a \- b
    ///       ^^
    /// ```
    CharacterEscape,
    /// Character escape marker.
    ///
    /// ## Info
    ///
    /// *   **Context**:
    ///     [`CharacterEscape`][TokenType::CharacterEscape]
    /// *   **Content model**:
    ///     void
    /// *   **Construct**:
    ///     [`character_escape`][crate::construct::character_escape]
    ///
    /// ## Example
    ///
    /// ```markdown
    /// > | a \- b
    ///       ^
    /// ```
    CharacterEscapeMarker,
    /// Character escape value.
    ///
    /// ## Info
    ///
    /// *   **Context**:
    ///     [`CharacterEscape`][TokenType::CharacterEscape]
    /// *   **Content model**:
    ///     void
    /// *   **Construct**:
    ///     [`character_escape`][crate::construct::character_escape]
    ///
    /// ## Example
    ///
    /// ```markdown
    /// > | a \- b
    ///        ^
    /// ```
    CharacterEscapeValue,
    /// Whole character reference.
    ///
    /// ## Info
    ///
    /// *   **Context**:
    ///     [string content][crate::content::string] or
    ///     [text content][crate::content::text]
    /// *   **Content model**:
    ///     [`CharacterReferenceMarker`][TokenType::CharacterReferenceMarker],
    ///     [`CharacterReferenceMarkerHexadecimal`][TokenType::CharacterReferenceMarkerHexadecimal],
    ///     [`CharacterReferenceMarkerNumeric`][TokenType::CharacterReferenceMarkerNumeric],
    ///     [`CharacterReferenceMarkerSemi`][TokenType::CharacterReferenceMarkerSemi],
    ///     [`CharacterReferenceValue`][TokenType::CharacterReferenceValue]
    /// *   **Construct**:
    ///     [`character_reference`][crate::construct::character_reference]
    ///
    /// ## Example
    ///
    /// ```markdown
    /// > | a &amp; b &#8800; c &#x1D306; d
    ///       ^^^^^   ^^^^^^^   ^^^^^^^^^
    /// ```
    CharacterReference,
    /// Character reference opening marker.
    ///
    /// ## Info
    ///
    /// *   **Context**:
    ///     [`CharacterReference`][TokenType::CharacterReference]
    /// *   **Content model**:
    ///     void
    /// *   **Construct**:
    ///     [`character_reference`][crate::construct::character_reference]
    ///
    /// ## Example
    ///
    /// ```markdown
    /// > | a &amp; b &#8800; c &#x1D306; d
    ///       ^       ^         ^
    /// ```
    CharacterReferenceMarker,
    /// Character reference numeric marker.
    ///
    /// ## Info
    ///
    /// *   **Context**:
    ///     [`CharacterReference`][TokenType::CharacterReference]
    /// *   **Content model**:
    ///     void
    /// *   **Construct**:
    ///     [`character_reference`][crate::construct::character_reference]
    ///
    /// ## Example
    ///
    /// ```markdown
    /// > | a &amp; b &#8800; c &#x1D306; d
    ///                ^         ^
    /// ```
    CharacterReferenceMarkerNumeric,
    /// Character reference hexadecimal numeric marker.
    ///
    /// ## Info
    ///
    /// *   **Context**:
    ///     [`CharacterReference`][TokenType::CharacterReference]
    /// *   **Content model**:
    ///     void
    /// *   **Construct**:
    ///     [`character_reference`][crate::construct::character_reference]
    ///
    /// ## Example
    ///
    /// ```markdown
    /// > | a &amp; b &#8800; c &#x1D306; d
    ///                           ^
    /// ```
    CharacterReferenceMarkerHexadecimal,
    /// Character reference closing marker.
    ///
    /// ## Info
    ///
    /// *   **Context**:
    ///     [`CharacterReference`][TokenType::CharacterReference]
    /// *   **Content model**:
    ///     void
    /// *   **Construct**:
    ///     [`character_reference`][crate::construct::character_reference]
    ///
    /// ## Example
    ///
    /// ```markdown
    /// > | a &amp; b &#8800; c &#x1D306; d
    ///           ^         ^           ^
    /// ```
    CharacterReferenceMarkerSemi,
    /// Character reference value.
    ///
    /// ## Info
    ///
    /// *   **Context**:
    ///     [`CharacterReference`][TokenType::CharacterReference]
    /// *   **Content model**:
    ///     void
    /// *   **Construct**:
    ///     [`character_reference`][crate::construct::character_reference]
    ///
    /// ## Example
    ///
    /// ```markdown
    /// > | a &amp; b &#8800; c &#x1D306; d
    ///        ^^^      ^^^^       ^^^^^
    /// ```
    CharacterReferenceValue,
    /// Whole code (fenced).
    ///
    /// ## Info
    ///
    /// *   **Context**:
    ///     [flow content][crate::content::flow]
    /// *   **Content model**:
    ///     [`CodeFencedFence`][TokenType::CodeFencedFence],
    ///     [`CodeFlowChunk`][TokenType::CodeFlowChunk],
    ///     [`LineEnding`][TokenType::LineEnding],
    ///     [`SpaceOrTab`][TokenType::SpaceOrTab]
    /// *   **Construct**:
    ///     [`code_fenced`][crate::construct::code_fenced]
    ///
    /// ## Example
    ///
    /// ````markdown
    /// > | ```js
    ///     ^^^^^
    /// > | console.log(1)
    ///     ^^^^^^^^^^^^^^
    /// > | ```
    ///     ^^^
    /// ````
    CodeFenced,
    /// A code (fenced) fence.
    ///
    /// ## Info
    ///
    /// *   **Context**:
    ///     [`CodeFenced`][TokenType::CodeFenced]
    /// *   **Content model**:
    ///     [`CodeFencedFenceInfo`][TokenType::CodeFencedFenceInfo],
    ///     [`CodeFencedFenceMeta`][TokenType::CodeFencedFenceMeta],
    ///     [`CodeFencedFenceSequence`][TokenType::CodeFencedFenceSequence],
    ///     [`SpaceOrTab`][TokenType::SpaceOrTab]
    /// *   **Construct**:
    ///     [`code_fenced`][crate::construct::code_fenced]
    ///
    /// ## Example
    ///
    /// ````markdown
    /// > | ```js
    ///     ^^^^^
    ///   | console.log(1)
    /// > | ```
    ///     ^^^
    /// ````
    CodeFencedFence,
    /// A code (fenced) fence sequence.
    ///
    /// ## Info
    ///
    /// *   **Context**:
    ///     [`CodeFencedFenceSequence`][TokenType::CodeFencedFenceSequence]
    /// *   **Content model**:
    ///     void
    /// *   **Construct**:
    ///     [`code_fenced`][crate::construct::code_fenced]
    ///
    /// ## Example
    ///
    /// ````markdown
    /// > | ```js
    ///     ^^^
    ///   | console.log(1)
    /// > | ```
    ///     ^^^
    /// ````
    CodeFencedFenceSequence,
    /// A code (fenced) fence info word.
    ///
    /// ## Info
    ///
    /// *   **Context**:
    ///     [`CodeFencedFence`][TokenType::CodeFencedFence]
    /// *   **Content model**:
    ///     [string content][crate::content::string]
    /// *   **Construct**:
    ///     [`code_fenced`][crate::construct::code_fenced]
    ///
    /// ## Example
    ///
    /// ````markdown
    /// > | ```js
    ///        ^^
    ///   | console.log(1)
    ///   | ```
    /// ````
    CodeFencedFenceInfo,
    /// A code (fenced) fence meta string.
    ///
    /// ## Info
    ///
    /// *   **Context**:
    ///     [`CodeFencedFence`][TokenType::CodeFencedFence]
    /// *   **Content model**:
    ///     [string content][crate::content::string]
    /// *   **Construct**:
    ///     [`code_fenced`][crate::construct::code_fenced]
    ///
    /// ## Example
    ///
    /// ````markdown
    /// > | ```js highlight="1"
    ///           ^^^^^^^^^^^^^
    ///   | console.log(1)
    ///   | ```
    /// ````
    CodeFencedFenceMeta,
    /// A code (fenced, indented) chunk.
    ///
    /// ## Info
    ///
    /// *   **Context**:
    ///     [`CodeFenced`][TokenType::CodeFenced],
    ///     [`CodeIndented`][TokenType::CodeIndented]
    /// *   **Content model**:
    ///     void
    /// *   **Construct**:
    ///     [`code_fenced`][crate::construct::code_fenced],
    ///     [`code_indented`][crate::construct::code_indented]
    ///
    /// ## Example
    ///
    /// ````markdown
    ///   | ```js
    /// > | console.log(1)
    ///     ^^^^^^^^^^^^^^
    ///   | ```
    /// ````
    ///
    /// ```markdown
    /// > | ␠␠␠␠console.log(1)
    ///         ^^^^^^^^^^^^^^
    /// ```
    CodeFlowChunk,
    /// Whole code (indented).
    ///
    /// ## Info
    ///
    /// *   **Context**:
    ///     [flow content][crate::content::flow]
    /// *   **Content model**:
    ///     [`CodeFlowChunk`][TokenType::CodeFlowChunk],
    ///     [`LineEnding`][TokenType::LineEnding],
    ///     [`SpaceOrTab`][TokenType::SpaceOrTab]
    /// *   **Construct**:
    ///     [`code_fenced`][crate::construct::code_fenced]
    ///
    /// ## Example
    ///
    /// ```markdown
    /// ␠␠␠␠console.log(1)
    /// ^^^^^^^^^^^^^^^^^^
    /// ```
    CodeIndented,
    /// Whole code (text).
    ///
    /// ## Info
    ///
    /// *   **Context**:
    ///     [text content][crate::content::text]
    /// *   **Content model**:
    ///     [`CodeTextData`][TokenType::CodeTextData],
    ///     [`CodeTextSequence`][TokenType::CodeTextSequence],
    ///     [`CodeTextLineEnding`][TokenType::CodeTextLineEnding]
    /// *   **Construct**:
    ///     [`code_text`][crate::construct::code_text]
    ///
    /// ## Example
    ///
    /// ```markdown
    /// > | a `b` c
    ///       ^^^
    /// ```
    CodeText,
    /// Code (text) data.
    ///
    /// ## Info
    ///
    /// *   **Context**:
    ///     [`CodeText`][TokenType::CodeText],
    /// *   **Content model**:
    ///     void
    /// *   **Construct**:
    ///     [`code_text`][crate::construct::code_text]
    ///
    /// ## Example
    ///
    /// ```markdown
    /// > | a `b` c
    ///        ^
    /// ```
    CodeTextData,
    /// Code (text) sequence.
    ///
    /// ## Info
    ///
    /// *   **Context**:
    ///     [`CodeText`][TokenType::CodeText],
    /// *   **Content model**:
    ///     void
    /// *   **Construct**:
    ///     [`code_text`][crate::construct::code_text]
    ///
    /// ## Example
    ///
    /// ```markdown
    /// > | a `b` c
    ///       ^ ^
    /// ```
    CodeTextSequence,
    /// Line ending in code (text).
    ///
    /// ## Info
    ///
    /// *   **Context**:
    ///     [`CodeText`][TokenType::CodeText],
    /// *   **Content model**:
    ///     void
    /// *   **Construct**:
    ///     [`code_text`][crate::construct::code_text]
    ///
    /// ## Example
    ///
    /// ```markdown
    /// > | a `b␊
    ///         ^
    ///   | c` d
    /// ```
    CodeTextLineEnding,
    /// Data.
    ///
    /// ## Info
    ///
    /// *   **Context**:
    ///     [string content][crate::content::string],
    ///     [text content][crate::content::text]
    /// *   **Content model**:
    ///     void
    /// *   **Construct**:
    ///     [`partial_data`][crate::construct::partial_data]
    ///
    /// ## Example
    ///
    /// ```markdown
    /// > | aa *bb* cc
    ///     ^^^ ^^ ^^^
    /// ```
    Data,
    /// Whole definition.
    ///
    /// ## Info
    ///
    /// *   **Context**:
    ///     [flow content][crate::content::flow]
    /// *   **Content model**:
    ///     [`DefinitionMarker`][TokenType::DefinitionMarker],
    ///     [`DefinitionLabel`][TokenType::DefinitionLabel],
    ///     [`DefinitionDestination`][TokenType::DefinitionDestination],
    ///     [`DefinitionTitle`][TokenType::DefinitionTitle],
    ///     [`LineEnding`][TokenType::LineEnding],
    ///     [`SpaceOrTab`][TokenType::SpaceOrTab]
    /// *   **Construct**:
    ///     [`definition`][crate::construct::definition]
    ///
    /// ## Example
    ///
    /// ```markdown
    /// > | [a]: b "c"
    ///     ^^^^^^^^^^
    /// ```
    Definition,
    /// Whole definition label.
    ///
    /// ## Info
    ///
    /// *   **Context**:
    ///     [`Definition`][TokenType::Definition]
    /// *   **Content model**:
    ///     [`DefinitionLabelMarker`][TokenType::DefinitionLabelMarker],
    ///     [`DefinitionLabelString`][TokenType::DefinitionLabelString],
    ///     [`LineEnding`][TokenType::LineEnding],
    ///     [`SpaceOrTab`][TokenType::SpaceOrTab]
    /// *   **Construct**:
    ///     [`partial_label`][crate::construct::partial_label]
    ///
    /// ## Example
    ///
    /// ```markdown
    /// > | [a]: b "c"
    ///     ^^^
    /// ```
    DefinitionLabel,
    /// Definition label marker.
    ///
    /// ## Info
    ///
    /// *   **Context**:
    ///     [`DefinitionLabel`][TokenType::DefinitionLabel]
    /// *   **Content model**:
    ///     void
    /// *   **Construct**:
    ///     [`partial_label`][crate::construct::partial_label]
    ///
    /// ## Example
    ///
    /// ```markdown
    /// > | [a]: b "c"
    ///     ^ ^
    /// ```
    DefinitionLabelMarker,
    /// Definition label data.
    ///
    /// ## Info
    ///
    /// *   **Context**:
    ///     [`DefinitionLabel`][TokenType::DefinitionLabel]
    /// *   **Content model**:
    ///     [string content][crate::content::string]
    /// *   **Construct**:
    ///     [`partial_label`][crate::construct::partial_label]
    ///
    /// ## Example
    ///
    /// ```markdown
    /// > | [a]: b "c"
    ///      ^
    /// ```
    DefinitionLabelString,
    /// Definition marker.
    ///
    /// ## Info
    ///
    /// *   **Context**:
    ///     [`Definition`][TokenType::Definition]
    /// *   **Content model**:
    ///     void
    /// *   **Construct**:
    ///     [`definition`][crate::construct::definition]
    ///
    /// ## Example
    ///
    /// ```markdown
    /// > | [a]: b "c"
    ///        ^
    /// ```
    DefinitionMarker,
    /// Whole definition destination.
    ///
    /// ## Info
    ///
    /// *   **Context**:
    ///     [`Definition`][TokenType::Definition]
    /// *   **Content model**:
    ///     [`DefinitionDestinationLiteral`][TokenType::DefinitionDestinationLiteral],
    ///     [`DefinitionDestinationRaw`][TokenType::DefinitionDestinationRaw]
    /// *   **Construct**:
    ///     [`partial_destination`][crate::construct::partial_destination]
    ///
    /// ## Example
    ///
    /// ```markdown
    /// > | [a]: b "c"
    ///          ^
    /// > | [a]: <b> "c"
    ///          ^^^
    /// ```
    DefinitionDestination,
    /// Definition destination literal.
    ///
    /// ## Info
    ///
    /// *   **Context**:
    ///     [`DefinitionDestination`][TokenType::DefinitionDestination]
    /// *   **Content model**:
    ///     [`DefinitionDestinationLiteralMarker`][TokenType::DefinitionDestinationLiteralMarker],
    ///     [`DefinitionDestinationString`][TokenType::DefinitionDestinationString]
    /// *   **Construct**:
    ///     [`partial_destination`][crate::construct::partial_destination]
    ///
    /// ## Example
    ///
    /// ```markdown
    /// > | [a]: <b> "c"
    ///          ^^^
    /// ```
    DefinitionDestinationLiteral,
    /// Definition destination literal marker.
    ///
    /// ## Info
    ///
    /// *   **Context**:
    ///     [`DefinitionDestinationLiteral`][TokenType::DefinitionDestinationLiteral]
    /// *   **Content model**:
    ///     void
    /// *   **Construct**:
    ///     [`partial_destination`][crate::construct::partial_destination]
    ///
    /// ## Example
    ///
    /// ```markdown
    /// > | [a]: <b> "c"
    ///          ^ ^
    /// ```
    DefinitionDestinationLiteralMarker,
    /// Definition destination raw.
    ///
    /// ## Info
    ///
    /// *   **Context**:
    ///     [`DefinitionDestination`][TokenType::DefinitionDestination]
    /// *   **Content model**:
    ///     [`DefinitionDestinationString`][TokenType::DefinitionDestinationString]
    /// *   **Construct**:
    ///     [`partial_destination`][crate::construct::partial_destination]
    ///
    /// ## Example
    ///
    /// ```markdown
    /// > | [a]: b "c"
    ///          ^
    /// ```
    DefinitionDestinationRaw,
    /// Definition destination data.
    ///
    /// ## Info
    ///
    /// *   **Context**:
    ///     [`DefinitionDestinationLiteral`][TokenType::DefinitionDestinationLiteral],
    ///     [`DefinitionDestinationRaw`][TokenType::DefinitionDestinationRaw]
    /// *   **Content model**:
    ///     [string content][crate::content::string]
    /// *   **Construct**:
    ///     [`partial_destination`][crate::construct::partial_destination]
    ///
    /// ## Example
    ///
    /// ```markdown
    /// > | [a]: b "c"
    ///          ^
    /// > | [a]: <b> "c"
    ///           ^
    /// ```
    DefinitionDestinationString,
    /// Whole definition title.
    ///
    /// ## Info
    ///
    /// *   **Context**:
    ///     [`Definition`][TokenType::Definition]
    /// *   **Content model**:
    ///     [`DefinitionTitleMarker`][TokenType::DefinitionTitleMarker],
    ///     [`DefinitionTitleString`][TokenType::DefinitionTitleString],
    ///     [`LineEnding`][TokenType::LineEnding],
    ///     [`SpaceOrTab`][TokenType::SpaceOrTab]
    /// *   **Construct**:
    ///     [`partial_title`][crate::construct::partial_title]
    ///
    /// ## Example
    ///
    /// ```markdown
    /// > | [a]: b "c"
    ///            ^^^
    /// ```
    DefinitionTitle,
    /// Definition title marker.
    ///
    /// ## Info
    ///
    /// *   **Context**:
    ///     [`DefinitionTitle`][TokenType::DefinitionTitle]
    /// *   **Content model**:
    ///     void
    /// *   **Construct**:
    ///     [`partial_title`][crate::construct::partial_title]
    ///
    /// ## Example
    ///
    /// ```markdown
    /// > | [a]: b "c"
    ///            ^ ^
    /// ```
    DefinitionTitleMarker,
    /// Definition title data.
    ///
    /// ## Info
    ///
    /// *   **Context**:
    ///     [`DefinitionTitle`][TokenType::DefinitionTitle]
    /// *   **Content model**:
    ///     [string content][crate::content::string]
    /// *   **Construct**:
    ///     [`partial_title`][crate::construct::partial_title]
    ///
    /// ## Example
    ///
    /// ```markdown
    /// > | [a]: b "c"
    ///             ^
    /// ```
    DefinitionTitleString,
    /// Whole hard break (escape).
    ///
    /// ## Info
    ///
    /// *   **Context**:
    ///     [text content][crate::content::text]
    /// *   **Content model**:
    ///     [`HardBreakEscapeMarker`][TokenType::HardBreakEscapeMarker]
    /// *   **Construct**:
    ///     [`hard_break_escape`][crate::construct::hard_break_escape]
    ///
    /// ## Example
    ///
    /// ```markdown
    /// > | a\␊
    ///      ^^
    /// > | b
    /// ```
    HardBreakEscape,
    /// Hard break (escape) marker.
    ///
    /// ## Info
    ///
    /// *   **Context**:
    ///     [text content][crate::content::text]
    /// *   **Content model**:
    ///     void
    /// *   **Construct**:
    ///     [`hard_break_escape`][crate::construct::hard_break_escape]
    ///
    /// ## Example
    ///
    /// ```markdown
    /// > | a\␊
    ///      ^
    /// > | b
    /// ```
    HardBreakEscapeMarker,
    /// Whole hard break (trailing).
    ///
    /// ## Info
    ///
    /// *   **Context**:
    ///     [text content][crate::content::text]
    /// *   **Content model**:
    ///     [`HardBreakTrailingSpace`][TokenType::HardBreakTrailingSpace]
    /// *   **Construct**:
    ///     [`hard_break_trailing`][crate::construct::hard_break_trailing]
    ///
    /// ## Example
    ///
    /// ```markdown
    /// > | a␠␠␊
    ///      ^^^
    /// > | b
    /// ```
    HardBreakTrailing,
    /// Hard break (trailing) spaces.
    ///
    /// ## Info
    ///
    /// *   **Context**:
    ///     [`HardBreakTrailing`][TokenType::HardBreakTrailing]
    /// *   **Content model**:
    ///     void
    /// *   **Construct**:
    ///     [`hard_break_trailing`][crate::construct::hard_break_trailing]
    ///
    /// ## Example
    ///
    /// ```markdown
    /// > | a␠␠␊
    ///      ^^
    /// > | b
    /// ```
    HardBreakTrailingSpace,
    /// Whole heading (atx).
    ///
    /// ## Info
    ///
    /// *   **Context**:
    ///     [flow content][crate::content::flow]
    /// *   **Content model**:
    ///     [`HeadingAtxSequence`][TokenType::HeadingAtxSequence],
    ///     [`HeadingAtxText`][TokenType::HeadingAtxText],
    ///     [`SpaceOrTab`][TokenType::SpaceOrTab]
    /// *   **Construct**:
    ///     [`heading_atx`][crate::construct::heading_atx]
    ///
    /// ## Example
    ///
    /// ```markdown
    /// > | # alpha
    ///     ^^^^^^^
    /// ```
    HeadingAtx,
    /// Heading (atx) sequence.
    ///
    /// ## Info
    ///
    /// *   **Context**:
    ///     [`HeadingAtx`][TokenType::HeadingAtx]
    /// *   **Content model**:
    ///     void
    /// *   **Construct**:
    ///     [`heading_atx`][crate::construct::heading_atx]
    ///
    /// ## Example
    ///
    /// ```markdown
    /// > | # alpha
    ///     ^
    /// ```
    HeadingAtxSequence,
    /// Heading (atx) data.
    ///
    /// ## Info
    ///
    /// *   **Context**:
    ///     [`HeadingAtx`][TokenType::HeadingAtx],
    /// *   **Content model**:
    ///     [text content][crate::content::text]
    /// *   **Construct**:
    ///     [`heading_atx`][crate::construct::heading_atx]
    ///
    /// ## Example
    ///
    /// ```markdown
    /// > | # alpha
    ///       ^^^^^
    /// ```
    HeadingAtxText,
    /// Whole heading (setext).
    ///
    /// ## Info
    ///
    /// *   **Context**:
    ///     [flow content][crate::content::flow]
    /// *   **Content model**:
    ///     [`HeadingSetextText`][TokenType::HeadingSetextText],
    ///     [`HeadingSetextUnderline`][TokenType::HeadingSetextUnderline],
    ///     [`LineEnding`][TokenType::LineEnding],
    ///     [`SpaceOrTab`][TokenType::SpaceOrTab]
    /// *   **Construct**:
    ///     [`heading_setext`][crate::construct::heading_setext]
    ///
    /// ## Example
    ///
    /// ```markdown
    /// > | alpha
    ///     ^^^^^
    /// > | =====
    ///     ^^^^^
    /// ```
    HeadingSetext,
    /// Heading (setext) data.
    ///
    /// ## Info
    ///
    /// *   **Context**:
    ///     [`HeadingSetext`][TokenType::HeadingSetext]
    /// *   **Content model**:
    ///     [text content][crate::content::text]
    /// *   **Construct**:
    ///     [`heading_setext`][crate::construct::heading_setext]
    ///
    /// ## Example
    ///
    /// ```markdown
    /// > | alpha
    ///     ^^^^^
    ///   | =====
    /// ```
    HeadingSetextText,
    /// Heading (setext) underline.
    ///
    /// ## Info
    ///
    /// *   **Context**:
    ///     [`HeadingSetext`][TokenType::HeadingSetext]
    /// *   **Content model**:
    ///     void
    /// *   **Construct**:
    ///     [`heading_setext`][crate::construct::heading_setext]
    ///
    /// ## Example
    ///
    /// ```markdown
    ///   | alpha
    /// > | =====
    ///     ^^^^^
    /// ```
    HeadingSetextUnderline,
    /// Whole html (flow).
    ///
    /// ## Info
    ///
    /// *   **Context**:
    ///     [flow content][crate::content::flow]
    /// *   **Content model**:
    ///     [`HtmlFlowData`][TokenType::HtmlFlowData],
    ///     [`LineEnding`][TokenType::LineEnding],
    ///     [`SpaceOrTab`][TokenType::SpaceOrTab]
    /// *   **Construct**:
    ///     [`html_flow`][crate::construct::html_flow]
    ///
    /// ## Example
    ///
    /// ```markdown
    /// > | <div>
    ///     ^^^^^
    /// ```
    HtmlFlow,
    /// HTML (flow) data.
    ///
    /// ## Info
    ///
    /// *   **Context**:
    ///     [`HtmlFlow`][TokenType::HtmlFlow],
    /// *   **Content model**:
    ///     void
    /// *   **Construct**:
    ///     [`html_flow`][crate::construct::html_flow]
    ///
    /// ## Example
    ///
    /// ```markdown
    /// > | <div>
    ///     ^^^^^
    /// ```
    HtmlFlowData,
    /// Whole html (text).
    ///
    /// ## Info
    ///
    /// *   **Context**:
    ///     [text content][crate::content::text]
    /// *   **Content model**:
    ///     [`HtmlTextData`][TokenType::HtmlTextData],
    ///     [`LineEnding`][TokenType::LineEnding],
    ///     [`SpaceOrTab`][TokenType::SpaceOrTab]
    /// *   **Construct**:
    ///     [`html_text`][crate::construct::html_text]
    ///
    /// ## Example
    ///
    /// ```markdown
    /// > | a <b> c
    ///       ^^^
    /// ```
    HtmlText,
    /// HTML (text) data.
    ///
    /// ## Info
    ///
    /// *   **Context**:
    ///     [`HtmlText`][TokenType::HtmlText]
    /// *   **Content model**:
    ///     void
    /// *   **Construct**:
    ///     [`html_text`][crate::construct::html_text]
    ///
    /// ## Example
    ///
    /// ```markdown
    /// > | a <b> c
    ///       ^^^
    /// ```
    HtmlTextData,
    /// To do,
    LabelImage,
    /// To do,
    LabelImageMarker,
    /// To do,
    LabelLink,
    /// To do,
    LabelMarker,
    LabelEnd,
    Resource,
    ResourceMarker,
    ResourceDestination,
    ResourceDestinationLiteral,
    ResourceDestinationLiteralMarker,
    ResourceDestinationRaw,
    ResourceDestinationString,
    ResourceTitle,
    ResourceTitleMarker,
    ResourceTitleString,
    Reference,
    ReferenceMarker,
    ReferenceString,
    Link,
    Image,
    Label,
    LabelText,
    /// Line ending.
    ///
    /// ## Info
    ///
    /// *   **Context**:
    ///     basically everywhere
    /// *   **Content model**:
    ///     void
    /// *   **Construct**:
    ///     n/a
    ///
    /// ## Example
    ///
    /// ```markdown
    /// > | a␊
    ///      ^
    ///   | b
    /// ```
    LineEnding,
    /// Whole paragraph.
    ///
    /// ## Info
    ///
    /// *   **Context**:
    ///     [flow content][crate::content::flow]
    /// *   **Content model**:
    ///     [text content][crate::content::text]
    /// *   **Construct**:
    ///     [`paragraph`][crate::construct::paragraph]
    ///
    /// ## Example
    ///
    /// ```markdown
    /// > | a b
    ///     ^^^
    /// > | c.
    ///     ^^
    /// ```
    Paragraph,
    /// Whole thematic break.
    ///
    /// ## Info
    ///
    /// *   **Context**:
    ///     [flow content][crate::content::flow]
    /// *   **Content model**:
    ///     [`ThematicBreakSequence`][TokenType::ThematicBreakSequence],
    ///     [`SpaceOrTab`][TokenType::SpaceOrTab]
    /// *   **Construct**:
    ///     [`thematic_break`][crate::construct::thematic_break]
    ///
    /// ## Example
    ///
    /// ```markdown
    /// > | * * *
    ///     ^^^^^
    /// ```
    ThematicBreak,
    /// Thematic break sequence.
    ///
    /// ## Info
    ///
    /// *   **Context**:
    ///     [`ThematicBreak`][TokenType::ThematicBreak]
    /// *   **Content model**:
    ///     void
    /// *   **Construct**:
    ///     [`thematic_break`][crate::construct::thematic_break]
    ///
    /// ## Example
    ///
    /// ```markdown
    /// > | * * *
    ///     ^ ^ ^
    /// ```
    ThematicBreakSequence,
    /// SpaceOrTab.
    ///
    /// ## Info
    ///
    /// *   **Context**:
    ///     basically everywhere
    /// *   **Content model**:
    ///     void
    /// *   **Construct**:
    ///     n/a
    ///
    /// ## Example
    ///
    /// ```markdown
    /// > | ␠* * *␠
    ///     ^ ^ ^ ^
    /// ```
    SpaceOrTab,
}

/// To do
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ContentType {
    Text,
    String,
}

/// Enum representing a character code.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Code {
    /// End of the input stream (called eof).
    None,
    /// Used to make parsing line endings easier as it represents both
    /// `Code::Char('\r')` and `Code::Char('\n')` combined.
    CarriageReturnLineFeed,
    /// the expansion of a tab (`Code::Char('\t')`), depending on where the tab
    /// ocurred, it’s followed by 0 to 3 (both inclusive) `Code::VirtualSpace`s.
    VirtualSpace,
    /// The most frequent variant of this enum is `Code::Char(char)`, which just
    /// represents a char, but micromark adds meaning to certain other values.
    Char(char),
}

/// A location in the document (`line`/`column`/`offset`).
///
/// The interface for the location in the document comes from unist `Point`:
/// <https://github.com/syntax-tree/unist#point>.
#[derive(Debug, Clone, PartialEq)]
pub struct Point {
    /// 1-indexed line number.
    pub line: usize,
    /// 1-indexed column number.
    /// This is increases up to a tab stop for tabs.
    /// Some editors count tabs as 1 character, so this position is not always
    /// the same as editors.
    pub column: usize,
    /// 0-indexed position in the document.
    pub offset: usize,
}

/// Possible event types.
#[derive(Debug, PartialEq, Clone)]
pub enum EventType {
    /// The start of something.
    Enter,
    /// The end of something.
    Exit,
}

/// Something semantic happening somewhere.
#[derive(Debug, Clone)]
pub struct Event {
    pub event_type: EventType,
    pub token_type: TokenType,
    pub point: Point,
    pub index: usize,
    pub previous: Option<usize>,
    pub next: Option<usize>,
    pub content_type: Option<ContentType>,
}

/// The essence of the state machine are functions: `StateFn`.
/// It’s responsible for dealing with that single passed [`Code`][].
/// It yields a [`StateFnResult`][].
pub type StateFn = dyn FnOnce(&mut Tokenizer, Code) -> StateFnResult;

/// Each [`StateFn`][] yields something back: primarily the state.
/// In certain cases, it can also yield back up parsed codes that were passed down.
pub type StateFnResult = (State, Option<Vec<Code>>);

/// To do.
pub type Resolver = dyn FnOnce(&mut Tokenizer) -> Vec<Event>;

/// The result of a state.
pub enum State {
    /// There is a future state: a boxed [`StateFn`][] to pass the next code to.
    Fn(Box<StateFn>),
    /// The state is successful.
    Ok,
    /// The state is not successful.
    Nok,
}

/// To do.
#[derive(Debug)]
pub struct LabelStart {
    /// To do.
    pub start: (usize, usize),
    /// A boolean used internally to figure out if a label start link can’t be
    /// used (because links in links are incorrect).
    pub inactive: bool,
    /// A boolean used internally to figure out if a label is balanced: they’re
    /// not media, it’s just balanced braces.
    pub balanced: bool,
}

/// To do.
#[derive(Debug)]
pub struct Media {
    /// To do.
    pub start: (usize, usize),
    /// To do.
    pub end: (usize, usize),
    /// To do.
    pub id: String,
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
    previous: Code,
    /// Current code.
    current: Code,
    /// `index` in codes of the current code.
    index: usize,
    /// Current relative and absolute position in the file.
    point: Point,
}

// #[derive(Debug)]

/// A tokenizer itself.
pub struct Tokenizer<'a> {
    column_start: HashMap<usize, usize>,
    /// Track whether a character is expected to be consumed, and whether it’s
    /// actually consumed
    ///
    /// Tracked to make sure everything’s valid.
    consumed: bool,
    /// Semantic labels of one or more codes in `codes`.
    pub events: Vec<Event>,
    /// Hierarchy of semantic labels.
    ///
    /// Tracked to make sure everything’s valid.
    stack: Vec<TokenType>,
    /// Previous character code.
    pub previous: Code,
    /// Current character code.
    current: Code,
    /// `index` in codes of the current code.
    index: usize,
    /// Current relative and absolute place in the file.
    point: Point,
    /// To do.
    pub parse_state: &'a ParseState,
    /// To do.
    pub label_start_stack: Vec<LabelStart>,
    /// To do.
    pub label_start_list_loose: Vec<LabelStart>,
    /// To do.
    pub media_list: Vec<Media>,
    /// To do.
    resolvers: Vec<Box<Resolver>>,
    resolver_ids: Vec<String>,
}

impl<'a> Tokenizer<'a> {
    /// Create a new tokenizer.
    pub fn new(point: Point, index: usize, parse_state: &'a ParseState) -> Tokenizer {
        Tokenizer {
            previous: Code::None,
            current: Code::None,
            column_start: HashMap::new(),
            index,
            consumed: true,
            point,
            stack: vec![],
            events: vec![],
            parse_state,
            label_start_stack: vec![],
            label_start_list_loose: vec![],
            media_list: vec![],
            resolvers: vec![],
            resolver_ids: vec![],
        }
    }

    /// To do.
    pub fn register_resolver(&mut self, id: String, resolver: Box<Resolver>) {
        if !self.resolver_ids.contains(&id) {
            self.resolver_ids.push(id);
            self.resolvers.push(resolver);
        }
    }

    /// Prepare for a next code to get consumed.
    fn expect(&mut self, code: Code) {
        assert!(self.consumed, "expected previous character to be consumed");
        self.consumed = false;
        self.current = code;
    }

    /// Define a jump between two places.
    ///
    /// This defines how much columns are increased when consuming a line
    /// ending.
    /// `index` is currently not used (yet).
    // To do: remove `index` as a parameter if not needed.
    pub fn define_skip(&mut self, point: &Point, index: usize) {
        self.column_start.insert(point.line, point.column);
        self.account_for_potential_skip();
        log::debug!("position: define skip: `{:?}` ({:?})", point, index);
    }

    /// Increment the current positional info if we’re right after a line
    /// ending, which has a skip defined.
    fn account_for_potential_skip(&mut self) {
        if self.point.column == 1 {
            match self.column_start.get(&self.point.line) {
                None => {}
                Some(next_column) => {
                    let col = *next_column;
                    self.point.column = col;
                    self.point.offset += col - 1;
                    self.index += col - 1;
                }
            };
        }
    }

    /// Consume the current character.
    /// Each [`StateFn`][] is expected to call this to signal that this code is
    /// used, or call a next `StateFn`.
    pub fn consume(&mut self, code: Code) {
        assert_eq!(
            code, self.current,
            "expected given code to equal expected code"
        );
        log::debug!("consume: `{:?}` ({:?})", code, self.point);
        assert!(!self.consumed, "expected code to not have been consumed: this might be because `x(code)` instead of `x` was returned");

        match code {
            Code::CarriageReturnLineFeed | Code::Char('\n' | '\r') => {
                self.point.line += 1;
                self.point.column = 1;
                self.point.offset += if code == Code::CarriageReturnLineFeed {
                    2
                } else {
                    1
                };
                self.account_for_potential_skip();
                log::debug!("position: after eol: `{:?}`", self.point);
            }
            Code::VirtualSpace => {
                // Empty.
            }
            _ => {
                self.point.column += 1;
                self.point.offset += 1;
            }
        }

        self.index += 1;
        self.previous = code;
        // Mark as consumed.
        self.consumed = true;
    }

    /// Mark the start of a semantic label.
    pub fn enter(&mut self, token_type: TokenType) {
        self.enter_with_content(token_type, None);
    }

    pub fn enter_with_content(&mut self, token_type: TokenType, content_type: Option<ContentType>) {
        log::debug!("enter `{:?}` ({:?})", token_type, self.point);
        self.events.push(Event {
            event_type: EventType::Enter,
            token_type: token_type.clone(),
            point: self.point.clone(),
            index: self.index,
            previous: None,
            next: None,
            content_type,
        });
        self.stack.push(token_type);
    }

    /// Mark the end of a semantic label.
    pub fn exit(&mut self, token_type: TokenType) {
        let current_token = self.stack.pop().expect("cannot close w/o open tokens");

        assert_eq!(
            current_token, token_type,
            "expected exit token to match current token"
        );

        let previous = self.events.last().expect("cannot close w/o open event");
        let point = self.point.clone();

        assert!(
            current_token != previous.token_type || previous.point != point,
            "expected non-empty token"
        );

        log::debug!("exit `{:?}` ({:?})", token_type, self.point);
        self.events.push(Event {
            event_type: EventType::Exit,
            token_type,
            point,
            index: self.index,
            previous: None,
            next: None,
            content_type: None,
        });
    }

    /// Capture the internal state.
    fn capture(&mut self) -> InternalState {
        InternalState {
            index: self.index,
            previous: self.previous,
            current: self.current,
            point: self.point.clone(),
            events_len: self.events.len(),
            stack_len: self.stack.len(),
        }
    }

    /// Apply the internal state.
    fn free(&mut self, previous: InternalState) {
        self.index = previous.index;
        self.previous = previous.previous;
        self.current = previous.current;
        self.point = previous.point;
        assert!(
            self.events.len() >= previous.events_len,
            "expected to restore less events than before"
        );
        self.events.truncate(previous.events_len);
        assert!(
            self.stack.len() >= previous.stack_len,
            "expected to restore less stack items than before"
        );
        self.stack.truncate(previous.stack_len);
    }

    /// Parse with `state_fn` and its future states, switching to `ok` when
    /// successful, and passing [`State::Nok`][] back up if it occurs.
    ///
    /// This function does not capture the current state, in case of
    /// `State::Nok`, as it is assumed that this `go` is itself wrapped in
    /// another `attempt`.
    #[allow(clippy::unused_self)]
    pub fn go(
        &mut self,
        state_fn: impl FnOnce(&mut Tokenizer, Code) -> StateFnResult + 'static,
        after: impl FnOnce(&mut Tokenizer, Code) -> StateFnResult + 'static,
    ) -> Box<StateFn> {
        attempt_impl(
            state_fn,
            vec![],
            |result: (Vec<Code>, Vec<Code>), ok, tokenizer: &mut Tokenizer| {
                if ok {
                    tokenizer.feed(&if ok { result.1 } else { result.0 }, after, false)
                } else {
                    (State::Nok, None)
                }
            },
        )
    }

    /// Parse with `state_fn` and its future states, to check if it result in
    /// [`State::Ok`][] or [`State::Nok`][], revert on both cases, and then
    /// call `done` with whether it was successful or not.
    ///
    /// This captures the current state of the tokenizer, returns a wrapped
    /// state that captures all codes and feeds them to `state_fn` and its
    /// future states until it yields `State::Ok` or `State::Nok`.
    /// It then applies the captured state, calls `done`, and feeds all
    /// captured codes to its future states.
    pub fn check(
        &mut self,
        state_fn: impl FnOnce(&mut Tokenizer, Code) -> StateFnResult + 'static,
        done: impl FnOnce(bool) -> Box<StateFn> + 'static,
    ) -> Box<StateFn> {
        let previous = self.capture();

        attempt_impl(
            state_fn,
            vec![],
            |result: (Vec<Code>, Vec<Code>), ok, tokenizer: &mut Tokenizer| {
                tokenizer.free(previous);
                tokenizer.feed(&result.0, done(ok), false)
            },
        )
    }

    /// Parse with `state_fn` and its future states, to check if it results in
    /// [`State::Ok`][] or [`State::Nok`][], revert on the case of
    /// `State::Nok`, and then call `done` with whether it was successful or
    /// not.
    ///
    /// This captures the current state of the tokenizer, returns a wrapped
    /// state that captures all codes and feeds them to `state_fn` and its
    /// future states until it yields `State::Ok`, at which point it calls
    /// `done` and yields its result.
    /// If instead `State::Nok` was yielded, the captured state is applied,
    /// `done` is called, and all captured codes are fed to its future states.
    pub fn attempt(
        &mut self,
        state_fn: impl FnOnce(&mut Tokenizer, Code) -> StateFnResult + 'static,
        done: impl FnOnce(bool) -> Box<StateFn> + 'static,
    ) -> Box<StateFn> {
        let previous = self.capture();

        attempt_impl(
            state_fn,
            vec![],
            |result: (Vec<Code>, Vec<Code>), ok, tokenizer: &mut Tokenizer| {
                if !ok {
                    tokenizer.free(previous);
                }

                let codes = if ok { result.1 } else { result.0 };

                log::debug!(
                    "attempt: {:?}, codes: {:?}, at {:?}",
                    ok,
                    codes,
                    tokenizer.point
                );
                tokenizer.feed(&codes, done(ok), false)
            },
        )
    }

    /// Just like [`attempt`][Tokenizer::attempt], but many.
    pub fn attempt_n(
        &mut self,
        mut state_fns: Vec<Box<StateFn>>,
        done: impl FnOnce(bool) -> Box<StateFn> + 'static,
    ) -> Box<StateFn> {
        if state_fns.is_empty() {
            done(false)
        } else {
            let state_fn = state_fns.remove(0);
            self.attempt(state_fn, move |ok| {
                if ok {
                    done(ok)
                } else {
                    Box::new(|t, code| t.attempt_n(state_fns, done)(t, code))
                }
            })
        }
    }

    /// Just like [`attempt`][Tokenizer::attempt], but for when you don’t care
    /// about `ok`.
    pub fn attempt_opt(
        &mut self,
        state_fn: impl FnOnce(&mut Tokenizer, Code) -> StateFnResult + 'static,
        after: impl FnOnce(&mut Tokenizer, Code) -> StateFnResult + 'static,
    ) -> Box<StateFn> {
        self.attempt(state_fn, |_ok| Box::new(after))
    }

    /// Feed a list of `codes` into `start`.
    ///
    /// This is set up to support repeatedly calling `feed`, and thus streaming
    /// markdown into the state machine, and normally pauses after feeding.
    /// When `done: true` is passed, the EOF is fed.
    // To do: call this `feed_impl`, and rename `push` to `feed`?
    fn feed(
        &mut self,
        codes: &[Code],
        start: impl FnOnce(&mut Tokenizer, Code) -> StateFnResult + 'static,
        drain: bool,
    ) -> StateFnResult {
        let codes = codes;
        let mut state = State::Fn(Box::new(start));
        let mut index = 0;

        self.consumed = true;

        while index < codes.len() {
            let code = codes[index];

            match state {
                State::Nok | State::Ok => {
                    break;
                }
                State::Fn(func) => {
                    log::debug!("main: passing `{:?}`", code);
                    self.expect(code);
                    let (next, remainder) = check_statefn_result(func(self, code));
                    state = next;
                    index = index + 1
                        - (if let Some(ref x) = remainder {
                            x.len()
                        } else {
                            0
                        });
                }
            }
        }

        // Yield to a higher loop if we shouldn’t feed EOFs.
        if !drain {
            return check_statefn_result((state, Some(codes[index..].to_vec())));
        }

        loop {
            // Feed EOF.
            match state {
                State::Ok | State::Nok => break,
                State::Fn(func) => {
                    let code = Code::None;
                    log::debug!("main: passing eof");
                    self.expect(code);
                    let (next, remainder) = check_statefn_result(func(self, code));
                    assert!(remainder.is_none(), "expected no remainder");
                    state = next;
                }
            }
        }

        match state {
            State::Ok => {}
            _ => unreachable!("expected final state to be `State::Ok`"),
        }

        check_statefn_result((state, None))
    }

    /// To do.
    // To do: set a `drained` to prevent passing after draining?
    pub fn push(
        &mut self,
        codes: &[Code],
        start: impl FnOnce(&mut Tokenizer, Code) -> StateFnResult + 'static,
        drain: bool,
    ) -> StateFnResult {
        let result = self.feed(codes, start, drain);

        if drain {
            while !self.resolvers.is_empty() {
                let resolver = self.resolvers.remove(0);
                self.events = resolver(self);
            }
        }

        result
    }
}

/// Internal utility to wrap states to also capture codes.
///
/// Recurses into itself.
/// Used in [`Tokenizer::attempt`][Tokenizer::attempt] and  [`Tokenizer::check`][Tokenizer::check].
fn attempt_impl(
    state: impl FnOnce(&mut Tokenizer, Code) -> StateFnResult + 'static,
    mut codes: Vec<Code>,
    done: impl FnOnce((Vec<Code>, Vec<Code>), bool, &mut Tokenizer) -> StateFnResult + 'static,
) -> Box<StateFn> {
    Box::new(|tokenizer, code| {
        let (next, remainder) = check_statefn_result(state(tokenizer, code));

        match code {
            Code::None => {}
            _ => {
                codes.push(code);
            }
        }

        if let Some(ref list) = remainder {
            assert!(
                list.len() <= codes.len(),
                "`remainder` must be less than or equal to `codes`"
            );
        }

        match next {
            State::Ok => {
                let remaining = if let Some(x) = remainder { x } else { vec![] };
                check_statefn_result(done((codes, remaining), true, tokenizer))
            }
            State::Nok => check_statefn_result(done((codes, vec![]), false, tokenizer)),
            State::Fn(func) => {
                assert!(remainder.is_none(), "expected no remainder");
                check_statefn_result((State::Fn(attempt_impl(func, codes, done)), None))
            }
        }
    })
}

/// Turn a string into codes.
pub fn as_codes(value: &str) -> Vec<Code> {
    let mut codes: Vec<Code> = vec![];
    let mut at_start = true;
    let mut at_carriage_return = false;
    let mut column = 1;

    for char in value.chars() {
        if at_start {
            if char == '\u{feff}' {
                // Ignore.
                continue;
            }

            at_start = false;
        }

        // Send a CRLF.
        if at_carriage_return && '\n' == char {
            at_carriage_return = false;
            codes.push(Code::CarriageReturnLineFeed);
        } else {
            // Send the previous CR: we’re not at a next `\n`.
            if at_carriage_return {
                at_carriage_return = false;
                codes.push(Code::Char('\r'));
            }

            match char {
                // Send a replacement character.
                '\0' => {
                    column += 1;
                    codes.push(Code::Char('�'));
                }
                // Send a tab and virtual spaces.
                '\t' => {
                    let remainder = column % TAB_SIZE;
                    let mut virtual_spaces = if remainder == 0 {
                        0
                    } else {
                        TAB_SIZE - remainder
                    };
                    codes.push(Code::Char(char));
                    column += 1;
                    while virtual_spaces > 0 {
                        codes.push(Code::VirtualSpace);
                        column += 1;
                        virtual_spaces -= 1;
                    }
                }
                // Send an LF.
                '\n' => {
                    column = 1;
                    codes.push(Code::Char(char));
                }
                // Don’t send anything yet.
                '\r' => {
                    column = 1;
                    at_carriage_return = true;
                }
                // Send the char.
                _ => {
                    column += 1;
                    codes.push(Code::Char(char));
                }
            }
        };
    }

    // Send the last CR: we’re not at a next `\n`.
    if at_carriage_return {
        codes.push(Code::Char('\r'));
    }

    codes
}

/// Check a [`StateFnResult`][], make sure its valid (that there are no bugs),
/// and clean a final eof passed back in `remainder`.
fn check_statefn_result(result: StateFnResult) -> StateFnResult {
    let (state, mut remainder) = result;

    // Remove an eof.
    // For convencience, feeding back an eof is allowed, but cleaned here.
    // Most states handle eof and eol in the same branch, and hence pass
    // all back.
    // This might not be needed, because if EOF is passed back, we’re at the EOF.
    // But they’re not supposed to be in codes, so here we remove them.
    if let Some(ref mut list) = remainder {
        if Some(&Code::None) == list.last() {
            list.pop();
        }

        if list.is_empty() {
            return (state, None);
        }
    }

    (state, remainder)
}
