use markdown::{to_mdast, ParseOptions};
use pretty_assertions::assert_eq;


#[test]
fn test_attention() {
    let input = include_str!("fixtures2/attention.md");
    let output = include_str!("fixtures2/attention.json");

    let mdast = to_mdast(input, &ParseOptions::default()).expect("could not parse fixture's input to mdast");

    let expected_tree: serde_json::Value = serde_json::from_str(output).expect("a fixture's tree contains invalid json");

    let actual_tree = serde_json::to_value(&mdast).expect("could not serialize mdast to json");

    assert_eq!(expected_tree, actual_tree);
}

#[test]
fn test_autolink() {
    let input = include_str!("fixtures2/autolink.md");
    let output = include_str!("fixtures2/autolink.json");

    let mdast = to_mdast(input, &ParseOptions::default()).expect("could not parse fixture's input to mdast");

    let expected_tree: serde_json::Value = serde_json::from_str(output).expect("a fixture's tree contains invalid json");

    let actual_tree = serde_json::to_value(&mdast).expect("could not serialize mdast to json");

    assert_eq!(expected_tree, actual_tree);
}

#[test]
fn test_blockquote() {
    let input = include_str!("fixtures2/blockquote.md");
    let output = include_str!("fixtures2/blockquote.json");

    let mdast = to_mdast(input, &ParseOptions::default()).expect("could not parse fixture's input to mdast");

    let expected_tree: serde_json::Value = serde_json::from_str(output).expect("a fixture's tree contains invalid json");

    let actual_tree = serde_json::to_value(&mdast).expect("could not serialize mdast to json");

    assert_eq!(expected_tree, actual_tree);
}

#[test]
fn test_character_escape() {
    let input = include_str!("fixtures2/character-escape.md");
    let output = include_str!("fixtures2/character-escape.json");

    let mdast = to_mdast(input, &ParseOptions::default()).expect("could not parse fixture's input to mdast");

    let expected_tree: serde_json::Value = serde_json::from_str(output).expect("a fixture's tree contains invalid json");

    let actual_tree = serde_json::to_value(&mdast).expect("could not serialize mdast to json");

    assert_eq!(expected_tree, actual_tree);
}

#[test]
fn test_character_reference() {
    let input = include_str!("fixtures2/character-reference.md");
    let output = include_str!("fixtures2/character-reference.json");

    let mdast = to_mdast(input, &ParseOptions::default()).expect("could not parse fixture's input to mdast");

    let expected_tree: serde_json::Value = serde_json::from_str(output).expect("a fixture's tree contains invalid json");

    let actual_tree = serde_json::to_value(&mdast).expect("could not serialize mdast to json");

    assert_eq!(expected_tree, actual_tree);
}

#[test]
fn test_character_references_everywhere() {
    let input = include_str!("fixtures2/character-references-everywhere.md");
    let output = include_str!("fixtures2/character-references-everywhere.json");

    let mdast = to_mdast(input, &ParseOptions::default()).expect("could not parse fixture's input to mdast");

    let expected_tree: serde_json::Value = serde_json::from_str(output).expect("a fixture's tree contains invalid json");

    let actual_tree = serde_json::to_value(&mdast).expect("could not serialize mdast to json");

    assert_eq!(expected_tree, actual_tree);
}

#[test]
fn test_code_fenced() {
    let input = include_str!("fixtures2/code-fenced.md");
    let output = include_str!("fixtures2/code-fenced.json");

    let mdast = to_mdast(input, &ParseOptions::default()).expect("could not parse fixture's input to mdast");

    let expected_tree: serde_json::Value = serde_json::from_str(output).expect("a fixture's tree contains invalid json");

    let actual_tree = serde_json::to_value(&mdast).expect("could not serialize mdast to json");

    assert_eq!(expected_tree, actual_tree);
}

#[test]
fn test_code_indented() {
    let input = include_str!("fixtures2/code-indented.md");
    let output = include_str!("fixtures2/code-indented.json");

    let mdast = to_mdast(input, &ParseOptions::default()).expect("could not parse fixture's input to mdast");

    let expected_tree: serde_json::Value = serde_json::from_str(output).expect("a fixture's tree contains invalid json");

    let actual_tree = serde_json::to_value(&mdast).expect("could not serialize mdast to json");

    assert_eq!(expected_tree, actual_tree);
}

#[test]
fn test_code_text() {
    let input = include_str!("fixtures2/code-text.md");
    let output = include_str!("fixtures2/code-text.json");

    let mdast = to_mdast(input, &ParseOptions::default()).expect("could not parse fixture's input to mdast");

    let expected_tree: serde_json::Value = serde_json::from_str(output).expect("a fixture's tree contains invalid json");

    let actual_tree = serde_json::to_value(&mdast).expect("could not serialize mdast to json");

    assert_eq!(expected_tree, actual_tree);
}

#[test]
fn test_definition() {
    let input = include_str!("fixtures2/definition.md");
    let output = include_str!("fixtures2/definition.json");

    let mdast = to_mdast(input, &ParseOptions::default()).expect("could not parse fixture's input to mdast");

    let expected_tree: serde_json::Value = serde_json::from_str(output).expect("a fixture's tree contains invalid json");

    let actual_tree = serde_json::to_value(&mdast).expect("could not serialize mdast to json");

    assert_eq!(expected_tree, actual_tree);
}

#[test]
fn test_empty() {
    let input = include_str!("fixtures2/empty.md");
    let output = include_str!("fixtures2/empty.json");

    let mdast = to_mdast(input, &ParseOptions::default()).expect("could not parse fixture's input to mdast");

    let expected_tree: serde_json::Value = serde_json::from_str(output).expect("a fixture's tree contains invalid json");

    let actual_tree = serde_json::to_value(&mdast).expect("could not serialize mdast to json");

    assert_eq!(expected_tree, actual_tree);
}

#[test]
fn test_hard_break_escape() {
    let input = include_str!("fixtures2/hard-break-escape.md");
    let output = include_str!("fixtures2/hard-break-escape.json");

    let mdast = to_mdast(input, &ParseOptions::default()).expect("could not parse fixture's input to mdast");

    let expected_tree: serde_json::Value = serde_json::from_str(output).expect("a fixture's tree contains invalid json");

    let actual_tree = serde_json::to_value(&mdast).expect("could not serialize mdast to json");

    assert_eq!(expected_tree, actual_tree);
}

#[test]
fn test_hard_break_prefix() {
    let input = include_str!("fixtures2/hard-break-prefix.md");
    let output = include_str!("fixtures2/hard-break-prefix.json");

    let mdast = to_mdast(input, &ParseOptions::default()).expect("could not parse fixture's input to mdast");

    let expected_tree: serde_json::Value = serde_json::from_str(output).expect("a fixture's tree contains invalid json");

    let actual_tree = serde_json::to_value(&mdast).expect("could not serialize mdast to json");

    assert_eq!(expected_tree, actual_tree);
}

#[test]
fn test_heading_atx() {
    let input = include_str!("fixtures2/heading-atx.md");
    let output = include_str!("fixtures2/heading-atx.json");

    let mdast = to_mdast(input, &ParseOptions::default()).expect("could not parse fixture's input to mdast");

    let expected_tree: serde_json::Value = serde_json::from_str(output).expect("a fixture's tree contains invalid json");

    let actual_tree = serde_json::to_value(&mdast).expect("could not serialize mdast to json");

    assert_eq!(expected_tree, actual_tree);
}

#[test]
fn test_heading_setext() {
    let input = include_str!("fixtures2/heading-setext.md");
    let output = include_str!("fixtures2/heading-setext.json");

    let mdast = to_mdast(input, &ParseOptions::default()).expect("could not parse fixture's input to mdast");

    let expected_tree: serde_json::Value = serde_json::from_str(output).expect("a fixture's tree contains invalid json");

    let actual_tree = serde_json::to_value(&mdast).expect("could not serialize mdast to json");

    assert_eq!(expected_tree, actual_tree);
}

#[test]
fn test_html_flow() {
    let input = include_str!("fixtures2/html-flow.md");
    let output = include_str!("fixtures2/html-flow.json");

    let mdast = to_mdast(input, &ParseOptions::default()).expect("could not parse fixture's input to mdast");

    let expected_tree: serde_json::Value = serde_json::from_str(output).expect("a fixture's tree contains invalid json");

    let actual_tree = serde_json::to_value(&mdast).expect("could not serialize mdast to json");

    assert_eq!(expected_tree, actual_tree);
}

#[test]
fn test_html_text() {
    let input = include_str!("fixtures2/html-text.md");
    let output = include_str!("fixtures2/html-text.json");

    let mdast = to_mdast(input, &ParseOptions::default()).expect("could not parse fixture's input to mdast");

    let expected_tree: serde_json::Value = serde_json::from_str(output).expect("a fixture's tree contains invalid json");

    let actual_tree = serde_json::to_value(&mdast).expect("could not serialize mdast to json");

    assert_eq!(expected_tree, actual_tree);
}

#[test]
fn test_image_reference() {
    let input = include_str!("fixtures2/image-reference.md");
    let output = include_str!("fixtures2/image-reference.json");

    let mdast = to_mdast(input, &ParseOptions::default()).expect("could not parse fixture's input to mdast");

    let expected_tree: serde_json::Value = serde_json::from_str(output).expect("a fixture's tree contains invalid json");

    let actual_tree = serde_json::to_value(&mdast).expect("could not serialize mdast to json");

    assert_eq!(expected_tree, actual_tree);
}

#[test]
fn test_image_resource_eol() {
    let input = include_str!("fixtures2/image-resource-eol.md");
    let output = include_str!("fixtures2/image-resource-eol.json");

    let mdast = to_mdast(input, &ParseOptions::default()).expect("could not parse fixture's input to mdast");

    let expected_tree: serde_json::Value = serde_json::from_str(output).expect("a fixture's tree contains invalid json");

    let actual_tree = serde_json::to_value(&mdast).expect("could not serialize mdast to json");

    assert_eq!(expected_tree, actual_tree);
}

#[test]
fn test_image_resource() {
    let input = include_str!("fixtures2/image-resource.md");
    let output = include_str!("fixtures2/image-resource.json");

    let mdast = to_mdast(input, &ParseOptions::default()).expect("could not parse fixture's input to mdast");

    let expected_tree: serde_json::Value = serde_json::from_str(output).expect("a fixture's tree contains invalid json");

    let actual_tree = serde_json::to_value(&mdast).expect("could not serialize mdast to json");

    assert_eq!(expected_tree, actual_tree);
}

#[test]
fn test_link_reference_with_phrasing() {
    let input = include_str!("fixtures2/link-reference-with-phrasing.md");
    let output = include_str!("fixtures2/link-reference-with-phrasing.json");

    let mdast = to_mdast(input, &ParseOptions::default()).expect("could not parse fixture's input to mdast");

    let expected_tree: serde_json::Value = serde_json::from_str(output).expect("a fixture's tree contains invalid json");

    let actual_tree = serde_json::to_value(&mdast).expect("could not serialize mdast to json");

    assert_eq!(expected_tree, actual_tree);
}

#[test]
fn test_link_reference() {
    let input = include_str!("fixtures2/link-reference.md");
    let output = include_str!("fixtures2/link-reference.json");

    let mdast = to_mdast(input, &ParseOptions::default()).expect("could not parse fixture's input to mdast");

    let expected_tree: serde_json::Value = serde_json::from_str(output).expect("a fixture's tree contains invalid json");

    let actual_tree = serde_json::to_value(&mdast).expect("could not serialize mdast to json");

    assert_eq!(expected_tree, actual_tree);
}

#[test]
fn test_link_resource_eol() {
    let input = include_str!("fixtures2/link-resource-eol.md");
    let output = include_str!("fixtures2/link-resource-eol.json");

    let mdast = to_mdast(input, &ParseOptions::default()).expect("could not parse fixture's input to mdast");

    let expected_tree: serde_json::Value = serde_json::from_str(output).expect("a fixture's tree contains invalid json");

    let actual_tree = serde_json::to_value(&mdast).expect("could not serialize mdast to json");

    assert_eq!(expected_tree, actual_tree);
}

#[test]
fn test_link_resource() {
    let input = include_str!("fixtures2/link-resource.md");
    let output = include_str!("fixtures2/link-resource.json");

    let mdast = to_mdast(input, &ParseOptions::default()).expect("could not parse fixture's input to mdast");

    let expected_tree: serde_json::Value = serde_json::from_str(output).expect("a fixture's tree contains invalid json");

    let actual_tree = serde_json::to_value(&mdast).expect("could not serialize mdast to json");

    assert_eq!(expected_tree, actual_tree);
}

#[test]
fn test_list() {
    let input = include_str!("fixtures2/list.md");
    let output = include_str!("fixtures2/list.json");

    let mdast = to_mdast(input, &ParseOptions::default()).expect("could not parse fixture's input to mdast");

    let expected_tree: serde_json::Value = serde_json::from_str(output).expect("a fixture's tree contains invalid json");

    let actual_tree = serde_json::to_value(&mdast).expect("could not serialize mdast to json");

    assert_eq!(expected_tree, actual_tree);
}

#[test]
fn test_paragraph() {
    let input = include_str!("fixtures2/paragraph.md");
    let output = include_str!("fixtures2/paragraph.json");

    let mdast = to_mdast(input, &ParseOptions::default()).expect("could not parse fixture's input to mdast");

    let expected_tree: serde_json::Value = serde_json::from_str(output).expect("a fixture's tree contains invalid json");

    let actual_tree = serde_json::to_value(&mdast).expect("could not serialize mdast to json");

    assert_eq!(expected_tree, actual_tree);
}

#[test]
fn test_thematic_break() {
    let input = include_str!("fixtures2/thematic-break.md");
    let output = include_str!("fixtures2/thematic-break.json");

    let mdast = to_mdast(input, &ParseOptions::default()).expect("could not parse fixture's input to mdast");

    let expected_tree: serde_json::Value = serde_json::from_str(output).expect("a fixture's tree contains invalid json");

    let actual_tree = serde_json::to_value(&mdast).expect("could not serialize mdast to json");

    assert_eq!(expected_tree, actual_tree);
}

#[test]
fn test_simple() {
    let input = "y’r";

    let mdast = to_mdast(input, &ParseOptions::default()).unwrap();
}
