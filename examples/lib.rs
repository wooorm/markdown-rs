extern crate micromark;
use micromark::{micromark, micromark_to_mdast, micromark_with_options, Constructs, Options};

fn main() -> Result<(), String> {
    // Turn on debugging.
    // You can show it with `RUST_LOG=debug cargo run --example lib`
    env_logger::init();

    // Safely turn (untrusted?) markdown into HTML.
    println!("{:?}", micromark("## Hello, *world*!"));

    // Turn trusted markdown into HTML.
    println!(
        "{:?}",
        micromark_with_options(
            "<div style=\"color: tomato\">\n\n# Hello, tomato!\n\n</div>",
            &Options {
                allow_dangerous_html: true,
                allow_dangerous_protocol: true,
                ..Options::default()
            }
        )
    );

    // Support GFM extensions.
    println!(
        "{}",
        micromark_with_options(
            "* [x] contact@example.com ~~strikethrough~~",
            &Options {
                constructs: Constructs::gfm(),
                gfm_tagfilter: true,
                ..Options::default()
            }
        )?
    );

    // Access syntax tree and support MDX extensions:
    println!(
        "{:?}",
        micromark_to_mdast(
            "# <HelloMessage />, {username}!",
            &Options {
                constructs: Constructs::mdx(),
                ..Options::default()
            }
        )?
    );

    Ok(())
}
