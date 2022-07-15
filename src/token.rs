/// Semantic label of a span.
#[derive(Debug, Clone, PartialEq, Hash, Eq)]
pub enum Token {
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
    ///     [`AutolinkEmail`][Token::AutolinkEmail],
    ///     [`AutolinkMarker`][Token::AutolinkMarker],
    ///     [`AutolinkProtocol`][Token::AutolinkProtocol]
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
    ///     [`Autolink`][Token::Autolink]
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
    ///     [`Autolink`][Token::Autolink]
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
    ///     [`Autolink`][Token::Autolink]
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
    ///     [`BlockQuotePrefix`][Token::BlockQuotePrefix],
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
    ///     [`BlockQuotePrefix`][Token::BlockQuotePrefix]
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
    ///     [`BlockQuote`][Token::BlockQuote]
    /// *   **Content model**:
    ///     [`BlockQuoteMarker`][Token::BlockQuoteMarker],
    ///     [`SpaceOrTab`][Token::SpaceOrTab]
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
    /// Whole character escape.
    ///
    /// ## Info
    ///
    /// *   **Context**:
    ///     [string content][crate::content::string] or
    ///     [text content][crate::content::text]
    /// *   **Content model**:
    ///     [`CharacterEscapeMarker`][Token::CharacterEscapeMarker],
    ///     [`CharacterEscapeValue`][Token::CharacterEscapeValue]
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
    ///     [`CharacterEscape`][Token::CharacterEscape]
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
    ///     [`CharacterEscape`][Token::CharacterEscape]
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
    ///     [`CharacterReferenceMarker`][Token::CharacterReferenceMarker],
    ///     [`CharacterReferenceMarkerHexadecimal`][Token::CharacterReferenceMarkerHexadecimal],
    ///     [`CharacterReferenceMarkerNumeric`][Token::CharacterReferenceMarkerNumeric],
    ///     [`CharacterReferenceMarkerSemi`][Token::CharacterReferenceMarkerSemi],
    ///     [`CharacterReferenceValue`][Token::CharacterReferenceValue]
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
    ///     [`CharacterReference`][Token::CharacterReference]
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
    ///     [`CharacterReference`][Token::CharacterReference]
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
    ///     [`CharacterReference`][Token::CharacterReference]
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
    ///     [`CharacterReference`][Token::CharacterReference]
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
    ///     [`CharacterReference`][Token::CharacterReference]
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
    ///     [`CodeFencedFence`][Token::CodeFencedFence],
    ///     [`CodeFlowChunk`][Token::CodeFlowChunk],
    ///     [`LineEnding`][Token::LineEnding],
    ///     [`SpaceOrTab`][Token::SpaceOrTab]
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
    ///     [`CodeFenced`][Token::CodeFenced]
    /// *   **Content model**:
    ///     [`CodeFencedFenceInfo`][Token::CodeFencedFenceInfo],
    ///     [`CodeFencedFenceMeta`][Token::CodeFencedFenceMeta],
    ///     [`CodeFencedFenceSequence`][Token::CodeFencedFenceSequence],
    ///     [`SpaceOrTab`][Token::SpaceOrTab]
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
    ///     [`CodeFencedFence`][Token::CodeFencedFence]
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
    ///     [`CodeFencedFence`][Token::CodeFencedFence]
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
    ///     [`CodeFencedFenceSequence`][Token::CodeFencedFenceSequence]
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
    ///     [`CodeFenced`][Token::CodeFenced],
    ///     [`CodeIndented`][Token::CodeIndented]
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
    ///     [`CodeFlowChunk`][Token::CodeFlowChunk],
    ///     [`LineEnding`][Token::LineEnding],
    ///     [`SpaceOrTab`][Token::SpaceOrTab]
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
    ///     [`CodeTextData`][Token::CodeTextData],
    ///     [`CodeTextSequence`][Token::CodeTextSequence],
    ///     [`CodeTextLineEnding`][Token::CodeTextLineEnding]
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
    ///     [`CodeText`][Token::CodeText],
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
    /// Line ending in code (text).
    ///
    /// ## Info
    ///
    /// *   **Context**:
    ///     [`CodeText`][Token::CodeText],
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
    /// Code (text) sequence.
    ///
    /// ## Info
    ///
    /// *   **Context**:
    ///     [`CodeText`][Token::CodeText],
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
    ///     [`DefinitionMarker`][Token::DefinitionMarker],
    ///     [`DefinitionLabel`][Token::DefinitionLabel],
    ///     [`DefinitionDestination`][Token::DefinitionDestination],
    ///     [`DefinitionTitle`][Token::DefinitionTitle],
    ///     [`LineEnding`][Token::LineEnding],
    ///     [`SpaceOrTab`][Token::SpaceOrTab]
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
    ///     [`Definition`][Token::Definition]
    /// *   **Content model**:
    ///     [`DefinitionDestinationLiteral`][Token::DefinitionDestinationLiteral],
    ///     [`DefinitionDestinationRaw`][Token::DefinitionDestinationRaw]
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
    ///     [`DefinitionDestination`][Token::DefinitionDestination]
    /// *   **Content model**:
    ///     [`DefinitionDestinationLiteralMarker`][Token::DefinitionDestinationLiteralMarker],
    ///     [`DefinitionDestinationString`][Token::DefinitionDestinationString]
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
    ///     [`DefinitionDestinationLiteral`][Token::DefinitionDestinationLiteral]
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
    ///     [`DefinitionDestination`][Token::DefinitionDestination]
    /// *   **Content model**:
    ///     [`DefinitionDestinationString`][Token::DefinitionDestinationString]
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
    ///     [`DefinitionDestinationLiteral`][Token::DefinitionDestinationLiteral],
    ///     [`DefinitionDestinationRaw`][Token::DefinitionDestinationRaw]
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
    ///     [`Definition`][Token::Definition]
    /// *   **Content model**:
    ///     [`DefinitionLabelMarker`][Token::DefinitionLabelMarker],
    ///     [`DefinitionLabelString`][Token::DefinitionLabelString],
    ///     [`LineEnding`][Token::LineEnding],
    ///     [`SpaceOrTab`][Token::SpaceOrTab]
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
    ///     [`DefinitionLabel`][Token::DefinitionLabel]
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
    ///     [`DefinitionLabel`][Token::DefinitionLabel]
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
    ///     [`Definition`][Token::Definition]
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
    ///     [`Definition`][Token::Definition]
    /// *   **Content model**:
    ///     [`DefinitionTitleMarker`][Token::DefinitionTitleMarker],
    ///     [`DefinitionTitleString`][Token::DefinitionTitleString],
    ///     [`LineEnding`][Token::LineEnding],
    ///     [`SpaceOrTab`][Token::SpaceOrTab]
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
    ///     [`DefinitionTitle`][Token::DefinitionTitle]
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
    ///     [`DefinitionTitle`][Token::DefinitionTitle]
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
    ///     [`EmphasisSequence`][Token::EmphasisSequence],
    ///     [`EmphasisText`][Token::EmphasisText]
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
    ///     [`Emphasis`][Token::Emphasis]
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
    ///     [`Emphasis`][Token::Emphasis]
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
    ///     [`HardBreakEscapeMarker`][Token::HardBreakEscapeMarker]
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
    ///     [`HardBreakTrailingSpace`][Token::HardBreakTrailingSpace]
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
    ///     [`HardBreakTrailing`][Token::HardBreakTrailing]
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
    ///     [`HeadingAtxSequence`][Token::HeadingAtxSequence],
    ///     [`HeadingAtxText`][Token::HeadingAtxText],
    ///     [`SpaceOrTab`][Token::SpaceOrTab]
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
    ///     [`HeadingAtx`][Token::HeadingAtx]
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
    ///     [`HeadingAtx`][Token::HeadingAtx],
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
    ///     [`HeadingSetextText`][Token::HeadingSetextText],
    ///     [`HeadingSetextUnderline`][Token::HeadingSetextUnderline],
    ///     [`LineEnding`][Token::LineEnding],
    ///     [`SpaceOrTab`][Token::SpaceOrTab]
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
    ///     [`HeadingSetext`][Token::HeadingSetext]
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
    ///     [`HeadingSetext`][Token::HeadingSetext]
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
    ///     [`HtmlFlowData`][Token::HtmlFlowData],
    ///     [`LineEnding`][Token::LineEnding],
    ///     [`SpaceOrTab`][Token::SpaceOrTab]
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
    ///     [`HtmlFlow`][Token::HtmlFlow],
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
    ///     [`HtmlTextData`][Token::HtmlTextData],
    ///     [`LineEnding`][Token::LineEnding],
    ///     [`SpaceOrTab`][Token::SpaceOrTab]
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
    ///     [`HtmlText`][Token::HtmlText]
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
    ///     [`Label`][Token::Label],
    ///     [`Resource`][Token::Resource],
    ///     [`Reference`][Token::Reference]
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
    ///     [`Image`][Token::Image],
    ///     [`Link`][Token::Link]
    /// *   **Content model**:
    ///     [`LabelImage`][Token::LabelImage],
    ///     [`LabelLink`][Token::LabelLink],
    ///     [`LabelEnd`][Token::LabelEnd],
    ///     [`LabelText`][Token::LabelText]
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
    ///     [`Label`][Token::Label]
    /// *   **Content model**:
    ///     [`LabelMarker`][Token::LabelMarker]
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
    ///     [`Label`][Token::Label]
    /// *   **Content model**:
    ///     [`LabelImageMarker`][Token::LabelImageMarker],
    ///     [`LabelMarker`][Token::LabelMarker]
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
    ///     [`LabelImage`][Token::LabelImage]
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
    ///     [`Label`][Token::Label]
    /// *   **Content model**:
    ///     [`LabelMarker`][Token::LabelMarker]
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
    ///     [`LabelImage`][Token::LabelImage],
    ///     [`LabelLink`][Token::LabelLink],
    ///     [`LabelEnd`][Token::LabelEnd]
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
    ///     [`Label`][Token::Label]
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
    ///     [`Label`][Token::Label],
    ///     [`Resource`][Token::Resource],
    ///     [`Reference`][Token::Reference]
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
    ///     [`ListOrdered`][Token::ListOrdered],
    ///     [`ListUnordered`][Token::ListUnordered],
    /// *   **Content model**:
    ///     [`ListItemPrefix`][Token::ListItemPrefix],
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
    ///     [`ListItemPrefix`][Token::ListItemPrefix]
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
    ///     [`ListItem`][Token::ListItem]
    /// *   **Content model**:
    ///     [`ListItemMarker`][Token::ListItemMarker],
    ///     [`ListItemValue`][Token::ListItemValue],
    ///     [`SpaceOrTab`][Token::SpaceOrTab]
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
    ///     [`ListItemPrefix`][Token::ListItemPrefix]
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
    ///     [`BlankLineEnding`][Token::BlankLineEnding],
    ///     [`BlockQuotePrefix`][Token::BlockQuotePrefix],
    ///     [`ListItem`][Token::ListItem],
    ///     [`LineEnding`][Token::LineEnding],
    ///     [`SpaceOrTab`][Token::SpaceOrTab]
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
    ///     [`BlankLineEnding`][Token::BlankLineEnding],
    ///     [`BlockQuotePrefix`][Token::BlockQuotePrefix],
    ///     [`ListItem`][Token::ListItem],
    ///     [`LineEnding`][Token::LineEnding],
    ///     [`SpaceOrTab`][Token::SpaceOrTab]
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
    ///     [`Image`][Token::Image],
    ///     [`Link`][Token::Link]
    /// *   **Content model**:
    ///     [`ReferenceMarker`][Token::ReferenceMarker],
    ///     [`ReferenceString`][Token::ReferenceString]
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
    ///     [`Reference`][Token::Reference]
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
    ///     [`Reference`][Token::Reference]
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
    ///     [`Image`][Token::Image],
    ///     [`Link`][Token::Link]
    /// *   **Content model**:
    ///     [`ResourceMarker`][Token::ResourceMarker],
    ///     [`ResourceDestination`][Token::ResourceDestination],
    ///     [`ResourceTitle`][Token::ResourceTitle],
    ///     [`SpaceOrTab`][Token::SpaceOrTab],
    ///     [`LineEnding`][Token::LineEnding]
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
    ///     [`Resource`][Token::Resource]
    /// *   **Content model**:
    ///     [`ResourceDestinationLiteral`][Token::ResourceDestinationLiteral],
    ///     [`ResourceDestinationRaw`][Token::ResourceDestinationRaw]
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
    ///     [`ResourceDestination`][Token::ResourceDestination]
    /// *   **Content model**:
    ///     [`ResourceDestinationLiteralMarker`][Token::ResourceDestinationLiteralMarker],
    ///     [`ResourceDestinationString`][Token::ResourceDestinationString]
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
    ///     [`ResourceDestinationLiteral`][Token::ResourceDestinationLiteral]
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
    ///     [`ResourceDestination`][Token::ResourceDestination]
    /// *   **Content model**:
    ///     [`ResourceDestinationString`][Token::ResourceDestinationString]
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
    ///     [`ResourceDestinationLiteral`][Token::ResourceDestinationLiteral],
    ///     [`ResourceDestinationRaw`][Token::ResourceDestinationRaw]
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
    ///     [`Resource`][Token::Resource]
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
    ///     [`Resource`][Token::Resource]
    /// *   **Content model**:
    ///     [`ResourceTitleMarker`][Token::ResourceTitleMarker],
    ///     [`ResourceTitleString`][Token::ResourceTitleString]
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
    ///     [`ResourceTitle`][Token::ResourceTitle]
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
    ///     [`ResourceTitle`][Token::ResourceTitle]
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
    ///     [`StrongSequence`][Token::StrongSequence],
    ///     [`StrongText`][Token::StrongText]
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
    ///     [`Strong`][Token::Strong]
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
    ///     [`Strong`][Token::Strong]
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
    ///     [`ThematicBreakSequence`][Token::ThematicBreakSequence],
    ///     [`SpaceOrTab`][Token::SpaceOrTab]
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
    ///     [`ThematicBreak`][Token::ThematicBreak]
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
