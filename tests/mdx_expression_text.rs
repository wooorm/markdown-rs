extern crate micromark;
mod test_utils;
use micromark::{
    mdast::{MdxTextExpression, Node, Paragraph, Position, Root, Text},
    micromark_to_mdast, micromark_with_options, Constructs, Options,
};
use pretty_assertions::assert_eq;
use test_utils::{parse_esm, parse_expression};

#[test]
fn mdx_expression_text_gnostic_core() -> Result<(), String> {
    let swc = Options {
        constructs: Constructs::mdx(),
        mdx_esm_parse: Some(Box::new(parse_esm)),
        mdx_expression_parse: Some(Box::new(parse_expression)),
        ..Options::default()
    };

    assert_eq!(
        micromark_with_options("a {} b", &swc)?,
        "<p>a  b</p>",
        "should support an empty expression (1)"
    );

    assert_eq!(
        micromark_with_options("a { \t\r\n} b", &swc)?,
        "<p>a  b</p>",
        "should support an empty expression (2)"
    );

    assert_eq!(
        micromark_with_options("a {/**/} b", &swc)?,
        "<p>a  b</p>",
        "should support a multiline comment (1)"
    );

    assert_eq!(
        micromark_with_options("a {  /*\n*/\t} b", &swc)?,
        "<p>a  b</p>",
        "should support a multiline comment (2)"
    );

    assert_eq!(
        micromark_with_options("a {/*b*//*c*/} d", &swc)?,
        "<p>a  d</p>",
        "should support a multiline comment (3)"
    );

    assert_eq!(
        micromark_with_options("a {b/*c*/} d", &swc)?,
        "<p>a  d</p>",
        "should support a multiline comment (4)"
    );

    assert_eq!(
        micromark_with_options("a {/*b*/c} d", &swc)?,
        "<p>a  d</p>",
        "should support a multiline comment (4)"
    );

    assert_eq!(
        micromark_with_options("a {//} b", &swc).err().unwrap(),
        "1:4: Could not parse expression with swc: Unexpected eof",
        "should crash on an incorrect line comment (1)"
    );

    assert_eq!(
        micromark_with_options("a { // b } c", &swc).err().unwrap(),
        "1:4: Could not parse expression with swc: Unexpected eof",
        "should crash on an incorrect line comment (2)"
    );

    assert_eq!(
        micromark_with_options("a {//\n} b", &swc)?,
        "<p>a  b</p>",
        "should support a line comment followed by a line ending"
    );

    assert_eq!(
        micromark_with_options("a {// b\nd} d", &swc)?,
        "<p>a  d</p>",
        "should support a line comment followed by a line ending and an expression"
    );

    assert_eq!(
        micromark_with_options("a {b// c\n} d", &swc)?,
        "<p>a  d</p>",
        "should support an expression followed by a line comment and a line ending"
    );

    assert_eq!(
        micromark_with_options("a {/*b*/ // c\n} d", &swc)?,
        "<p>a  d</p>",
        "should support comments (1)"
    );

    assert_eq!(
        micromark_with_options("a {b.c} d", &swc)?,
        "<p>a  d</p>",
        "should support expression statements (1)"
    );

    assert_eq!(
        micromark_with_options("a {1 + 1} b", &swc)?,
        "<p>a  b</p>",
        "should support expression statements (2)"
    );

    assert_eq!(
        micromark_with_options("a {function () {}} b", &swc)?,
        "<p>a  b</p>",
        "should support expression statements (3)"
    );

    assert_eq!(
        micromark_with_options("a {var b = \"c\"} d", &swc).err().unwrap(),
        "1:7: Could not parse expression with swc: Unexpected token `var`. Expected this, import, async, function, [ for array literal, { for object literal, @ for decorator, function, class, null, true, false, number, bigint, string, regexp, ` for template literal, (, or an identifier",
        "should crash on non-expressions"
    );

    assert_eq!(
        micromark_with_options("> a {\n> b} c", &swc)?,
        "<blockquote>\n<p>a  c</p>\n</blockquote>",
        "should support expressions in containers"
    );

    assert_eq!(
        micromark_with_options("> a {\n> b<} c", &swc)
            .err()
            .unwrap(),
        "2:8: Could not parse expression with swc: Unexpected eof",
        "should crash on incorrect expressions in containers (1)"
    );

    assert_eq!(
        micromark_with_options("> a {\n> b\n> c} d", &swc)
            .err()
            .unwrap(),
        "3:3: Could not parse expression with swc: Unexpected content after expression",
        "should crash on incorrect expressions in containers (2)"
    );

    Ok(())
}

#[test]
fn mdx_expression_text_agnostic() -> Result<(), String> {
    let mdx = Options {
        constructs: Constructs::mdx(),
        ..Options::default()
    };

    assert_eq!(
        micromark_with_options("a {b} c", &mdx)?,
        "<p>a  c</p>",
        "should support an expression"
    );

    assert_eq!(
        micromark_with_options("a {} b", &mdx)?,
        "<p>a  b</p>",
        "should support an empty expression"
    );

    assert_eq!(
        micromark_with_options("a {b c", &mdx).err().unwrap(),
        "1:7: Unexpected end of file in expression, expected a corresponding closing brace for `{`",
        "should crash if no closing brace is found (1)"
    );

    assert_eq!(
        micromark_with_options("a {b { c } d", &mdx)
            .err()
            .unwrap(),
        "1:13: Unexpected end of file in expression, expected a corresponding closing brace for `{`",
        "should crash if no closing brace is found (2)"
    );

    assert_eq!(
        micromark_with_options("a {\n} b", &mdx)?,
        "<p>a  b</p>",
        "should support a line ending in an expression"
    );

    assert_eq!(
        micromark_with_options("a } b", &mdx)?,
        "<p>a } b</p>",
        "should support just a closing brace"
    );

    assert_eq!(
        micromark_with_options("{ a } b", &mdx)?,
        "<p> b</p>",
        "should support expressions as the first thing when following by other things"
    );

    assert_eq!(
        micromark_to_mdast("a {alpha} b.", &mdx)?,
        Node::Root(Root {
            children: vec![Node::Paragraph(Paragraph {
                children: vec![
                    Node::Text(Text {
                        value: "a ".to_string(),
                        position: Some(Position::new(1, 1, 0, 1, 3, 2))
                    }),
                    Node::MdxTextExpression(MdxTextExpression {
                        value: "alpha".to_string(),
                        position: Some(Position::new(1, 3, 2, 1, 10, 9))
                    }),
                    Node::Text(Text {
                        value: " b.".to_string(),
                        position: Some(Position::new(1, 10, 9, 1, 13, 12))
                    })
                ],
                position: Some(Position::new(1, 1, 0, 1, 13, 12))
            })],
            position: Some(Position::new(1, 1, 0, 1, 13, 12))
        }),
        "should support mdx expressions (text) as `MdxTextExpression`s in mdast"
    );

    Ok(())
}

#[test]
fn mdx_expression_text_gnostic() -> Result<(), String> {
    let swc = Options {
        constructs: Constructs::mdx(),
        mdx_esm_parse: Some(Box::new(parse_esm)),
        mdx_expression_parse: Some(Box::new(parse_expression)),
        ..Options::default()
    };

    assert_eq!(
        micromark_with_options("a {b} c", &swc)?,
        "<p>a  c</p>",
        "should support an expression"
    );

    assert_eq!(
        micromark_with_options("a {??} b", &swc).err().unwrap(),
        "1:9: Could not parse expression with swc: Unexpected eof",
        "should crash on an incorrect expression"
    );

    assert_eq!(
        micromark_with_options("a {} b", &swc)?,
        "<p>a  b</p>",
        "should support an empty expression"
    );

    assert_eq!(
        micromark_with_options("a {b c", &swc).err().unwrap(),
        "1:7: Unexpected end of file in expression, expected a corresponding closing brace for `{`",
        "should crash if no closing brace is found (1)"
    );

    assert_eq!(
        micromark_with_options("a {b { c } d", &swc).err().unwrap(),
        "1:6: Could not parse expression with swc: Unexpected content after expression",
        "should crash if no closing brace is found (2)"
    );

    assert_eq!(
        micromark_with_options("a {\n} b", &swc)?,
        "<p>a  b</p>",
        "should support a line ending in an expression"
    );

    assert_eq!(
        micromark_with_options("a } b", &swc)?,
        "<p>a } b</p>",
        "should support just a closing brace"
    );

    assert_eq!(
        micromark_with_options("{ a } b", &swc)?,
        "<p> b</p>",
        "should support expressions as the first thing when following by other things"
    );

    assert_eq!(
        micromark_with_options("a { /* { */ } b", &swc)?,
        "<p>a  b</p>",
        "should support an unbalanced opening brace (if JS permits)"
    );

    assert_eq!(
        micromark_with_options("a { /* } */ } b", &swc)?,
        "<p>a  b</p>",
        "should support an unbalanced closing brace (if JS permits)"
    );

    Ok(())
}
