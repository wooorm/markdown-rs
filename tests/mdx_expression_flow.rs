extern crate micromark;
mod test_utils;
use micromark::{
    mdast::{MdxFlowExpression, Node, Root},
    micromark_to_mdast, micromark_with_options,
    unist::Position,
    Constructs, Options,
};
use pretty_assertions::assert_eq;
use test_utils::swc::{parse_esm, parse_expression};

#[test]
fn mdx_expression_flow_agnostic() -> Result<(), String> {
    let mdx = Options {
        constructs: Constructs::mdx(),
        ..Options::default()
    };

    assert_eq!(
        micromark_with_options("{a}", &mdx)?,
        "",
        "should support an expression"
    );

    assert_eq!(
        micromark_with_options("{}", &mdx)?,
        "",
        "should support an empty expression"
    );

    assert_eq!(
        micromark_with_options("{a", &mdx).err().unwrap(),
        "1:3: Unexpected end of file in expression, expected a corresponding closing brace for `{`",
        "should crash if no closing brace is found (1)"
    );

    assert_eq!(
        micromark_with_options("{b { c }", &mdx).err().unwrap(),
        "1:9: Unexpected end of file in expression, expected a corresponding closing brace for `{`",
        "should crash if no closing brace is found (2)"
    );

    assert_eq!(
        micromark_with_options("{\n}\na", &mdx)?,
        "<p>a</p>",
        "should support a line ending in an expression"
    );

    assert_eq!(
        micromark_with_options("{ a } \t\nb", &mdx)?,
        "<p>b</p>",
        "should support expressions followed by spaces"
    );

    assert_eq!(
        micromark_with_options("  { a }\nb", &mdx)?,
        "<p>b</p>",
        "should support expressions preceded by spaces"
    );

    assert_eq!(
        micromark_with_options("> {a\nb}", &mdx)
            .err()
            .unwrap(),
        "2:1: Unexpected lazy line in expression in container, expected line to be prefixed with `>` when in a block quote, whitespace when in a list, etc",
        "should not support lazyness (1)"
    );

    assert_eq!(
        micromark_with_options("> a\n{b}", &mdx)?,
        "<blockquote>\n<p>a</p>\n</blockquote>\n",
        "should not support lazyness (2)"
    );

    assert_eq!(
        micromark_with_options("> {a}\nb", &mdx)?,
        "<blockquote>\n</blockquote>\n<p>b</p>",
        "should not support lazyness (3)"
    );

    assert_eq!(
        micromark_with_options("> {\n> a\nb}", &mdx)
            .err()
            .unwrap(),
        "3:1: Unexpected lazy line in expression in container, expected line to be prefixed with `>` when in a block quote, whitespace when in a list, etc",
        "should not support lazyness (4)"
    );

    assert_eq!(
        micromark_to_mdast("{alpha +\nbravo}", &mdx)?,
        Node::Root(Root {
            children: vec![Node::MdxFlowExpression(MdxFlowExpression {
                value: "alpha +\nbravo".to_string(),
                position: Some(Position::new(1, 1, 0, 2, 7, 15))
            })],
            position: Some(Position::new(1, 1, 0, 2, 7, 15))
        }),
        "should support mdx expressions (flow) as `MdxFlowExpression`s in mdast"
    );

    Ok(())
}

#[test]
fn mdx_expression_flow_gnostic() -> Result<(), String> {
    let swc = Options {
        constructs: Constructs::mdx(),
        mdx_esm_parse: Some(Box::new(parse_esm)),
        mdx_expression_parse: Some(Box::new(parse_expression)),
        ..Options::default()
    };

    assert_eq!(
        micromark_with_options("{a}", &swc)?,
        "",
        "should support an expression"
    );

    assert_eq!(
        micromark_with_options("{}", &swc)?,
        "",
        "should support an empty expression"
    );

    assert_eq!(
        micromark_with_options("{a", &swc).err().unwrap(),
        "1:3: Unexpected end of file in expression, expected a corresponding closing brace for `{`",
        "should crash if no closing brace is found (1)"
    );

    assert_eq!(
        micromark_with_options("{b { c }", &swc).err().unwrap(),
        "1:4: Could not parse expression with swc: Unexpected content after expression",
        "should crash if no closing brace is found (2)"
    );

    assert_eq!(
        micromark_with_options("{\n}\na", &swc)?,
        "<p>a</p>",
        "should support a line ending in an expression"
    );

    assert_eq!(
        micromark_with_options("{ a } \t\nb", &swc)?,
        "<p>b</p>",
        "should support expressions followed by spaces"
    );

    assert_eq!(
        micromark_with_options("  { a }\nb", &swc)?,
        "<p>b</p>",
        "should support expressions preceded by spaces"
    );

    assert_eq!(
        micromark_with_options("  {`\n    a\n  `}", &swc)?,
        "",
        "should support indented expressions"
    );

    assert_eq!(
        micromark_with_options("a{(b)}c", &swc)?,
        "<p>ac</p>",
        "should support expressions padded w/ parens"
    );

    assert_eq!(
        micromark_with_options("a{/* b */ ( (c) /* d */ + (e) )}f", &swc)?,
        "<p>af</p>",
        "should support expressions padded w/ parens and comments"
    );

    Ok(())
}

#[test]
fn mdx_expression_spread() -> Result<(), String> {
    let swc = Options {
        constructs: Constructs::mdx(),
        mdx_esm_parse: Some(Box::new(parse_esm)),
        mdx_expression_parse: Some(Box::new(parse_expression)),
        ..Options::default()
    };

    assert_eq!(
        micromark_with_options("<a {...b} />", &swc)?,
        "",
        "should support spreads for attribute expression"
    );

    assert_eq!(
        micromark_with_options("<a {b} />", &swc).err().unwrap(),
        "1:5: Expected a single spread value, such as `...x`",
        "should crash if not a spread"
    );

    assert_eq!(
        micromark_with_options("<a {...?} />", &swc).err().unwrap(),
        "1:13: Could not parse expression with swc: Unexpected token `?`. Expected this, import, async, function, [ for array literal, { for object literal, @ for decorator, function, class, null, true, false, number, bigint, string, regexp, ` for template literal, (, or an identifier",
        "should crash on an incorrect spread"
    );

    assert_eq!(
        micromark_with_options("<a {...b,c} d>", &swc)
            .err()
            .unwrap(),
        "1:5: Expected a single spread value, such as `...x`",
        "should crash if a spread and other things"
    );

    assert_eq!(
        micromark_with_options("<a {} />", &swc).err().unwrap(),
        "1:5: Expected a single spread value, such as `...x`",
        "should crash on an empty spread"
    );

    assert_eq!(
        micromark_with_options("<a {a=b} />", &swc).err().unwrap(),
        "1:12: Could not parse expression with swc: assignment property is invalid syntax",
        "should crash if not an identifier"
    );

    assert_eq!(
        micromark_with_options("<a {/* b */} />", &swc)
            .err()
            .unwrap(),
        "1:5: Expected a single spread value, such as `...x`",
        "should crash on a comment spread"
    );

    Ok(())
}
