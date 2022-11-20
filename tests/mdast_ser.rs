use markdown::{to_mdast, ParseOptions};
use pretty_assertions::assert_eq;


#[test]
fn test_amps_and_angles_encoding() {
    let input = include_str!("./fixtures/input/amps-and-angles-encoding.text");
    let output = include_str!("./fixtures/tree/amps-and-angles-encoding.json");

    let mdast = to_mdast(input, &ParseOptions::default()).expect("could not parse fixture's input to mdast");

    let expected_tree: serde_json::Value = serde_json::from_str(output).expect("a fixture's tree contains invalid json");

    let actual_tree = serde_json::to_value(&mdast).expect("could not serialize mdast to json");

    assert_eq!(expected_tree, actual_tree);
}

#[test]
fn test_auto_link_invalid() {
    let input = include_str!("./fixtures/input/auto-link-invalid.text");
    let output = include_str!("./fixtures/tree/auto-link-invalid.json");

    let mdast = to_mdast(input, &ParseOptions::default()).expect("could not parse fixture's input to mdast");

    let expected_tree: serde_json::Value = serde_json::from_str(output).expect("a fixture's tree contains invalid json");

    let actual_tree = serde_json::to_value(&mdast).expect("could not serialize mdast to json");

    assert_eq!(expected_tree, actual_tree);
}

#[test]
fn test_auto_link_lines() {
    let input = include_str!("./fixtures/input/auto-link-lines.text");
    let output = include_str!("./fixtures/tree/auto-link-lines.json");

    let mdast = to_mdast(input, &ParseOptions::default()).expect("could not parse fixture's input to mdast");

    let expected_tree: serde_json::Value = serde_json::from_str(output).expect("a fixture's tree contains invalid json");

    let actual_tree = serde_json::to_value(&mdast).expect("could not serialize mdast to json");

    assert_eq!(expected_tree, actual_tree);
}

#[test]
fn test_auto_link_syntax() {
    let input = include_str!("./fixtures/input/auto-link-syntax.text");
    let output = include_str!("./fixtures/tree/auto-link-syntax.json");

    let mdast = to_mdast(input, &ParseOptions::default()).expect("could not parse fixture's input to mdast");

    let expected_tree: serde_json::Value = serde_json::from_str(output).expect("a fixture's tree contains invalid json");

    let actual_tree = serde_json::to_value(&mdast).expect("could not serialize mdast to json");

    assert_eq!(expected_tree, actual_tree);
}

#[test]
fn test_auto_link_url_invalid() {
    let input = include_str!("./fixtures/input/auto-link-url-invalid.text");
    let output = include_str!("./fixtures/tree/auto-link-url-invalid.json");

    let mdast = to_mdast(input, &ParseOptions::default()).expect("could not parse fixture's input to mdast");

    let expected_tree: serde_json::Value = serde_json::from_str(output).expect("a fixture's tree contains invalid json");

    let actual_tree = serde_json::to_value(&mdast).expect("could not serialize mdast to json");

    assert_eq!(expected_tree, actual_tree);
}

#[test]
fn test_auto_link_url() {
    let input = include_str!("./fixtures/input/auto-link-url.text");
    let output = include_str!("./fixtures/tree/auto-link-url.json");

    let mdast = to_mdast(input, &ParseOptions::default()).expect("could not parse fixture's input to mdast");

    let expected_tree: serde_json::Value = serde_json::from_str(output).expect("a fixture's tree contains invalid json");

    let actual_tree = serde_json::to_value(&mdast).expect("could not serialize mdast to json");

    assert_eq!(expected_tree, actual_tree);
}

#[test]
fn test_auto_link() {
    let input = include_str!("./fixtures/input/auto-link.text");
    let output = include_str!("./fixtures/tree/auto-link.json");

    let mdast = to_mdast(input, &ParseOptions::default()).expect("could not parse fixture's input to mdast");

    let expected_tree: serde_json::Value = serde_json::from_str(output).expect("a fixture's tree contains invalid json");

    let actual_tree = serde_json::to_value(&mdast).expect("could not serialize mdast to json");

    assert_eq!(expected_tree, actual_tree);
}

#[test]
fn test_backslash_escapes() {
    let input = include_str!("./fixtures/input/backslash-escapes.text");
    let output = include_str!("./fixtures/tree/backslash-escapes.json");

    let mdast = to_mdast(input, &ParseOptions::default()).expect("could not parse fixture's input to mdast");

    let expected_tree: serde_json::Value = serde_json::from_str(output).expect("a fixture's tree contains invalid json");

    let actual_tree = serde_json::to_value(&mdast).expect("could not serialize mdast to json");

    assert_eq!(expected_tree, actual_tree);
}

#[test]
fn test_block_elements() {
    let input = include_str!("./fixtures/input/block-elements.text");
    let output = include_str!("./fixtures/tree/block-elements.json");

    let mdast = to_mdast(input, &ParseOptions::default()).expect("could not parse fixture's input to mdast");

    let expected_tree: serde_json::Value = serde_json::from_str(output).expect("a fixture's tree contains invalid json");

    let actual_tree = serde_json::to_value(&mdast).expect("could not serialize mdast to json");

    assert_eq!(expected_tree, actual_tree);
}

#[test]
fn test_blockquote_indented() {
    let input = include_str!("./fixtures/input/blockquote-indented.text");
    let output = include_str!("./fixtures/tree/blockquote-indented.json");

    let mdast = to_mdast(input, &ParseOptions::default()).expect("could not parse fixture's input to mdast");

    let expected_tree: serde_json::Value = serde_json::from_str(output).expect("a fixture's tree contains invalid json");

    let actual_tree = serde_json::to_value(&mdast).expect("could not serialize mdast to json");

    assert_eq!(expected_tree, actual_tree);
}

#[test]
fn test_blockquote_lazy_code() {
    let input = include_str!("./fixtures/input/blockquote-lazy-code.text");
    let output = include_str!("./fixtures/tree/blockquote-lazy-code.json");

    let mdast = to_mdast(input, &ParseOptions::default()).expect("could not parse fixture's input to mdast");

    let expected_tree: serde_json::Value = serde_json::from_str(output).expect("a fixture's tree contains invalid json");

    let actual_tree = serde_json::to_value(&mdast).expect("could not serialize mdast to json");

    assert_eq!(expected_tree, actual_tree);
}

#[test]
fn test_blockquote_lazy_fence() {
    let input = include_str!("./fixtures/input/blockquote-lazy-fence.text");
    let output = include_str!("./fixtures/tree/blockquote-lazy-fence.json");

    let mdast = to_mdast(input, &ParseOptions::default()).expect("could not parse fixture's input to mdast");

    let expected_tree: serde_json::Value = serde_json::from_str(output).expect("a fixture's tree contains invalid json");

    let actual_tree = serde_json::to_value(&mdast).expect("could not serialize mdast to json");

    assert_eq!(expected_tree, actual_tree);
}

#[test]
fn test_blockquote_lazy_list() {
    let input = include_str!("./fixtures/input/blockquote-lazy-list.text");
    let output = include_str!("./fixtures/tree/blockquote-lazy-list.json");

    let mdast = to_mdast(input, &ParseOptions::default()).expect("could not parse fixture's input to mdast");

    let expected_tree: serde_json::Value = serde_json::from_str(output).expect("a fixture's tree contains invalid json");

    let actual_tree = serde_json::to_value(&mdast).expect("could not serialize mdast to json");

    assert_eq!(expected_tree, actual_tree);
}

#[test]
fn test_blockquote_lazy_rule() {
    let input = include_str!("./fixtures/input/blockquote-lazy-rule.text");
    let output = include_str!("./fixtures/tree/blockquote-lazy-rule.json");

    let mdast = to_mdast(input, &ParseOptions::default()).expect("could not parse fixture's input to mdast");

    let expected_tree: serde_json::Value = serde_json::from_str(output).expect("a fixture's tree contains invalid json");

    let actual_tree = serde_json::to_value(&mdast).expect("could not serialize mdast to json");

    assert_eq!(expected_tree, actual_tree);
}

#[test]
fn test_blockquote_list_item() {
    let input = include_str!("./fixtures/input/blockquote-list-item.text");
    let output = include_str!("./fixtures/tree/blockquote-list-item.json");

    let mdast = to_mdast(input, &ParseOptions::default()).expect("could not parse fixture's input to mdast");

    let expected_tree: serde_json::Value = serde_json::from_str(output).expect("a fixture's tree contains invalid json");

    let actual_tree = serde_json::to_value(&mdast).expect("could not serialize mdast to json");

    assert_eq!(expected_tree, actual_tree);
}

#[test]
fn test_blockquotes_empty_lines_output() {
    let input = include_str!("./fixtures/input/blockquotes-empty-lines.output.text");
    let output = include_str!("./fixtures/tree/blockquotes-empty-lines.output.json");

    let mdast = to_mdast(input, &ParseOptions::default()).expect("could not parse fixture's input to mdast");

    let expected_tree: serde_json::Value = serde_json::from_str(output).expect("a fixture's tree contains invalid json");

    let actual_tree = serde_json::to_value(&mdast).expect("could not serialize mdast to json");

    assert_eq!(expected_tree, actual_tree);
}

#[test]
fn test_blockquotes_with_code_blocks() {
    let input = include_str!("./fixtures/input/blockquotes-with-code-blocks.text");
    let output = include_str!("./fixtures/tree/blockquotes-with-code-blocks.json");

    let mdast = to_mdast(input, &ParseOptions::default()).expect("could not parse fixture's input to mdast");

    let expected_tree: serde_json::Value = serde_json::from_str(output).expect("a fixture's tree contains invalid json");

    let actual_tree = serde_json::to_value(&mdast).expect("could not serialize mdast to json");

    assert_eq!(expected_tree, actual_tree);
}

#[test]
fn test_blockquotes() {
    let input = include_str!("./fixtures/input/blockquotes.text");
    let output = include_str!("./fixtures/tree/blockquotes.json");

    let mdast = to_mdast(input, &ParseOptions::default()).expect("could not parse fixture's input to mdast");

    let expected_tree: serde_json::Value = serde_json::from_str(output).expect("a fixture's tree contains invalid json");

    let actual_tree = serde_json::to_value(&mdast).expect("could not serialize mdast to json");

    assert_eq!(expected_tree, actual_tree);
}

#[test]
fn test_bom() {
    let input = include_str!("./fixtures/input/bom.text");
    let output = include_str!("./fixtures/tree/bom.json");

    let mdast = to_mdast(input, &ParseOptions::default()).expect("could not parse fixture's input to mdast");

    let expected_tree: serde_json::Value = serde_json::from_str(output).expect("a fixture's tree contains invalid json");

    let actual_tree = serde_json::to_value(&mdast).expect("could not serialize mdast to json");

    assert_eq!(expected_tree, actual_tree);
}

#[test]
fn test_breaks_hard() {
    let input = include_str!("./fixtures/input/breaks-hard.text");
    let output = include_str!("./fixtures/tree/breaks-hard.json");

    let mdast = to_mdast(input, &ParseOptions::default()).expect("could not parse fixture's input to mdast");

    let expected_tree: serde_json::Value = serde_json::from_str(output).expect("a fixture's tree contains invalid json");

    let actual_tree = serde_json::to_value(&mdast).expect("could not serialize mdast to json");

    assert_eq!(expected_tree, actual_tree);
}

#[test]
fn test_case_insensitive_refs() {
    let input = include_str!("./fixtures/input/case-insensitive-refs.text");
    let output = include_str!("./fixtures/tree/case-insensitive-refs.json");

    let mdast = to_mdast(input, &ParseOptions::default()).expect("could not parse fixture's input to mdast");

    let expected_tree: serde_json::Value = serde_json::from_str(output).expect("a fixture's tree contains invalid json");

    let actual_tree = serde_json::to_value(&mdast).expect("could not serialize mdast to json");

    assert_eq!(expected_tree, actual_tree);
}

#[test]
fn test_code_block_indentation_nooutput() {
    let input = include_str!("./fixtures/input/code-block-indentation.nooutput.text");
    let output = include_str!("./fixtures/tree/code-block-indentation.nooutput.json");

    let mdast = to_mdast(input, &ParseOptions::default()).expect("could not parse fixture's input to mdast");

    let expected_tree: serde_json::Value = serde_json::from_str(output).expect("a fixture's tree contains invalid json");

    let actual_tree = serde_json::to_value(&mdast).expect("could not serialize mdast to json");

    assert_eq!(expected_tree, actual_tree);
}

#[test]
fn test_code_block_nesting_bug() {
    let input = include_str!("./fixtures/input/code-block-nesting-bug.text");
    let output = include_str!("./fixtures/tree/code-block-nesting-bug.json");

    let mdast = to_mdast(input, &ParseOptions::default()).expect("could not parse fixture's input to mdast");

    let expected_tree: serde_json::Value = serde_json::from_str(output).expect("a fixture's tree contains invalid json");

    let actual_tree = serde_json::to_value(&mdast).expect("could not serialize mdast to json");

    assert_eq!(expected_tree, actual_tree);
}

#[test]
fn test_code_block_output_fence() {
    let input = include_str!("./fixtures/input/code-block.output.fence=`.text");
    let output = include_str!("./fixtures/tree/code-block.output.fence=`.json");

    let mdast = to_mdast(input, &ParseOptions::default()).expect("could not parse fixture's input to mdast");

    let expected_tree: serde_json::Value = serde_json::from_str(output).expect("a fixture's tree contains invalid json");

    let actual_tree = serde_json::to_value(&mdast).expect("could not serialize mdast to json");

    assert_eq!(expected_tree, actual_tree);
}

#[test]
fn test_code_block_output_fence_2() {
    let input = include_str!("./fixtures/input/code-block.output.fence=~.text");
    let output = include_str!("./fixtures/tree/code-block.output.fence=~.json");

    let mdast = to_mdast(input, &ParseOptions::default()).expect("could not parse fixture's input to mdast");

    let expected_tree: serde_json::Value = serde_json::from_str(output).expect("a fixture's tree contains invalid json");

    let actual_tree = serde_json::to_value(&mdast).expect("could not serialize mdast to json");

    assert_eq!(expected_tree, actual_tree);
}

#[test]
fn test_code_blocks_output_fences() {
    let input = include_str!("./fixtures/input/code-blocks.output.fences.text");
    let output = include_str!("./fixtures/tree/code-blocks.output.fences.json");

    let mdast = to_mdast(input, &ParseOptions::default()).expect("could not parse fixture's input to mdast");

    let expected_tree: serde_json::Value = serde_json::from_str(output).expect("a fixture's tree contains invalid json");

    let actual_tree = serde_json::to_value(&mdast).expect("could not serialize mdast to json");

    assert_eq!(expected_tree, actual_tree);
}

#[test]
fn test_code_blocks_output() {
    let input = include_str!("./fixtures/input/code-blocks.output.text");
    let output = include_str!("./fixtures/tree/code-blocks.output.json");

    let mdast = to_mdast(input, &ParseOptions::default()).expect("could not parse fixture's input to mdast");

    let expected_tree: serde_json::Value = serde_json::from_str(output).expect("a fixture's tree contains invalid json");

    let actual_tree = serde_json::to_value(&mdast).expect("could not serialize mdast to json");

    assert_eq!(expected_tree, actual_tree);
}

#[test]
fn test_code_blocks() {
    let input = include_str!("./fixtures/input/code-blocks.text");
    let output = include_str!("./fixtures/tree/code-blocks.json");

    let mdast = to_mdast(input, &ParseOptions::default()).expect("could not parse fixture's input to mdast");

    let expected_tree: serde_json::Value = serde_json::from_str(output).expect("a fixture's tree contains invalid json");

    let actual_tree = serde_json::to_value(&mdast).expect("could not serialize mdast to json");

    assert_eq!(expected_tree, actual_tree);
}

#[test]
fn test_code_inline_whitespace() {
    let input = include_str!("./fixtures/input/code-inline-whitespace.text");
    let output = include_str!("./fixtures/tree/code-inline-whitespace.json");

    let mdast = to_mdast(input, &ParseOptions::default()).expect("could not parse fixture's input to mdast");

    let expected_tree: serde_json::Value = serde_json::from_str(output).expect("a fixture's tree contains invalid json");

    let actual_tree = serde_json::to_value(&mdast).expect("could not serialize mdast to json");

    assert_eq!(expected_tree, actual_tree);
}

#[test]
fn test_code_spans() {
    let input = include_str!("./fixtures/input/code-spans.text");
    let output = include_str!("./fixtures/tree/code-spans.json");

    let mdast = to_mdast(input, &ParseOptions::default()).expect("could not parse fixture's input to mdast");

    let expected_tree: serde_json::Value = serde_json::from_str(output).expect("a fixture's tree contains invalid json");

    let actual_tree = serde_json::to_value(&mdast).expect("could not serialize mdast to json");

    assert_eq!(expected_tree, actual_tree);
}

#[test]
fn test_def_blocks() {
    let input = include_str!("./fixtures/input/def-blocks.text");
    let output = include_str!("./fixtures/tree/def-blocks.json");

    let mdast = to_mdast(input, &ParseOptions::default()).expect("could not parse fixture's input to mdast");

    let expected_tree: serde_json::Value = serde_json::from_str(output).expect("a fixture's tree contains invalid json");

    let actual_tree = serde_json::to_value(&mdast).expect("could not serialize mdast to json");

    assert_eq!(expected_tree, actual_tree);
}

#[test]
fn test_definition_in_list_and_blockquote() {
    let input = include_str!("./fixtures/input/definition-in-list-and-blockquote.text");
    let output = include_str!("./fixtures/tree/definition-in-list-and-blockquote.json");

    let mdast = to_mdast(input, &ParseOptions::default()).expect("could not parse fixture's input to mdast");

    let expected_tree: serde_json::Value = serde_json::from_str(output).expect("a fixture's tree contains invalid json");

    let actual_tree = serde_json::to_value(&mdast).expect("could not serialize mdast to json");

    assert_eq!(expected_tree, actual_tree);
}

#[test]
fn test_definition_newline() {
    let input = include_str!("./fixtures/input/definition-newline.text");
    let output = include_str!("./fixtures/tree/definition-newline.json");

    let mdast = to_mdast(input, &ParseOptions::default()).expect("could not parse fixture's input to mdast");

    let expected_tree: serde_json::Value = serde_json::from_str(output).expect("a fixture's tree contains invalid json");

    let actual_tree = serde_json::to_value(&mdast).expect("could not serialize mdast to json");

    assert_eq!(expected_tree, actual_tree);
}

#[test]
fn test_definition_unclosed_attribute() {
    let input = include_str!("./fixtures/input/definition-unclosed-attribute.text");
    let output = include_str!("./fixtures/tree/definition-unclosed-attribute.json");

    let mdast = to_mdast(input, &ParseOptions::default()).expect("could not parse fixture's input to mdast");

    let expected_tree: serde_json::Value = serde_json::from_str(output).expect("a fixture's tree contains invalid json");

    let actual_tree = serde_json::to_value(&mdast).expect("could not serialize mdast to json");

    assert_eq!(expected_tree, actual_tree);
}

#[test]
fn test_definition_unclosed() {
    let input = include_str!("./fixtures/input/definition-unclosed.text");
    let output = include_str!("./fixtures/tree/definition-unclosed.json");

    let mdast = to_mdast(input, &ParseOptions::default()).expect("could not parse fixture's input to mdast");

    let expected_tree: serde_json::Value = serde_json::from_str(output).expect("a fixture's tree contains invalid json");

    let actual_tree = serde_json::to_value(&mdast).expect("could not serialize mdast to json");

    assert_eq!(expected_tree, actual_tree);
}

#[test]
fn test_definition_url_entities() {
    let input = include_str!("./fixtures/input/definition-url-entities.text");
    let output = include_str!("./fixtures/tree/definition-url-entities.json");

    let mdast = to_mdast(input, &ParseOptions::default()).expect("could not parse fixture's input to mdast");

    let expected_tree: serde_json::Value = serde_json::from_str(output).expect("a fixture's tree contains invalid json");

    let actual_tree = serde_json::to_value(&mdast).expect("could not serialize mdast to json");

    assert_eq!(expected_tree, actual_tree);
}

#[test]
fn test_deletion() {
    let input = include_str!("./fixtures/input/deletion.text");
    let output = include_str!("./fixtures/tree/deletion.json");

    let mdast = to_mdast(input, &ParseOptions::default()).expect("could not parse fixture's input to mdast");

    let expected_tree: serde_json::Value = serde_json::from_str(output).expect("a fixture's tree contains invalid json");

    let actual_tree = serde_json::to_value(&mdast).expect("could not serialize mdast to json");

    assert_eq!(expected_tree, actual_tree);
}

#[test]
fn test_double_link() {
    let input = include_str!("./fixtures/input/double-link.text");
    let output = include_str!("./fixtures/tree/double-link.json");

    let mdast = to_mdast(input, &ParseOptions::default()).expect("could not parse fixture's input to mdast");

    let expected_tree: serde_json::Value = serde_json::from_str(output).expect("a fixture's tree contains invalid json");

    let actual_tree = serde_json::to_value(&mdast).expect("could not serialize mdast to json");

    assert_eq!(expected_tree, actual_tree);
}

#[test]
fn test_emphasis_empty() {
    let input = include_str!("./fixtures/input/emphasis-empty.text");
    let output = include_str!("./fixtures/tree/emphasis-empty.json");

    let mdast = to_mdast(input, &ParseOptions::default()).expect("could not parse fixture's input to mdast");

    let expected_tree: serde_json::Value = serde_json::from_str(output).expect("a fixture's tree contains invalid json");

    let actual_tree = serde_json::to_value(&mdast).expect("could not serialize mdast to json");

    assert_eq!(expected_tree, actual_tree);
}

#[test]
fn test_emphasis_escaped_final_marker() {
    let input = include_str!("./fixtures/input/emphasis-escaped-final-marker.text");
    let output = include_str!("./fixtures/tree/emphasis-escaped-final-marker.json");

    let mdast = to_mdast(input, &ParseOptions::default()).expect("could not parse fixture's input to mdast");

    let expected_tree: serde_json::Value = serde_json::from_str(output).expect("a fixture's tree contains invalid json");

    let actual_tree = serde_json::to_value(&mdast).expect("could not serialize mdast to json");

    assert_eq!(expected_tree, actual_tree);
}

#[test]
fn test_emphasis_internal() {
    let input = include_str!("./fixtures/input/emphasis-internal.text");
    let output = include_str!("./fixtures/tree/emphasis-internal.json");

    let mdast = to_mdast(input, &ParseOptions::default()).expect("could not parse fixture's input to mdast");

    let expected_tree: serde_json::Value = serde_json::from_str(output).expect("a fixture's tree contains invalid json");

    let actual_tree = serde_json::to_value(&mdast).expect("could not serialize mdast to json");

    assert_eq!(expected_tree, actual_tree);
}

#[test]
fn test_emphasis_output_emphasis_asterisk_strong() {
    let input = include_str!("./fixtures/input/emphasis.output.emphasis=-asterisk-.strong=_.text");
    let output = include_str!("./fixtures/tree/emphasis.output.emphasis=-asterisk-.strong=_.json");

    let mdast = to_mdast(input, &ParseOptions::default()).expect("could not parse fixture's input to mdast");

    let expected_tree: serde_json::Value = serde_json::from_str(output).expect("a fixture's tree contains invalid json");

    let actual_tree = serde_json::to_value(&mdast).expect("could not serialize mdast to json");

    assert_eq!(expected_tree, actual_tree);
}

#[test]
fn test_emphasis_output_emphasis_strong_asterisk() {
    let input = include_str!("./fixtures/input/emphasis.output.emphasis=_.strong=-asterisk-.text");
    let output = include_str!("./fixtures/tree/emphasis.output.emphasis=_.strong=-asterisk-.json");

    let mdast = to_mdast(input, &ParseOptions::default()).expect("could not parse fixture's input to mdast");

    let expected_tree: serde_json::Value = serde_json::from_str(output).expect("a fixture's tree contains invalid json");

    let actual_tree = serde_json::to_value(&mdast).expect("could not serialize mdast to json");

    assert_eq!(expected_tree, actual_tree);
}

#[test]
fn test_empty() {
    let input = include_str!("./fixtures/input/empty.text");
    let output = include_str!("./fixtures/tree/empty.json");

    let mdast = to_mdast(input, &ParseOptions::default()).expect("could not parse fixture's input to mdast");

    let expected_tree: serde_json::Value = serde_json::from_str(output).expect("a fixture's tree contains invalid json");

    let actual_tree = serde_json::to_value(&mdast).expect("could not serialize mdast to json");

    assert_eq!(expected_tree, actual_tree);
}

#[test]
fn test_entities_advanced() {
    let input = include_str!("./fixtures/input/entities-advanced.text");
    let output = include_str!("./fixtures/tree/entities-advanced.json");

    let mdast = to_mdast(input, &ParseOptions::default()).expect("could not parse fixture's input to mdast");

    let expected_tree: serde_json::Value = serde_json::from_str(output).expect("a fixture's tree contains invalid json");

    let actual_tree = serde_json::to_value(&mdast).expect("could not serialize mdast to json");

    assert_eq!(expected_tree, actual_tree);
}

#[test]
fn test_entities() {
    let input = include_str!("./fixtures/input/entities.text");
    let output = include_str!("./fixtures/tree/entities.json");

    let mdast = to_mdast(input, &ParseOptions::default()).expect("could not parse fixture's input to mdast");

    let expected_tree: serde_json::Value = serde_json::from_str(output).expect("a fixture's tree contains invalid json");

    let actual_tree = serde_json::to_value(&mdast).expect("could not serialize mdast to json");

    assert_eq!(expected_tree, actual_tree);
}

#[test]
fn test_escaped_angles() {
    let input = include_str!("./fixtures/input/escaped-angles.text");
    let output = include_str!("./fixtures/tree/escaped-angles.json");

    let mdast = to_mdast(input, &ParseOptions::default()).expect("could not parse fixture's input to mdast");

    let expected_tree: serde_json::Value = serde_json::from_str(output).expect("a fixture's tree contains invalid json");

    let actual_tree = serde_json::to_value(&mdast).expect("could not serialize mdast to json");

    assert_eq!(expected_tree, actual_tree);
}

#[test]
fn test_fenced_code_empty() {
    let input = include_str!("./fixtures/input/fenced-code-empty.text");
    let output = include_str!("./fixtures/tree/fenced-code-empty.json");

    let mdast = to_mdast(input, &ParseOptions::default()).expect("could not parse fixture's input to mdast");

    let expected_tree: serde_json::Value = serde_json::from_str(output).expect("a fixture's tree contains invalid json");

    let actual_tree = serde_json::to_value(&mdast).expect("could not serialize mdast to json");

    assert_eq!(expected_tree, actual_tree);
}

#[test]
fn test_fenced_code_info_string() {
    let input = include_str!("./fixtures/input/fenced-code-info-string.text");
    let output = include_str!("./fixtures/tree/fenced-code-info-string.json");

    let mdast = to_mdast(input, &ParseOptions::default()).expect("could not parse fixture's input to mdast");

    let expected_tree: serde_json::Value = serde_json::from_str(output).expect("a fixture's tree contains invalid json");

    let actual_tree = serde_json::to_value(&mdast).expect("could not serialize mdast to json");

    assert_eq!(expected_tree, actual_tree);
}

#[test]
fn test_fenced_code_info_with_marker() {
    let input = include_str!("./fixtures/input/fenced-code-info-with-marker.text");
    let output = include_str!("./fixtures/tree/fenced-code-info-with-marker.json");

    let mdast = to_mdast(input, &ParseOptions::default()).expect("could not parse fixture's input to mdast");

    let expected_tree: serde_json::Value = serde_json::from_str(output).expect("a fixture's tree contains invalid json");

    let actual_tree = serde_json::to_value(&mdast).expect("could not serialize mdast to json");

    assert_eq!(expected_tree, actual_tree);
}

#[test]
fn test_fenced_code_initial_final_newlines() {
    let input = include_str!("./fixtures/input/fenced-code-initial-final-newlines.text");
    let output = include_str!("./fixtures/tree/fenced-code-initial-final-newlines.json");

    let mdast = to_mdast(input, &ParseOptions::default()).expect("could not parse fixture's input to mdast");

    let expected_tree: serde_json::Value = serde_json::from_str(output).expect("a fixture's tree contains invalid json");

    let actual_tree = serde_json::to_value(&mdast).expect("could not serialize mdast to json");

    assert_eq!(expected_tree, actual_tree);
}

#[test]
fn test_fenced_code_lang_unescape() {
    let input = include_str!("./fixtures/input/fenced-code-lang-unescape.text");
    let output = include_str!("./fixtures/tree/fenced-code-lang-unescape.json");

    let mdast = to_mdast(input, &ParseOptions::default()).expect("could not parse fixture's input to mdast");

    let expected_tree: serde_json::Value = serde_json::from_str(output).expect("a fixture's tree contains invalid json");

    let actual_tree = serde_json::to_value(&mdast).expect("could not serialize mdast to json");

    assert_eq!(expected_tree, actual_tree);
}

#[test]
fn test_fenced_code_trailing_characters_2_nooutput() {
    let input = include_str!("./fixtures/input/fenced-code-trailing-characters-2.nooutput.text");
    let output = include_str!("./fixtures/tree/fenced-code-trailing-characters-2.nooutput.json");

    let mdast = to_mdast(input, &ParseOptions::default()).expect("could not parse fixture's input to mdast");

    let expected_tree: serde_json::Value = serde_json::from_str(output).expect("a fixture's tree contains invalid json");

    let actual_tree = serde_json::to_value(&mdast).expect("could not serialize mdast to json");

    assert_eq!(expected_tree, actual_tree);
}

#[test]
fn test_fenced_code_trailing_characters_nooutput() {
    let input = include_str!("./fixtures/input/fenced-code-trailing-characters.nooutput.text");
    let output = include_str!("./fixtures/tree/fenced-code-trailing-characters.nooutput.json");

    let mdast = to_mdast(input, &ParseOptions::default()).expect("could not parse fixture's input to mdast");

    let expected_tree: serde_json::Value = serde_json::from_str(output).expect("a fixture's tree contains invalid json");

    let actual_tree = serde_json::to_value(&mdast).expect("could not serialize mdast to json");

    assert_eq!(expected_tree, actual_tree);
}

#[test]
fn test_fenced_code_white_space_after_flag() {
    let input = include_str!("./fixtures/input/fenced-code-white-space-after-flag.text");
    let output = include_str!("./fixtures/tree/fenced-code-white-space-after-flag.json");

    let mdast = to_mdast(input, &ParseOptions::default()).expect("could not parse fixture's input to mdast");

    let expected_tree: serde_json::Value = serde_json::from_str(output).expect("a fixture's tree contains invalid json");

    let actual_tree = serde_json::to_value(&mdast).expect("could not serialize mdast to json");

    assert_eq!(expected_tree, actual_tree);
}

#[test]
fn test_fenced_code() {
    let input = include_str!("./fixtures/input/fenced-code.text");
    let output = include_str!("./fixtures/tree/fenced-code.json");

    let mdast = to_mdast(input, &ParseOptions::default()).expect("could not parse fixture's input to mdast");

    let expected_tree: serde_json::Value = serde_json::from_str(output).expect("a fixture's tree contains invalid json");

    let actual_tree = serde_json::to_value(&mdast).expect("could not serialize mdast to json");

    assert_eq!(expected_tree, actual_tree);
}

#[test]
fn test_gh_218() {
    let input = include_str!("./fixtures/input/gh-218.text");
    let output = include_str!("./fixtures/tree/gh-218.json");

    let mdast = to_mdast(input, &ParseOptions::default()).expect("could not parse fixture's input to mdast");

    let expected_tree: serde_json::Value = serde_json::from_str(output).expect("a fixture's tree contains invalid json");

    let actual_tree = serde_json::to_value(&mdast).expect("could not serialize mdast to json");

    assert_eq!(expected_tree, actual_tree);
}

#[test]
fn test_gh_324() {
    let input = include_str!("./fixtures/input/gh-324.text");
    let output = include_str!("./fixtures/tree/gh-324.json");

    let mdast = to_mdast(input, &ParseOptions::default()).expect("could not parse fixture's input to mdast");

    let expected_tree: serde_json::Value = serde_json::from_str(output).expect("a fixture's tree contains invalid json");

    let actual_tree = serde_json::to_value(&mdast).expect("could not serialize mdast to json");

    assert_eq!(expected_tree, actual_tree);
}

#[test]
fn test_gh_398() {
    let input = include_str!("./fixtures/input/gh-398.text");
    let output = include_str!("./fixtures/tree/gh-398.json");

    let mdast = to_mdast(input, &ParseOptions::default()).expect("could not parse fixture's input to mdast");

    let expected_tree: serde_json::Value = serde_json::from_str(output).expect("a fixture's tree contains invalid json");

    let actual_tree = serde_json::to_value(&mdast).expect("could not serialize mdast to json");

    assert_eq!(expected_tree, actual_tree);
}

#[test]
fn test_gh_402() {
    let input = include_str!("./fixtures/input/gh-402.text");
    let output = include_str!("./fixtures/tree/gh-402.json");

    let mdast = to_mdast(input, &ParseOptions::default()).expect("could not parse fixture's input to mdast");

    let expected_tree: serde_json::Value = serde_json::from_str(output).expect("a fixture's tree contains invalid json");

    let actual_tree = serde_json::to_value(&mdast).expect("could not serialize mdast to json");

    assert_eq!(expected_tree, actual_tree);
}

#[test]
fn test_gh_407() {
    let input = include_str!("./fixtures/input/gh-407.text");
    let output = include_str!("./fixtures/tree/gh-407.json");

    let mdast = to_mdast(input, &ParseOptions::default()).expect("could not parse fixture's input to mdast");

    let expected_tree: serde_json::Value = serde_json::from_str(output).expect("a fixture's tree contains invalid json");

    let actual_tree = serde_json::to_value(&mdast).expect("could not serialize mdast to json");

    assert_eq!(expected_tree, actual_tree);
}

#[test]
fn test_gh_450() {
    let input = include_str!("./fixtures/input/gh-450.text");
    let output = include_str!("./fixtures/tree/gh-450.json");

    let mdast = to_mdast(input, &ParseOptions::default()).expect("could not parse fixture's input to mdast");

    let expected_tree: serde_json::Value = serde_json::from_str(output).expect("a fixture's tree contains invalid json");

    let actual_tree = serde_json::to_value(&mdast).expect("could not serialize mdast to json");

    assert_eq!(expected_tree, actual_tree);
}

#[test]
fn test_gh_459() {
    let input = include_str!("./fixtures/input/gh-459.text");
    let output = include_str!("./fixtures/tree/gh-459.json");

    let mdast = to_mdast(input, &ParseOptions::default()).expect("could not parse fixture's input to mdast");

    let expected_tree: serde_json::Value = serde_json::from_str(output).expect("a fixture's tree contains invalid json");

    let actual_tree = serde_json::to_value(&mdast).expect("could not serialize mdast to json");

    assert_eq!(expected_tree, actual_tree);
}

#[test]
fn test_gh_493() {
    let input = include_str!("./fixtures/input/gh-493.text");
    let output = include_str!("./fixtures/tree/gh-493.json");

    let mdast = to_mdast(input, &ParseOptions::default()).expect("could not parse fixture's input to mdast");

    let expected_tree: serde_json::Value = serde_json::from_str(output).expect("a fixture's tree contains invalid json");

    let actual_tree = serde_json::to_value(&mdast).expect("could not serialize mdast to json");

    assert_eq!(expected_tree, actual_tree);
}

#[test]
fn test_gh_494() {
    let input = include_str!("./fixtures/input/gh-494.text");
    let output = include_str!("./fixtures/tree/gh-494.json");

    let mdast = to_mdast(input, &ParseOptions::default()).expect("could not parse fixture's input to mdast");

    let expected_tree: serde_json::Value = serde_json::from_str(output).expect("a fixture's tree contains invalid json");

    let actual_tree = serde_json::to_value(&mdast).expect("could not serialize mdast to json");

    assert_eq!(expected_tree, actual_tree);
}

#[test]
fn test_gh_497() {
    let input = include_str!("./fixtures/input/gh-497.text");
    let output = include_str!("./fixtures/tree/gh-497.json");

    let mdast = to_mdast(input, &ParseOptions::default()).expect("could not parse fixture's input to mdast");

    let expected_tree: serde_json::Value = serde_json::from_str(output).expect("a fixture's tree contains invalid json");

    let actual_tree = serde_json::to_value(&mdast).expect("could not serialize mdast to json");

    assert_eq!(expected_tree, actual_tree);
}

#[test]
fn test_gh_504() {
    let input = include_str!("./fixtures/input/gh-504.text");
    let output = include_str!("./fixtures/tree/gh-504.json");

    let mdast = to_mdast(input, &ParseOptions::default()).expect("could not parse fixture's input to mdast");

    let expected_tree: serde_json::Value = serde_json::from_str(output).expect("a fixture's tree contains invalid json");

    let actual_tree = serde_json::to_value(&mdast).expect("could not serialize mdast to json");

    assert_eq!(expected_tree, actual_tree);
}

#[test]
fn test_gh_517() {
    let input = include_str!("./fixtures/input/gh-517.text");
    let output = include_str!("./fixtures/tree/gh-517.json");

    let mdast = to_mdast(input, &ParseOptions::default()).expect("could not parse fixture's input to mdast");

    let expected_tree: serde_json::Value = serde_json::from_str(output).expect("a fixture's tree contains invalid json");

    let actual_tree = serde_json::to_value(&mdast).expect("could not serialize mdast to json");

    assert_eq!(expected_tree, actual_tree);
}

#[test]
fn test_gh_521_output() {
    let input = include_str!("./fixtures/input/gh-521.output.text");
    let output = include_str!("./fixtures/tree/gh-521.output.json");

    let mdast = to_mdast(input, &ParseOptions::default()).expect("could not parse fixture's input to mdast");

    let expected_tree: serde_json::Value = serde_json::from_str(output).expect("a fixture's tree contains invalid json");

    let actual_tree = serde_json::to_value(&mdast).expect("could not serialize mdast to json");

    assert_eq!(expected_tree, actual_tree);
}

#[test]
fn test_gh_523() {
    let input = include_str!("./fixtures/input/gh-523.text");
    let output = include_str!("./fixtures/tree/gh-523.json");

    let mdast = to_mdast(input, &ParseOptions::default()).expect("could not parse fixture's input to mdast");

    let expected_tree: serde_json::Value = serde_json::from_str(output).expect("a fixture's tree contains invalid json");

    let actual_tree = serde_json::to_value(&mdast).expect("could not serialize mdast to json");

    assert_eq!(expected_tree, actual_tree);
}

#[test]
fn test_hard_wrapped_paragraphs_with_list_like_lines() {
    let input = include_str!("./fixtures/input/hard-wrapped-paragraphs-with-list-like-lines.text");
    let output = include_str!("./fixtures/tree/hard-wrapped-paragraphs-with-list-like-lines.json");

    let mdast = to_mdast(input, &ParseOptions::default()).expect("could not parse fixture's input to mdast");

    let expected_tree: serde_json::Value = serde_json::from_str(output).expect("a fixture's tree contains invalid json");

    let actual_tree = serde_json::to_value(&mdast).expect("could not serialize mdast to json");

    assert_eq!(expected_tree, actual_tree);
}

#[test]
fn test_heading_atx_closed_trailing_white_space() {
    let input = include_str!("./fixtures/input/heading-atx-closed-trailing-white-space.text");
    let output = include_str!("./fixtures/tree/heading-atx-closed-trailing-white-space.json");

    let mdast = to_mdast(input, &ParseOptions::default()).expect("could not parse fixture's input to mdast");

    let expected_tree: serde_json::Value = serde_json::from_str(output).expect("a fixture's tree contains invalid json");

    let actual_tree = serde_json::to_value(&mdast).expect("could not serialize mdast to json");

    assert_eq!(expected_tree, actual_tree);
}

#[test]
fn test_heading_atx_empty() {
    let input = include_str!("./fixtures/input/heading-atx-empty.text");
    let output = include_str!("./fixtures/tree/heading-atx-empty.json");

    let mdast = to_mdast(input, &ParseOptions::default()).expect("could not parse fixture's input to mdast");

    let expected_tree: serde_json::Value = serde_json::from_str(output).expect("a fixture's tree contains invalid json");

    let actual_tree = serde_json::to_value(&mdast).expect("could not serialize mdast to json");

    assert_eq!(expected_tree, actual_tree);
}

#[test]
fn test_heading_atx_no_space_before_trailing_number_sign() {
    let input = include_str!("./fixtures/input/heading-atx-no-space-before-trailing-number-sign.text");
    let output = include_str!("./fixtures/tree/heading-atx-no-space-before-trailing-number-sign.json");

    let mdast = to_mdast(input, &ParseOptions::default()).expect("could not parse fixture's input to mdast");

    let expected_tree: serde_json::Value = serde_json::from_str(output).expect("a fixture's tree contains invalid json");

    let actual_tree = serde_json::to_value(&mdast).expect("could not serialize mdast to json");

    assert_eq!(expected_tree, actual_tree);
}

#[test]
fn test_heading_in_blockquote() {
    let input = include_str!("./fixtures/input/heading-in-blockquote.text");
    let output = include_str!("./fixtures/tree/heading-in-blockquote.json");

    let mdast = to_mdast(input, &ParseOptions::default()).expect("could not parse fixture's input to mdast");

    let expected_tree: serde_json::Value = serde_json::from_str(output).expect("a fixture's tree contains invalid json");

    let actual_tree = serde_json::to_value(&mdast).expect("could not serialize mdast to json");

    assert_eq!(expected_tree, actual_tree);
}

#[test]
fn test_heading_in_paragraph() {
    let input = include_str!("./fixtures/input/heading-in-paragraph.text");
    let output = include_str!("./fixtures/tree/heading-in-paragraph.json");

    let mdast = to_mdast(input, &ParseOptions::default()).expect("could not parse fixture's input to mdast");

    let expected_tree: serde_json::Value = serde_json::from_str(output).expect("a fixture's tree contains invalid json");

    let actual_tree = serde_json::to_value(&mdast).expect("could not serialize mdast to json");

    assert_eq!(expected_tree, actual_tree);
}

#[test]
fn test_heading_not_atx() {
    let input = include_str!("./fixtures/input/heading-not-atx.text");
    let output = include_str!("./fixtures/tree/heading-not-atx.json");

    let mdast = to_mdast(input, &ParseOptions::default()).expect("could not parse fixture's input to mdast");

    let expected_tree: serde_json::Value = serde_json::from_str(output).expect("a fixture's tree contains invalid json");

    let actual_tree = serde_json::to_value(&mdast).expect("could not serialize mdast to json");

    assert_eq!(expected_tree, actual_tree);
}

#[test]
fn test_heading_setext_with_initial_spacing() {
    let input = include_str!("./fixtures/input/heading-setext-with-initial-spacing.text");
    let output = include_str!("./fixtures/tree/heading-setext-with-initial-spacing.json");

    let mdast = to_mdast(input, &ParseOptions::default()).expect("could not parse fixture's input to mdast");

    let expected_tree: serde_json::Value = serde_json::from_str(output).expect("a fixture's tree contains invalid json");

    let actual_tree = serde_json::to_value(&mdast).expect("could not serialize mdast to json");

    assert_eq!(expected_tree, actual_tree);
}

#[test]
fn test_heading_output_close_atx() {
    let input = include_str!("./fixtures/input/heading.output.close-atx.text");
    let output = include_str!("./fixtures/tree/heading.output.close-atx.json");

    let mdast = to_mdast(input, &ParseOptions::default()).expect("could not parse fixture's input to mdast");

    let expected_tree: serde_json::Value = serde_json::from_str(output).expect("a fixture's tree contains invalid json");

    let actual_tree = serde_json::to_value(&mdast).expect("could not serialize mdast to json");

    assert_eq!(expected_tree, actual_tree);
}

#[test]
fn test_heading_output_setext() {
    let input = include_str!("./fixtures/input/heading.output.setext.text");
    let output = include_str!("./fixtures/tree/heading.output.setext.json");

    let mdast = to_mdast(input, &ParseOptions::default()).expect("could not parse fixture's input to mdast");

    let expected_tree: serde_json::Value = serde_json::from_str(output).expect("a fixture's tree contains invalid json");

    let actual_tree = serde_json::to_value(&mdast).expect("could not serialize mdast to json");

    assert_eq!(expected_tree, actual_tree);
}

#[test]
fn test_horizontal_rules_adjacent() {
    let input = include_str!("./fixtures/input/horizontal-rules-adjacent.text");
    let output = include_str!("./fixtures/tree/horizontal-rules-adjacent.json");

    let mdast = to_mdast(input, &ParseOptions::default()).expect("could not parse fixture's input to mdast");

    let expected_tree: serde_json::Value = serde_json::from_str(output).expect("a fixture's tree contains invalid json");

    let actual_tree = serde_json::to_value(&mdast).expect("could not serialize mdast to json");

    assert_eq!(expected_tree, actual_tree);
}

#[test]
fn test_horizontal_rules() {
    let input = include_str!("./fixtures/input/horizontal-rules.text");
    let output = include_str!("./fixtures/tree/horizontal-rules.json");

    let mdast = to_mdast(input, &ParseOptions::default()).expect("could not parse fixture's input to mdast");

    let expected_tree: serde_json::Value = serde_json::from_str(output).expect("a fixture's tree contains invalid json");

    let actual_tree = serde_json::to_value(&mdast).expect("could not serialize mdast to json");

    assert_eq!(expected_tree, actual_tree);
}

#[test]
fn test_hr_list_break_nooutput() {
    let input = include_str!("./fixtures/input/hr-list-break.nooutput.text");
    let output = include_str!("./fixtures/tree/hr-list-break.nooutput.json");

    let mdast = to_mdast(input, &ParseOptions::default()).expect("could not parse fixture's input to mdast");

    let expected_tree: serde_json::Value = serde_json::from_str(output).expect("a fixture's tree contains invalid json");

    let actual_tree = serde_json::to_value(&mdast).expect("could not serialize mdast to json");

    assert_eq!(expected_tree, actual_tree);
}

#[test]
fn test_hr_output_norule_spaces() {
    let input = include_str!("./fixtures/input/hr.output.norule-spaces.text");
    let output = include_str!("./fixtures/tree/hr.output.norule-spaces.json");

    let mdast = to_mdast(input, &ParseOptions::default()).expect("could not parse fixture's input to mdast");

    let expected_tree: serde_json::Value = serde_json::from_str(output).expect("a fixture's tree contains invalid json");

    let actual_tree = serde_json::to_value(&mdast).expect("could not serialize mdast to json");

    assert_eq!(expected_tree, actual_tree);
}

#[test]
fn test_hr_output_rule_repetition_5() {
    let input = include_str!("./fixtures/input/hr.output.rule-repetition=5.text");
    let output = include_str!("./fixtures/tree/hr.output.rule-repetition=5.json");

    let mdast = to_mdast(input, &ParseOptions::default()).expect("could not parse fixture's input to mdast");

    let expected_tree: serde_json::Value = serde_json::from_str(output).expect("a fixture's tree contains invalid json");

    let actual_tree = serde_json::to_value(&mdast).expect("could not serialize mdast to json");

    assert_eq!(expected_tree, actual_tree);
}

#[test]
fn test_hr_output_rule() {
    let input = include_str!("./fixtures/input/hr.output.rule=-.text");
    let output = include_str!("./fixtures/tree/hr.output.rule=-.json");

    let mdast = to_mdast(input, &ParseOptions::default()).expect("could not parse fixture's input to mdast");

    let expected_tree: serde_json::Value = serde_json::from_str(output).expect("a fixture's tree contains invalid json");

    let actual_tree = serde_json::to_value(&mdast).expect("could not serialize mdast to json");

    assert_eq!(expected_tree, actual_tree);
}

#[test]
fn test_hr_output_rule_asterisk() {
    let input = include_str!("./fixtures/input/hr.output.rule=-asterisk-.text");
    let output = include_str!("./fixtures/tree/hr.output.rule=-asterisk-.json");

    let mdast = to_mdast(input, &ParseOptions::default()).expect("could not parse fixture's input to mdast");

    let expected_tree: serde_json::Value = serde_json::from_str(output).expect("a fixture's tree contains invalid json");

    let actual_tree = serde_json::to_value(&mdast).expect("could not serialize mdast to json");

    assert_eq!(expected_tree, actual_tree);
}

#[test]
fn test_hr_output_rule_2() {
    let input = include_str!("./fixtures/input/hr.output.rule=_.text");
    let output = include_str!("./fixtures/tree/hr.output.rule=_.json");

    let mdast = to_mdast(input, &ParseOptions::default()).expect("could not parse fixture's input to mdast");

    let expected_tree: serde_json::Value = serde_json::from_str(output).expect("a fixture's tree contains invalid json");

    let actual_tree = serde_json::to_value(&mdast).expect("could not serialize mdast to json");

    assert_eq!(expected_tree, actual_tree);
}

#[test]
fn test_html_advanced() {
    let input = include_str!("./fixtures/input/html-advanced.text");
    let output = include_str!("./fixtures/tree/html-advanced.json");

    let mdast = to_mdast(input, &ParseOptions::default()).expect("could not parse fixture's input to mdast");

    let expected_tree: serde_json::Value = serde_json::from_str(output).expect("a fixture's tree contains invalid json");

    let actual_tree = serde_json::to_value(&mdast).expect("could not serialize mdast to json");

    assert_eq!(expected_tree, actual_tree);
}

#[test]
fn test_html_attributes() {
    let input = include_str!("./fixtures/input/html-attributes.text");
    let output = include_str!("./fixtures/tree/html-attributes.json");

    let mdast = to_mdast(input, &ParseOptions::default()).expect("could not parse fixture's input to mdast");

    let expected_tree: serde_json::Value = serde_json::from_str(output).expect("a fixture's tree contains invalid json");

    let actual_tree = serde_json::to_value(&mdast).expect("could not serialize mdast to json");

    assert_eq!(expected_tree, actual_tree);
}

#[test]
fn test_html_cdata() {
    let input = include_str!("./fixtures/input/html-cdata.text");
    let output = include_str!("./fixtures/tree/html-cdata.json");

    let mdast = to_mdast(input, &ParseOptions::default()).expect("could not parse fixture's input to mdast");

    let expected_tree: serde_json::Value = serde_json::from_str(output).expect("a fixture's tree contains invalid json");

    let actual_tree = serde_json::to_value(&mdast).expect("could not serialize mdast to json");

    assert_eq!(expected_tree, actual_tree);
}

#[test]
fn test_html_comments() {
    let input = include_str!("./fixtures/input/html-comments.text");
    let output = include_str!("./fixtures/tree/html-comments.json");

    let mdast = to_mdast(input, &ParseOptions::default()).expect("could not parse fixture's input to mdast");

    let expected_tree: serde_json::Value = serde_json::from_str(output).expect("a fixture's tree contains invalid json");

    let actual_tree = serde_json::to_value(&mdast).expect("could not serialize mdast to json");

    assert_eq!(expected_tree, actual_tree);
}

#[test]
fn test_html_declaration() {
    let input = include_str!("./fixtures/input/html-declaration.text");
    let output = include_str!("./fixtures/tree/html-declaration.json");

    let mdast = to_mdast(input, &ParseOptions::default()).expect("could not parse fixture's input to mdast");

    let expected_tree: serde_json::Value = serde_json::from_str(output).expect("a fixture's tree contains invalid json");

    let actual_tree = serde_json::to_value(&mdast).expect("could not serialize mdast to json");

    assert_eq!(expected_tree, actual_tree);
}

#[test]
fn test_html_indented() {
    let input = include_str!("./fixtures/input/html-indented.text");
    let output = include_str!("./fixtures/tree/html-indented.json");

    let mdast = to_mdast(input, &ParseOptions::default()).expect("could not parse fixture's input to mdast");

    let expected_tree: serde_json::Value = serde_json::from_str(output).expect("a fixture's tree contains invalid json");

    let actual_tree = serde_json::to_value(&mdast).expect("could not serialize mdast to json");

    assert_eq!(expected_tree, actual_tree);
}

#[test]
fn test_html_processing_instruction() {
    let input = include_str!("./fixtures/input/html-processing-instruction.text");
    let output = include_str!("./fixtures/tree/html-processing-instruction.json");

    let mdast = to_mdast(input, &ParseOptions::default()).expect("could not parse fixture's input to mdast");

    let expected_tree: serde_json::Value = serde_json::from_str(output).expect("a fixture's tree contains invalid json");

    let actual_tree = serde_json::to_value(&mdast).expect("could not serialize mdast to json");

    assert_eq!(expected_tree, actual_tree);
}

#[test]
fn test_html_simple() {
    let input = include_str!("./fixtures/input/html-simple.text");
    let output = include_str!("./fixtures/tree/html-simple.json");

    let mdast = to_mdast(input, &ParseOptions::default()).expect("could not parse fixture's input to mdast");

    let expected_tree: serde_json::Value = serde_json::from_str(output).expect("a fixture's tree contains invalid json");

    let actual_tree = serde_json::to_value(&mdast).expect("could not serialize mdast to json");

    assert_eq!(expected_tree, actual_tree);
}

#[test]
fn test_html_tags() {
    let input = include_str!("./fixtures/input/html-tags.text");
    let output = include_str!("./fixtures/tree/html-tags.json");

    let mdast = to_mdast(input, &ParseOptions::default()).expect("could not parse fixture's input to mdast");

    let expected_tree: serde_json::Value = serde_json::from_str(output).expect("a fixture's tree contains invalid json");

    let actual_tree = serde_json::to_value(&mdast).expect("could not serialize mdast to json");

    assert_eq!(expected_tree, actual_tree);
}

#[test]
fn test_image_empty_alt() {
    let input = include_str!("./fixtures/input/image-empty-alt.text");
    let output = include_str!("./fixtures/tree/image-empty-alt.json");

    let mdast = to_mdast(input, &ParseOptions::default()).expect("could not parse fixture's input to mdast");

    let expected_tree: serde_json::Value = serde_json::from_str(output).expect("a fixture's tree contains invalid json");

    let actual_tree = serde_json::to_value(&mdast).expect("could not serialize mdast to json");

    assert_eq!(expected_tree, actual_tree);
}

#[test]
fn test_image_in_link() {
    let input = include_str!("./fixtures/input/image-in-link.text");
    let output = include_str!("./fixtures/tree/image-in-link.json");

    let mdast = to_mdast(input, &ParseOptions::default()).expect("could not parse fixture's input to mdast");

    let expected_tree: serde_json::Value = serde_json::from_str(output).expect("a fixture's tree contains invalid json");

    let actual_tree = serde_json::to_value(&mdast).expect("could not serialize mdast to json");

    assert_eq!(expected_tree, actual_tree);
}

#[test]
fn test_image_with_pipe() {
    let input = include_str!("./fixtures/input/image-with-pipe.text");
    let output = include_str!("./fixtures/tree/image-with-pipe.json");

    let mdast = to_mdast(input, &ParseOptions::default()).expect("could not parse fixture's input to mdast");

    let expected_tree: serde_json::Value = serde_json::from_str(output).expect("a fixture's tree contains invalid json");

    let actual_tree = serde_json::to_value(&mdast).expect("could not serialize mdast to json");

    assert_eq!(expected_tree, actual_tree);
}

#[test]
fn test_images_output_noreference_images() {
    let input = include_str!("./fixtures/input/images.output.noreference-images.text");
    let output = include_str!("./fixtures/tree/images.output.noreference-images.json");

    let mdast = to_mdast(input, &ParseOptions::default()).expect("could not parse fixture's input to mdast");

    let expected_tree: serde_json::Value = serde_json::from_str(output).expect("a fixture's tree contains invalid json");

    let actual_tree = serde_json::to_value(&mdast).expect("could not serialize mdast to json");

    assert_eq!(expected_tree, actual_tree);
}

#[test]
fn test_isolated_hard_break() {
    let input = include_str!("./fixtures/input/isolated-hard-break.text");
    let output = include_str!("./fixtures/tree/isolated-hard-break.json");

    let mdast = to_mdast(input, &ParseOptions::default()).expect("could not parse fixture's input to mdast");

    let expected_tree: serde_json::Value = serde_json::from_str(output).expect("a fixture's tree contains invalid json");

    let actual_tree = serde_json::to_value(&mdast).expect("could not serialize mdast to json");

    assert_eq!(expected_tree, actual_tree);
}

#[test]
fn test_lazy_blockquotes() {
    let input = include_str!("./fixtures/input/lazy-blockquotes.text");
    let output = include_str!("./fixtures/tree/lazy-blockquotes.json");

    let mdast = to_mdast(input, &ParseOptions::default()).expect("could not parse fixture's input to mdast");

    let expected_tree: serde_json::Value = serde_json::from_str(output).expect("a fixture's tree contains invalid json");

    let actual_tree = serde_json::to_value(&mdast).expect("could not serialize mdast to json");

    assert_eq!(expected_tree, actual_tree);
}

#[test]
fn test_link_in_link() {
    let input = include_str!("./fixtures/input/link-in-link.text");
    let output = include_str!("./fixtures/tree/link-in-link.json");

    let mdast = to_mdast(input, &ParseOptions::default()).expect("could not parse fixture's input to mdast");

    let expected_tree: serde_json::Value = serde_json::from_str(output).expect("a fixture's tree contains invalid json");

    let actual_tree = serde_json::to_value(&mdast).expect("could not serialize mdast to json");

    assert_eq!(expected_tree, actual_tree);
}

#[test]
fn test_link_spaces() {
    let input = include_str!("./fixtures/input/link-spaces.text");
    let output = include_str!("./fixtures/tree/link-spaces.json");

    let mdast = to_mdast(input, &ParseOptions::default()).expect("could not parse fixture's input to mdast");

    let expected_tree: serde_json::Value = serde_json::from_str(output).expect("a fixture's tree contains invalid json");

    let actual_tree = serde_json::to_value(&mdast).expect("could not serialize mdast to json");

    assert_eq!(expected_tree, actual_tree);
}

#[test]
fn test_link_whitespace() {
    let input = include_str!("./fixtures/input/link-whitespace.text");
    let output = include_str!("./fixtures/tree/link-whitespace.json");

    let mdast = to_mdast(input, &ParseOptions::default()).expect("could not parse fixture's input to mdast");

    let expected_tree: serde_json::Value = serde_json::from_str(output).expect("a fixture's tree contains invalid json");

    let actual_tree = serde_json::to_value(&mdast).expect("could not serialize mdast to json");

    assert_eq!(expected_tree, actual_tree);
}

#[test]
fn test_link_with_spaces() {
    let input = include_str!("./fixtures/input/link-with-spaces.text");
    let output = include_str!("./fixtures/tree/link-with-spaces.json");

    let mdast = to_mdast(input, &ParseOptions::default()).expect("could not parse fixture's input to mdast");

    let expected_tree: serde_json::Value = serde_json::from_str(output).expect("a fixture's tree contains invalid json");

    let actual_tree = serde_json::to_value(&mdast).expect("could not serialize mdast to json");

    assert_eq!(expected_tree, actual_tree);
}

#[test]
fn test_links_inline_style() {
    let input = include_str!("./fixtures/input/links-inline-style.text");
    let output = include_str!("./fixtures/tree/links-inline-style.json");

    let mdast = to_mdast(input, &ParseOptions::default()).expect("could not parse fixture's input to mdast");

    let expected_tree: serde_json::Value = serde_json::from_str(output).expect("a fixture's tree contains invalid json");

    let actual_tree = serde_json::to_value(&mdast).expect("could not serialize mdast to json");

    assert_eq!(expected_tree, actual_tree);
}

#[test]
fn test_links_reference_proto() {
    let input = include_str!("./fixtures/input/links-reference-proto.text");
    let output = include_str!("./fixtures/tree/links-reference-proto.json");

    let mdast = to_mdast(input, &ParseOptions::default()).expect("could not parse fixture's input to mdast");

    let expected_tree: serde_json::Value = serde_json::from_str(output).expect("a fixture's tree contains invalid json");

    let actual_tree = serde_json::to_value(&mdast).expect("could not serialize mdast to json");

    assert_eq!(expected_tree, actual_tree);
}

#[test]
fn test_links_reference_style_nooutput() {
    let input = include_str!("./fixtures/input/links-reference-style.nooutput.text");
    let output = include_str!("./fixtures/tree/links-reference-style.nooutput.json");

    let mdast = to_mdast(input, &ParseOptions::default()).expect("could not parse fixture's input to mdast");

    let expected_tree: serde_json::Value = serde_json::from_str(output).expect("a fixture's tree contains invalid json");

    let actual_tree = serde_json::to_value(&mdast).expect("could not serialize mdast to json");

    assert_eq!(expected_tree, actual_tree);
}

#[test]
fn test_links_shortcut_references_nooutput() {
    let input = include_str!("./fixtures/input/links-shortcut-references.nooutput.text");
    let output = include_str!("./fixtures/tree/links-shortcut-references.nooutput.json");

    let mdast = to_mdast(input, &ParseOptions::default()).expect("could not parse fixture's input to mdast");

    let expected_tree: serde_json::Value = serde_json::from_str(output).expect("a fixture's tree contains invalid json");

    let actual_tree = serde_json::to_value(&mdast).expect("could not serialize mdast to json");

    assert_eq!(expected_tree, actual_tree);
}

#[test]
fn test_links_text_delimiters() {
    let input = include_str!("./fixtures/input/links-text-delimiters.text");
    let output = include_str!("./fixtures/tree/links-text-delimiters.json");

    let mdast = to_mdast(input, &ParseOptions::default()).expect("could not parse fixture's input to mdast");

    let expected_tree: serde_json::Value = serde_json::from_str(output).expect("a fixture's tree contains invalid json");

    let actual_tree = serde_json::to_value(&mdast).expect("could not serialize mdast to json");

    assert_eq!(expected_tree, actual_tree);
}

#[test]
fn test_links_text_empty() {
    let input = include_str!("./fixtures/input/links-text-empty.text");
    let output = include_str!("./fixtures/tree/links-text-empty.json");

    let mdast = to_mdast(input, &ParseOptions::default()).expect("could not parse fixture's input to mdast");

    let expected_tree: serde_json::Value = serde_json::from_str(output).expect("a fixture's tree contains invalid json");

    let actual_tree = serde_json::to_value(&mdast).expect("could not serialize mdast to json");

    assert_eq!(expected_tree, actual_tree);
}

#[test]
fn test_links_text_entity_delimiters() {
    let input = include_str!("./fixtures/input/links-text-entity-delimiters.text");
    let output = include_str!("./fixtures/tree/links-text-entity-delimiters.json");

    let mdast = to_mdast(input, &ParseOptions::default()).expect("could not parse fixture's input to mdast");

    let expected_tree: serde_json::Value = serde_json::from_str(output).expect("a fixture's tree contains invalid json");

    let actual_tree = serde_json::to_value(&mdast).expect("could not serialize mdast to json");

    assert_eq!(expected_tree, actual_tree);
}

#[test]
fn test_links_text_escaped_delimiters() {
    let input = include_str!("./fixtures/input/links-text-escaped-delimiters.text");
    let output = include_str!("./fixtures/tree/links-text-escaped-delimiters.json");

    let mdast = to_mdast(input, &ParseOptions::default()).expect("could not parse fixture's input to mdast");

    let expected_tree: serde_json::Value = serde_json::from_str(output).expect("a fixture's tree contains invalid json");

    let actual_tree = serde_json::to_value(&mdast).expect("could not serialize mdast to json");

    assert_eq!(expected_tree, actual_tree);
}

#[test]
fn test_links_text_mismatched_delimiters() {
    let input = include_str!("./fixtures/input/links-text-mismatched-delimiters.text");
    let output = include_str!("./fixtures/tree/links-text-mismatched-delimiters.json");

    let mdast = to_mdast(input, &ParseOptions::default()).expect("could not parse fixture's input to mdast");

    let expected_tree: serde_json::Value = serde_json::from_str(output).expect("a fixture's tree contains invalid json");

    let actual_tree = serde_json::to_value(&mdast).expect("could not serialize mdast to json");

    assert_eq!(expected_tree, actual_tree);
}

#[test]
fn test_links_title_double_quotes_delimiters() {
    let input = include_str!("./fixtures/input/links-title-double-quotes-delimiters.text");
    let output = include_str!("./fixtures/tree/links-title-double-quotes-delimiters.json");

    let mdast = to_mdast(input, &ParseOptions::default()).expect("could not parse fixture's input to mdast");

    let expected_tree: serde_json::Value = serde_json::from_str(output).expect("a fixture's tree contains invalid json");

    let actual_tree = serde_json::to_value(&mdast).expect("could not serialize mdast to json");

    assert_eq!(expected_tree, actual_tree);
}

#[test]
fn test_links_title_double_quotes_entity_delimiters() {
    let input = include_str!("./fixtures/input/links-title-double-quotes-entity-delimiters.text");
    let output = include_str!("./fixtures/tree/links-title-double-quotes-entity-delimiters.json");

    let mdast = to_mdast(input, &ParseOptions::default()).expect("could not parse fixture's input to mdast");

    let expected_tree: serde_json::Value = serde_json::from_str(output).expect("a fixture's tree contains invalid json");

    let actual_tree = serde_json::to_value(&mdast).expect("could not serialize mdast to json");

    assert_eq!(expected_tree, actual_tree);
}

#[test]
fn test_links_title_double_quotes_escaped_delimiters() {
    let input = include_str!("./fixtures/input/links-title-double-quotes-escaped-delimiters.text");
    let output = include_str!("./fixtures/tree/links-title-double-quotes-escaped-delimiters.json");

    let mdast = to_mdast(input, &ParseOptions::default()).expect("could not parse fixture's input to mdast");

    let expected_tree: serde_json::Value = serde_json::from_str(output).expect("a fixture's tree contains invalid json");

    let actual_tree = serde_json::to_value(&mdast).expect("could not serialize mdast to json");

    assert_eq!(expected_tree, actual_tree);
}

#[test]
fn test_links_title_double_quotes_mismatched_delimiters() {
    let input = include_str!("./fixtures/input/links-title-double-quotes-mismatched-delimiters.text");
    let output = include_str!("./fixtures/tree/links-title-double-quotes-mismatched-delimiters.json");

    let mdast = to_mdast(input, &ParseOptions::default()).expect("could not parse fixture's input to mdast");

    let expected_tree: serde_json::Value = serde_json::from_str(output).expect("a fixture's tree contains invalid json");

    let actual_tree = serde_json::to_value(&mdast).expect("could not serialize mdast to json");

    assert_eq!(expected_tree, actual_tree);
}

#[test]
fn test_links_title_double_quotes() {
    let input = include_str!("./fixtures/input/links-title-double-quotes.text");
    let output = include_str!("./fixtures/tree/links-title-double-quotes.json");

    let mdast = to_mdast(input, &ParseOptions::default()).expect("could not parse fixture's input to mdast");

    let expected_tree: serde_json::Value = serde_json::from_str(output).expect("a fixture's tree contains invalid json");

    let actual_tree = serde_json::to_value(&mdast).expect("could not serialize mdast to json");

    assert_eq!(expected_tree, actual_tree);
}

#[test]
fn test_links_title_empty_double_quotes() {
    let input = include_str!("./fixtures/input/links-title-empty-double-quotes.text");
    let output = include_str!("./fixtures/tree/links-title-empty-double-quotes.json");

    let mdast = to_mdast(input, &ParseOptions::default()).expect("could not parse fixture's input to mdast");

    let expected_tree: serde_json::Value = serde_json::from_str(output).expect("a fixture's tree contains invalid json");

    let actual_tree = serde_json::to_value(&mdast).expect("could not serialize mdast to json");

    assert_eq!(expected_tree, actual_tree);
}

#[test]
fn test_links_title_empty_parentheses() {
    let input = include_str!("./fixtures/input/links-title-empty-parentheses.text");
    let output = include_str!("./fixtures/tree/links-title-empty-parentheses.json");

    let mdast = to_mdast(input, &ParseOptions::default()).expect("could not parse fixture's input to mdast");

    let expected_tree: serde_json::Value = serde_json::from_str(output).expect("a fixture's tree contains invalid json");

    let actual_tree = serde_json::to_value(&mdast).expect("could not serialize mdast to json");

    assert_eq!(expected_tree, actual_tree);
}

#[test]
fn test_links_title_empty_single_quotes() {
    let input = include_str!("./fixtures/input/links-title-empty-single-quotes.text");
    let output = include_str!("./fixtures/tree/links-title-empty-single-quotes.json");

    let mdast = to_mdast(input, &ParseOptions::default()).expect("could not parse fixture's input to mdast");

    let expected_tree: serde_json::Value = serde_json::from_str(output).expect("a fixture's tree contains invalid json");

    let actual_tree = serde_json::to_value(&mdast).expect("could not serialize mdast to json");

    assert_eq!(expected_tree, actual_tree);
}

#[test]
fn test_links_title_parentheses() {
    let input = include_str!("./fixtures/input/links-title-parentheses.text");
    let output = include_str!("./fixtures/tree/links-title-parentheses.json");

    let mdast = to_mdast(input, &ParseOptions::default()).expect("could not parse fixture's input to mdast");

    let expected_tree: serde_json::Value = serde_json::from_str(output).expect("a fixture's tree contains invalid json");

    let actual_tree = serde_json::to_value(&mdast).expect("could not serialize mdast to json");

    assert_eq!(expected_tree, actual_tree);
}

#[test]
fn test_links_title_single_quotes_delimiters() {
    let input = include_str!("./fixtures/input/links-title-single-quotes-delimiters.text");
    let output = include_str!("./fixtures/tree/links-title-single-quotes-delimiters.json");

    let mdast = to_mdast(input, &ParseOptions::default()).expect("could not parse fixture's input to mdast");

    let expected_tree: serde_json::Value = serde_json::from_str(output).expect("a fixture's tree contains invalid json");

    let actual_tree = serde_json::to_value(&mdast).expect("could not serialize mdast to json");

    assert_eq!(expected_tree, actual_tree);
}

#[test]
fn test_links_title_single_quotes_entity_delimiters() {
    let input = include_str!("./fixtures/input/links-title-single-quotes-entity-delimiters.text");
    let output = include_str!("./fixtures/tree/links-title-single-quotes-entity-delimiters.json");

    let mdast = to_mdast(input, &ParseOptions::default()).expect("could not parse fixture's input to mdast");

    let expected_tree: serde_json::Value = serde_json::from_str(output).expect("a fixture's tree contains invalid json");

    let actual_tree = serde_json::to_value(&mdast).expect("could not serialize mdast to json");

    assert_eq!(expected_tree, actual_tree);
}

#[test]
fn test_links_title_single_quotes_escaped_delimiters() {
    let input = include_str!("./fixtures/input/links-title-single-quotes-escaped-delimiters.text");
    let output = include_str!("./fixtures/tree/links-title-single-quotes-escaped-delimiters.json");

    let mdast = to_mdast(input, &ParseOptions::default()).expect("could not parse fixture's input to mdast");

    let expected_tree: serde_json::Value = serde_json::from_str(output).expect("a fixture's tree contains invalid json");

    let actual_tree = serde_json::to_value(&mdast).expect("could not serialize mdast to json");

    assert_eq!(expected_tree, actual_tree);
}

#[test]
fn test_links_title_single_quotes_mismatched_delimiters() {
    let input = include_str!("./fixtures/input/links-title-single-quotes-mismatched-delimiters.text");
    let output = include_str!("./fixtures/tree/links-title-single-quotes-mismatched-delimiters.json");

    let mdast = to_mdast(input, &ParseOptions::default()).expect("could not parse fixture's input to mdast");

    let expected_tree: serde_json::Value = serde_json::from_str(output).expect("a fixture's tree contains invalid json");

    let actual_tree = serde_json::to_value(&mdast).expect("could not serialize mdast to json");

    assert_eq!(expected_tree, actual_tree);
}

#[test]
fn test_links_title_single_quotes() {
    let input = include_str!("./fixtures/input/links-title-single-quotes.text");
    let output = include_str!("./fixtures/tree/links-title-single-quotes.json");

    let mdast = to_mdast(input, &ParseOptions::default()).expect("could not parse fixture's input to mdast");

    let expected_tree: serde_json::Value = serde_json::from_str(output).expect("a fixture's tree contains invalid json");

    let actual_tree = serde_json::to_value(&mdast).expect("could not serialize mdast to json");

    assert_eq!(expected_tree, actual_tree);
}

#[test]
fn test_links_title_unclosed() {
    let input = include_str!("./fixtures/input/links-title-unclosed.text");
    let output = include_str!("./fixtures/tree/links-title-unclosed.json");

    let mdast = to_mdast(input, &ParseOptions::default()).expect("could not parse fixture's input to mdast");

    let expected_tree: serde_json::Value = serde_json::from_str(output).expect("a fixture's tree contains invalid json");

    let actual_tree = serde_json::to_value(&mdast).expect("could not serialize mdast to json");

    assert_eq!(expected_tree, actual_tree);
}

#[test]
fn test_links_url_empty_title_double_quotes() {
    let input = include_str!("./fixtures/input/links-url-empty-title-double-quotes.text");
    let output = include_str!("./fixtures/tree/links-url-empty-title-double-quotes.json");

    let mdast = to_mdast(input, &ParseOptions::default()).expect("could not parse fixture's input to mdast");

    let expected_tree: serde_json::Value = serde_json::from_str(output).expect("a fixture's tree contains invalid json");

    let actual_tree = serde_json::to_value(&mdast).expect("could not serialize mdast to json");

    assert_eq!(expected_tree, actual_tree);
}

#[test]
fn test_links_url_empty_title_parentheses() {
    let input = include_str!("./fixtures/input/links-url-empty-title-parentheses.text");
    let output = include_str!("./fixtures/tree/links-url-empty-title-parentheses.json");

    let mdast = to_mdast(input, &ParseOptions::default()).expect("could not parse fixture's input to mdast");

    let expected_tree: serde_json::Value = serde_json::from_str(output).expect("a fixture's tree contains invalid json");

    let actual_tree = serde_json::to_value(&mdast).expect("could not serialize mdast to json");

    assert_eq!(expected_tree, actual_tree);
}

#[test]
fn test_links_url_empty_title_single_quotes() {
    let input = include_str!("./fixtures/input/links-url-empty-title-single-quotes.text");
    let output = include_str!("./fixtures/tree/links-url-empty-title-single-quotes.json");

    let mdast = to_mdast(input, &ParseOptions::default()).expect("could not parse fixture's input to mdast");

    let expected_tree: serde_json::Value = serde_json::from_str(output).expect("a fixture's tree contains invalid json");

    let actual_tree = serde_json::to_value(&mdast).expect("could not serialize mdast to json");

    assert_eq!(expected_tree, actual_tree);
}

#[test]
fn test_links_url_empty() {
    let input = include_str!("./fixtures/input/links-url-empty.text");
    let output = include_str!("./fixtures/tree/links-url-empty.json");

    let mdast = to_mdast(input, &ParseOptions::default()).expect("could not parse fixture's input to mdast");

    let expected_tree: serde_json::Value = serde_json::from_str(output).expect("a fixture's tree contains invalid json");

    let actual_tree = serde_json::to_value(&mdast).expect("could not serialize mdast to json");

    assert_eq!(expected_tree, actual_tree);
}

#[test]
fn test_links_url_entities() {
    let input = include_str!("./fixtures/input/links-url-entities.text");
    let output = include_str!("./fixtures/tree/links-url-entities.json");

    let mdast = to_mdast(input, &ParseOptions::default()).expect("could not parse fixture's input to mdast");

    let expected_tree: serde_json::Value = serde_json::from_str(output).expect("a fixture's tree contains invalid json");

    let actual_tree = serde_json::to_value(&mdast).expect("could not serialize mdast to json");

    assert_eq!(expected_tree, actual_tree);
}

#[test]
fn test_links_url_entity_parentheses() {
    let input = include_str!("./fixtures/input/links-url-entity-parentheses.text");
    let output = include_str!("./fixtures/tree/links-url-entity-parentheses.json");

    let mdast = to_mdast(input, &ParseOptions::default()).expect("could not parse fixture's input to mdast");

    let expected_tree: serde_json::Value = serde_json::from_str(output).expect("a fixture's tree contains invalid json");

    let actual_tree = serde_json::to_value(&mdast).expect("could not serialize mdast to json");

    assert_eq!(expected_tree, actual_tree);
}

#[test]
fn test_links_url_escaped_parentheses() {
    let input = include_str!("./fixtures/input/links-url-escaped-parentheses.text");
    let output = include_str!("./fixtures/tree/links-url-escaped-parentheses.json");

    let mdast = to_mdast(input, &ParseOptions::default()).expect("could not parse fixture's input to mdast");

    let expected_tree: serde_json::Value = serde_json::from_str(output).expect("a fixture's tree contains invalid json");

    let actual_tree = serde_json::to_value(&mdast).expect("could not serialize mdast to json");

    assert_eq!(expected_tree, actual_tree);
}

#[test]
fn test_links_url_mismatched_parentheses() {
    let input = include_str!("./fixtures/input/links-url-mismatched-parentheses.text");
    let output = include_str!("./fixtures/tree/links-url-mismatched-parentheses.json");

    let mdast = to_mdast(input, &ParseOptions::default()).expect("could not parse fixture's input to mdast");

    let expected_tree: serde_json::Value = serde_json::from_str(output).expect("a fixture's tree contains invalid json");

    let actual_tree = serde_json::to_value(&mdast).expect("could not serialize mdast to json");

    assert_eq!(expected_tree, actual_tree);
}

#[test]
fn test_links_url_nested_parentheses() {
    let input = include_str!("./fixtures/input/links-url-nested-parentheses.text");
    let output = include_str!("./fixtures/tree/links-url-nested-parentheses.json");

    let mdast = to_mdast(input, &ParseOptions::default()).expect("could not parse fixture's input to mdast");

    let expected_tree: serde_json::Value = serde_json::from_str(output).expect("a fixture's tree contains invalid json");

    let actual_tree = serde_json::to_value(&mdast).expect("could not serialize mdast to json");

    assert_eq!(expected_tree, actual_tree);
}

#[test]
fn test_links_url_new_line() {
    let input = include_str!("./fixtures/input/links-url-new-line.text");
    let output = include_str!("./fixtures/tree/links-url-new-line.json");

    let mdast = to_mdast(input, &ParseOptions::default()).expect("could not parse fixture's input to mdast");

    let expected_tree: serde_json::Value = serde_json::from_str(output).expect("a fixture's tree contains invalid json");

    let actual_tree = serde_json::to_value(&mdast).expect("could not serialize mdast to json");

    assert_eq!(expected_tree, actual_tree);
}

#[test]
fn test_links_url_unclosed() {
    let input = include_str!("./fixtures/input/links-url-unclosed.text");
    let output = include_str!("./fixtures/tree/links-url-unclosed.json");

    let mdast = to_mdast(input, &ParseOptions::default()).expect("could not parse fixture's input to mdast");

    let expected_tree: serde_json::Value = serde_json::from_str(output).expect("a fixture's tree contains invalid json");

    let actual_tree = serde_json::to_value(&mdast).expect("could not serialize mdast to json");

    assert_eq!(expected_tree, actual_tree);
}

#[test]
fn test_links_url_white_space() {
    let input = include_str!("./fixtures/input/links-url-white-space.text");
    let output = include_str!("./fixtures/tree/links-url-white-space.json");

    let mdast = to_mdast(input, &ParseOptions::default()).expect("could not parse fixture's input to mdast");

    let expected_tree: serde_json::Value = serde_json::from_str(output).expect("a fixture's tree contains invalid json");

    let actual_tree = serde_json::to_value(&mdast).expect("could not serialize mdast to json");

    assert_eq!(expected_tree, actual_tree);
}

#[test]
fn test_links_output_noreference_links() {
    let input = include_str!("./fixtures/input/links.output.noreference-links.text");
    let output = include_str!("./fixtures/tree/links.output.noreference-links.json");

    let mdast = to_mdast(input, &ParseOptions::default()).expect("could not parse fixture's input to mdast");

    let expected_tree: serde_json::Value = serde_json::from_str(output).expect("a fixture's tree contains invalid json");

    let actual_tree = serde_json::to_value(&mdast).expect("could not serialize mdast to json");

    assert_eq!(expected_tree, actual_tree);
}

#[test]
fn test_list_after_list() {
    let input = include_str!("./fixtures/input/list-after-list.text");
    let output = include_str!("./fixtures/tree/list-after-list.json");

    let mdast = to_mdast(input, &ParseOptions::default()).expect("could not parse fixture's input to mdast");

    let expected_tree: serde_json::Value = serde_json::from_str(output).expect("a fixture's tree contains invalid json");

    let actual_tree = serde_json::to_value(&mdast).expect("could not serialize mdast to json");

    assert_eq!(expected_tree, actual_tree);
}

#[test]
fn test_list_and_code() {
    let input = include_str!("./fixtures/input/list-and-code.text");
    let output = include_str!("./fixtures/tree/list-and-code.json");

    let mdast = to_mdast(input, &ParseOptions::default()).expect("could not parse fixture's input to mdast");

    let expected_tree: serde_json::Value = serde_json::from_str(output).expect("a fixture's tree contains invalid json");

    let actual_tree = serde_json::to_value(&mdast).expect("could not serialize mdast to json");

    assert_eq!(expected_tree, actual_tree);
}

#[test]
fn test_list_continuation() {
    let input = include_str!("./fixtures/input/list-continuation.text");
    let output = include_str!("./fixtures/tree/list-continuation.json");

    let mdast = to_mdast(input, &ParseOptions::default()).expect("could not parse fixture's input to mdast");

    let expected_tree: serde_json::Value = serde_json::from_str(output).expect("a fixture's tree contains invalid json");

    let actual_tree = serde_json::to_value(&mdast).expect("could not serialize mdast to json");

    assert_eq!(expected_tree, actual_tree);
}

#[test]
fn test_list_indentation_nooutput() {
    let input = include_str!("./fixtures/input/list-indentation.nooutput.text");
    let output = include_str!("./fixtures/tree/list-indentation.nooutput.json");

    let mdast = to_mdast(input, &ParseOptions::default()).expect("could not parse fixture's input to mdast");

    let expected_tree: serde_json::Value = serde_json::from_str(output).expect("a fixture's tree contains invalid json");

    let actual_tree = serde_json::to_value(&mdast).expect("could not serialize mdast to json");

    assert_eq!(expected_tree, actual_tree);
}

#[test]
fn test_list_interrupt() {
    let input = include_str!("./fixtures/input/list-interrupt.text");
    let output = include_str!("./fixtures/tree/list-interrupt.json");

    let mdast = to_mdast(input, &ParseOptions::default()).expect("could not parse fixture's input to mdast");

    let expected_tree: serde_json::Value = serde_json::from_str(output).expect("a fixture's tree contains invalid json");

    let actual_tree = serde_json::to_value(&mdast).expect("could not serialize mdast to json");

    assert_eq!(expected_tree, actual_tree);
}

#[test]
fn test_list_item_empty_with_white_space() {
    let input = include_str!("./fixtures/input/list-item-empty-with-white-space.text");
    let output = include_str!("./fixtures/tree/list-item-empty-with-white-space.json");

    let mdast = to_mdast(input, &ParseOptions::default()).expect("could not parse fixture's input to mdast");

    let expected_tree: serde_json::Value = serde_json::from_str(output).expect("a fixture's tree contains invalid json");

    let actual_tree = serde_json::to_value(&mdast).expect("could not serialize mdast to json");

    assert_eq!(expected_tree, actual_tree);
}

#[test]
fn test_list_item_empty_without_white_space_eof() {
    let input = include_str!("./fixtures/input/list-item-empty-without-white-space-eof.text");
    let output = include_str!("./fixtures/tree/list-item-empty-without-white-space-eof.json");

    let mdast = to_mdast(input, &ParseOptions::default()).expect("could not parse fixture's input to mdast");

    let expected_tree: serde_json::Value = serde_json::from_str(output).expect("a fixture's tree contains invalid json");

    let actual_tree = serde_json::to_value(&mdast).expect("could not serialize mdast to json");

    assert_eq!(expected_tree, actual_tree);
}

#[test]
fn test_list_item_empty_without_white_space() {
    let input = include_str!("./fixtures/input/list-item-empty-without-white-space.text");
    let output = include_str!("./fixtures/tree/list-item-empty-without-white-space.json");

    let mdast = to_mdast(input, &ParseOptions::default()).expect("could not parse fixture's input to mdast");

    let expected_tree: serde_json::Value = serde_json::from_str(output).expect("a fixture's tree contains invalid json");

    let actual_tree = serde_json::to_value(&mdast).expect("could not serialize mdast to json");

    assert_eq!(expected_tree, actual_tree);
}

#[test]
fn test_list_item_empty() {
    let input = include_str!("./fixtures/input/list-item-empty.text");
    let output = include_str!("./fixtures/tree/list-item-empty.json");

    let mdast = to_mdast(input, &ParseOptions::default()).expect("could not parse fixture's input to mdast");

    let expected_tree: serde_json::Value = serde_json::from_str(output).expect("a fixture's tree contains invalid json");

    let actual_tree = serde_json::to_value(&mdast).expect("could not serialize mdast to json");

    assert_eq!(expected_tree, actual_tree);
}

#[test]
fn test_list_item_indent_list_item_indent_1_output() {
    let input = include_str!("./fixtures/input/list-item-indent.list-item-indent=1.output.text");
    let output = include_str!("./fixtures/tree/list-item-indent.list-item-indent=1.output.json");

    let mdast = to_mdast(input, &ParseOptions::default()).expect("could not parse fixture's input to mdast");

    let expected_tree: serde_json::Value = serde_json::from_str(output).expect("a fixture's tree contains invalid json");

    let actual_tree = serde_json::to_value(&mdast).expect("could not serialize mdast to json");

    assert_eq!(expected_tree, actual_tree);
}

#[test]
fn test_list_item_indent_list_item_indent_mixed_output() {
    let input = include_str!("./fixtures/input/list-item-indent.list-item-indent=mixed.output.text");
    let output = include_str!("./fixtures/tree/list-item-indent.list-item-indent=mixed.output.json");

    let mdast = to_mdast(input, &ParseOptions::default()).expect("could not parse fixture's input to mdast");

    let expected_tree: serde_json::Value = serde_json::from_str(output).expect("a fixture's tree contains invalid json");

    let actual_tree = serde_json::to_value(&mdast).expect("could not serialize mdast to json");

    assert_eq!(expected_tree, actual_tree);
}

#[test]
fn test_list_item_indent_list_item_indent_tab_output() {
    let input = include_str!("./fixtures/input/list-item-indent.list-item-indent=tab.output.text");
    let output = include_str!("./fixtures/tree/list-item-indent.list-item-indent=tab.output.json");

    let mdast = to_mdast(input, &ParseOptions::default()).expect("could not parse fixture's input to mdast");

    let expected_tree: serde_json::Value = serde_json::from_str(output).expect("a fixture's tree contains invalid json");

    let actual_tree = serde_json::to_value(&mdast).expect("could not serialize mdast to json");

    assert_eq!(expected_tree, actual_tree);
}

#[test]
fn test_list_item_newline_nooutput() {
    let input = include_str!("./fixtures/input/list-item-newline.nooutput.text");
    let output = include_str!("./fixtures/tree/list-item-newline.nooutput.json");

    let mdast = to_mdast(input, &ParseOptions::default()).expect("could not parse fixture's input to mdast");

    let expected_tree: serde_json::Value = serde_json::from_str(output).expect("a fixture's tree contains invalid json");

    let actual_tree = serde_json::to_value(&mdast).expect("could not serialize mdast to json");

    assert_eq!(expected_tree, actual_tree);
}

#[test]
fn test_list_item_text() {
    let input = include_str!("./fixtures/input/list-item-text.text");
    let output = include_str!("./fixtures/tree/list-item-text.json");

    let mdast = to_mdast(input, &ParseOptions::default()).expect("could not parse fixture's input to mdast");

    let expected_tree: serde_json::Value = serde_json::from_str(output).expect("a fixture's tree contains invalid json");

    let actual_tree = serde_json::to_value(&mdast).expect("could not serialize mdast to json");

    assert_eq!(expected_tree, actual_tree);
}

#[test]
fn test_list_mixed_indentation() {
    let input = include_str!("./fixtures/input/list-mixed-indentation.text");
    let output = include_str!("./fixtures/tree/list-mixed-indentation.json");

    let mdast = to_mdast(input, &ParseOptions::default()).expect("could not parse fixture's input to mdast");

    let expected_tree: serde_json::Value = serde_json::from_str(output).expect("a fixture's tree contains invalid json");

    let actual_tree = serde_json::to_value(&mdast).expect("could not serialize mdast to json");

    assert_eq!(expected_tree, actual_tree);
}

#[test]
fn test_list_ordered_empty_no_space_single_item() {
    let input = include_str!("./fixtures/input/list-ordered-empty-no-space-single-item.text");
    let output = include_str!("./fixtures/tree/list-ordered-empty-no-space-single-item.json");

    let mdast = to_mdast(input, &ParseOptions::default()).expect("could not parse fixture's input to mdast");

    let expected_tree: serde_json::Value = serde_json::from_str(output).expect("a fixture's tree contains invalid json");

    let actual_tree = serde_json::to_value(&mdast).expect("could not serialize mdast to json");

    assert_eq!(expected_tree, actual_tree);
}

#[test]
fn test_list_ordered_empty_no_space() {
    let input = include_str!("./fixtures/input/list-ordered-empty-no-space.text");
    let output = include_str!("./fixtures/tree/list-ordered-empty-no-space.json");

    let mdast = to_mdast(input, &ParseOptions::default()).expect("could not parse fixture's input to mdast");

    let expected_tree: serde_json::Value = serde_json::from_str(output).expect("a fixture's tree contains invalid json");

    let actual_tree = serde_json::to_value(&mdast).expect("could not serialize mdast to json");

    assert_eq!(expected_tree, actual_tree);
}

#[test]
fn test_list_ordered_increment_list_marker_output() {
    let input = include_str!("./fixtures/input/list-ordered.increment-list-marker.output.text");
    let output = include_str!("./fixtures/tree/list-ordered.increment-list-marker.output.json");

    let mdast = to_mdast(input, &ParseOptions::default()).expect("could not parse fixture's input to mdast");

    let expected_tree: serde_json::Value = serde_json::from_str(output).expect("a fixture's tree contains invalid json");

    let actual_tree = serde_json::to_value(&mdast).expect("could not serialize mdast to json");

    assert_eq!(expected_tree, actual_tree);
}

#[test]
fn test_list_ordered_noincrement_list_marker_output() {
    let input = include_str!("./fixtures/input/list-ordered.noincrement-list-marker.output.text");
    let output = include_str!("./fixtures/tree/list-ordered.noincrement-list-marker.output.json");

    let mdast = to_mdast(input, &ParseOptions::default()).expect("could not parse fixture's input to mdast");

    let expected_tree: serde_json::Value = serde_json::from_str(output).expect("a fixture's tree contains invalid json");

    let actual_tree = serde_json::to_value(&mdast).expect("could not serialize mdast to json");

    assert_eq!(expected_tree, actual_tree);
}

#[test]
fn test_list_unordered_empty_no_space_single_item() {
    let input = include_str!("./fixtures/input/list-unordered-empty-no-space-single-item.text");
    let output = include_str!("./fixtures/tree/list-unordered-empty-no-space-single-item.json");

    let mdast = to_mdast(input, &ParseOptions::default()).expect("could not parse fixture's input to mdast");

    let expected_tree: serde_json::Value = serde_json::from_str(output).expect("a fixture's tree contains invalid json");

    let actual_tree = serde_json::to_value(&mdast).expect("could not serialize mdast to json");

    assert_eq!(expected_tree, actual_tree);
}

#[test]
fn test_list_unordered_empty_no_space() {
    let input = include_str!("./fixtures/input/list-unordered-empty-no-space.text");
    let output = include_str!("./fixtures/tree/list-unordered-empty-no-space.json");

    let mdast = to_mdast(input, &ParseOptions::default()).expect("could not parse fixture's input to mdast");

    let expected_tree: serde_json::Value = serde_json::from_str(output).expect("a fixture's tree contains invalid json");

    let actual_tree = serde_json::to_value(&mdast).expect("could not serialize mdast to json");

    assert_eq!(expected_tree, actual_tree);
}

#[test]
fn test_list_output_bullet() {
    let input = include_str!("./fixtures/input/list.output.bullet=+.text");
    let output = include_str!("./fixtures/tree/list.output.bullet=+.json");

    let mdast = to_mdast(input, &ParseOptions::default()).expect("could not parse fixture's input to mdast");

    let expected_tree: serde_json::Value = serde_json::from_str(output).expect("a fixture's tree contains invalid json");

    let actual_tree = serde_json::to_value(&mdast).expect("could not serialize mdast to json");

    assert_eq!(expected_tree, actual_tree);
}

#[test]
fn test_list_output_bullet_2() {
    let input = include_str!("./fixtures/input/list.output.bullet=-.text");
    let output = include_str!("./fixtures/tree/list.output.bullet=-.json");

    let mdast = to_mdast(input, &ParseOptions::default()).expect("could not parse fixture's input to mdast");

    let expected_tree: serde_json::Value = serde_json::from_str(output).expect("a fixture's tree contains invalid json");

    let actual_tree = serde_json::to_value(&mdast).expect("could not serialize mdast to json");

    assert_eq!(expected_tree, actual_tree);
}

#[test]
fn test_list_output_bullet_asterisk() {
    let input = include_str!("./fixtures/input/list.output.bullet=-asterisk-.text");
    let output = include_str!("./fixtures/tree/list.output.bullet=-asterisk-.json");

    let mdast = to_mdast(input, &ParseOptions::default()).expect("could not parse fixture's input to mdast");

    let expected_tree: serde_json::Value = serde_json::from_str(output).expect("a fixture's tree contains invalid json");

    let actual_tree = serde_json::to_value(&mdast).expect("could not serialize mdast to json");

    assert_eq!(expected_tree, actual_tree);
}

#[test]
fn test_lists_with_code_and_rules() {
    let input = include_str!("./fixtures/input/lists-with-code-and-rules.text");
    let output = include_str!("./fixtures/tree/lists-with-code-and-rules.json");

    let mdast = to_mdast(input, &ParseOptions::default()).expect("could not parse fixture's input to mdast");

    let expected_tree: serde_json::Value = serde_json::from_str(output).expect("a fixture's tree contains invalid json");

    let actual_tree = serde_json::to_value(&mdast).expect("could not serialize mdast to json");

    assert_eq!(expected_tree, actual_tree);
}

#[test]
fn test_literal_email() {
    let input = include_str!("./fixtures/input/literal-email.text");
    let output = include_str!("./fixtures/tree/literal-email.json");

    let mdast = to_mdast(input, &ParseOptions::default()).expect("could not parse fixture's input to mdast");

    let expected_tree: serde_json::Value = serde_json::from_str(output).expect("a fixture's tree contains invalid json");

    let actual_tree = serde_json::to_value(&mdast).expect("could not serialize mdast to json");

    assert_eq!(expected_tree, actual_tree);
}

#[test]
fn test_literal_url() {
    let input = include_str!("./fixtures/input/literal-url.text");
    let output = include_str!("./fixtures/tree/literal-url.json");

    let mdast = to_mdast(input, &ParseOptions::default()).expect("could not parse fixture's input to mdast");

    let expected_tree: serde_json::Value = serde_json::from_str(output).expect("a fixture's tree contains invalid json");

    let actual_tree = serde_json::to_value(&mdast).expect("could not serialize mdast to json");

    assert_eq!(expected_tree, actual_tree);
}

#[test]
fn test_loose_lists() {
    let input = include_str!("./fixtures/input/loose-lists.text");
    let output = include_str!("./fixtures/tree/loose-lists.json");

    let mdast = to_mdast(input, &ParseOptions::default()).expect("could not parse fixture's input to mdast");

    let expected_tree: serde_json::Value = serde_json::from_str(output).expect("a fixture's tree contains invalid json");

    let actual_tree = serde_json::to_value(&mdast).expect("could not serialize mdast to json");

    assert_eq!(expected_tree, actual_tree);
}

#[test]
fn test_main() {
    let input = include_str!("./fixtures/input/main.text");
    let output = include_str!("./fixtures/tree/main.json");

    let mdast = to_mdast(input, &ParseOptions::default()).expect("could not parse fixture's input to mdast");

    let expected_tree: serde_json::Value = serde_json::from_str(output).expect("a fixture's tree contains invalid json");

    let actual_tree = serde_json::to_value(&mdast).expect("could not serialize mdast to json");

    assert_eq!(expected_tree, actual_tree);
}

#[test]
fn test_markdown_documentation_basics() {
    let input = include_str!("./fixtures/input/markdown-documentation-basics.text");
    let output = include_str!("./fixtures/tree/markdown-documentation-basics.json");

    let mdast = to_mdast(input, &ParseOptions::default()).expect("could not parse fixture's input to mdast");

    let expected_tree: serde_json::Value = serde_json::from_str(output).expect("a fixture's tree contains invalid json");

    let actual_tree = serde_json::to_value(&mdast).expect("could not serialize mdast to json");

    assert_eq!(expected_tree, actual_tree);
}

#[test]
fn test_markdown_documentation_syntax() {
    let input = include_str!("./fixtures/input/markdown-documentation-syntax.text");
    let output = include_str!("./fixtures/tree/markdown-documentation-syntax.json");

    let mdast = to_mdast(input, &ParseOptions::default()).expect("could not parse fixture's input to mdast");

    let expected_tree: serde_json::Value = serde_json::from_str(output).expect("a fixture's tree contains invalid json");

    let actual_tree = serde_json::to_value(&mdast).expect("could not serialize mdast to json");

    assert_eq!(expected_tree, actual_tree);
}

#[test]
fn test_mixed_indentation() {
    let input = include_str!("./fixtures/input/mixed-indentation.text");
    let output = include_str!("./fixtures/tree/mixed-indentation.json");

    let mdast = to_mdast(input, &ParseOptions::default()).expect("could not parse fixture's input to mdast");

    let expected_tree: serde_json::Value = serde_json::from_str(output).expect("a fixture's tree contains invalid json");

    let actual_tree = serde_json::to_value(&mdast).expect("could not serialize mdast to json");

    assert_eq!(expected_tree, actual_tree);
}

#[test]
fn test_nested_blockquotes() {
    let input = include_str!("./fixtures/input/nested-blockquotes.text");
    let output = include_str!("./fixtures/tree/nested-blockquotes.json");

    let mdast = to_mdast(input, &ParseOptions::default()).expect("could not parse fixture's input to mdast");

    let expected_tree: serde_json::Value = serde_json::from_str(output).expect("a fixture's tree contains invalid json");

    let actual_tree = serde_json::to_value(&mdast).expect("could not serialize mdast to json");

    assert_eq!(expected_tree, actual_tree);
}

#[test]
fn test_nested_code() {
    let input = include_str!("./fixtures/input/nested-code.text");
    let output = include_str!("./fixtures/tree/nested-code.json");

    let mdast = to_mdast(input, &ParseOptions::default()).expect("could not parse fixture's input to mdast");

    let expected_tree: serde_json::Value = serde_json::from_str(output).expect("a fixture's tree contains invalid json");

    let actual_tree = serde_json::to_value(&mdast).expect("could not serialize mdast to json");

    assert_eq!(expected_tree, actual_tree);
}

#[test]
fn test_nested_em_nooutput() {
    let input = include_str!("./fixtures/input/nested-em.nooutput.text");
    let output = include_str!("./fixtures/tree/nested-em.nooutput.json");

    let mdast = to_mdast(input, &ParseOptions::default()).expect("could not parse fixture's input to mdast");

    let expected_tree: serde_json::Value = serde_json::from_str(output).expect("a fixture's tree contains invalid json");

    let actual_tree = serde_json::to_value(&mdast).expect("could not serialize mdast to json");

    assert_eq!(expected_tree, actual_tree);
}

#[test]
fn test_nested_references() {
    let input = include_str!("./fixtures/input/nested-references.text");
    let output = include_str!("./fixtures/tree/nested-references.json");

    let mdast = to_mdast(input, &ParseOptions::default()).expect("could not parse fixture's input to mdast");

    let expected_tree: serde_json::Value = serde_json::from_str(output).expect("a fixture's tree contains invalid json");

    let actual_tree = serde_json::to_value(&mdast).expect("could not serialize mdast to json");

    assert_eq!(expected_tree, actual_tree);
}

#[test]
fn test_nested_square_link() {
    let input = include_str!("./fixtures/input/nested-square-link.text");
    let output = include_str!("./fixtures/tree/nested-square-link.json");

    let mdast = to_mdast(input, &ParseOptions::default()).expect("could not parse fixture's input to mdast");

    let expected_tree: serde_json::Value = serde_json::from_str(output).expect("a fixture's tree contains invalid json");

    let actual_tree = serde_json::to_value(&mdast).expect("could not serialize mdast to json");

    assert_eq!(expected_tree, actual_tree);
}

#[test]
fn test_not_a_link() {
    let input = include_str!("./fixtures/input/not-a-link.text");
    let output = include_str!("./fixtures/tree/not-a-link.json");

    let mdast = to_mdast(input, &ParseOptions::default()).expect("could not parse fixture's input to mdast");

    let expected_tree: serde_json::Value = serde_json::from_str(output).expect("a fixture's tree contains invalid json");

    let actual_tree = serde_json::to_value(&mdast).expect("could not serialize mdast to json");

    assert_eq!(expected_tree, actual_tree);
}

#[test]
fn test_ordered_and_unordered_lists() {
    let input = include_str!("./fixtures/input/ordered-and-unordered-lists.text");
    let output = include_str!("./fixtures/tree/ordered-and-unordered-lists.json");

    let mdast = to_mdast(input, &ParseOptions::default()).expect("could not parse fixture's input to mdast");

    let expected_tree: serde_json::Value = serde_json::from_str(output).expect("a fixture's tree contains invalid json");

    let actual_tree = serde_json::to_value(&mdast).expect("could not serialize mdast to json");

    assert_eq!(expected_tree, actual_tree);
}

#[test]
fn test_ordered_different_types_nooutput() {
    let input = include_str!("./fixtures/input/ordered-different-types.nooutput.text");
    let output = include_str!("./fixtures/tree/ordered-different-types.nooutput.json");

    let mdast = to_mdast(input, &ParseOptions::default()).expect("could not parse fixture's input to mdast");

    let expected_tree: serde_json::Value = serde_json::from_str(output).expect("a fixture's tree contains invalid json");

    let actual_tree = serde_json::to_value(&mdast).expect("could not serialize mdast to json");

    assert_eq!(expected_tree, actual_tree);
}

#[test]
fn test_ordered_with_parentheses() {
    let input = include_str!("./fixtures/input/ordered-with-parentheses.text");
    let output = include_str!("./fixtures/tree/ordered-with-parentheses.json");

    let mdast = to_mdast(input, &ParseOptions::default()).expect("could not parse fixture's input to mdast");

    let expected_tree: serde_json::Value = serde_json::from_str(output).expect("a fixture's tree contains invalid json");

    let actual_tree = serde_json::to_value(&mdast).expect("could not serialize mdast to json");

    assert_eq!(expected_tree, actual_tree);
}

#[test]
fn test_paragraphs_and_indentation() {
    let input = include_str!("./fixtures/input/paragraphs-and-indentation.text");
    let output = include_str!("./fixtures/tree/paragraphs-and-indentation.json");

    let mdast = to_mdast(input, &ParseOptions::default()).expect("could not parse fixture's input to mdast");

    let expected_tree: serde_json::Value = serde_json::from_str(output).expect("a fixture's tree contains invalid json");

    let actual_tree = serde_json::to_value(&mdast).expect("could not serialize mdast to json");

    assert_eq!(expected_tree, actual_tree);
}

#[test]
fn test_paragraphs_empty() {
    let input = include_str!("./fixtures/input/paragraphs-empty.text");
    let output = include_str!("./fixtures/tree/paragraphs-empty.json");

    let mdast = to_mdast(input, &ParseOptions::default()).expect("could not parse fixture's input to mdast");

    let expected_tree: serde_json::Value = serde_json::from_str(output).expect("a fixture's tree contains invalid json");

    let actual_tree = serde_json::to_value(&mdast).expect("could not serialize mdast to json");

    assert_eq!(expected_tree, actual_tree);
}

#[test]
fn test_ref_paren() {
    let input = include_str!("./fixtures/input/ref-paren.text");
    let output = include_str!("./fixtures/tree/ref-paren.json");

    let mdast = to_mdast(input, &ParseOptions::default()).expect("could not parse fixture's input to mdast");

    let expected_tree: serde_json::Value = serde_json::from_str(output).expect("a fixture's tree contains invalid json");

    let actual_tree = serde_json::to_value(&mdast).expect("could not serialize mdast to json");

    assert_eq!(expected_tree, actual_tree);
}

#[test]
fn test_reference_image_empty_alt() {
    let input = include_str!("./fixtures/input/reference-image-empty-alt.text");
    let output = include_str!("./fixtures/tree/reference-image-empty-alt.json");

    let mdast = to_mdast(input, &ParseOptions::default()).expect("could not parse fixture's input to mdast");

    let expected_tree: serde_json::Value = serde_json::from_str(output).expect("a fixture's tree contains invalid json");

    let actual_tree = serde_json::to_value(&mdast).expect("could not serialize mdast to json");

    assert_eq!(expected_tree, actual_tree);
}

#[test]
fn test_reference_link_escape_nooutput() {
    let input = include_str!("./fixtures/input/reference-link-escape.nooutput.text");
    let output = include_str!("./fixtures/tree/reference-link-escape.nooutput.json");

    let mdast = to_mdast(input, &ParseOptions::default()).expect("could not parse fixture's input to mdast");

    let expected_tree: serde_json::Value = serde_json::from_str(output).expect("a fixture's tree contains invalid json");

    let actual_tree = serde_json::to_value(&mdast).expect("could not serialize mdast to json");

    assert_eq!(expected_tree, actual_tree);
}

#[test]
fn test_reference_link_not_closed() {
    let input = include_str!("./fixtures/input/reference-link-not-closed.text");
    let output = include_str!("./fixtures/tree/reference-link-not-closed.json");

    let mdast = to_mdast(input, &ParseOptions::default()).expect("could not parse fixture's input to mdast");

    let expected_tree: serde_json::Value = serde_json::from_str(output).expect("a fixture's tree contains invalid json");

    let actual_tree = serde_json::to_value(&mdast).expect("could not serialize mdast to json");

    assert_eq!(expected_tree, actual_tree);
}

#[test]
fn test_reference_link_with_angle_brackets() {
    let input = include_str!("./fixtures/input/reference-link-with-angle-brackets.text");
    let output = include_str!("./fixtures/tree/reference-link-with-angle-brackets.json");

    let mdast = to_mdast(input, &ParseOptions::default()).expect("could not parse fixture's input to mdast");

    let expected_tree: serde_json::Value = serde_json::from_str(output).expect("a fixture's tree contains invalid json");

    let actual_tree = serde_json::to_value(&mdast).expect("could not serialize mdast to json");

    assert_eq!(expected_tree, actual_tree);
}

#[test]
fn test_reference_link_with_multiple_definitions() {
    let input = include_str!("./fixtures/input/reference-link-with-multiple-definitions.text");
    let output = include_str!("./fixtures/tree/reference-link-with-multiple-definitions.json");

    let mdast = to_mdast(input, &ParseOptions::default()).expect("could not parse fixture's input to mdast");

    let expected_tree: serde_json::Value = serde_json::from_str(output).expect("a fixture's tree contains invalid json");

    let actual_tree = serde_json::to_value(&mdast).expect("could not serialize mdast to json");

    assert_eq!(expected_tree, actual_tree);
}

#[test]
fn test_remarkjs_remark_lint_gh_111() {
    let input = include_str!("./fixtures/input/remarkjs-remark-lint-gh-111.text");
    let output = include_str!("./fixtures/tree/remarkjs-remark-lint-gh-111.json");

    let mdast = to_mdast(input, &ParseOptions::default()).expect("could not parse fixture's input to mdast");

    let expected_tree: serde_json::Value = serde_json::from_str(output).expect("a fixture's tree contains invalid json");

    let actual_tree = serde_json::to_value(&mdast).expect("could not serialize mdast to json");

    assert_eq!(expected_tree, actual_tree);
}

#[test]
fn test_same_bullet_nooutput() {
    let input = include_str!("./fixtures/input/same-bullet.nooutput.text");
    let output = include_str!("./fixtures/tree/same-bullet.nooutput.json");

    let mdast = to_mdast(input, &ParseOptions::default()).expect("could not parse fixture's input to mdast");

    let expected_tree: serde_json::Value = serde_json::from_str(output).expect("a fixture's tree contains invalid json");

    let actual_tree = serde_json::to_value(&mdast).expect("could not serialize mdast to json");

    assert_eq!(expected_tree, actual_tree);
}

#[test]
fn test_stringify_escape() {
    let input = include_str!("./fixtures/input/stringify-escape.text");
    let output = include_str!("./fixtures/tree/stringify-escape.json");

    let mdast = to_mdast(input, &ParseOptions::default()).expect("could not parse fixture's input to mdast");

    let expected_tree: serde_json::Value = serde_json::from_str(output).expect("a fixture's tree contains invalid json");

    let actual_tree = serde_json::to_value(&mdast).expect("could not serialize mdast to json");

    assert_eq!(expected_tree, actual_tree);
}

#[test]
fn test_strong_and_em_together_one() {
    let input = include_str!("./fixtures/input/strong-and-em-together-one.text");
    let output = include_str!("./fixtures/tree/strong-and-em-together-one.json");

    let mdast = to_mdast(input, &ParseOptions::default()).expect("could not parse fixture's input to mdast");

    let expected_tree: serde_json::Value = serde_json::from_str(output).expect("a fixture's tree contains invalid json");

    let actual_tree = serde_json::to_value(&mdast).expect("could not serialize mdast to json");

    assert_eq!(expected_tree, actual_tree);
}

#[test]
fn test_strong_and_em_together_two_nooutput() {
    let input = include_str!("./fixtures/input/strong-and-em-together-two.nooutput.text");
    let output = include_str!("./fixtures/tree/strong-and-em-together-two.nooutput.json");

    let mdast = to_mdast(input, &ParseOptions::default()).expect("could not parse fixture's input to mdast");

    let expected_tree: serde_json::Value = serde_json::from_str(output).expect("a fixture's tree contains invalid json");

    let actual_tree = serde_json::to_value(&mdast).expect("could not serialize mdast to json");

    assert_eq!(expected_tree, actual_tree);
}

#[test]
fn test_strong_emphasis() {
    let input = include_str!("./fixtures/input/strong-emphasis.text");
    let output = include_str!("./fixtures/tree/strong-emphasis.json");

    let mdast = to_mdast(input, &ParseOptions::default()).expect("could not parse fixture's input to mdast");

    let expected_tree: serde_json::Value = serde_json::from_str(output).expect("a fixture's tree contains invalid json");

    let actual_tree = serde_json::to_value(&mdast).expect("could not serialize mdast to json");

    assert_eq!(expected_tree, actual_tree);
}

#[test]
fn test_strong_initial_white_space() {
    let input = include_str!("./fixtures/input/strong-initial-white-space.text");
    let output = include_str!("./fixtures/tree/strong-initial-white-space.json");

    let mdast = to_mdast(input, &ParseOptions::default()).expect("could not parse fixture's input to mdast");

    let expected_tree: serde_json::Value = serde_json::from_str(output).expect("a fixture's tree contains invalid json");

    let actual_tree = serde_json::to_value(&mdast).expect("could not serialize mdast to json");

    assert_eq!(expected_tree, actual_tree);
}

#[test]
fn test_tabs_and_spaces() {
    let input = include_str!("./fixtures/input/tabs-and-spaces.text");
    let output = include_str!("./fixtures/tree/tabs-and-spaces.json");

    let mdast = to_mdast(input, &ParseOptions::default()).expect("could not parse fixture's input to mdast");

    let expected_tree: serde_json::Value = serde_json::from_str(output).expect("a fixture's tree contains invalid json");

    let actual_tree = serde_json::to_value(&mdast).expect("could not serialize mdast to json");

    assert_eq!(expected_tree, actual_tree);
}

#[test]
fn test_tabs() {
    let input = include_str!("./fixtures/input/tabs.text");
    let output = include_str!("./fixtures/tree/tabs.json");

    let mdast = to_mdast(input, &ParseOptions::default()).expect("could not parse fixture's input to mdast");

    let expected_tree: serde_json::Value = serde_json::from_str(output).expect("a fixture's tree contains invalid json");

    let actual_tree = serde_json::to_value(&mdast).expect("could not serialize mdast to json");

    assert_eq!(expected_tree, actual_tree);
}

#[test]
fn test_tidyness() {
    let input = include_str!("./fixtures/input/tidyness.text");
    let output = include_str!("./fixtures/tree/tidyness.json");

    let mdast = to_mdast(input, &ParseOptions::default()).expect("could not parse fixture's input to mdast");

    let expected_tree: serde_json::Value = serde_json::from_str(output).expect("a fixture's tree contains invalid json");

    let actual_tree = serde_json::to_value(&mdast).expect("could not serialize mdast to json");

    assert_eq!(expected_tree, actual_tree);
}

#[test]
fn test_title_attributes() {
    let input = include_str!("./fixtures/input/title-attributes.text");
    let output = include_str!("./fixtures/tree/title-attributes.json");

    let mdast = to_mdast(input, &ParseOptions::default()).expect("could not parse fixture's input to mdast");

    let expected_tree: serde_json::Value = serde_json::from_str(output).expect("a fixture's tree contains invalid json");

    let actual_tree = serde_json::to_value(&mdast).expect("could not serialize mdast to json");

    assert_eq!(expected_tree, actual_tree);
}

#[test]
fn test_toplevel_paragraphs_nooutput() {
    let input = include_str!("./fixtures/input/toplevel-paragraphs.nooutput.text");
    let output = include_str!("./fixtures/tree/toplevel-paragraphs.nooutput.json");

    let mdast = to_mdast(input, &ParseOptions::default()).expect("could not parse fixture's input to mdast");

    let expected_tree: serde_json::Value = serde_json::from_str(output).expect("a fixture's tree contains invalid json");

    let actual_tree = serde_json::to_value(&mdast).expect("could not serialize mdast to json");

    assert_eq!(expected_tree, actual_tree);
}

#[test]
fn test_tricky_list() {
    let input = include_str!("./fixtures/input/tricky-list.text");
    let output = include_str!("./fixtures/tree/tricky-list.json");

    let mdast = to_mdast(input, &ParseOptions::default()).expect("could not parse fixture's input to mdast");

    let expected_tree: serde_json::Value = serde_json::from_str(output).expect("a fixture's tree contains invalid json");

    let actual_tree = serde_json::to_value(&mdast).expect("could not serialize mdast to json");

    assert_eq!(expected_tree, actual_tree);
}

#[test]
fn test_simple() {
    let input = "hello world\n* how are you";
    let mdast = to_mdast(input, &ParseOptions::default()).expect("could not parse fixture's input to mdast");
    println!("{}", serde_json::to_string_pretty(&mdast).unwrap());
}
