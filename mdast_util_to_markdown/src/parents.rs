use alloc::vec::Vec;
use markdown::mdast::{Emphasis, List, Node, Paragraph, Root, Strong};

pub trait Parent {
    fn children(&self) -> &Vec<Node>;

    fn spreadable(&self) -> Option<bool> {
        None
    }
}

impl Parent for List {
    fn children(&self) -> &Vec<Node> {
        &self.children
    }

    fn spreadable(&self) -> Option<bool> {
        Some(self.spread)
    }
}

macro_rules! impl_Parent {
    (for $($t:ty),+) => {
        $(impl Parent for $t {
            fn children(&self) -> &Vec<Node> {
                &self.children
            }
        })*
    }
}

impl_Parent!(for Root, Paragraph, Strong, Emphasis);
