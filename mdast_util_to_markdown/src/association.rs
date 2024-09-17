use alloc::string::String;
use markdown::mdast::Definition;

pub trait Association {
    fn identifier(&self) -> &String;
    fn label(&self) -> &Option<String>;
}

impl Association for Definition {
    fn identifier(&self) -> &String {
        &self.identifier
    }

    fn label(&self) -> &Option<String> {
        &self.label
    }
}
