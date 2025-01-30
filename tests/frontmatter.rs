use markdown::{
    mdast::{Node, Root, Toml, Yaml},
    message, to_html, to_html_with_options, to_mdast,
    unist::Position,
    Constructs, Options, ParseOptions,
};
use pretty_assertions::assert_eq;

#[test]
fn frontmatter() -> Result<(), message::Message> {
    let frontmatter = Options {
        parse: ParseOptions {
            constructs: Constructs {
                frontmatter: true,
                ..Default::default()
            },
            ..Default::default()
        },
        ..Default::default()
    };

    assert_eq!(
        to_html("---\ntitle: Jupyter\n---"),
        "<hr />\n<h2>title: Jupyter</h2>",
        "should not support frontmatter by default"
    );

    assert_eq!(
        to_html_with_options("---\ntitle: Jupyter\n---", &frontmatter)?,
        "",
        "should support frontmatter (yaml)"
    );

    assert_eq!(
        to_html_with_options("+++\ntitle = \"Jupyter\"\n+++", &frontmatter)?,
        "",
        "should support frontmatter (toml)"
    );

    assert_eq!(
        to_html_with_options("---\n---", &frontmatter)?,
        "",
        "should support empty frontmatter"
    );

    assert_eq!(
        to_html_with_options("--\n---", &frontmatter)?,
        "<h2>--</h2>",
        "should not support 2 markers in an opening fence"
    );

    assert_eq!(
        to_html_with_options("----\n---", &frontmatter)?,
        "<hr />\n<hr />",
        "should not support 4 markers in an opening fence"
    );

    assert_eq!(
        to_html_with_options("---\n--", &frontmatter)?,
        "<hr />\n<p>--</p>",
        "should not support 2 markers in a closing fence"
    );

    assert_eq!(
        to_html_with_options("---\n--\n", &frontmatter)?,
        "<hr />\n<p>--</p>\n",
        "should not panic if newline after 2 marker closing fence"
    );

    assert_eq!(
        to_html_with_options("---\n----", &frontmatter)?,
        "<hr />\n<hr />",
        "should not support 4 markers in a closing fence"
    );

    assert_eq!(
        to_html_with_options("---\n---\n## Neptune", &frontmatter)?,
        "<h2>Neptune</h2>",
        "should support content after frontmatter"
    );

    assert_eq!(
        to_html_with_options("--- \t\n---", &frontmatter)?,
        "",
        "should support spaces and tabs after opening fence"
    );

    assert_eq!(
        to_html_with_options("---\n---\t ", &frontmatter)?,
        "",
        "should support spaces and tabs after closing fence"
    );

    assert_eq!(
        to_html_with_options("---\n---\na\nb", &frontmatter)?,
        "<p>a\nb</p>",
        "should support line endings after frontmatter"
    );

    assert_eq!(
        to_html_with_options("--- a\n---", &frontmatter)?,
        "<h2>--- a</h2>",
        "should not support content after opening fence"
    );

    assert_eq!(
        to_html_with_options("---\n--- b", &frontmatter)?,
        "<hr />\n<p>--- b</p>",
        "should not support content after closing fence"
    );

    assert_eq!(
        to_html_with_options("## Neptune\n---\n---", &frontmatter)?,
        "<h2>Neptune</h2>\n<hr />\n<hr />",
        "should not support frontmatter after content"
    );

    assert_eq!(
        to_html_with_options("> ---\n> ---\n> ## Neptune", &frontmatter)?,
        "<blockquote>\n<hr />\n<hr />\n<h2>Neptune</h2>\n</blockquote>",
        "should not support frontmatter in a container"
    );

    assert_eq!(
        to_html_with_options("---", &frontmatter)?,
        "<hr />",
        "should not support just an opening fence"
    );

    assert_eq!(
        to_html_with_options("---\ntitle: Neptune", &frontmatter)?,
        "<hr />\n<p>title: Neptune</p>",
        "should not support a missing closing fence"
    );

    assert_eq!(
        to_html_with_options("---\na\n\nb\n \t\nc\n---", &frontmatter)?,
        "",
        "should support blank lines in frontmatter"
    );

    assert_eq!(
        to_mdast("---\na: b\n---", &frontmatter.parse)?,
        Node::Root(Root {
            children: vec![Node::Yaml(Yaml {
                value: "a: b".into(),
                position: Some(Position::new(1, 1, 0, 3, 4, 12))
            })],
            position: Some(Position::new(1, 1, 0, 3, 4, 12))
        }),
        "should support yaml as `Yaml`s in mdast"
    );

    assert_eq!(
        to_mdast("+++\ntitle = \"Jupyter\"\n+++", &frontmatter.parse)?,
        Node::Root(Root {
            children: vec![Node::Toml(Toml {
                value: "title = \"Jupyter\"".into(),
                position: Some(Position::new(1, 1, 0, 3, 4, 25))
            })],
            position: Some(Position::new(1, 1, 0, 3, 4, 25))
        }),
        "should support toml as `Toml`s in mdast"
    );

    Ok(())
}
