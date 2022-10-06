extern crate micromark;
use micromark::{
    mdast::{Node, Root, Toml, Yaml},
    micromark, micromark_to_mdast, micromark_with_options,
    unist::Position,
    Constructs, Options, ParseOptions,
};
use pretty_assertions::assert_eq;

#[test]
fn frontmatter() -> Result<(), String> {
    let frontmatter = Options {
        parse: ParseOptions {
            constructs: Constructs {
                frontmatter: true,
                ..Constructs::default()
            },
            ..ParseOptions::default()
        },
        ..Options::default()
    };

    assert_eq!(
        micromark("---\ntitle: Jupyter\n---"),
        "<hr />\n<h2>title: Jupyter</h2>",
        "should not support frontmatter by default"
    );

    assert_eq!(
        micromark_with_options("---\ntitle: Jupyter\n---", &frontmatter)?,
        "",
        "should support frontmatter (yaml)"
    );

    assert_eq!(
        micromark_with_options("+++\ntitle = \"Jupyter\"\n+++", &frontmatter)?,
        "",
        "should support frontmatter (toml)"
    );

    assert_eq!(
        micromark_with_options("---\n---", &frontmatter)?,
        "",
        "should support empty frontmatter"
    );

    assert_eq!(
        micromark_with_options("---\n---\n## Neptune", &frontmatter)?,
        "<h2>Neptune</h2>",
        "should support content after frontmatter"
    );

    assert_eq!(
        micromark_with_options("## Neptune\n---\n---", &frontmatter)?,
        "<h2>Neptune</h2>\n<hr />\n<hr />",
        "should not support frontmatter after content"
    );

    assert_eq!(
        micromark_with_options("> ---\n> ---\n> ## Neptune", &frontmatter)?,
        "<blockquote>\n<hr />\n<hr />\n<h2>Neptune</h2>\n</blockquote>",
        "should not support frontmatter in a container"
    );

    assert_eq!(
        micromark_with_options("---", &frontmatter)?,
        "<hr />",
        "should not support just an opening fence"
    );

    assert_eq!(
        micromark_with_options("---\ntitle: Neptune", &frontmatter)?,
        "<hr />\n<p>title: Neptune</p>",
        "should not support a missing closing fence"
    );

    assert_eq!(
        micromark_to_mdast("---\na: b\n---", &frontmatter.parse)?,
        Node::Root(Root {
            children: vec![Node::Yaml(Yaml {
                value: "a: b".to_string(),
                position: Some(Position::new(1, 1, 0, 3, 4, 12))
            })],
            position: Some(Position::new(1, 1, 0, 3, 4, 12))
        }),
        "should support yaml as `Yaml`s in mdast"
    );

    assert_eq!(
        micromark_to_mdast("+++\ntitle = \"Jupyter\"\n+++", &frontmatter.parse)?,
        Node::Root(Root {
            children: vec![Node::Toml(Toml {
                value: "title = \"Jupyter\"".to_string(),
                position: Some(Position::new(1, 1, 0, 3, 4, 25))
            })],
            position: Some(Position::new(1, 1, 0, 3, 4, 25))
        }),
        "should support toml as `Toml`s in mdast"
    );

    Ok(())
}
