extern crate micromark;
use micromark::{micromark_with_options, Constructs, Options};
use pretty_assertions::assert_eq;

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

    Ok(())
}

// To do: swc.
// #[test]
// fn mdx_expression_flow_gnostic() -> Result<(), String> {
//     assert_eq!(
//         micromark_with_options("{a}", &swc),
//         "",
//         "should support an expression"
//     );

//     assert_eq!(
//         micromark_with_options("{}", &swc)?,
//         "",
//         "should support an empty expression"
//     );

//     //   To do: errors.
//     // t.throws(
//     //     () => {
//     //     micromark_with_options("{a", &swc);
//     //     },
//     //     /Unexpected end of file in expression, expected a corresponding closing brace for `{`/,
//     //     "should crash if no closing brace is found (1)"
//     // );

//     //   To do: errors.
//     // t.throws(
//     //     () => {
//     //     micromark_with_options("{b { c }", &swc);
//     //     },
//     //     /Could not parse expression with swc: Unexpected content after expression/,
//     //     "should crash if no closing brace is found (2)"
//     // );

//     assert_eq!(
//         micromark_with_options("{\n}\na", &swc)?,
//         "<p>a</p>",
//         "should support a line ending in an expression"
//     );

//     assert_eq!(
//         micromark_with_options("{ a } \t\nb", &swc)?,
//         "<p>b</p>",
//         "should support expressions followed by spaces"
//     );

//     assert_eq!(
//         micromark_with_options("  { a }\nb", &swc)?,
//         "<p>b</p>",
//         "should support expressions preceded by spaces"
//     );

//     assert_eq!(
//         micromark_with_options("  {`\n    a\n  `}", &swc)?,
//         "",
//         "should support indented expressions"
//     );

//     assert_eq!(
//         micromark_with_options("a{(b)}c", &swc)?,
//         "<p>ac</p>",
//         "should support expressions padded w/ parens"
//     );

//     assert_eq!(
//         micromark_with_options("a{/* b */ ( (c) /* d */ + (e) )}f", &swc)?,
//         "<p>af</p>",
//         "should support expressions padded w/ parens and comments"
//     );

//     Ok(())
// }

// To do: move to JSX, actually test spread in expressions?
// To do: swc.
// #[test]
// fn mdx_expression_spread() -> Result<(), String> {
//     //   To do: errors.
//     // t.throws(
//     //     () => {
//     //     micromark_with_options("a {b} c", &swc);
//     //     },
//     //     /Unexpected `Property` in code: only spread elements are supported/,
//     //     "should crash if not a spread"
//     // );

//     //   To do: errors.
//     // t.throws(
//     //     () => {
//     //     micromark_with_options("a {...?} c", &swc);
//     //     },
//     //     /Could not parse expression with swc: Unexpected token/,
//     //     "should crash on an incorrect spread"
//     // );

//     //   To do: errors.
//     // t.throws(
//     //     () => {
//     //     micromark_with_options("a {...b,c} d", &swc);
//     //     },
//     //     /Unexpected extra content in spread: only a single spread is supported/,
//     //     "should crash if a spread and other things"
//     // );

//     assert_eq!(
//         micromark_with_options("a {} b", &swc)?,
//         "<p>a  b</p>",
//         "should support an empty spread"
//     );

//     //   To do: errors.
//     // t.throws(
//     //     () => {
//     //     micromark_with_options("a {} b", &swc);
//     //     },
//     //     /Unexpected empty expression/,
//     //     "should crash on an empty spread w/ `allowEmpty: false`"
//     // );

//     //   To do: errors.
//     // t.throws(
//     //     () => {
//     //     micromark_with_options("{a=b}", &swc);
//     //     },
//     //     /Could not parse expression with swc: Shorthand property assignments are valid only in destructuring patterns/,
//     //     "should crash if not a spread w/ `allowEmpty`"
//     // );

//     assert_eq!(
//         micromark_with_options("a {/* b */} c", &swc)?,
//         "<p>a  c</p>",
//         "should support a comment spread"
//     );

//     //   To do: errors.
//     // t.throws(
//     //     () => {
//     //     micromark_with_options("a {/* b */} c", &swc);
//     //     },
//     //     /Unexpected empty expression/,
//     //     "should crash on a comment spread w/ `allowEmpty: false`"
//     // );

//     assert_eq!(
//         micromark_with_options("a {...b} c", &swc)?,
//         "<p>a  c</p>",
//         "should support a spread"
//     );

//     Ok(())
// }
