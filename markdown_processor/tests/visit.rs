use markdown::{mdast, to_mdast, ParseOptions};
use markdown_processor::visit::visit_mut;
use pretty_assertions::assert_eq;

#[test]
fn visits_in_preorder() {
    let mut tree = to_mdast("# a *b*\n\nc", &ParseOptions::default()).unwrap();
    let mut seen = vec![];

    visit_mut(&mut tree, &mut |node| {
        seen.push(match node {
            mdast::Node::Root(_) => "root",
            mdast::Node::Heading(_) => "heading",
            mdast::Node::Emphasis(_) => "emphasis",
            mdast::Node::Paragraph(_) => "paragraph",
            mdast::Node::Text(_) => "text",
            _ => "other",
        });
    });

    assert_eq!(
        seen,
        vec![
            "root",
            "heading",
            "text",
            "emphasis",
            "text",
            "paragraph",
            "text"
        ],
        "should visit parents before children, in document order"
    );
}

#[test]
fn visits_children_of_a_replacement() {
    let mut tree = to_mdast("*a*", &ParseOptions::default()).unwrap();
    let mut seen = vec![];

    visit_mut(&mut tree, &mut |node| {
        if let mdast::Node::Emphasis(_) = node {
            *node = mdast::Node::Strong(mdast::Strong {
                children: vec![mdast::Node::Text(mdast::Text {
                    value: "b".into(),
                    position: None,
                })],
                position: None,
            });
        } else if let mdast::Node::Text(text) = node {
            seen.push(text.value.clone());
        }
    });

    assert!(
        matches!(
            &tree.children().unwrap()[0].children().unwrap()[0],
            mdast::Node::Strong(_)
        ),
        "should replace the node in place"
    );
    assert_eq!(
        seen,
        vec!["b"],
        "should visit the replacement's children, not the original's"
    );
}
