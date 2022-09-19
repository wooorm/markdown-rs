//! [mdast][] syntax tree.
//!
//! [mdast]: https://github.com/syntax-tree/mdast

// To do: example.
// To do: math.

use alloc::{string::String, vec::Vec};

/// One place in a source file.
#[derive(Clone, Debug)]
pub struct Point {
    /// 1-indexed integer representing a line in a source file.
    pub line: usize,
    /// 1-indexed integer representing a column in a source file.
    pub column: usize,
    /// 0-indexed integer representing a character in a source file.
    pub offset: usize,
}

/// Location of a node in a source file.
#[derive(Clone, Debug)]
pub struct Position {
    /// Represents the place of the first character of the parsed source region.
    pub start: Point,
    /// Represents the place of the first character after the parsed source
    /// region, whether it exists or not.
    pub end: Point,
}

/// Explicitness of a reference.
#[derive(Clone, Debug)]
pub enum ReferenceKind {
    /// The reference is implicit, its identifier inferred from its content.
    Shortcut,
    /// The reference is explicit, its identifier inferred from its content.
    Collapsed,
    /// The reference is explicit, its identifier explicitly set.
    Full,
}

/// Represents how phrasing content is aligned.
#[derive(Clone, Debug)]
pub enum AlignKind {
    /// See the `left` value of the `text-align` CSS property.
    Left,
    /// See the `right` value of the `text-align` CSS property.
    Right,
    /// See the `center` value of the `text-align` CSS property.
    Center,
    /// Phrasing content is aligned as defined by the host environment.
    None,
}

/// Node type.
#[derive(Clone, Debug)]
pub enum Kind {
    /// Root node.
    Root,
    /// Paragraph node.
    Paragraph,
    /// Heading node.
    Heading,
    /// Thematic break node.
    ThematicBreak,
    /// Block quote node.
    BlockQuote,
    /// List node.
    List,
    /// List item node.
    ListItem,
    /// Html node.
    Html,
    /// Code node.
    Code,
    /// Definition node.
    Definition,
    /// Text node.
    Text,
    /// Emphasis node.
    Emphasis,
    /// Strong node.
    Strong,
    /// Code (inline) node.
    InlineCode,
    /// Break node.
    Break,
    /// Link node.
    Link,
    /// Image node.
    Image,
    /// Link reference node.
    LinkReference,
    /// Image reference node.
    ImageReference,
    /// Footnote definition node.
    FootnoteDefinition,
    /// Footnote reference node.
    FootnoteReference,
    /// Table node.
    Table,
    /// Table row node.
    TableRow,
    /// Table cell node.
    TableCell,
    /// Strong node.
    Delete,
    /// Yaml node.
    Yaml,
    /// Toml node.
    Toml,
    /// MDX: ESM node.
    MdxjsEsm,
    /// MDX: expression (flow).
    MdxFlowExpression,
    /// MDX: expression (phrasing).
    MdxTextExpression,
    /// MDX: JSX element (flow).
    MdxJsxFlowElement,
    /// MDX: JSX element (phrasing).
    MdxJsxTextElement,
    /// MDX: JSX attribute expression.
    MdxJsxExpressionAttribute,
    /// MDX: JSX attribute.
    MdxJsxAttribute,
    /// MDX: JSX attribute value expression.
    MdxJsxAttributeValueExpression,
}

/// Document content.
#[derive(Clone, Debug)]
pub enum DocumentContent {
    /// Container content.
    Container(ContainerContent),
    /// Frontmatter content.
    Frontmatter(FrontmatterContent),
}

/// Container content.
#[derive(Clone, Debug)]
pub enum ContainerContent {
    /// Block quote.
    BlockQuote(BlockQuote),
    /// Flow content.
    Flow(FlowContent),
    /// Footnote definition.
    FootnoteDefinition(FootnoteDefinition),
    /// MDX: JSX element (container).
    JsxElement(MdxJsxFlowElement),
    /// List.
    List(List),
}

/// Frontmatter content.
#[derive(Clone, Debug)]
pub enum FrontmatterContent {
    /// MDX.js ESM.
    Esm(MdxjsEsm),
    /// Toml.
    Toml(Toml),
    /// Yaml.
    Yaml(Yaml),
}

/// Phrasing content.
#[derive(Clone, Debug)]
pub enum PhrasingContent {
    /// Break.
    Break(Break),
    /// Code (phrasing).
    Code(InlineCode),
    /// Delete.
    Delete(Delete),
    /// Emphasis.
    Emphasis(Emphasis),
    // MDX: expression (text).
    Expression(MdxTextExpression),
    /// Footnote reference.
    FootnoteReference(FootnoteReference),
    /// Html (phrasing).
    Html(Html),
    /// Image.
    Image(Image),
    /// Image reference.
    ImageReference(ImageReference),
    // MDX: JSX element (text).
    JsxElement(MdxJsxTextElement),
    /// Link.
    Link(Link),
    /// Link reference.
    LinkReference(LinkReference),
    /// Strong
    Strong(Strong),
    /// Text.
    Text(Text),
}

/// Flow content.
#[derive(Clone, Debug)]
pub enum FlowContent {
    /// Code (flow).
    Code(Code),
    /// Content.
    Content(ContentContent),
    // MDX: expression (flow).
    Expression(MdxFlowExpression),
    /// Heading.
    Heading(Heading),
    /// Html (flow).
    Html(Html),
    /// Table.
    Table(Table),
    /// Thematic break.
    ThematicBreak(ThematicBreak),
}

/// Table content.
#[derive(Clone, Debug)]
pub enum TableContent {
    /// Table row.
    Row(TableRow),
}

/// Row content.
#[derive(Clone, Debug)]
pub enum RowContent {
    /// Table cell.
    Cell(TableCell),
}

/// List content.
#[derive(Clone, Debug)]
pub enum ListContent {
    /// List item.
    Item(ListItem),
}

/// Content.
#[derive(Clone, Debug)]
pub enum ContentContent {
    /// Definition.
    Definition(Definition),
    /// Paragraph.
    Paragraph(Paragraph),
}

/// MDX: attribute content.
#[derive(Clone, Debug)]
pub enum AttributeContent {
    /// MDX: JSX attribute expression.
    Expression(MdxJsxExpressionAttribute),
    /// MDX: JSX attribute.
    Property(MdxJsxAttribute),
}

/// MDX: attribute value.
#[derive(Clone, Debug)]
pub enum AttributeValue {
    /// Expression value.
    Expression(MdxJsxAttributeValueExpression),
    /// Static value.
    Literal(String),
}

/// Document.
///
/// ```markdown
/// > | a
///     ^
/// ```
#[derive(Clone, Debug)]
pub struct Root {
    // Parent.
    /// Node type.
    pub kind: Kind, // `Kind::Root`.
    /// Content model.
    pub children: Vec<DocumentContent>,
    /// Positional info.
    pub position: Option<Position>,
}

/// Paragraph.
///
/// ```markdown
/// > | a
///     ^
/// ```
#[derive(Clone, Debug)]
pub struct Paragraph {
    // Parent.
    /// Node type.
    pub kind: Kind, // `Kind::Paragraph`.
    /// Content model.
    pub children: Vec<PhrasingContent>,
    /// Positional info.
    pub position: Option<Position>,
}

/// Heading.
///
/// ```markdown
/// > | # a
///     ^^^
/// ```
#[derive(Clone, Debug)]
pub struct Heading {
    // Parent.
    /// Node type.
    pub kind: Kind, // `Kind::Heading`.
    /// Content model.
    pub children: Vec<PhrasingContent>,
    /// Positional info.
    pub position: Option<Position>,
    // Extra.
    /// Rank (between `1` and `6`, both including).
    pub depth: u8,
}

/// Thematic break.
///
/// ```markdown
/// > | ***
///     ^^^
/// ```
#[derive(Clone, Debug)]
pub struct ThematicBreak {
    // Void.
    /// Node type.
    pub kind: Kind, // `Kind::ThematicBreak`.
    /// Positional info.
    pub position: Option<Position>,
}

/// Block quote.
///
/// ```markdown
/// > | > a
///     ^^^
/// ```
#[derive(Clone, Debug)]
pub struct BlockQuote {
    // Parent.
    /// Node type.
    pub kind: Kind, // `Kind::BlockQuote`.
    /// Content model.
    pub children: Vec<ContainerContent>,
    /// Positional info.
    pub position: Option<Position>,
}

/// List.
///
/// ```markdown
/// > | * a
///     ^^^
/// ```
#[derive(Clone, Debug)]
pub struct List {
    // Parent.
    /// Node type.
    pub kind: Kind, // `Kind::List`.
    /// Content model.
    pub children: Vec<ListContent>,
    /// Positional info.
    pub position: Option<Position>,
    // Extra.
    /// Ordered (`true`) or unordered (`false`).
    pub ordered: bool,
    /// Starting number of the list.
    /// `None` when unordered.
    pub start: Option<u8>,
    /// One or more of its children are separated with a blank line from its
    /// siblings (when `true`), or not (when `false`).
    pub spread: bool,
}

/// List item.
///
/// ```markdown
/// > | * a
///     ^^^
/// ```
#[derive(Clone, Debug)]
pub struct ListItem {
    // Parent.
    /// Node type.
    pub kind: Kind, // `Kind::ListItem`.
    /// Content model.
    pub children: Vec<ContainerContent>,
    /// Positional info.
    pub position: Option<Position>,
    // Extra.
    /// The item contains two or more children separated by a blank line
    /// (when `true`), or not (when `false`).
    pub spread: bool,
    /// GFM: whether the item is done (when `true`), not done (when `false`),
    /// or indeterminate or not applicable (`None`).
    pub checked: Option<bool>,
}

/// Html (flow or phrasing).
///
/// ```markdown
/// > | <a>
///     ^^^
/// ```
#[derive(Clone, Debug)]
pub struct Html {
    // Text.
    /// Node type.
    pub kind: Kind, // `Kind::Html`.
    /// Content model.
    pub value: String,
    /// Positional info.
    pub position: Option<Position>,
}

/// Code (flow).
///
/// ```markdown
/// > | ~~~
///     ^^^
/// > | a
///     ^^^
/// > | ~~~
///     ^^^
/// ```
#[derive(Clone, Debug)]
pub struct Code {
    // Text.
    /// Node type.
    pub kind: Kind, // `Kind::Code`.
    /// Content model.
    pub value: String,
    /// Positional info.
    pub position: Option<Position>,
    // Extra.
    /// The language of computer code being marked up.
    pub lang: Option<String>,
    /// Custom info relating to the node.
    pub meta: Option<String>,
}

/// Definition.
///
/// ```markdown
/// > | [a]: b
///     ^^^^^^
/// ```
#[derive(Clone, Debug)]
pub struct Definition {
    // Void.
    /// Node type.
    pub kind: Kind, // `Kind::Definition`.
    /// Positional info.
    pub position: Option<Position>,
    // Resource.
    /// URL to the referenced resource.
    pub url: String,
    /// Advisory info for the resource, such as something that would be
    /// appropriate for a tooltip.
    pub title: Option<String>,
    // Association.
    /// Value that can match another node.
    /// `identifier` is a source value: character escapes and character references
    /// are *not* parsed.
    /// Its value must be normalized.
    pub identifier: String,
    /// `label` is a string value: it works just like `title` on a link or a
    /// `lang` on code: character escapes and character references are parsed.
    ///
    /// To normalize a value, collapse markdown whitespace (`[\t\n\r ]+`) to a
    /// space, trim the optional initial and/or final space, and perform
    /// case-folding.
    pub label: Option<String>,
}

/// Text.
///
/// ```markdown
/// > | a
///     ^
/// ```
#[derive(Clone, Debug)]
pub struct Text {
    // Text.
    /// Node type.
    pub kind: Kind, // `Kind::Text`.
    /// Content model.
    pub value: String,
    /// Positional info.
    pub position: Option<Position>,
}

/// Emphasis.
///
/// ```markdown
/// > | *a*
///     ^^^
/// ```
#[derive(Clone, Debug)]
pub struct Emphasis {
    // Parent.
    /// Node type.
    pub kind: Kind, // `Kind::Emphasis`.
    /// Content model.
    pub children: Vec<PhrasingContent>,
    /// Positional info.
    pub position: Option<Position>,
}

/// Strong.
///
/// ```markdown
/// > | **a**
///     ^^^^^
/// ```
#[derive(Clone, Debug)]
pub struct Strong {
    // Parent.
    /// Node type.
    pub kind: Kind, // `Kind::Strong`.
    /// Content model.
    pub children: Vec<PhrasingContent>,
    /// Positional info.
    pub position: Option<Position>,
}

/// Code (phrasing).
///
/// ```markdown
/// > | `a`
///     ^^^
/// ```
#[derive(Clone, Debug)]
pub struct InlineCode {
    // Text.
    /// Node type.
    pub kind: Kind, // `Kind::InlineCode`.
    /// Content model.
    pub value: String,
    /// Positional info.
    pub position: Option<Position>,
}

/// Break.
///
/// ```markdown
/// > | a\
///      ^
///   | b
/// ```
#[derive(Clone, Debug)]
pub struct Break {
    // Void.
    /// Node type.
    pub kind: Kind, // `Kind::Break`.
    /// Positional info.
    pub position: Option<Position>,
}

/// Link.
///
/// ```markdown
/// > | [a](b)
///     ^^^^^^
/// ```
#[derive(Clone, Debug)]
pub struct Link {
    // Parent.
    /// Node type.
    pub kind: Kind, // `Kind::Link`.
    /// Content model.
    pub children: Vec<PhrasingContent>,
    /// Positional info.
    pub position: Option<Position>,
    // Resource.
    /// URL to the referenced resource.
    pub url: String,
    /// Advisory info for the resource, such as something that would be
    /// appropriate for a tooltip.
    pub title: Option<String>,
}

/// Image.
///
/// ```markdown
/// > | ![a](b)
///     ^^^^^^^
/// ```
#[derive(Clone, Debug)]
pub struct Image {
    // Void.
    /// Node type.
    pub kind: Kind, // `Kind::Image`.
    /// Positional info.
    pub position: Option<Position>,
    // Alternative.
    /// Equivalent content for environments that cannot represent the node as
    /// intended.
    pub alt: String,
    // Resource.
    /// URL to the referenced resource.
    pub url: String,
    /// Advisory info for the resource, such as something that would be
    /// appropriate for a tooltip.
    pub title: Option<String>,
}

/// Link reference.
///
/// ```markdown
/// > | [a]
///     ^^^
/// ```
#[derive(Clone, Debug)]
pub struct LinkReference {
    // Parent.
    /// Node type.
    pub kind: Kind, // `Kind::LinkReference`.
    /// Content model.
    pub children: Vec<PhrasingContent>,
    /// Positional info.
    pub position: Option<Position>,
    // Reference.
    /// Explicitness of a reference.
    pub reference_kind: ReferenceKind,
    // Association.
    /// Value that can match another node.
    /// `identifier` is a source value: character escapes and character references
    /// are *not* parsed.
    /// Its value must be normalized.
    pub identifier: String,
    /// `label` is a string value: it works just like `title` on a link or a
    /// `lang` on code: character escapes and character references are parsed.
    ///
    /// To normalize a value, collapse markdown whitespace (`[\t\n\r ]+`) to a
    /// space, trim the optional initial and/or final space, and perform
    /// case-folding.
    pub label: Option<String>,
}

/// Image reference.
///
/// ```markdown
/// > | ![a]
///     ^^^^
/// ```
#[derive(Clone, Debug)]
pub struct ImageReference {
    // Void.
    /// Node type.
    pub kind: Kind, // `Kind::ImageReference`.
    /// Positional info.
    pub position: Option<Position>,
    // Alternative.
    /// Equivalent content for environments that cannot represent the node as
    /// intended.
    pub alt: String,
    // Reference.
    /// Explicitness of a reference.
    pub reference_kind: ReferenceKind,
    // Association.
    /// Value that can match another node.
    /// `identifier` is a source value: character escapes and character references
    /// are *not* parsed.
    /// Its value must be normalized.
    pub identifier: String,
    /// `label` is a string value: it works just like `title` on a link or a
    /// `lang` on code: character escapes and character references are parsed.
    ///
    /// To normalize a value, collapse markdown whitespace (`[\t\n\r ]+`) to a
    /// space, trim the optional initial and/or final space, and perform
    /// case-folding.
    pub label: Option<String>,
}

/// Footnote definition (GFM).
///
/// ```markdown
/// > | [^a]: b
///     ^^^^^^^
/// ```
#[derive(Clone, Debug)]
pub struct FootnoteDefinition {
    // Parent.
    /// Node type.
    pub kind: Kind, // `Kind::FootnoteDefinition`.
    /// Content model.
    pub children: Vec<ContainerContent>,
    /// Positional info.
    pub position: Option<Position>,
    // Association.
    /// Value that can match another node.
    /// `identifier` is a source value: character escapes and character references
    /// are *not* parsed.
    /// Its value must be normalized.
    pub identifier: String,
    /// `label` is a string value: it works just like `title` on a link or a
    /// `lang` on code: character escapes and character references are parsed.
    ///
    /// To normalize a value, collapse markdown whitespace (`[\t\n\r ]+`) to a
    /// space, trim the optional initial and/or final space, and perform
    /// case-folding.
    pub label: Option<String>,
}

/// Footnote reference (GFM).
///
/// ```markdown
/// > | [^a]
///     ^^^^
/// ```
#[derive(Clone, Debug)]
pub struct FootnoteReference {
    // Void.
    /// Node type.
    pub kind: Kind, // `Kind::FootnoteReference`.
    /// Positional info.
    pub position: Option<Position>,
    // Association.
    /// Value that can match another node.
    /// `identifier` is a source value: character escapes and character references
    /// are *not* parsed.
    /// Its value must be normalized.
    pub identifier: String,
    /// `label` is a string value: it works just like `title` on a link or a
    /// `lang` on code: character escapes and character references are parsed.
    ///
    /// To normalize a value, collapse markdown whitespace (`[\t\n\r ]+`) to a
    /// space, trim the optional initial and/or final space, and perform
    /// case-folding.
    pub label: Option<String>,
}

/// Table (GFM).
///
/// ```markdown
/// > | | a |
///     ^^^^^
/// > | | - |
///     ^^^^^
/// ```
#[derive(Clone, Debug)]
pub struct Table {
    // Parent.
    /// Node type.
    pub kind: Kind, // `Kind::Table`.
    /// Content model.
    pub children: Vec<TableContent>,
    /// Positional info.
    pub position: Option<Position>,
    // Extra.
    /// Represents how cells in columns are aligned.
    pub align: Vec<AlignKind>,
}

/// Table row (GFM).
///
/// ```markdown
/// > | | a |
///     ^^^^^
/// ```
#[derive(Clone, Debug)]
pub struct TableRow {
    // Parent.
    /// Node type.
    pub kind: Kind, // `Kind::TableRow`.
    /// Content model.
    pub children: Vec<RowContent>,
    /// Positional info.
    pub position: Option<Position>,
}

/// Table cell (GFM).
///
/// ```markdown
/// > | | a |
///     ^^^^^
/// ```
#[derive(Clone, Debug)]
pub struct TableCell {
    // Parent.
    /// Node type.
    pub kind: Kind, // `Kind::TableCell`.
    /// Content model.
    pub children: Vec<PhrasingContent>,
    /// Positional info.
    pub position: Option<Position>,
}

/// Delete (GFM).
///
/// ```markdown
/// > | ~~a~~
///     ^^^^^
/// ```
#[derive(Clone, Debug)]
pub struct Delete {
    // Parent.
    /// Node type.
    pub kind: Kind, // `Kind::Delete`.
    /// Content model.
    pub children: Vec<PhrasingContent>,
    /// Positional info.
    pub position: Option<Position>,
}

/// Yaml (frontmatter).
///
/// ```markdown
/// > | ---
///     ^^^
/// > | a: b
///     ^^^^
/// > | ---
///     ^^^
/// ```
#[derive(Clone, Debug)]
pub struct Yaml {
    // Void.
    /// Node type.
    pub kind: Kind, // `Kind::Yaml`.
    /// Content model.
    pub value: String,
    /// Positional info.
    pub position: Option<Position>,
}

/// Toml (frontmatter).
///
/// ```markdown
/// > | +++
///     ^^^
/// > | a: b
///     ^^^^
/// > | +++
///     ^^^
/// ```
#[derive(Clone, Debug)]
pub struct Toml {
    // Void.
    /// Node type.
    pub kind: Kind, // `Kind::Toml`.
    /// Content model.
    pub value: String,
    /// Positional info.
    pub position: Option<Position>,
}

/// MDX: ESM.
///
/// ```markdown
/// > | import a from 'b'
///     ^^^^^^^^^^^^^^^^^
/// ```
#[derive(Clone, Debug)]
pub struct MdxjsEsm {
    // Literal.
    /// Node type.
    pub kind: Kind, // `Kind::MdxjsEsm`.
    /// Content model.
    pub value: String,
    /// Positional info.
    pub position: Option<Position>,
}

/// MDX: expression (flow).
///
/// ```markdown
/// > | {a}
///     ^^^
/// ```
#[derive(Clone, Debug)]
pub struct MdxFlowExpression {
    // Literal.
    /// Node type.
    pub kind: Kind, // `Kind::MdxFlowExpression`.
    /// Content model.
    pub value: String,
    /// Positional info.
    pub position: Option<Position>,
}

/// MDX: expression (text).
///
/// ```markdown
/// > | a {b}
///       ^^^
/// ```
#[derive(Clone, Debug)]
pub struct MdxTextExpression {
    // Literal.
    /// Node type.
    pub kind: Kind, // `Kind::MdxTextExpression`.
    /// Content model.
    pub value: String,
    /// Positional info.
    pub position: Option<Position>,
}

/// MDX: JSX element (container).
///
/// ```markdown
/// > | <a />
///     ^^^^^
/// ```
#[derive(Clone, Debug)]
pub struct MdxJsxFlowElement {
    // Parent.
    /// Node type.
    pub kind: Kind, // `Kind::MdxJsxFlowElement`.
    /// Content model.
    pub children: Vec<ContainerContent>,
    /// Positional info.
    pub position: Option<Position>,
    // JSX element.
    /// Name.
    ///
    /// Fragments have no name.
    pub name: Option<String>,
    /// Attributes.
    pub attributes: Vec<AttributeContent>,
}

/// MDX: JSX element (text).
///
/// ```markdown
/// > | <a />.
///     ^^^^^
/// ```
#[derive(Clone, Debug)]
pub struct MdxJsxTextElement {
    // Parent.
    /// Node type.
    pub kind: Kind, // `Kind::MdxJsxTextElement`.
    /// Content model.
    pub children: Vec<PhrasingContent>,
    /// Positional info.
    pub position: Option<Position>,
    // JSX element.
    /// Name.
    ///
    /// Fragments have no name.
    pub name: Option<String>,
    /// Attributes.
    pub attributes: Vec<AttributeContent>,
}

/// MDX: JSX attribute expression.
///
/// ```markdown
/// > | <a {...b} />
///        ^^^^^^
/// ```
#[derive(Clone, Debug)]
pub struct MdxJsxExpressionAttribute {
    // Literal.
    /// Node type.
    pub kind: Kind, // `Kind::MdxJsxExpressionAttribute`.
    /// Content model.
    pub value: String,
    /// Positional info.
    pub position: Option<Position>,
}

/// MDX: JSX attribute.
///
/// ```markdown
/// > | <a b />
///        ^
/// ```
#[derive(Clone, Debug)]
pub struct MdxJsxAttribute {
    // Void.
    /// Node type.
    pub kind: Kind, // `Kind::MdxJsxAttribute`.
    /// Positional info.
    pub position: Option<Position>,
    /// Key.
    pub name: String,
    /// Value.
    pub value: Option<AttributeValue>,
}

/// MDX: JSX attribute value expression.
///
/// ```markdown
/// > | <a b={c} />
///          ^^^
/// ```
#[derive(Clone, Debug)]
pub struct MdxJsxAttributeValueExpression {
    // Literal.
    /// Node type.
    pub kind: Kind, // `Kind::MdxJsxAttributeValueExpression`.
    /// Content model.
    pub value: String,
    /// Positional info.
    pub position: Option<Position>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use alloc::{string::ToString, vec};

    #[test]
    fn test() {
        let text = Text {
            kind: Kind::Text,
            value: "a".to_string(),
            position: Some(Position {
                start: Point {
                    line: 1,
                    column: 1,
                    offset: 0,
                },
                end: Point {
                    line: 1,
                    column: 2,
                    offset: 1,
                },
            }),
        };

        let paragraph = Paragraph {
            kind: Kind::Paragraph,
            children: vec![PhrasingContent::Text(text)],
            position: Some(Position {
                start: Point {
                    line: 1,
                    column: 1,
                    offset: 0,
                },
                end: Point {
                    line: 1,
                    column: 2,
                    offset: 1,
                },
            }),
        };

        assert_eq!(paragraph.children.len(), 1);
        assert!(matches!(&paragraph.children[0], PhrasingContent::Text(_)));
    }
}
