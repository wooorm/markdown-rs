use crate::{
    construct_name::ConstructName,
    r#unsafe::{Construct, Unsafe},
};

pub fn pattern_in_scope(stack: &[ConstructName], pattern: &Unsafe) -> bool {
    list_in_scope(stack, &pattern.in_construct, true)
        && !list_in_scope(stack, &pattern.not_in_construct, false)
}

fn list_in_scope(stack: &[ConstructName], list: &Option<Construct>, none: bool) -> bool {
    let Some(list) = list else {
        return none;
    };
    match list {
        Construct::Single(construct_name) => {
            if stack.contains(construct_name) {
                return true;
            }

            false
        }
        Construct::List(constructs_names) => {
            if constructs_names.is_empty() {
                return none;
            }

            for construct_name in constructs_names {
                if stack.contains(construct_name) {
                    return true;
                }
            }

            false
        }
    }
}
