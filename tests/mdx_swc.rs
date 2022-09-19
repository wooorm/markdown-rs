extern crate micromark;
mod test_utils;
use micromark::{micromark_with_options, Constructs, Options};
use pretty_assertions::assert_eq;
use test_utils::{parse_esm, parse_expression};

#[test]
fn mdx_swc() -> Result<(), String> {
    let mdx = Options {
        constructs: Constructs::mdx(),
        mdx_esm_parse: Some(Box::new(parse_esm)),
        mdx_expression_parse: Some(Box::new(parse_expression)),
        ..Options::default()
    };

    assert_eq!(
        micromark_with_options("{'}'}", &mdx)?,
        "",
        "should support JavaScript-aware flow expressions w/ `mdx_expression_parse`"
    );

    assert_eq!(
        micromark_with_options("a {'}'} b", &mdx)?,
        "<p>a  b</p>",
        "should support JavaScript-aware text expressions w/ `mdx_expression_parse`"
    );

    assert_eq!(
        micromark_with_options("<a {...a/*}*/} />", &mdx)?,
        "",
        "should support JavaScript-aware attribute expressions w/ `mdx_expression_parse`"
    );

    assert_eq!(
        micromark_with_options("<a b={'}'} />", &mdx)?,
        "",
        "should support JavaScript-aware attribute value expressions w/ `mdx_expression_parse`"
    );

    assert_eq!(
        micromark_with_options("import a from 'b'\n\nexport {a}\n\n# c", &mdx)?,
        "<h1>c</h1>",
        "should support JavaScript-aware ESM w/ `mdx_esm_parse`"
    );

    Ok(())
}
