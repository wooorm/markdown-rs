//! JS equivalent: https://github.com/syntax-tree/mdast-util-mdxjs-esm/blob/main/lib/index.js#L79

use super::Handle;
use crate::state::{Info, State};
use markdown::{
    mdast::{MdxjsEsm, Node},
    message::Message,
};

impl Handle for MdxjsEsm {
    fn handle(
        &self,
        _state: &mut State,
        _info: &Info,
        _parent: Option<&Node>,
        _node: &Node,
    ) -> Result<alloc::string::String, Message> {
        Ok(self.value.clone())
    }
}
