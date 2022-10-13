extern crate markdown;
extern crate swc_common;
extern crate swc_ecma_ast;
extern crate swc_ecma_codegen;
mod test_utils;
use pretty_assertions::assert_eq;
use test_utils::mdx::{mdx, JsxRuntime, Options};

#[test]
fn mdx_test() -> Result<(), String> {
    // To do: JSX should be compiled away.
    assert_eq!(
        mdx("", Some("example.mdx".into()), &Options::default())?,
        "function _createMdxContent(props) {
    return <></>;
}
function MDXContent(props = {}) {
    const { wrapper: MDXLayout  } = props.components || {};
    return MDXLayout ? <MDXLayout {...props}><_createMdxContent {...props}/></MDXLayout> : _createMdxContent(props);
}
export default MDXContent;
",
        "should work",
    );

    // To do: JSX should be compiled away.
    assert_eq!(
        mdx("<A />", Some("example.mdx".into()), &Options {
            development: true,
            ..Default::default()
        })?,
        "function _createMdxContent(props) {
    const { A  } = props.components || {};
    if (!A) _missingMdxReference(\"A\", true, \"1:1-1:6\");
    return <A />;
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
        "should support `options.development: true`",
    );

    // To do: JSX should be compiled away.
    assert_eq!(
        mdx("<A />", Some("example.mdx".into()), &Options {
            provider_import_source: Some("@mdx-js/react".into()),
            ..Default::default()
        })?,
        "import { useMDXComponents as _provideComponents } from \"@mdx-js/react\";
function _createMdxContent(props) {
    const { A  } = Object.assign({}, _provideComponents(), props.components);
    if (!A) _missingMdxReference(\"A\", true);
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
        "should support `options.provider_import_source`",
    );

    assert_eq!(
        mdx("", Some("example.mdx".into()), &Options {
            jsx: true,
            ..Default::default()
        })?,
        "function _createMdxContent(props) {
    return <></>;
}
function MDXContent(props = {}) {
    const { wrapper: MDXLayout  } = props.components || {};
    return MDXLayout ? <MDXLayout {...props}><_createMdxContent {...props}/></MDXLayout> : _createMdxContent(props);
}
export default MDXContent;
",
        "should support `options.jsx: true`",
    );

    // To do: JSX should be compiled away.
    // To do: should use calls of `React.createElement` / `React.Fragment`.
    assert_eq!(
        mdx("", Some("example.mdx".into()), &Options {
            jsx_runtime: Some(JsxRuntime::Classic),
            ..Default::default()
        })?,
        "import { React } from \"react\";
function _createMdxContent(props) {
    return <></>;
}
function MDXContent(props = {}) {
    const { wrapper: MDXLayout  } = props.components || {};
    return MDXLayout ? <MDXLayout {...props}><_createMdxContent {...props}/></MDXLayout> : _createMdxContent(props);
}
export default MDXContent;
",
        "should support `options.jsx_runtime: JsxRuntime::Classic`",
    );

    // To do: JSX should be compiled away.
    // To do: should import `_jsx` and such.
    // To do: should use calls of `_jsx`, etc.
    assert_eq!(
        mdx("", Some("example.mdx".into()), &Options {
            jsx_import_source: Some("preact".into()),
            ..Default::default()
        })?,
        "function _createMdxContent(props) {
    return <></>;
}
function MDXContent(props = {}) {
    const { wrapper: MDXLayout  } = props.components || {};
    return MDXLayout ? <MDXLayout {...props}><_createMdxContent {...props}/></MDXLayout> : _createMdxContent(props);
}
export default MDXContent;
",
        "should support `options.jsx_import_source: Some(\"preact\".into())`",
    );

    // To do: JSX should be compiled away.
    // To do: should use calls of `a.b`, symbol of `a.c`.
    assert_eq!(
        mdx("", Some("example.mdx".into()), &Options {
            jsx_runtime: Some(JsxRuntime::Classic),
            pragma: Some("a.b".into()),
            pragma_frag: Some("a.c".into()),
            pragma_import_source: Some("d".into()),
            ..Default::default()
        })?,
        "import { a } from \"d\";
function _createMdxContent(props) {
    return <></>;
}
function MDXContent(props = {}) {
    const { wrapper: MDXLayout  } = props.components || {};
    return MDXLayout ? <MDXLayout {...props}><_createMdxContent {...props}/></MDXLayout> : _createMdxContent(props);
}
export default MDXContent;
",
        "should support `options.pragma`, `options.pragma_frag`, `options.pragma_import_source`",
    );

    Ok(())
}
