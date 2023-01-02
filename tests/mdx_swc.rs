mod test_utils;
use markdown::{to_html_with_options, Constructs, OptionsBuilder, ParseOptionsBuilder};
use pretty_assertions::assert_eq;
use test_utils::swc::{parse_esm, parse_expression};

#[test]
fn mdx_swc() -> Result<(), String> {
    let swc = OptionsBuilder::default()
        .parse(
            ParseOptionsBuilder::default()
                .constructs(Constructs::mdx())
                .mdx_esm_parse(Some(Box::new(parse_esm)))
                .mdx_expression_parse(Some(Box::new(parse_expression)))
                .build(),
        )
        .build();

    assert_eq!(
        to_html_with_options("{'}'}", &swc)?,
        "",
        "should support JavaScript-aware flow expressions w/ `mdx_expression_parse`"
    );

    assert_eq!(
        to_html_with_options("a {'}'} b", &swc)?,
        "<p>a  b</p>",
        "should support JavaScript-aware text expressions w/ `mdx_expression_parse`"
    );

    assert_eq!(
        to_html_with_options("<a {...a/*}*/} />", &swc)?,
        "",
        "should support JavaScript-aware attribute expressions w/ `mdx_expression_parse`"
    );

    assert_eq!(
        to_html_with_options("<a b={'}'} />", &swc)?,
        "",
        "should support JavaScript-aware attribute value expressions w/ `mdx_expression_parse`"
    );

    assert_eq!(
        to_html_with_options("import a from 'b'\n\nexport {a}\n\n# c", &swc)?,
        "<h1>c</h1>",
        "should support JavaScript-aware ESM w/ `mdx_esm_parse`"
    );

    Ok(())
}
