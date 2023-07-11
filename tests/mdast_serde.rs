use markdown::mdast::Node;

#[cfg_attr(feature = "serde", test)]
fn test_serde() {
    let markdown = "This is a **test**\n\n> quote content\n> continue";
    let tree = markdown::to_mdast(&markdown, &markdown::ParseOptions::default()).unwrap();
    let json = serde_json::to_string(&tree).unwrap();
    let tree2: Node = serde_json::from_str(&json).unwrap();
    assert!(tree == tree2);
}
