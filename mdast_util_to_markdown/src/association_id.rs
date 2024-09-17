use alloc::string::String;
use markdown::mdast::Definition;

pub trait AssociationId {
    fn identifier(&self) -> &String;
    fn label(&self) -> &Option<String>;
}

impl AssociationId for Definition {
    fn identifier(&self) -> &String {
        &self.identifier
    }

    fn label(&self) -> &Option<String> {
        &self.label
    }
}
