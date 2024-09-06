use markdown::mdast::{Node, ThematicBreak};
use mdast_util_to_markdown::to_markdown as to;
use mdast_util_to_markdown::to_markdown_with_options as to_md_with_opts;

use mdast_util_to_markdown::Options;
use pretty_assertions::assert_eq;

#[test]
fn thematic_break() {
    assert_eq!(
        to(&Node::ThematicBreak(ThematicBreak { position: None })).unwrap(),
        "***\n",
        "should support a thematic break"
    );

    assert_eq!(
        to_md_with_opts(
            &Node::ThematicBreak(ThematicBreak { position: None }),
            &Options {
                rule: '-',
                ..Default::default()
            }
        )
        .unwrap(),
        "---\n",
        "should support a thematic break w/ dashes when `rule: \"-\"`"
    );

    assert_eq!(
        to_md_with_opts(
            &Node::ThematicBreak(ThematicBreak { position: None }),
            &Options {
                rule: '_',
                ..Default::default()
            }
        )
        .unwrap(),
        "___\n",
        "should support a thematic break w/ underscores when `rule: \"_\"`"
    );

    assert_eq!(
        to_md_with_opts(
            &Node::ThematicBreak(ThematicBreak { position: None }),
            &Options {
                rule: '.',
                ..Default::default()
            }
        ),
        Err("Cannot serialize rules with `.` for `options.rule`, expected `*`, `-`, or `_`".into()),
        "should throw on when given an incorrect `rule`"
    );

    assert_eq!(
        to_md_with_opts(
            &Node::ThematicBreak(ThematicBreak { position: None }),
            &Options {
                rule_repetition: 5,
                ..Default::default()
            }
        )
        .unwrap(),
        "*****\n",
        "should support a thematic break w/ more repetitions w/ `rule_repetition`"
    );

    assert_eq!(
        to_md_with_opts(
            &Node::ThematicBreak(ThematicBreak { position: None }),
            &Options {
                rule_repetition: 2,
                ..Default::default()
            }
        ),
        Err("Cannot serialize rules with repetition `2` for `options.rule_repetition`, expected `3` or more".into()),
        "should throw on when given an incorrect `ruleRepetition`"
    );

    assert_eq!(
        to_md_with_opts(
            &Node::ThematicBreak(ThematicBreak { position: None }),
            &Options {
                rule_spaces: true,
                ..Default::default()
            }
        )
        .unwrap(),
        "* * *\n",
        "should support a thematic break w/ spaces w/ `rule_spaces`"
    );
}
