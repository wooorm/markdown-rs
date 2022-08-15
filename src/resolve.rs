//! Resolve events.

use crate::construct;
use crate::tokenizer::Tokenizer;

/// Names of resolvers.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Name {
    /// Resolve labels.
    ///
    /// Labels are parsed as starts and ends, and when they match, merged
    /// together to form media (links and images), and otherwise turned into
    /// data.
    Label,
    /// Resolve attention.
    ///
    /// Attention sequences are parsed and finally matched together to form
    /// attention (emphasis and strong) based on which characters they contain,
    /// and what occurs before and after each sequence.
    /// Otherwise they are turned into data.
    Attention,
    /// Resolve heading (atx).
    ///
    /// Heading (atx) contains further sequences and data.
    /// At the end, a final sequence is kept that way, while the rest is merged
    /// with the data.
    HeadingAtx,
    /// Resolve heading (setext).
    ///
    /// Heading (setext) is parsed as an underline that is preceded by a
    /// paragraph, both will form the whole construct.
    HeadingSetext,
    /// Resolve list item.
    ///
    /// List items are parsed on their own.
    /// They are wrapped into ordered or unordered lists based on whether items
    /// with the same marker occur next to each other.
    ListItem,
    /// Resolve paragraphs.
    ///
    /// Paragraphs are parsed as single line paragraphs, as what remains if
    /// other flow constructs don’t match.
    /// But, when they occur next to each other, they need to be merged.
    Paragraph,
    /// Resolve data.
    ///
    /// Data is parsed as many small bits, due to many punctuation characters
    /// potentially starting something in particularly text content.
    /// It helps performance to merge them together if those markers did not
    /// match anything and hence they occur next to each other.
    Data,
    /// Resolve whitespace in `string`.
    String,
    /// Resolve whitespace in `text`.
    Text,
}

/// Call the corresponding resolver.
pub fn call(tokenizer: &mut Tokenizer, name: Name) {
    let func = match name {
        Name::Label => construct::label_end::resolve,
        Name::Attention => construct::attention::resolve,
        Name::HeadingAtx => construct::heading_atx::resolve,
        Name::HeadingSetext => construct::heading_setext::resolve,
        Name::ListItem => construct::list_item::resolve,
        Name::Paragraph => construct::paragraph::resolve,
        Name::Data => construct::partial_data::resolve,
        Name::String => construct::string::resolve,
        Name::Text => construct::text::resolve,
    };

    func(tokenizer);
}
