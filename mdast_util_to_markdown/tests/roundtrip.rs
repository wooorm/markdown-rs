use mdast_util_to_markdown::to_markdown as to;

use markdown::to_mdast as from;
use pretty_assertions::assert_eq;

#[test]
fn round_trip() {
    let doc: String = vec![
        "> * Lorem ipsum dolor sit amet",
        ">",
        "> * consectetur adipisicing elit",
        "",
    ]
    .join("\n");
    assert_eq!(to(&from(&doc, &Default::default()).unwrap()).unwrap(), doc);
}
