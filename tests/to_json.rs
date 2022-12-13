#[cfg(feature = "json")]
mod to_json_tests {
    use markdown::{to_json, ParseOptions};
    use pretty_assertions::assert_eq;

    #[test]
    fn test_attention() {
        let input = include_str!("fixtures/to_json/attention.md");
        let output = include_str!("fixtures/to_json/attention.json");

        let actual_tree = to_json(input, &ParseOptions::default())
            .expect("could not parse fixture's input to mdast");

        let expected_tree: serde_json::Value =
            serde_json::from_str(output).expect("a fixture's tree contains invalid json");

        assert_eq!(expected_tree, actual_tree);
    }

    #[test]
    fn test_autolink() {
        let input = include_str!("fixtures/to_json/autolink.md");
        let output = include_str!("fixtures/to_json/autolink.json");

        let actual_tree = to_json(input, &ParseOptions::default())
            .expect("could not parse fixture's input to mdast");

        let expected_tree: serde_json::Value =
            serde_json::from_str(output).expect("a fixture's tree contains invalid json");

        assert_eq!(expected_tree, actual_tree);
    }

    #[test]
    fn test_blockquote() {
        let input = include_str!("fixtures/to_json/blockquote.md");
        let output = include_str!("fixtures/to_json/blockquote.json");

        let actual_tree = to_json(input, &ParseOptions::default())
            .expect("could not parse fixture's input to mdast");

        let expected_tree: serde_json::Value =
            serde_json::from_str(output).expect("a fixture's tree contains invalid json");

        assert_eq!(expected_tree, actual_tree);
    }

    #[test]
    fn test_character_escape() {
        let input = include_str!("fixtures/to_json/character-escape.md");
        let output = include_str!("fixtures/to_json/character-escape.json");

        let actual_tree = to_json(input, &ParseOptions::default())
            .expect("could not parse fixture's input to mdast");

        let expected_tree: serde_json::Value =
            serde_json::from_str(output).expect("a fixture's tree contains invalid json");

        assert_eq!(expected_tree, actual_tree);
    }

    #[test]
    fn test_character_reference() {
        let input = include_str!("fixtures/to_json/character-reference.md");
        let output = include_str!("fixtures/to_json/character-reference.json");

        let actual_tree = to_json(input, &ParseOptions::default())
            .expect("could not parse fixture's input to mdast");

        let expected_tree: serde_json::Value =
            serde_json::from_str(output).expect("a fixture's tree contains invalid json");

        assert_eq!(expected_tree, actual_tree);
    }

    #[test]
    fn test_character_references_everywhere() {
        let input = include_str!("fixtures/to_json/character-references-everywhere.md");
        let output = include_str!("fixtures/to_json/character-references-everywhere.json");

        let actual_tree = to_json(input, &ParseOptions::default())
            .expect("could not parse fixture's input to mdast");

        let expected_tree: serde_json::Value =
            serde_json::from_str(output).expect("a fixture's tree contains invalid json");

        assert_eq!(expected_tree, actual_tree);
    }

    #[test]
    fn test_code_fenced() {
        let input = include_str!("fixtures/to_json/code-fenced.md");
        let output = include_str!("fixtures/to_json/code-fenced.json");

        let actual_tree = to_json(input, &ParseOptions::default())
            .expect("could not parse fixture's input to mdast");

        let expected_tree: serde_json::Value =
            serde_json::from_str(output).expect("a fixture's tree contains invalid json");

        assert_eq!(expected_tree, actual_tree);
    }

    #[test]
    fn test_code_indented() {
        let input = include_str!("fixtures/to_json/code-indented.md");
        let output = include_str!("fixtures/to_json/code-indented.json");

        let actual_tree = to_json(input, &ParseOptions::default())
            .expect("could not parse fixture's input to mdast");

        let expected_tree: serde_json::Value =
            serde_json::from_str(output).expect("a fixture's tree contains invalid json");

        assert_eq!(expected_tree, actual_tree);
    }

    #[test]
    fn test_code_text() {
        let input = include_str!("fixtures/to_json/code-text.md");
        let output = include_str!("fixtures/to_json/code-text.json");

        let actual_tree = to_json(input, &ParseOptions::default())
            .expect("could not parse fixture's input to mdast");

        let expected_tree: serde_json::Value =
            serde_json::from_str(output).expect("a fixture's tree contains invalid json");

        assert_eq!(expected_tree, actual_tree);
    }

    #[test]
    fn test_definition() {
        let input = include_str!("fixtures/to_json/definition.md");
        let output = include_str!("fixtures/to_json/definition.json");

        let actual_tree = to_json(input, &ParseOptions::default())
            .expect("could not parse fixture's input to mdast");

        let expected_tree: serde_json::Value =
            serde_json::from_str(output).expect("a fixture's tree contains invalid json");

        assert_eq!(expected_tree, actual_tree);
    }

    #[test]
    fn test_empty() {
        let input = include_str!("fixtures/to_json/empty.md");
        let output = include_str!("fixtures/to_json/empty.json");

        let actual_tree = to_json(input, &ParseOptions::default())
            .expect("could not parse fixture's input to mdast");

        let expected_tree: serde_json::Value =
            serde_json::from_str(output).expect("a fixture's tree contains invalid json");

        assert_eq!(expected_tree, actual_tree);
    }

    #[test]
    fn test_hard_break_escape() {
        let input = include_str!("fixtures/to_json/hard-break-escape.md");
        let output = include_str!("fixtures/to_json/hard-break-escape.json");

        let actual_tree = to_json(input, &ParseOptions::default())
            .expect("could not parse fixture's input to mdast");

        let expected_tree: serde_json::Value =
            serde_json::from_str(output).expect("a fixture's tree contains invalid json");

        assert_eq!(expected_tree, actual_tree);
    }

    #[test]
    fn test_hard_break_prefix() {
        let input = include_str!("fixtures/to_json/hard-break-prefix.md");
        let output = include_str!("fixtures/to_json/hard-break-prefix.json");

        let actual_tree = to_json(input, &ParseOptions::default())
            .expect("could not parse fixture's input to mdast");

        let expected_tree: serde_json::Value =
            serde_json::from_str(output).expect("a fixture's tree contains invalid json");

        assert_eq!(expected_tree, actual_tree);
    }

    #[test]
    fn test_heading_atx() {
        let input = include_str!("fixtures/to_json/heading-atx.md");
        let output = include_str!("fixtures/to_json/heading-atx.json");

        let actual_tree = to_json(input, &ParseOptions::default())
            .expect("could not parse fixture's input to mdast");

        let expected_tree: serde_json::Value =
            serde_json::from_str(output).expect("a fixture's tree contains invalid json");

        assert_eq!(expected_tree, actual_tree);
    }

    #[test]
    fn test_heading_setext() {
        let input = include_str!("fixtures/to_json/heading-setext.md");
        let output = include_str!("fixtures/to_json/heading-setext.json");

        let actual_tree = to_json(input, &ParseOptions::default())
            .expect("could not parse fixture's input to mdast");

        let expected_tree: serde_json::Value =
            serde_json::from_str(output).expect("a fixture's tree contains invalid json");

        assert_eq!(expected_tree, actual_tree);
    }

    #[test]
    fn test_html_flow() {
        let input = include_str!("fixtures/to_json/html-flow.md");
        let output = include_str!("fixtures/to_json/html-flow.json");

        let actual_tree = to_json(input, &ParseOptions::default())
            .expect("could not parse fixture's input to mdast");

        let expected_tree: serde_json::Value =
            serde_json::from_str(output).expect("a fixture's tree contains invalid json");

        assert_eq!(expected_tree, actual_tree);
    }

    #[test]
    fn test_html_text() {
        let input = include_str!("fixtures/to_json/html-text.md");
        let output = include_str!("fixtures/to_json/html-text.json");

        let actual_tree = to_json(input, &ParseOptions::default())
            .expect("could not parse fixture's input to mdast");

        let expected_tree: serde_json::Value =
            serde_json::from_str(output).expect("a fixture's tree contains invalid json");

        assert_eq!(expected_tree, actual_tree);
    }

    #[test]
    fn test_image_reference() {
        let input = include_str!("fixtures/to_json/image-reference.md");
        let output = include_str!("fixtures/to_json/image-reference.json");

        let actual_tree = to_json(input, &ParseOptions::default())
            .expect("could not parse fixture's input to mdast");

        let expected_tree: serde_json::Value =
            serde_json::from_str(output).expect("a fixture's tree contains invalid json");

        assert_eq!(expected_tree, actual_tree);
    }

    #[test]
    fn test_image_resource_eol() {
        let input = include_str!("fixtures/to_json/image-resource-eol.md");
        let output = include_str!("fixtures/to_json/image-resource-eol.json");

        let actual_tree = to_json(input, &ParseOptions::default())
            .expect("could not parse fixture's input to mdast");

        let expected_tree: serde_json::Value =
            serde_json::from_str(output).expect("a fixture's tree contains invalid json");

        assert_eq!(expected_tree, actual_tree);
    }

    #[test]
    fn test_image_resource() {
        let input = include_str!("fixtures/to_json/image-resource.md");
        let output = include_str!("fixtures/to_json/image-resource.json");

        let actual_tree = to_json(input, &ParseOptions::default())
            .expect("could not parse fixture's input to mdast");

        let expected_tree: serde_json::Value =
            serde_json::from_str(output).expect("a fixture's tree contains invalid json");

        assert_eq!(expected_tree, actual_tree);
    }

    #[test]
    fn test_link_reference_with_phrasing() {
        let input = include_str!("fixtures/to_json/link-reference-with-phrasing.md");
        let output = include_str!("fixtures/to_json/link-reference-with-phrasing.json");

        let actual_tree = to_json(input, &ParseOptions::default())
            .expect("could not parse fixture's input to mdast");

        let expected_tree: serde_json::Value =
            serde_json::from_str(output).expect("a fixture's tree contains invalid json");

        assert_eq!(expected_tree, actual_tree);
    }

    #[test]
    fn test_link_reference() {
        let input = include_str!("fixtures/to_json/link-reference.md");
        let output = include_str!("fixtures/to_json/link-reference.json");

        let actual_tree = to_json(input, &ParseOptions::default())
            .expect("could not parse fixture's input to mdast");

        let expected_tree: serde_json::Value =
            serde_json::from_str(output).expect("a fixture's tree contains invalid json");

        assert_eq!(expected_tree, actual_tree);
    }

    #[test]
    fn test_link_resource_eol() {
        let input = include_str!("fixtures/to_json/link-resource-eol.md");
        let output = include_str!("fixtures/to_json/link-resource-eol.json");

        let actual_tree = to_json(input, &ParseOptions::default())
            .expect("could not parse fixture's input to mdast");

        let expected_tree: serde_json::Value =
            serde_json::from_str(output).expect("a fixture's tree contains invalid json");

        assert_eq!(expected_tree, actual_tree);
    }

    #[test]
    fn test_link_resource() {
        let input = include_str!("fixtures/to_json/link-resource.md");
        let output = include_str!("fixtures/to_json/link-resource.json");

        let actual_tree = to_json(input, &ParseOptions::default())
            .expect("could not parse fixture's input to mdast");

        let expected_tree: serde_json::Value =
            serde_json::from_str(output).expect("a fixture's tree contains invalid json");

        assert_eq!(expected_tree, actual_tree);
    }

    #[test]
    fn test_list() {
        let input = include_str!("fixtures/to_json/list.md");
        let output = include_str!("fixtures/to_json/list.json");

        let actual_tree = to_json(input, &ParseOptions::default())
            .expect("could not parse fixture's input to mdast");

        let expected_tree: serde_json::Value =
            serde_json::from_str(output).expect("a fixture's tree contains invalid json");

        assert_eq!(expected_tree, actual_tree);
    }

    #[test]
    fn test_paragraph() {
        let input = include_str!("fixtures/to_json/paragraph.md");
        let output = include_str!("fixtures/to_json/paragraph.json");

        let actual_tree = to_json(input, &ParseOptions::default())
            .expect("could not parse fixture's input to mdast");

        let expected_tree: serde_json::Value =
            serde_json::from_str(output).expect("a fixture's tree contains invalid json");

        assert_eq!(expected_tree, actual_tree);
    }

    #[test]
    fn test_thematic_break() {
        let input = include_str!("fixtures/to_json/thematic-break.md");
        let output = include_str!("fixtures/to_json/thematic-break.json");

        let actual_tree = to_json(input, &ParseOptions::default())
            .expect("could not parse fixture's input to mdast");

        let expected_tree: serde_json::Value =
            serde_json::from_str(output).expect("a fixture's tree contains invalid json");

        assert_eq!(expected_tree, actual_tree);
    }
}
