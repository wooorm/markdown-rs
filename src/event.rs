/// Semantic label of a span.
#[derive(Debug, Clone, PartialEq, Hash, Eq)]
pub enum Name {
    /// Attention sequence.
    ///
    /// > 👉 **Note**: this is used while parsing but compiled away.
    AttentionSequence,
    /// Whole autolink.
    ///
    /// ## Info
    ///
    /// *   **Context**:
    ///     [text content][crate::content::text]
    /// *   **Content model**:
    ///     [`AutolinkEmail`][Name::AutolinkEmail],
    ///     [`AutolinkMarker`][Name::AutolinkMarker],
    ///     [`AutolinkProtocol`][Name::AutolinkProtocol]
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
    ///     [`Autolink`][Name::Autolink]
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
    ///     [`Autolink`][Name::Autolink]
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
    ///     [`Autolink`][Name::Autolink]
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
    /// Whole block quote.
    ///
    /// ## Info
    ///
    /// *   **Context**:
    ///     [document content][crate::content::document]
    /// *   **Content model**:
    ///     [`BlockQuotePrefix`][Name::BlockQuotePrefix],
    ///     [flow content][crate::content::flow]
    /// *   **Construct**:
    ///     [`block_quote`][crate::construct::block_quote]
    ///
    /// ## Example
    ///
    /// ```markdown
    /// > | > a
    ///     ^^^
    /// > | b
    ///     ^
    /// ```
    BlockQuote,
    /// Block quote marker.
    ///
    /// ## Info
    ///
    /// *   **Context**:
    ///     [`BlockQuotePrefix`][Name::BlockQuotePrefix]
    /// *   **Content model**:
    ///     void
    /// *   **Construct**:
    ///     [`block_quote`][crate::construct::block_quote]
    ///
    /// ## Example
    ///
    /// ```markdown
    /// > | > a
    ///     ^
    ///   | b
    /// ```
    BlockQuoteMarker,
    /// Block quote prefix.
    ///
    /// ## Info
    ///
    /// *   **Context**:
    ///     [`BlockQuote`][Name::BlockQuote]
    /// *   **Content model**:
    ///     [`BlockQuoteMarker`][Name::BlockQuoteMarker],
    ///     [`SpaceOrTab`][Name::SpaceOrTab]
    /// *   **Construct**:
    ///     [`block_quote`][crate::construct::block_quote]
    ///
    /// ## Example
    ///
    /// ```markdown
    /// > | > a
    ///     ^^
    ///   | b
    /// ```
    BlockQuotePrefix,
    /// Byte order mark.
    ///
    /// ## Info
    ///
    /// *   **Context**:
    ///     optional first event
    /// *   **Content model**:
    ///     void
    /// *   **Construct**:
    ///     [`document`][crate::content::document]
    ByteOrderMark,
    /// Whole character escape.
    ///
    /// ## Info
    ///
    /// *   **Context**:
    ///     [string content][crate::content::string] or
    ///     [text content][crate::content::text]
    /// *   **Content model**:
    ///     [`CharacterEscapeMarker`][Name::CharacterEscapeMarker],
    ///     [`CharacterEscapeValue`][Name::CharacterEscapeValue]
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
    ///     [`CharacterEscape`][Name::CharacterEscape]
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
    ///     [`CharacterEscape`][Name::CharacterEscape]
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
    ///     [`CharacterReferenceMarker`][Name::CharacterReferenceMarker],
    ///     [`CharacterReferenceMarkerHexadecimal`][Name::CharacterReferenceMarkerHexadecimal],
    ///     [`CharacterReferenceMarkerNumeric`][Name::CharacterReferenceMarkerNumeric],
    ///     [`CharacterReferenceMarkerSemi`][Name::CharacterReferenceMarkerSemi],
    ///     [`CharacterReferenceValue`][Name::CharacterReferenceValue]
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
    ///     [`CharacterReference`][Name::CharacterReference]
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
    /// Character reference hexadecimal numeric marker.
    ///
    /// ## Info
    ///
    /// *   **Context**:
    ///     [`CharacterReference`][Name::CharacterReference]
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
    /// Character reference numeric marker.
    ///
    /// ## Info
    ///
    /// *   **Context**:
    ///     [`CharacterReference`][Name::CharacterReference]
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
    /// Character reference closing marker.
    ///
    /// ## Info
    ///
    /// *   **Context**:
    ///     [`CharacterReference`][Name::CharacterReference]
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
    ///     [`CharacterReference`][Name::CharacterReference]
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
    ///     [`CodeFencedFence`][Name::CodeFencedFence],
    ///     [`CodeFlowChunk`][Name::CodeFlowChunk],
    ///     [`LineEnding`][Name::LineEnding],
    ///     [`SpaceOrTab`][Name::SpaceOrTab]
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
    ///     [`CodeFenced`][Name::CodeFenced]
    /// *   **Content model**:
    ///     [`CodeFencedFenceInfo`][Name::CodeFencedFenceInfo],
    ///     [`CodeFencedFenceMeta`][Name::CodeFencedFenceMeta],
    ///     [`CodeFencedFenceSequence`][Name::CodeFencedFenceSequence],
    ///     [`SpaceOrTab`][Name::SpaceOrTab]
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
    /// A code (fenced) fence info word.
    ///
    /// ## Info
    ///
    /// *   **Context**:
    ///     [`CodeFencedFence`][Name::CodeFencedFence]
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
    ///     [`CodeFencedFence`][Name::CodeFencedFence]
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
    /// A code (fenced) fence sequence.
    ///
    /// ## Info
    ///
    /// *   **Context**:
    ///     [`CodeFencedFenceSequence`][Name::CodeFencedFenceSequence]
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
    /// A code (fenced, indented) chunk.
    ///
    /// ## Info
    ///
    /// *   **Context**:
    ///     [`CodeFenced`][Name::CodeFenced],
    ///     [`CodeIndented`][Name::CodeIndented]
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
    ///     [`CodeFlowChunk`][Name::CodeFlowChunk],
    ///     [`LineEnding`][Name::LineEnding],
    ///     [`SpaceOrTab`][Name::SpaceOrTab]
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
    ///     [`CodeTextData`][Name::CodeTextData],
    ///     [`CodeTextSequence`][Name::CodeTextSequence],
    ///     [`LineEnding`][Name::LineEnding]
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
    ///     [`CodeText`][Name::CodeText],
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
    ///     [`CodeText`][Name::CodeText],
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
    ///     [`data`][crate::construct::partial_data]
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
    ///     [`DefinitionMarker`][Name::DefinitionMarker],
    ///     [`DefinitionLabel`][Name::DefinitionLabel],
    ///     [`DefinitionDestination`][Name::DefinitionDestination],
    ///     [`DefinitionTitle`][Name::DefinitionTitle],
    ///     [`LineEnding`][Name::LineEnding],
    ///     [`SpaceOrTab`][Name::SpaceOrTab]
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
    /// Whole definition destination.
    ///
    /// ## Info
    ///
    /// *   **Context**:
    ///     [`Definition`][Name::Definition]
    /// *   **Content model**:
    ///     [`DefinitionDestinationLiteral`][Name::DefinitionDestinationLiteral],
    ///     [`DefinitionDestinationRaw`][Name::DefinitionDestinationRaw]
    /// *   **Construct**:
    ///     [`destination`][crate::construct::partial_destination]
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
    ///     [`DefinitionDestination`][Name::DefinitionDestination]
    /// *   **Content model**:
    ///     [`DefinitionDestinationLiteralMarker`][Name::DefinitionDestinationLiteralMarker],
    ///     [`DefinitionDestinationString`][Name::DefinitionDestinationString]
    /// *   **Construct**:
    ///     [`destination`][crate::construct::partial_destination]
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
    ///     [`DefinitionDestinationLiteral`][Name::DefinitionDestinationLiteral]
    /// *   **Content model**:
    ///     void
    /// *   **Construct**:
    ///     [`destination`][crate::construct::partial_destination]
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
    ///     [`DefinitionDestination`][Name::DefinitionDestination]
    /// *   **Content model**:
    ///     [`DefinitionDestinationString`][Name::DefinitionDestinationString]
    /// *   **Construct**:
    ///     [`destination`][crate::construct::partial_destination]
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
    ///     [`DefinitionDestinationLiteral`][Name::DefinitionDestinationLiteral],
    ///     [`DefinitionDestinationRaw`][Name::DefinitionDestinationRaw]
    /// *   **Content model**:
    ///     [string content][crate::content::string]
    /// *   **Construct**:
    ///     [`destination`][crate::construct::partial_destination]
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
    /// Whole definition label.
    ///
    /// ## Info
    ///
    /// *   **Context**:
    ///     [`Definition`][Name::Definition]
    /// *   **Content model**:
    ///     [`DefinitionLabelMarker`][Name::DefinitionLabelMarker],
    ///     [`DefinitionLabelString`][Name::DefinitionLabelString],
    ///     [`LineEnding`][Name::LineEnding],
    ///     [`SpaceOrTab`][Name::SpaceOrTab]
    /// *   **Construct**:
    ///     [`label`][crate::construct::partial_label]
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
    ///     [`DefinitionLabel`][Name::DefinitionLabel]
    /// *   **Content model**:
    ///     void
    /// *   **Construct**:
    ///     [`label`][crate::construct::partial_label]
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
    ///     [`DefinitionLabel`][Name::DefinitionLabel]
    /// *   **Content model**:
    ///     [string content][crate::content::string]
    /// *   **Construct**:
    ///     [`label`][crate::construct::partial_label]
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
    ///     [`Definition`][Name::Definition]
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
    /// Whole definition title.
    ///
    /// ## Info
    ///
    /// *   **Context**:
    ///     [`Definition`][Name::Definition]
    /// *   **Content model**:
    ///     [`DefinitionTitleMarker`][Name::DefinitionTitleMarker],
    ///     [`DefinitionTitleString`][Name::DefinitionTitleString],
    ///     [`LineEnding`][Name::LineEnding],
    ///     [`SpaceOrTab`][Name::SpaceOrTab]
    /// *   **Construct**:
    ///     [`title`][crate::construct::partial_title]
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
    ///     [`DefinitionTitle`][Name::DefinitionTitle]
    /// *   **Content model**:
    ///     void
    /// *   **Construct**:
    ///     [`title`][crate::construct::partial_title]
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
    ///     [`DefinitionTitle`][Name::DefinitionTitle]
    /// *   **Content model**:
    ///     [string content][crate::content::string]
    /// *   **Construct**:
    ///     [`title`][crate::construct::partial_title]
    ///
    /// ## Example
    ///
    /// ```markdown
    /// > | [a]: b "c"
    ///             ^
    /// ```
    DefinitionTitleString,
    /// Emphasis.
    ///
    /// ## Info
    ///
    /// *   **Context**:
    ///     [text content][crate::content::text]
    /// *   **Content model**:
    ///     [`EmphasisSequence`][Name::EmphasisSequence],
    ///     [`EmphasisText`][Name::EmphasisText]
    /// *   **Construct**:
    ///     [`attention`][crate::construct::attention]
    ///
    /// ## Example
    ///
    /// ```markdown
    /// > | *a*
    ///     ^^^
    /// ```
    Emphasis,
    /// Emphasis sequence.
    ///
    /// ## Info
    ///
    /// *   **Context**:
    ///     [`Emphasis`][Name::Emphasis]
    /// *   **Content model**:
    ///     void
    /// *   **Construct**:
    ///     [`attention`][crate::construct::attention]
    ///
    /// ## Example
    ///
    /// ```markdown
    /// > | *a*
    ///     ^ ^
    /// ```
    EmphasisSequence,
    /// Emphasis text.
    ///
    /// ## Info
    ///
    /// *   **Context**:
    ///     [`Emphasis`][Name::Emphasis]
    /// *   **Content model**:
    ///     [text content][crate::content::text]
    /// *   **Construct**:
    ///     [`attention`][crate::construct::attention]
    ///
    /// ## Example
    ///
    /// ```markdown
    /// > | *a*
    ///      ^
    /// ```
    EmphasisText,
    /// Whole hard break (escape).
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
    HardBreakEscape,
    /// Whole hard break (trailing).
    ///
    /// ## Info
    ///
    /// *   **Context**:
    ///     [text content][crate::content::text]
    /// *   **Content model**:
    ///     void
    /// *   **Construct**:
    ///     [`whitespace`][crate::construct::partial_whitespace]
    ///
    /// ## Example
    ///
    /// ```markdown
    /// > | a␠␠␊
    ///      ^^
    /// > | b
    /// ```
    HardBreakTrailing,
    /// Whole heading (atx).
    ///
    /// ## Info
    ///
    /// *   **Context**:
    ///     [flow content][crate::content::flow]
    /// *   **Content model**:
    ///     [`HeadingAtxSequence`][Name::HeadingAtxSequence],
    ///     [`HeadingAtxText`][Name::HeadingAtxText],
    ///     [`SpaceOrTab`][Name::SpaceOrTab]
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
    ///     [`HeadingAtx`][Name::HeadingAtx]
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
    ///     [`HeadingAtx`][Name::HeadingAtx],
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
    ///     [`HeadingSetextText`][Name::HeadingSetextText],
    ///     [`HeadingSetextUnderline`][Name::HeadingSetextUnderline],
    ///     [`LineEnding`][Name::LineEnding],
    ///     [`SpaceOrTab`][Name::SpaceOrTab]
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
    ///     [`HeadingSetext`][Name::HeadingSetext]
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
    ///     [`HeadingSetext`][Name::HeadingSetext]
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
    ///     [`HtmlFlowData`][Name::HtmlFlowData],
    ///     [`LineEnding`][Name::LineEnding],
    ///     [`SpaceOrTab`][Name::SpaceOrTab]
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
    ///     [`HtmlFlow`][Name::HtmlFlow],
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
    ///     [`HtmlTextData`][Name::HtmlTextData],
    ///     [`LineEnding`][Name::LineEnding],
    ///     [`SpaceOrTab`][Name::SpaceOrTab]
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
    ///     [`HtmlText`][Name::HtmlText]
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
    /// Image.
    ///
    /// ## Info
    ///
    /// *   **Context**:
    ///     [text content][crate::content::text]
    /// *   **Content model**:
    ///     [`Label`][Name::Label],
    ///     [`Resource`][Name::Resource],
    ///     [`Reference`][Name::Reference]
    /// *   **Construct**:
    ///     [`label_end`][crate::construct::label_end]
    ///
    /// ## Example
    ///
    /// ```markdown
    /// > | a ![b] c
    ///       ^^^^
    /// > | a ![b][c] d
    ///       ^^^^^^^
    /// > | a ![b](c) d
    ///       ^^^^^^^
    /// ```
    Image,
    /// Label.
    ///
    /// ## Info
    ///
    /// *   **Context**:
    ///     [`Image`][Name::Image],
    ///     [`Link`][Name::Link]
    /// *   **Content model**:
    ///     [`LabelImage`][Name::LabelImage],
    ///     [`LabelLink`][Name::LabelLink],
    ///     [`LabelEnd`][Name::LabelEnd],
    ///     [`LabelText`][Name::LabelText]
    /// *   **Construct**:
    ///     [`label_end`][crate::construct::label_end]
    ///
    /// ## Example
    ///
    /// ```markdown
    /// > | a [b] c
    ///       ^^^
    /// > | a ![b][c] d
    ///       ^^^^
    /// > | a [b](c) d
    ///       ^^^
    /// ```
    Label,
    /// Label end.
    ///
    /// ## Info
    ///
    /// *   **Context**:
    ///     [`Label`][Name::Label]
    /// *   **Content model**:
    ///     [`LabelMarker`][Name::LabelMarker]
    /// *   **Construct**:
    ///     [`label_end`][crate::construct::label_end]
    ///
    /// ## Example
    ///
    /// ```markdown
    /// > | a ![b](c) d
    ///          ^
    /// > | a [b](c) d
    ///         ^
    /// ```
    LabelEnd,
    /// Label start (image).
    ///
    /// ## Info
    ///
    /// *   **Context**:
    ///     [`Label`][Name::Label]
    /// *   **Content model**:
    ///     [`LabelImageMarker`][Name::LabelImageMarker],
    ///     [`LabelMarker`][Name::LabelMarker]
    /// *   **Construct**:
    ///     [`label_start_image`][crate::construct::label_start_image]
    ///
    /// ## Example
    ///
    /// ```markdown
    /// > | a ![b](c) d
    ///       ^^
    /// ```
    LabelImage,
    /// Label start (image) marker.
    ///
    /// ## Info
    ///
    /// *   **Context**:
    ///     [`LabelImage`][Name::LabelImage]
    /// *   **Content model**:
    ///     void
    /// *   **Construct**:
    ///     [`label_start_image`][crate::construct::label_start_image]
    ///
    /// ## Example
    ///
    /// ```markdown
    /// > | a ![b](c) d
    ///       ^
    /// ```
    LabelImageMarker,
    /// Label start (link).
    ///
    /// ## Info
    ///
    /// *   **Context**:
    ///     [`Label`][Name::Label]
    /// *   **Content model**:
    ///     [`LabelMarker`][Name::LabelMarker]
    /// *   **Construct**:
    ///     [`label_start_link`][crate::construct::label_start_link]
    ///
    /// ## Example
    ///
    /// ```markdown
    /// > | a [b](c) d
    ///       ^
    /// ```
    LabelLink,
    /// Label marker.
    ///
    /// ## Info
    ///
    /// *   **Context**:
    ///     [`LabelImage`][Name::LabelImage],
    ///     [`LabelLink`][Name::LabelLink],
    ///     [`LabelEnd`][Name::LabelEnd]
    /// *   **Content model**:
    ///     void
    /// *   **Construct**:
    ///     [`label_start_image`][crate::construct::label_start_image],
    ///     [`label_start_link`][crate::construct::label_start_link],
    ///     [`label_end`][crate::construct::label_end]
    ///
    /// ## Example
    ///
    /// ```markdown
    /// > | a ![b](c) d
    ///        ^ ^
    /// > | a [b](c) d
    ///       ^ ^
    /// ```
    LabelMarker,
    /// Label text.
    ///
    /// ## Info
    ///
    /// *   **Context**:
    ///     [`Label`][Name::Label]
    /// *   **Content model**:
    ///     [text content][crate::content::text]
    /// *   **Construct**:
    ///     [`label_end`][crate::construct::label_end]
    ///
    /// ## Example
    ///
    /// ```markdown
    /// > | a [b] c
    ///        ^
    /// > | a ![b][c] d
    ///         ^
    /// > | a [b](c) d
    ///        ^
    /// ```
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
    /// Link.
    ///
    /// ## Info
    ///
    /// *   **Context**:
    ///     [text content][crate::content::text]
    /// *   **Content model**:
    ///     [`Label`][Name::Label],
    ///     [`Resource`][Name::Resource],
    ///     [`Reference`][Name::Reference]
    /// *   **Construct**:
    ///     [`label_end`][crate::construct::label_end]
    ///
    /// ## Example
    ///
    /// ```markdown
    /// > | a [b] c
    ///       ^^^
    /// > | a [b][c] d
    ///       ^^^^^^
    /// > | a [b](c) d
    ///       ^^^^^^
    /// ```
    Link,
    /// List item.
    ///
    /// ## Info
    ///
    /// *   **Context**:
    ///     [`ListOrdered`][Name::ListOrdered],
    ///     [`ListUnordered`][Name::ListUnordered],
    /// *   **Content model**:
    ///     [`ListItemPrefix`][Name::ListItemPrefix],
    ///     [flow content][crate::content::flow]
    /// *   **Construct**:
    ///     [`list`][crate::construct::list]
    ///
    /// ## Example
    ///
    /// ```markdown
    /// > | * a
    ///     ^^^
    /// > | 1. b
    ///     ^^^^
    /// ```
    ListItem,
    /// List item (marker).
    ///
    /// ## Info
    ///
    /// *   **Context**:
    ///     [`ListItemPrefix`][Name::ListItemPrefix]
    /// *   **Content model**:
    ///     void
    /// *   **Construct**:
    ///     [`list`][crate::construct::list]
    ///
    /// ## Example
    ///
    /// ```markdown
    /// > | * a
    ///     ^
    /// > | 1. b
    ///      ^
    /// ```
    ListItemMarker,
    /// List item (prefix).
    ///
    /// ## Info
    ///
    /// *   **Context**:
    ///     [`ListItem`][Name::ListItem]
    /// *   **Content model**:
    ///     [`ListItemMarker`][Name::ListItemMarker],
    ///     [`ListItemValue`][Name::ListItemValue],
    ///     [`SpaceOrTab`][Name::SpaceOrTab]
    /// *   **Construct**:
    ///     [`list`][crate::construct::list]
    ///
    /// ## Example
    ///
    /// ```markdown
    /// > | * a
    ///     ^^
    /// > |   b
    ///     ^^
    /// ```
    ListItemPrefix,
    /// List item (value).
    ///
    /// ## Info
    ///
    /// *   **Context**:
    ///     [`ListItemPrefix`][Name::ListItemPrefix]
    /// *   **Content model**:
    ///     void
    /// *   **Construct**:
    ///     [`list`][crate::construct::list]
    ///
    /// ## Example
    ///
    /// ```markdown
    /// > | 1. b
    ///     ^
    /// ```
    ListItemValue,
    /// List (ordered).
    ///
    /// ## Info
    ///
    /// *   **Context**:
    ///     [document content][crate::content::document]
    /// *   **Content model**:
    ///     [`BlankLineEnding`][Name::BlankLineEnding],
    ///     [`BlockQuotePrefix`][Name::BlockQuotePrefix],
    ///     [`ListItem`][Name::ListItem],
    ///     [`LineEnding`][Name::LineEnding],
    ///     [`SpaceOrTab`][Name::SpaceOrTab]
    /// *   **Construct**:
    ///     [`list`][crate::construct::list]
    ///
    /// ## Example
    ///
    /// ```markdown
    /// > | 1. a
    ///     ^^^^
    /// > | 2. b
    ///     ^^^^
    /// ```
    ListOrdered,
    /// List (unordered).
    ///
    /// ## Info
    ///
    /// *   **Context**:
    ///     [document content][crate::content::document]
    /// *   **Content model**:
    ///     [`BlankLineEnding`][Name::BlankLineEnding],
    ///     [`BlockQuotePrefix`][Name::BlockQuotePrefix],
    ///     [`ListItem`][Name::ListItem],
    ///     [`LineEnding`][Name::LineEnding],
    ///     [`SpaceOrTab`][Name::SpaceOrTab]
    /// *   **Construct**:
    ///     [`list`][crate::construct::list]
    ///
    /// ## Example
    ///
    /// ```markdown
    /// > | * a
    ///     ^^^
    /// > | * b
    ///     ^^^
    /// ```
    ListUnordered,
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
    /// Reference.
    ///
    /// ## Info
    ///
    /// *   **Context**:
    ///     [`Image`][Name::Image],
    ///     [`Link`][Name::Link]
    /// *   **Content model**:
    ///     [`ReferenceMarker`][Name::ReferenceMarker],
    ///     [`ReferenceString`][Name::ReferenceString]
    /// *   **Construct**:
    ///     [`label`][crate::construct::partial_label]
    ///
    /// ## Example
    ///
    /// ```markdown
    /// > | a ![b][c] d
    ///           ^^^
    /// ```
    Reference,
    /// Reference marker.
    ///
    /// ## Info
    ///
    /// *   **Context**:
    ///     [`Reference`][Name::Reference]
    /// *   **Content model**:
    ///     void
    /// *   **Construct**:
    ///     [`label`][crate::construct::partial_label]
    ///
    /// ## Example
    ///
    /// ```markdown
    /// > | a ![b][c] d
    ///           ^ ^
    /// ```
    ReferenceMarker,
    /// Reference string.
    ///
    /// ## Info
    ///
    /// *   **Context**:
    ///     [`Reference`][Name::Reference]
    /// *   **Content model**:
    ///     [string content][crate::content::string]
    /// *   **Construct**:
    ///     [`label`][crate::construct::partial_label]
    ///
    /// ## Example
    ///
    /// ```markdown
    /// > | a ![b][c] d
    ///            ^
    /// ```
    ReferenceString,
    /// Resource.
    ///
    /// ## Info
    ///
    /// *   **Context**:
    ///     [`Image`][Name::Image],
    ///     [`Link`][Name::Link]
    /// *   **Content model**:
    ///     [`ResourceMarker`][Name::ResourceMarker],
    ///     [`ResourceDestination`][Name::ResourceDestination],
    ///     [`ResourceTitle`][Name::ResourceTitle],
    ///     [`SpaceOrTab`][Name::SpaceOrTab],
    ///     [`LineEnding`][Name::LineEnding]
    /// *   **Construct**:
    ///     [`label_end`][crate::construct::label_end]
    ///
    /// ## Example
    ///
    /// ```markdown
    /// > | a ![b](c "d") e
    ///           ^^^^^^^
    /// > | a [b](c) d
    ///          ^^^
    /// ```
    Resource,
    /// Resource destination.
    ///
    /// ## Info
    ///
    /// *   **Context**:
    ///     [`Resource`][Name::Resource]
    /// *   **Content model**:
    ///     [`ResourceDestinationLiteral`][Name::ResourceDestinationLiteral],
    ///     [`ResourceDestinationRaw`][Name::ResourceDestinationRaw]
    /// *   **Construct**:
    ///     [`destination`][crate::construct::partial_destination]
    ///
    /// ## Example
    ///
    /// ```markdown
    /// > | a ![b](c "d") e
    ///            ^
    /// ```
    ResourceDestination,
    /// Resource destination literal.
    ///
    /// ## Info
    ///
    /// *   **Context**:
    ///     [`ResourceDestination`][Name::ResourceDestination]
    /// *   **Content model**:
    ///     [`ResourceDestinationLiteralMarker`][Name::ResourceDestinationLiteralMarker],
    ///     [`ResourceDestinationString`][Name::ResourceDestinationString]
    /// *   **Construct**:
    ///     [`destination`][crate::construct::partial_destination]
    ///
    /// ## Example
    ///
    /// ```markdown
    /// > | a ![b](<c> "d") e
    ///            ^^^
    /// ```
    ResourceDestinationLiteral,
    /// Resource destination literal marker.
    ///
    /// ## Info
    ///
    /// *   **Context**:
    ///     [`ResourceDestinationLiteral`][Name::ResourceDestinationLiteral]
    /// *   **Content model**:
    ///     void
    /// *   **Construct**:
    ///     [`destination`][crate::construct::partial_destination]
    ///
    /// ## Example
    ///
    /// ```markdown
    /// > | a ![b](<c> "d") e
    ///            ^ ^
    /// ```
    ResourceDestinationLiteralMarker,
    /// Resource destination raw.
    ///
    /// ## Info
    ///
    /// *   **Context**:
    ///     [`ResourceDestination`][Name::ResourceDestination]
    /// *   **Content model**:
    ///     [`ResourceDestinationString`][Name::ResourceDestinationString]
    /// *   **Construct**:
    ///     [`destination`][crate::construct::partial_destination]
    ///
    /// ## Example
    ///
    /// ```markdown
    /// > | a ![b](c "d") e
    ///            ^
    /// ```
    ResourceDestinationRaw,
    /// Resource destination raw.
    ///
    /// ## Info
    ///
    /// *   **Context**:
    ///     [`ResourceDestinationLiteral`][Name::ResourceDestinationLiteral],
    ///     [`ResourceDestinationRaw`][Name::ResourceDestinationRaw]
    /// *   **Content model**:
    ///     [string content][crate::content::string]
    /// *   **Construct**:
    ///     [`destination`][crate::construct::partial_destination]
    ///
    /// ## Example
    ///
    /// ```markdown
    /// > | a ![b](<c> "d") e
    ///             ^
    /// > | a ![b](c "d") e
    ///            ^
    /// ```
    ResourceDestinationString,
    /// Resource marker.
    ///
    /// ## Info
    ///
    /// *   **Context**:
    ///     [`Resource`][Name::Resource]
    /// *   **Content model**:
    ///     void
    /// *   **Construct**:
    ///     [`label_end`][crate::construct::label_end]
    ///
    /// ## Example
    ///
    /// ```markdown
    /// > | a ![b](c "d") e
    ///           ^     ^
    /// ```
    ResourceMarker,
    /// Resource title.
    ///
    /// ## Info
    ///
    /// *   **Context**:
    ///     [`Resource`][Name::Resource]
    /// *   **Content model**:
    ///     [`ResourceTitleMarker`][Name::ResourceTitleMarker],
    ///     [`ResourceTitleString`][Name::ResourceTitleString]
    /// *   **Construct**:
    ///     [`title`][crate::construct::partial_title]
    ///
    /// ## Example
    ///
    /// ```markdown
    /// > | a ![b](<c> "d") e
    ///                ^^^
    /// ```
    ResourceTitle,
    /// Resource title marker.
    ///
    /// ## Info
    ///
    /// *   **Context**:
    ///     [`ResourceTitle`][Name::ResourceTitle]
    /// *   **Content model**:
    ///     void
    /// *   **Construct**:
    ///     [`title`][crate::construct::partial_title]
    ///
    /// ## Example
    ///
    /// ```markdown
    /// > | a ![b](<c> "d") e
    ///                ^ ^
    /// ```
    ResourceTitleMarker,
    /// Resource title string.
    ///
    /// ## Info
    ///
    /// *   **Context**:
    ///     [`ResourceTitle`][Name::ResourceTitle]
    /// *   **Content model**:
    ///     [string content][crate::content::string]
    /// *   **Construct**:
    ///     [`title`][crate::construct::partial_title]
    ///
    /// ## Example
    ///
    /// ```markdown
    /// > | a ![b](<c> "d") e
    ///                 ^
    /// ```
    ResourceTitleString,
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
    /// Strong.
    ///
    /// ## Info
    ///
    /// *   **Context**:
    ///     [text content][crate::content::text]
    /// *   **Content model**:
    ///     [`StrongSequence`][Name::StrongSequence],
    ///     [`StrongText`][Name::StrongText]
    /// *   **Construct**:
    ///     [`attention`][crate::construct::attention]
    ///
    /// ## Example
    ///
    /// ```markdown
    /// > | **a**
    ///     ^^^^^
    /// ```
    Strong,
    /// Strong sequence.
    ///
    /// ## Info
    ///
    /// *   **Context**:
    ///     [`Strong`][Name::Strong]
    /// *   **Content model**:
    ///     void
    /// *   **Construct**:
    ///     [`attention`][crate::construct::attention]
    ///
    /// ## Example
    ///
    /// ```markdown
    /// > | **a**
    ///     ^^ ^^
    /// ```
    StrongSequence,
    /// Strong text.
    ///
    /// ## Info
    ///
    /// *   **Context**:
    ///     [`Strong`][Name::Strong]
    /// *   **Content model**:
    ///     [text content][crate::content::text]
    /// *   **Construct**:
    ///     [`attention`][crate::construct::attention]
    ///
    /// ## Example
    ///
    /// ```markdown
    /// > | **a**
    ///       ^
    /// ```
    StrongText,
    /// Whole thematic break.
    ///
    /// ## Info
    ///
    /// *   **Context**:
    ///     [flow content][crate::content::flow]
    /// *   **Content model**:
    ///     [`ThematicBreakSequence`][Name::ThematicBreakSequence],
    ///     [`SpaceOrTab`][Name::SpaceOrTab]
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
    ///     [`ThematicBreak`][Name::ThematicBreak]
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
}

/// List of void tokens, used to make sure everything is working well.
pub const VOID_EVENTS: [Name; 40] = [
    Name::AttentionSequence,
    Name::AutolinkEmail,
    Name::AutolinkMarker,
    Name::AutolinkProtocol,
    Name::BlankLineEnding,
    Name::BlockQuoteMarker,
    Name::ByteOrderMark,
    Name::CharacterEscapeMarker,
    Name::CharacterEscapeValue,
    Name::CharacterReferenceMarker,
    Name::CharacterReferenceMarkerHexadecimal,
    Name::CharacterReferenceMarkerNumeric,
    Name::CharacterReferenceMarkerSemi,
    Name::CharacterReferenceValue,
    Name::CodeFencedFenceSequence,
    Name::CodeFlowChunk,
    Name::CodeTextData,
    Name::CodeTextSequence,
    Name::Data,
    Name::DefinitionDestinationLiteralMarker,
    Name::DefinitionLabelMarker,
    Name::DefinitionMarker,
    Name::DefinitionTitleMarker,
    Name::EmphasisSequence,
    Name::HardBreakEscape,
    Name::HardBreakTrailing,
    Name::HeadingAtxSequence,
    Name::HeadingSetextUnderline,
    Name::HtmlFlowData,
    Name::HtmlTextData,
    Name::LabelImageMarker,
    Name::LabelMarker,
    Name::LineEnding,
    Name::ListItemMarker,
    Name::ListItemValue,
    Name::ReferenceMarker,
    Name::ResourceMarker,
    Name::ResourceTitleMarker,
    Name::StrongSequence,
    Name::ThematicBreakSequence,
];

/// Embedded content type.
#[derive(Debug, Clone, PartialEq)]
pub enum Content {
    /// Represents [flow content][crate::content::flow].
    Flow,
    /// Represents [string content][crate::content::string].
    String,
    /// Represents [text content][crate::content::text].
    Text,
}

/// Link to another event.
#[derive(Debug, Clone)]
pub struct Link {
    pub previous: Option<usize>,
    pub next: Option<usize>,
    pub content: Content,
}

/// Place in the document.
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

/// Event kinds.
#[derive(Debug, PartialEq, Clone)]
pub enum Kind {
    /// The start of something.
    Enter,
    /// The end of something.
    Exit,
}

/// Something semantic happening somewhere.
#[derive(Debug, Clone)]
pub struct Event {
    /// Kind of event.
    pub kind: Kind,
    /// Name of event.
    pub name: Name,
    /// Place where this happens.
    pub point: Point,
    /// Link to another event.
    pub link: Option<Link>,
}
