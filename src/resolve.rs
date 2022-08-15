use crate::construct;
use crate::tokenizer::Tokenizer;

/// Names of functions that resolve.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Name {
    Label,
    Attention,
    HeadingAtx,
    HeadingSetext,
    List,
    Paragraph,
    Data,
    String,
    Text,
}

/// Call the corresponding resolver.
pub fn call(tokenizer: &mut Tokenizer, name: Name) {
    let func = match name {
        Name::Label => construct::label_end::resolve,
        Name::Attention => construct::attention::resolve,
        Name::HeadingAtx => construct::heading_atx::resolve,
        Name::HeadingSetext => construct::heading_setext::resolve,
        Name::List => construct::list_item::resolve,
        Name::Paragraph => construct::paragraph::resolve,
        Name::Data => construct::partial_data::resolve,
        Name::String => construct::string::resolve,
        Name::Text => construct::text::resolve,
    };

    func(tokenizer);
}
