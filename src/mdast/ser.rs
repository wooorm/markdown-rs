use serde::ser::{Serialize, SerializeSeq, SerializeStruct, SerializeStructVariant, Serializer};

use super::{
    AlignKind, AttributeContent, AttributeValue, BlockQuote, Break, Code, Definition, Delete,
    Emphasis, FootnoteDefinition, FootnoteReference, Heading, Html, Image, ImageReference,
    InlineCode, InlineMath, Link, LinkReference, List, ListItem, Math, MdxFlowExpression,
    MdxJsxAttribute, MdxJsxFlowElement, MdxJsxTextElement, MdxTextExpression, MdxjsEsm, Node,
    Paragraph, Root, Strong, Table, TableCell, TableRow, Text, ThematicBreak, Toml, Yaml,
};

impl Serialize for Node {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        match self {
            Self::Root(node) => node.serialize(serializer),
            Self::BlockQuote(node) => node.serialize(serializer),
            Self::FootnoteDefinition(node) => node.serialize(serializer),
            Self::MdxJsxFlowElement(node) => node.serialize(serializer),
            Self::List(node) => node.serialize(serializer),
            Self::MdxjsEsm(node) => node.serialize(serializer),
            Self::Toml(node) => node.serialize(serializer),
            Self::Yaml(node) => node.serialize(serializer),
            Self::Break(node) => node.serialize(serializer),
            Self::InlineCode(node) => node.serialize(serializer),
            Self::InlineMath(node) => node.serialize(serializer),
            Self::Delete(node) => node.serialize(serializer),
            Self::Emphasis(node) => node.serialize(serializer),
            Self::MdxTextExpression(node) => node.serialize(serializer),
            Self::FootnoteReference(node) => node.serialize(serializer),
            Self::Html(node) => node.serialize(serializer),
            Self::Image(node) => node.serialize(serializer),
            Self::ImageReference(node) => node.serialize(serializer),
            Self::MdxJsxTextElement(node) => node.serialize(serializer),
            Self::Link(node) => node.serialize(serializer),
            Self::LinkReference(node) => node.serialize(serializer),
            Self::Strong(node) => node.serialize(serializer),
            Self::Text(node) => node.serialize(serializer),
            Self::Code(node) => node.serialize(serializer),
            Self::Math(node) => node.serialize(serializer),
            Self::MdxFlowExpression(node) => node.serialize(serializer),
            Self::Heading(node) => node.serialize(serializer),
            Self::Table(node) => node.serialize(serializer),
            Self::ThematicBreak(node) => node.serialize(serializer),
            Self::TableRow(node) => node.serialize(serializer),
            Self::TableCell(node) => node.serialize(serializer),
            Self::ListItem(node) => node.serialize(serializer),
            Self::Definition(node) => node.serialize(serializer),
            Self::Paragraph(node) => node.serialize(serializer),
        }
    }
}

impl Serialize for Root {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut state = serializer.serialize_struct("Root", 3)?;
        state.serialize_field("type", "root")?;
        state.serialize_field("children", &self.children)?;
        state.serialize_field("position", &self.position)?;
        state.end()
    }
}

impl Serialize for BlockQuote {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut state = serializer.serialize_struct("BlockQuote", 3)?;
        state.serialize_field("type", "blockquote")?;
        state.serialize_field("children", &self.children)?;
        state.serialize_field("position", &self.position)?;
        state.end()
    }
}

impl Serialize for FootnoteDefinition {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut state = serializer.serialize_struct("FootnoteDefinition", 5)?;
        state.serialize_field("type", "footnoteDefinition")?;
        state.serialize_field("identifier", &self.identifier)?;
        state.serialize_field("label", &self.label)?;
        state.serialize_field("children", &self.children)?;
        state.serialize_field("position", &self.position)?;
        state.end()
    }
}

impl Serialize for AttributeValue {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        match self {
            Self::Expression(expr, _) => {
                let mut state =
                    serializer.serialize_struct_variant("AttributeValue", 0, "Expression", 2)?;
                state.serialize_field("type", "mdxJsxAttributeValueExpression")?;
                state.serialize_field("value", expr)?;
                state.end()
            }
            Self::Literal(lit) => {
                let mut state =
                    serializer.serialize_struct_variant("AttributeValue", 1, "Literal", 2)?;
                state.serialize_field("type", "literal")?;
                state.serialize_field("value", lit)?;
                state.end()
            }
        }
    }
}

impl Serialize for MdxJsxAttribute {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut state = serializer.serialize_struct("MdxJsxAttribute", 3)?;
        state.serialize_field("type", "mdxJsxAttribute")?;
        state.serialize_field("name", &self.name)?;
        state.serialize_field("value", &self.value)?;
        state.end()
    }
}

impl Serialize for AttributeContent {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        match self {
            Self::Expression(expr, _) => {
                let mut state =
                    serializer.serialize_struct_variant("AttributeContent", 0, "Expression", 2)?;
                state.serialize_field("type", "mdxJsxExpressionAttribute")?;
                state.serialize_field("value", &expr)?;
                state.end()
            }
            Self::Property(prop) => {
                serializer.serialize_newtype_variant("AttributeContent", 1, "Property", &prop)
            }
        }
    }
}

impl Serialize for MdxJsxFlowElement {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut state = serializer.serialize_struct("MdxJsxFlowElement", 5)?;
        state.serialize_field("type", "mdxJsxFlowElement")?;
        state.serialize_field("name", &self.name)?;
        state.serialize_field("attributes", &self.attributes)?;
        state.serialize_field("children", &self.children)?;
        state.serialize_field("position", &self.position)?;
        state.end()
    }
}

impl Serialize for List {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut state = serializer.serialize_struct("List", 6)?;
        state.serialize_field("type", "list")?;
        state.serialize_field("ordered", &self.ordered)?;
        state.serialize_field("start", &self.start)?;
        state.serialize_field("spread", &self.spread)?;
        state.serialize_field("children", &self.children)?;
        state.serialize_field("position", &self.position)?;
        state.end()
    }
}

impl Serialize for MdxjsEsm {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut state = serializer.serialize_struct("MdxjsEsm", 3)?;
        state.serialize_field("type", "mdxjsEsm")?;
        state.serialize_field("value", &self.value)?;
        state.serialize_field("position", &self.position)?;
        state.end()
    }
}

impl Serialize for Toml {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut state = serializer.serialize_struct("Toml", 3)?;
        state.serialize_field("type", "toml")?;
        state.serialize_field("value", &self.value)?;
        state.serialize_field("position", &self.position)?;
        state.end()
    }
}

impl Serialize for Yaml {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut state = serializer.serialize_struct("Yaml", 3)?;
        state.serialize_field("type", "yaml")?;
        state.serialize_field("value", &self.value)?;
        state.serialize_field("position", &self.position)?;
        state.end()
    }
}

impl Serialize for Break {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut state = serializer.serialize_struct("Break", 2)?;
        state.serialize_field("type", "break")?;
        state.serialize_field("position", &self.position)?;
        state.end()
    }
}

impl Serialize for InlineCode {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut state = serializer.serialize_struct("InlineCode", 3)?;
        state.serialize_field("type", "inlineCode")?;
        state.serialize_field("value", &self.value)?;
        state.serialize_field("position", &self.position)?;
        state.end()
    }
}

impl Serialize for InlineMath {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut state = serializer.serialize_struct("InlineMath", 3)?;
        state.serialize_field("type", "inlineMath")?;
        state.serialize_field("value", &self.value)?;
        state.serialize_field("position", &self.position)?;
        state.end()
    }
}

impl Serialize for Delete {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut state = serializer.serialize_struct("Delete", 3)?;
        state.serialize_field("type", "delete")?;
        state.serialize_field("children", &self.children)?;
        state.serialize_field("position", &self.position)?;
        state.end()
    }
}

impl Serialize for Emphasis {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut state = serializer.serialize_struct("Emphasis", 3)?;
        state.serialize_field("type", "emphasis")?;
        state.serialize_field("children", &self.children)?;
        state.serialize_field("position", &self.position)?;
        state.end()
    }
}

impl Serialize for MdxTextExpression {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut state = serializer.serialize_struct("MdxTextExpression", 3)?;
        state.serialize_field("type", "mdxTextExpression")?;
        state.serialize_field("value", &self.value)?;
        state.serialize_field("position", &self.position)?;
        state.end()
    }
}

impl Serialize for FootnoteReference {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut state = serializer.serialize_struct("FootnoteReference", 4)?;
        state.serialize_field("type", "footnoteReference")?;
        state.serialize_field("identifier", &self.identifier)?;
        state.serialize_field("label", &self.label)?;
        state.serialize_field("position", &self.position)?;
        state.end()
    }
}

impl Serialize for Html {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut state = serializer.serialize_struct("Html", 3)?;
        state.serialize_field("type", "html")?;
        state.serialize_field("value", &self.value)?;
        state.serialize_field("position", &self.position)?;
        state.end()
    }
}

impl Serialize for Image {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut state = serializer.serialize_struct("Image", 5)?;
        state.serialize_field("type", "image")?;
        state.serialize_field("alt", &self.alt)?;
        state.serialize_field("url", &self.url)?;
        state.serialize_field("title", &self.title)?;
        state.serialize_field("position", &self.position)?;
        state.end()
    }
}

impl Serialize for ImageReference {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut state = serializer.serialize_struct("ImageReference", 6)?;
        state.serialize_field("type", "imageReference")?;
        state.serialize_field("identifier", &self.identifier)?;
        state.serialize_field("label", &self.label)?;
        state.serialize_field("referenceType", self.reference_kind.as_str())?;
        state.serialize_field("alt", &self.alt)?;
        state.serialize_field("position", &self.position)?;
        state.end()
    }
}

impl Serialize for MdxJsxTextElement {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut state = serializer.serialize_struct("MdxJsxTextElement", 5)?;
        state.serialize_field("type", "mdxJsxTextElement")?;
        state.serialize_field("name", &self.name)?;
        state.serialize_field("attributes", &self.attributes)?;
        state.serialize_field("children", &self.children)?;
        state.serialize_field("position", &self.position)?;
        state.end()
    }
}

impl Serialize for Link {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut state = serializer.serialize_struct("Link", 5)?;
        state.serialize_field("type", "link")?;
        state.serialize_field("url", &self.url)?;
        state.serialize_field("title", &self.title)?;
        state.serialize_field("children", &self.children)?;
        state.serialize_field("position", &self.position)?;
        state.end()
    }
}

impl Serialize for LinkReference {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut state = serializer.serialize_struct("LinkReference", 6)?;
        state.serialize_field("type", "linkReference")?;
        state.serialize_field("identifier", &self.identifier)?;
        state.serialize_field("label", &self.label)?;
        state.serialize_field("referenceType", self.reference_kind.as_str())?;
        state.serialize_field("children", &self.children)?;
        state.serialize_field("position", &self.position)?;
        state.end()
    }
}

impl Serialize for Strong {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut state = serializer.serialize_struct("Strong", 3)?;
        state.serialize_field("type", "strong")?;
        state.serialize_field("children", &self.children)?;
        state.serialize_field("position", &self.position)?;
        state.end()
    }
}

impl Serialize for Text {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut state = serializer.serialize_struct("Text", 3)?;
        state.serialize_field("type", "text")?;
        state.serialize_field("value", &self.value)?;
        state.serialize_field("position", &self.position)?;
        state.end()
    }
}

impl Serialize for Code {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut state = serializer.serialize_struct("Code", 4)?;
        state.serialize_field("type", "code")?;
        state.serialize_field("value", &self.value)?;
        state.serialize_field("lang", &self.lang)?;
        state.serialize_field("meta", &self.meta)?;
        state.serialize_field("position", &self.position)?;
        state.end()
    }
}

impl Serialize for Math {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut state = serializer.serialize_struct("Math", 4)?;
        state.serialize_field("type", "math")?;
        state.serialize_field("value", &self.value)?;
        state.serialize_field("meta", &self.meta)?;
        state.serialize_field("position", &self.position)?;
        state.end()
    }
}

impl Serialize for MdxFlowExpression {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut state = serializer.serialize_struct("MdxFlowExpression", 3)?;
        state.serialize_field("type", "mdxFlowExpression")?;
        state.serialize_field("value", &self.value)?;
        state.serialize_field("position", &self.position)?;
        state.end()
    }
}

impl Serialize for Heading {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut state = serializer.serialize_struct("Heading", 4)?;
        state.serialize_field("type", "heading")?;
        state.serialize_field("depth", &self.depth)?;
        state.serialize_field("children", &self.children)?;
        state.serialize_field("position", &self.position)?;
        state.end()
    }
}

impl Serialize for AlignKind {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        match self.as_str() {
            Some(value) => serializer.serialize_some(value),
            None => serializer.serialize_none(),
        }
    }
}

impl Serialize for Table {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut state = serializer.serialize_struct("Table", 4)?;
        state.serialize_field("type", "table")?;
        state.serialize_field("align", &self.align)?;
        state.serialize_field("children", &self.children)?;
        state.serialize_field("position", &self.position)?;
        state.end()
    }
}

impl Serialize for ThematicBreak {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut state = serializer.serialize_struct("ThematicBreak", 2)?;
        state.serialize_field("type", "thematicBreak")?;
        state.serialize_field("position", &self.position)?;
        state.end()
    }
}

impl Serialize for TableRow {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut state = serializer.serialize_struct("TableRow", 3)?;
        state.serialize_field("type", "tableRow")?;
        state.serialize_field("children", &self.children)?;
        state.serialize_field("position", &self.position)?;
        state.end()
    }
}

impl Serialize for TableCell {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut state = serializer.serialize_struct("TableCell", 3)?;
        state.serialize_field("type", "tableCell")?;
        state.serialize_field("children", &self.children)?;
        state.serialize_field("position", &self.position)?;
        state.end()
    }
}

impl Serialize for ListItem {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut state = serializer.serialize_struct("ListItem", 5)?;
        state.serialize_field("type", "listItem")?;
        state.serialize_field("spread", &self.spread)?;
        state.serialize_field("children", &self.children)?;
        state.serialize_field("checked", &self.checked)?;
        state.serialize_field("position", &self.position)?;
        state.end()
    }
}

impl Serialize for Definition {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut state = serializer.serialize_struct("Definition", 6)?;
        state.serialize_field("type", "definition")?;
        state.serialize_field("identifier", &self.identifier)?;
        state.serialize_field("url", &self.url)?;
        state.serialize_field("title", &self.title)?;
        state.serialize_field("label", &self.label)?;
        state.serialize_field("position", &self.position)?;
        state.end()
    }
}

impl Serialize for Paragraph {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut state = serializer.serialize_struct("Paragraph", 3)?;
        state.serialize_field("type", "paragraph")?;
        state.serialize_field("children", &self.children)?;
        state.serialize_field("position", &self.position)?;
        state.end()
    }
}
