use markdown::mdast;
use markdown_processor::{visit::visit_mut, Processor};
use pretty_assertions::assert_eq;
use std::borrow::Cow;

/// Plugin that turns emphasis into a custom `shout` node.
fn emphasis_to_shout(processor: &mut Processor) {
    processor.add_mdast_transform(|tree| {
        visit_mut(tree, &mut |node| {
            if let mdast::Node::Emphasis(emphasis) = node {
                *node = mdast::Node::Custom(mdast::Custom {
                    name: "shout".into(),
                    fields: vec![("level".into(), "2".into())].into_iter().collect(),
                    children: emphasis.children.clone(),
                    ..mdast::Custom::default()
                });
            }
        });
        Ok(())
    });
}

fn element(tag_name: impl Into<Cow<'static, str>>, children: Vec<hast::Node>) -> hast::Node {
    hast::Node::Element(hast::Element {
        tag_name: tag_name.into(),
        properties: vec![],
        children,
        position: None,
    })
}

#[test]
fn handler_turns_custom_node_into_hast() {
    let processor =
        Processor::new()
            .plugin(emphasis_to_shout)
            .plugin(|processor: &mut Processor| {
                processor.add_hast_handler("shout", |node, children| {
                    let level = node.fields.get("level").unwrap();
                    vec![element(["h", level].concat(), children)]
                });
            });

    assert_eq!(
        processor.process("a *b*").unwrap(),
        "<p>a <h2>b</h2></p>",
        "should pass the node and its converted children to the handler"
    );
}

#[test]
fn later_handler_replaces_earlier() {
    let processor =
        Processor::new()
            .plugin(emphasis_to_shout)
            .plugin(|processor: &mut Processor| {
                processor.add_hast_handler("shout", |_, children| vec![element("b", children)]);
                processor.add_hast_handler("shout", |_, children| vec![element("i", children)]);
            });

    assert_eq!(
        processor.process("*a*").unwrap(),
        "<p><i>a</i></p>",
        "should use the last handler added for a name"
    );
}

#[test]
fn unhandled_custom_parent_becomes_div() {
    assert_eq!(
        Processor::new()
            .plugin(emphasis_to_shout)
            .process("*a*")
            .unwrap(),
        "<p><div>a</div></p>",
        "should fall back to a `div` of the children, like mdast-util-to-hast"
    );
}

#[test]
fn unhandled_custom_literal_becomes_text() {
    let processor = Processor::new().plugin(|processor: &mut Processor| {
        processor.add_mdast_transform(|tree| {
            if let Some(children) = tree.children_mut() {
                children.push(mdast::Node::Custom(mdast::Custom {
                    name: "x".into(),
                    value: Some("<b>".into()),
                    ..mdast::Custom::default()
                }));
            }
            Ok(())
        });
    });

    assert_eq!(
        processor.process("").unwrap(),
        "&lt;b&gt;",
        "should fall back to text of the value"
    );
}
