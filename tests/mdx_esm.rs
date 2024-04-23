mod test_utils;
use markdown::{
    mdast::{MdxjsEsm, Node, Root},
    message, to_html_with_options, to_mdast,
    unist::Position,
    Constructs, Options, ParseOptions,
};
use pretty_assertions::assert_eq;
use test_utils::swc::{parse_esm, parse_expression};

#[test]
fn mdx_esm() -> Result<(), message::Message> {
    let swc = Options {
        parse: ParseOptions {
            constructs: Constructs::mdx(),
            mdx_esm_parse: Some(Box::new(parse_esm)),
            mdx_expression_parse: Some(Box::new(parse_expression)),
            ..Default::default()
        },
        ..Default::default()
    };

    assert_eq!(
        to_html_with_options("import a from 'b'\n\nc", &swc)?,
        "<p>c</p>",
        "should support an import"
    );

    assert_eq!(
        to_html_with_options("export default a\n\nb", &swc)?,
        "<p>b</p>",
        "should support an export"
    );

    assert_eq!(
        to_html_with_options("impossible", &swc)?,
        "<p>impossible</p>",
        "should not support other keywords (`impossible`)"
    );

    assert_eq!(
        to_html_with_options("exporting", &swc)?,
        "<p>exporting</p>",
        "should not support other keywords (`exporting`)"
    );

    assert_eq!(
        to_html_with_options("import.", &swc)?,
        "<p>import.</p>",
        "should not support a non-whitespace after the keyword"
    );

    assert_eq!(
        to_html_with_options("import('a')", &swc)?,
        "<p>import('a')</p>",
        "should not support a non-whitespace after the keyword (import-as-a-function)"
    );

    assert_eq!(
        to_html_with_options("  import a from 'b'\n  export default c", &swc)?,
        "<p>import a from 'b'\nexport default c</p>",
        "should not support an indent"
    );

    assert_eq!(
        to_html_with_options("- import a from 'b'\n> export default c", &swc)?,
        "<ul>\n<li>import a from 'b'</li>\n</ul>\n<blockquote>\n<p>export default c</p>\n</blockquote>",
        "should not support keywords in containers"
    );

    assert_eq!(
        to_html_with_options("import a from 'b'\nexport default c", &swc)?,
        "",
        "should support imports and exports in the same “block”"
    );

    assert_eq!(
        to_html_with_options("import a from 'b'\n\nexport default c", &swc)?,
        "",
        "should support imports and exports in separate “blocks”"
    );

    assert_eq!(
        to_html_with_options("a\n\nimport a from 'b'\n\nb\n\nexport default c", &swc)?,
        "<p>a</p>\n<p>b</p>\n",
        "should support imports and exports in between other constructs"
    );

    assert_eq!(
        to_html_with_options("a\nimport a from 'b'\n\nb\nexport default c", &swc)?,
        "<p>a\nimport a from 'b'</p>\n<p>b\nexport default c</p>",
        "should not support import/exports when interrupting paragraphs"
    );

    assert_eq!(
        to_html_with_options("import a", &swc)
            .err()
            .unwrap()
            .to_string(),
        "1:9: Could not parse esm with swc: Expected ',', got '<eof>' (mdx:swc)",
        "should crash on invalid import/exports (1)"
    );

    assert_eq!(
        to_html_with_options("import 1/1", &swc)
            .err()
            .unwrap()
            .to_string(),
        "1:8: Could not parse esm with swc: Expected 'from', got 'numeric literal (1, 1)' (mdx:swc)",
        "should crash on invalid import/exports (2)"
    );

    assert_eq!(
        to_html_with_options("export {\n  a\n} from 'b'\n\nc", &swc)?,
        "<p>c</p>",
        "should support line endings in import/exports"
    );

    assert_eq!(
        to_html_with_options("export {\n\n  a\n\n} from 'b'\n\nc", &swc)?,
        "<p>c</p>",
        "should support blank lines in import/exports"
    );

    assert_eq!(
        to_html_with_options("import a from 'b'\n*md*?", &swc)
            .err()
            .unwrap()
            .to_string(),
        "2:6: Could not parse esm with swc: Expression expected (mdx:swc)",
        "should crash on markdown after import/export w/o blank line"
    );

    assert_eq!(
        to_html_with_options("export var a = 1\n// b\n/* c */\n\nd", &swc)?,
        "<p>d</p>",
        "should support comments in “blocks”"
    );

    assert_eq!(
        to_html_with_options("export var a = 1\nvar b\n\nc", &swc)
            .err()
            .unwrap()
            .to_string(),
        "2:1: Unexpected statement in code: only import/exports are supported (mdx:swc)",
        "should crash on other statements in “blocks”"
    );

    assert_eq!(
        to_html_with_options("import ('a')\n\nb", &swc)
            .err()
            .unwrap()
            .to_string(),
        "1:1: Unexpected statement in code: only import/exports are supported (mdx:swc)",
        "should crash on import-as-a-function with a space `import (x)`"
    );

    assert_eq!(
        to_html_with_options("import a from 'b'\nexport {a}\n\nc", &swc)?,
        "<p>c</p>",
        "should support a reexport from another import"
    );

    assert_eq!(
        to_html_with_options("import a from 'b';\nexport {a};\n\nc", &swc)?,
        "<p>c</p>",
        "should support a reexport from another import w/ semicolons"
    );

    assert_eq!(
        to_html_with_options("import a from 'b'\nexport {a as default}\n\nc", &swc)?,
        "<p>c</p>",
        "should support a reexport default from another import"
    );

    assert_eq!(
        to_html_with_options("export var a = () => <b />", &swc)?,
        "",
        "should support JSX by default"
    );

    assert_eq!(
        to_html_with_options("export {a}\n", &swc)?,
        "",
        "should support EOF after EOL"
    );

    assert_eq!(
        to_html_with_options("import a from 'b'\n\nexport {a}\n\nc", &swc)?,
        "<p>c</p>",
        "should support a reexport from another esm block (1)"
    );

    assert_eq!(
        to_html_with_options("import a from 'b'\n\nexport {a}\n\n# c", &swc)?,
        "<h1>c</h1>",
        "should support a reexport from another esm block (2)"
    );

    let cases = vec![
        ("default", "import a from \"b\""),
        ("whole", "import * as a from \"b\""),
        ("destructuring", "import {a} from \"b\""),
        ("destructuring and rename", "import {a as b} from \"c\""),
        ("default and destructuring", "import a, {b as c} from \"d\""),
        ("default and whole", "import a, * as b from \"c\""),
        ("side-effects", "import \"a\""),
    ];

    for case in cases {
        assert_eq!(
            to_html_with_options(case.1, &swc)?,
            "",
            "should support imports: {}",
            case.0
        );
    }

    let cases = vec![
        ("var", "export var a = \"\""),
        ("const", "export const a = \"\""),
        ("let", "export let a = \"\""),
        ("multiple", "export var a, b"),
        ("multiple w/ assignment", "export var a = \"a\", b = \"b\""),
        ("function", "export function a() {}"),
        ("class", "export class a {}"),
        ("destructuring", "export var {a} = {}"),
        ("rename destructuring", "export var {a: b} = {}"),
        ("array destructuring", "export var [a] = []"),
        ("default", "export default a = 1"),
        ("default function", "export default function a() {}"),
        ("default class", "export default class a {}"),
        ("aggregate", "export * from \"a\""),
        ("whole reexport", "export * as a from \"b\""),
        ("reexport destructuring", "export {a} from \"b\""),
        (
            "reexport destructuring w rename",
            "export {a as b} from \"c\"",
        ),
        ("reexport as a default whole", "export {default} from \"b\""),
        (
            "reexport default and non-default",
            "export {default as a, b} from \"c\"",
        ),
    ];

    for case in cases {
        assert_eq!(
            to_html_with_options(case.1, &swc)?,
            "",
            "should support exports: {}",
            case.0
        );
    }

    assert_eq!(
        to_mdast("import a from 'b'\nexport {a}", &swc.parse)?,
        Node::Root(Root {
            children: vec![Node::MdxjsEsm(MdxjsEsm {
                value: "import a from 'b'\nexport {a}".into(),
                position: Some(Position::new(1, 1, 0, 2, 11, 28)),
                stops: vec![(0, 0), (17, 17), (18, 18)]
            })],
            position: Some(Position::new(1, 1, 0, 2, 11, 28))
        }),
        "should support mdx esm as `MdxjsEsm`s in mdast"
    );

    Ok(())
}
