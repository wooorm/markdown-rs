extern crate micromark;
extern crate swc_common;
extern crate swc_ecma_ast;
extern crate swc_ecma_codegen;
mod test_utils;
use micromark::{micromark_to_mdast, Constructs, Options};
use pretty_assertions::assert_eq;
use test_utils::{
    swc::{parse_esm, parse_expression, serialize},
    to_document::{to_document, Options as DocumentOptions},
    to_hast::to_hast,
    to_swc::to_swc,
};

fn from_markdown(value: &str) -> Result<String, String> {
    let mdast = micromark_to_mdast(
        value,
        &Options {
            constructs: Constructs::mdx(),
            mdx_esm_parse: Some(Box::new(parse_esm)),
            mdx_expression_parse: Some(Box::new(parse_expression)),
            ..Options::default()
        },
    )?;
    let hast = to_hast(&mdast);
    let program = to_document(to_swc(&hast)?, &DocumentOptions::default())?;
    let value = serialize(&program.module);
    Ok(value)
}

#[test]
fn document() -> Result<(), String> {
    assert_eq!(
        from_markdown("# hi\n\nAlpha *bravo* **charlie**.")?,
        "function _createMdxContent(props) {
    return <><h1 >{\"hi\"}</h1>{\"\\n\"}<p >{\"Alpha \"}<em >{\"bravo\"}</em>{\" \"}<strong >{\"charlie\"}</strong>{\".\"}</p></>;
}
function MDXContent(props = {}) {
    return MDXLayout ? <MDXLayout {...props}><_createMdxContent {...props}/></MDXLayout> : _createMdxContent(props);
}
export default MDXContent;
",
        "should support a small program",
    );

    assert_eq!(
        from_markdown("import a from 'b'\n\n# {a}")?,
        "import a from 'b';
function _createMdxContent(props) {
    return <h1 >{a}</h1>;
}
function MDXContent(props = {}) {
    return MDXLayout ? <MDXLayout {...props}><_createMdxContent {...props}/></MDXLayout> : _createMdxContent(props);
}
export default MDXContent;
",
        "should support an import",
    );

    assert_eq!(
        from_markdown("export * from 'a'\n\n# b")?,
        "export * from 'a';
function _createMdxContent(props) {
    return <h1 >{\"b\"}</h1>;
}
function MDXContent(props = {}) {
    return MDXLayout ? <MDXLayout {...props}><_createMdxContent {...props}/></MDXLayout> : _createMdxContent(props);
}
export default MDXContent;
",
        "should support an export all",
    );

    assert_eq!(
        from_markdown("export function a() {}")?,
        "export function a() {}
function _createMdxContent(props) {
    return <></>;
}
function MDXContent(props = {}) {
    return MDXLayout ? <MDXLayout {...props}><_createMdxContent {...props}/></MDXLayout> : _createMdxContent(props);
}
export default MDXContent;
",
        "should support an export declaration",
    );

    assert_eq!(
        from_markdown("export default a")?,
        "const MDXLayout = a;
function _createMdxContent(props) {
    return <></>;
}
function MDXContent(props = {}) {
    return <MDXLayout {...props}><_createMdxContent {...props}/></MDXLayout>;
}
export default MDXContent;
",
        "should support an export default expression",
    );

    assert_eq!(
        from_markdown("export default function () {}")?,
        "const MDXLayout = function() {};
function _createMdxContent(props) {
    return <></>;
}
function MDXContent(props = {}) {
    return <MDXLayout {...props}><_createMdxContent {...props}/></MDXLayout>;
}
export default MDXContent;
",
        "should support an export default declaration",
    );

    assert_eq!(
        from_markdown("export {a, b as default}")?,
        "export { a };
const MDXLayout = b;
function _createMdxContent(props) {
    return <></>;
}
function MDXContent(props = {}) {
    return <MDXLayout {...props}><_createMdxContent {...props}/></MDXLayout>;
}
export default MDXContent;
",
        "should support a named export w/o source, w/ a default specifier",
    );

    assert_eq!(
        from_markdown("export {a}")?,
        "export { a };
function _createMdxContent(props) {
    return <></>;
}
function MDXContent(props = {}) {
    return MDXLayout ? <MDXLayout {...props}><_createMdxContent {...props}/></MDXLayout> : _createMdxContent(props);
}
export default MDXContent;
",
        "should support a named export w/o source, w/o a default specifier",
    );

    assert_eq!(
        from_markdown("export {}")?,
        "export { };
function _createMdxContent(props) {
    return <></>;
}
function MDXContent(props = {}) {
    return MDXLayout ? <MDXLayout {...props}><_createMdxContent {...props}/></MDXLayout> : _createMdxContent(props);
}
export default MDXContent;
",
        "should support a named export w/o source, w/o a specifiers",
    );

    // ...........

    assert_eq!(
        from_markdown("export {a, b as default} from 'c'")?,
        "export { a } from 'c';
import { b as MDXLayout } from 'c';
function _createMdxContent(props) {
    return <></>;
}
function MDXContent(props = {}) {
    return <MDXLayout {...props}><_createMdxContent {...props}/></MDXLayout>;
}
export default MDXContent;
",
        "should support a named export w/ source, w/ a default specifier",
    );

    assert_eq!(
        from_markdown("export {a} from 'b'")?,
        "export { a } from 'b';
function _createMdxContent(props) {
    return <></>;
}
function MDXContent(props = {}) {
    return MDXLayout ? <MDXLayout {...props}><_createMdxContent {...props}/></MDXLayout> : _createMdxContent(props);
}
export default MDXContent;
",
        "should support a named export w/ source, w/o a default specifier",
    );

    assert_eq!(
        from_markdown("export {} from 'a'")?,
        "export { } from 'a';
function _createMdxContent(props) {
    return <></>;
}
function MDXContent(props = {}) {
    return MDXLayout ? <MDXLayout {...props}><_createMdxContent {...props}/></MDXLayout> : _createMdxContent(props);
}
export default MDXContent;
",
        "should support a named export w/ source, w/o a specifiers",
    );

    Ok(())
}