extern crate markdown;
extern crate swc_common;
extern crate swc_ecma_ast;
extern crate swc_ecma_codegen;
mod test_utils;
use markdown::{to_mdast, Constructs, Location, ParseOptions};
use pretty_assertions::assert_eq;
use test_utils::{
    hast_util_to_swc::hast_util_to_swc,
    mdast_util_to_hast::mdast_util_to_hast,
    mdx_plugin_recma_document::{mdx_plugin_recma_document, Options as DocumentOptions},
    mdx_plugin_recma_jsx_rewrite::{mdx_plugin_recma_jsx_rewrite, Options as RewriteOptions},
    swc::{parse_esm, parse_expression, serialize},
};

fn from_markdown(value: &str, options: &RewriteOptions) -> Result<String, String> {
    let location = Location::new(value.as_bytes());
    let mdast = to_mdast(
        value,
        &ParseOptions {
            constructs: Constructs::mdx(),
            mdx_esm_parse: Some(Box::new(parse_esm)),
            mdx_expression_parse: Some(Box::new(parse_expression)),
            ..ParseOptions::default()
        },
    )?;
    let hast = mdast_util_to_hast(&mdast);
    let mut program = hast_util_to_swc(&hast, Some("example.mdx".into()), Some(&location))?;
    mdx_plugin_recma_document(&mut program, &DocumentOptions::default(), Some(&location))?;
    mdx_plugin_recma_jsx_rewrite(&mut program, options, Some(&location));
    Ok(serialize(&program.module))
}

#[test]
fn mdx_plugin_recma_jsx_rewrite_test() -> Result<(), String> {
    assert_eq!(
        from_markdown("", &Default::default())?,
        "function _createMdxContent(props) {
    return <></>;
}
function MDXContent(props = {}) {
    const { wrapper: MDXLayout  } = props.components || {};
    return MDXLayout ? <MDXLayout {...props}><_createMdxContent {...props}/></MDXLayout> : _createMdxContent(props);
}
export default MDXContent;
",
        "should work on an empty file",
    );

    assert_eq!(
        from_markdown("# hi", &Default::default())?,
        "function _createMdxContent(props) {
    const _components = Object.assign({
        h1: \"h1\"
    }, props.components);
    return <_components.h1 >{\"hi\"}</_components.h1>;
}
function MDXContent(props = {}) {
    const { wrapper: MDXLayout  } = props.components || {};
    return MDXLayout ? <MDXLayout {...props}><_createMdxContent {...props}/></MDXLayout> : _createMdxContent(props);
}
export default MDXContent;
",
        "should support passing in a layout (as `wrapper`) and components for literal tags",
    );

    assert_eq!(
        from_markdown(
            "export {MyLayout as default} from './a.js'\n\n# hi",
            &Default::default()
        )?,
        "import { MyLayout as MDXLayout } from './a.js';
function _createMdxContent(props) {
    const _components = Object.assign({
        h1: \"h1\"
    }, props.components);
    return <_components.h1 >{\"hi\"}</_components.h1>;
}
function MDXContent(props = {}) {
    return <MDXLayout {...props}><_createMdxContent {...props}/></MDXLayout>;
}
export default MDXContent;
",
        "should not support passing in a layout if one is defined locally",
    );

    assert_eq!(
        from_markdown("# <Hi />", &Default::default())?,
        "function _createMdxContent(props) {
    const _components = Object.assign({
        h1: \"h1\"
    }, props.components), { Hi  } = _components;
    if (!Hi) _missingMdxReference(\"Hi\", true);
    return <_components.h1 ><Hi /></_components.h1>;
}
function MDXContent(props = {}) {
    const { wrapper: MDXLayout  } = props.components || {};
    return MDXLayout ? <MDXLayout {...props}><_createMdxContent {...props}/></MDXLayout> : _createMdxContent(props);
}
export default MDXContent;
function _missingMdxReference(id, component) {
    throw new Error(\"Expected \" + (component ? \"component\" : \"object\") + \" `\" + id + \"` to be defined: you likely forgot to import, pass, or provide it.\");
}
",
        "should support passing in a component",
    );

    assert_eq!(
        from_markdown("<X />, <X.y />, <Y.Z />", &Default::default())?,
        "function _createMdxContent(props) {
    const _components = Object.assign({
        p: \"p\"
    }, props.components), { X , Y  } = _components;
    if (!X) _missingMdxReference(\"X\", true);
    if (!X.y) _missingMdxReference(\"X.y\", true);
    if (!Y) _missingMdxReference(\"Y\", false);
    if (!Y.Z) _missingMdxReference(\"Y.Z\", true);
    return <_components.p ><X />{\", \"}<X.y />{\", \"}<Y.Z /></_components.p>;
}
function MDXContent(props = {}) {
    const { wrapper: MDXLayout  } = props.components || {};
    return MDXLayout ? <MDXLayout {...props}><_createMdxContent {...props}/></MDXLayout> : _createMdxContent(props);
}
export default MDXContent;
function _missingMdxReference(id, component) {
    throw new Error(\"Expected \" + (component ? \"component\" : \"object\") + \" `\" + id + \"` to be defined: you likely forgot to import, pass, or provide it.\");
}
",
        "should support passing in component objects",
    );

    assert_eq!(
        from_markdown("import {Hi} from './a.js'\n\n# <Hi />", &Default::default())?,
        "import { Hi } from './a.js';
function _createMdxContent(props) {
    const _components = Object.assign({
        h1: \"h1\"
    }, props.components);
    return <_components.h1 ><Hi /></_components.h1>;
}
function MDXContent(props = {}) {
    const { wrapper: MDXLayout  } = props.components || {};
    return MDXLayout ? <MDXLayout {...props}><_createMdxContent {...props}/></MDXLayout> : _createMdxContent(props);
}
export default MDXContent;
",
        "should not support passing in a component if one is defined locally",
    );

    assert_eq!(
        from_markdown("# <a-b />", &Default::default())?,
        "function _createMdxContent(props) {
    const _components = Object.assign({
        h1: \"h1\",
        \"a-b\": \"a-b\"
    }, props.components), _component0 = _components[\"a-b\"];
    return <_components.h1 ><_component0 /></_components.h1>;
}
function MDXContent(props = {}) {
    const { wrapper: MDXLayout  } = props.components || {};
    return MDXLayout ? <MDXLayout {...props}><_createMdxContent {...props}/></MDXLayout> : _createMdxContent(props);
}
export default MDXContent;
",
        "should support passing in a component for a JSX identifier that is not a valid JS identifier",
    );

    assert_eq!(
        from_markdown("# <Hi />", &RewriteOptions {
            provider_import_source: Some("x".into()),
            ..Default::default()
        })?,
        "import { useMDXComponents as _provideComponents } from \"x\";
function _createMdxContent(props) {
    const _components = Object.assign({
        h1: \"h1\"
    }, _provideComponents(), props.components), { Hi  } = _components;
    if (!Hi) _missingMdxReference(\"Hi\", true);
    return <_components.h1 ><Hi /></_components.h1>;
}
function MDXContent(props = {}) {
    const { wrapper: MDXLayout  } = Object.assign({}, _provideComponents(), props.components);
    return MDXLayout ? <MDXLayout {...props}><_createMdxContent {...props}/></MDXLayout> : _createMdxContent(props);
}
export default MDXContent;
function _missingMdxReference(id, component) {
    throw new Error(\"Expected \" + (component ? \"component\" : \"object\") + \" `\" + id + \"` to be defined: you likely forgot to import, pass, or provide it.\");
}
",
        "should support providing a layout, literal tags, and components",
    );

    assert_eq!(
        from_markdown("", &RewriteOptions {
            provider_import_source: Some("x".into()),
            ..Default::default()
        })?,
        "import { useMDXComponents as _provideComponents } from \"x\";
function _createMdxContent(props) {
    return <></>;
}
function MDXContent(props = {}) {
    const { wrapper: MDXLayout  } = Object.assign({}, _provideComponents(), props.components);
    return MDXLayout ? <MDXLayout {...props}><_createMdxContent {...props}/></MDXLayout> : _createMdxContent(props);
}
export default MDXContent;
",
        "should support a provider on an empty file",
    );

    assert_eq!(
        from_markdown("<X />, <X.y />, <Y.Z />", &RewriteOptions {
            provider_import_source: Some("x".into()),
            ..Default::default()
        })?,
        "import { useMDXComponents as _provideComponents } from \"x\";
function _createMdxContent(props) {
    const _components = Object.assign({
        p: \"p\"
    }, _provideComponents(), props.components), { X , Y  } = _components;
    if (!X) _missingMdxReference(\"X\", true);
    if (!X.y) _missingMdxReference(\"X.y\", true);
    if (!Y) _missingMdxReference(\"Y\", false);
    if (!Y.Z) _missingMdxReference(\"Y.Z\", true);
    return <_components.p ><X />{\", \"}<X.y />{\", \"}<Y.Z /></_components.p>;
}
function MDXContent(props = {}) {
    const { wrapper: MDXLayout  } = Object.assign({}, _provideComponents(), props.components);
    return MDXLayout ? <MDXLayout {...props}><_createMdxContent {...props}/></MDXLayout> : _createMdxContent(props);
}
export default MDXContent;
function _missingMdxReference(id, component) {
    throw new Error(\"Expected \" + (component ? \"component\" : \"object\") + \" `\" + id + \"` to be defined: you likely forgot to import, pass, or provide it.\");
}
",
        "should support providing component objects",
    );

    assert_eq!(
        from_markdown("export function A() {
    return <B />
}

<A />
", &Default::default())?,
        "export function A() {
    return <B />;
}
function _createMdxContent(props) {
    return <A />;
}
function MDXContent(props = {}) {
    const { wrapper: MDXLayout  } = props.components || {};
    return MDXLayout ? <MDXLayout {...props}><_createMdxContent {...props}/></MDXLayout> : _createMdxContent(props);
}
export default MDXContent;
",
        "should not support passing components in locally defined components",
    );

    assert_eq!(
        from_markdown("export function A() {
    return <B />
}

<A />
", &RewriteOptions {
    provider_import_source: Some("x".into()),
    ..Default::default()
})?,
        "import { useMDXComponents as _provideComponents } from \"x\";
export function A() {
    const { B  } = _provideComponents();
    if (!B) _missingMdxReference(\"B\", true);
    return <B />;
}
function _createMdxContent(props) {
    return <A />;
}
function MDXContent(props = {}) {
    const { wrapper: MDXLayout  } = Object.assign({}, _provideComponents(), props.components);
    return MDXLayout ? <MDXLayout {...props}><_createMdxContent {...props}/></MDXLayout> : _createMdxContent(props);
}
export default MDXContent;
function _missingMdxReference(id, component) {
    throw new Error(\"Expected \" + (component ? \"component\" : \"object\") + \" `\" + id + \"` to be defined: you likely forgot to import, pass, or provide it.\");
}
",
        "should support providing components in locally defined components",
    );

    assert_eq!(
        from_markdown("export function A() {
    return <b-c />
}

<A />
", &RewriteOptions {
    provider_import_source: Some("x".into()),
    ..Default::default()
})?,
        "import { useMDXComponents as _provideComponents } from \"x\";
export function A() {
    const _components = Object.assign({
        \"b-c\": \"b-c\"
    }, _provideComponents()), _component0 = _components[\"b-c\"];
    return <_component0 />;
}
function _createMdxContent(props) {
    return <A />;
}
function MDXContent(props = {}) {
    const { wrapper: MDXLayout  } = Object.assign({}, _provideComponents(), props.components);
    return MDXLayout ? <MDXLayout {...props}><_createMdxContent {...props}/></MDXLayout> : _createMdxContent(props);
}
export default MDXContent;
",
        "should support providing components with JSX identifiers that are not JS identifiers in locally defined components",
    );

    assert_eq!(
        from_markdown("export function A() {
    return <X />, <X.y />, <Y.Z />
}

<A />
", &RewriteOptions {
    provider_import_source: Some("x".into()),
    ..Default::default()
})?,
        "import { useMDXComponents as _provideComponents } from \"x\";
export function A() {
    const { X , Y  } = _provideComponents();
    if (!X) _missingMdxReference(\"X\", true);
    if (!X.y) _missingMdxReference(\"X.y\", true);
    if (!Y) _missingMdxReference(\"Y\", false);
    if (!Y.Z) _missingMdxReference(\"Y.Z\", true);
    return <X />, <X.y />, <Y.Z />;
}
function _createMdxContent(props) {
    return <A />;
}
function MDXContent(props = {}) {
    const { wrapper: MDXLayout  } = Object.assign({}, _provideComponents(), props.components);
    return MDXLayout ? <MDXLayout {...props}><_createMdxContent {...props}/></MDXLayout> : _createMdxContent(props);
}
export default MDXContent;
function _missingMdxReference(id, component) {
    throw new Error(\"Expected \" + (component ? \"component\" : \"object\") + \" `\" + id + \"` to be defined: you likely forgot to import, pass, or provide it.\");
}
",
        "should support providing components with JSX identifiers that are not JS identifiers in locally defined components",
    );

    assert_eq!(
        from_markdown("# <Hi />", &RewriteOptions {
            development: true,
            ..Default::default()
        })?,
        "function _createMdxContent(props) {
    const _components = Object.assign({
        h1: \"h1\"
    }, props.components), { Hi  } = _components;
    if (!Hi) _missingMdxReference(\"Hi\", true, \"1:3-1:9\");
    return <_components.h1 ><Hi /></_components.h1>;
}
function MDXContent(props = {}) {
    const { wrapper: MDXLayout  } = props.components || {};
    return MDXLayout ? <MDXLayout {...props}><_createMdxContent {...props}/></MDXLayout> : _createMdxContent(props);
}
export default MDXContent;
function _missingMdxReference(id, component, place) {
    throw new Error(\"Expected \" + (component ? \"component\" : \"object\") + \" `\" + id + \"` to be defined: you likely forgot to import, pass, or provide it.\" + (place ? \"\\nIt’s referenced in your code at `\" + place + \"` in `example.mdx`\" : \"\"));
}
",
        "should create missing reference helpers w/o positional info in `development` mode",
    );

    Ok(())
}
