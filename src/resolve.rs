use crate::construct;
use crate::content;
use crate::tokenizer::Tokenizer;

/// Names of functions that resolve.
#[derive(Debug, Clone, Copy, PartialEq)]
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
        Name::List => construct::list::resolve,
        Name::Paragraph => construct::paragraph::resolve,
        Name::Data => construct::partial_data::resolve,
        Name::String => content::string::resolve,
        Name::Text => content::text::resolve,
    };

    func(tokenizer);
}
