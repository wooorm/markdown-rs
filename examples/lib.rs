extern crate micromark;
use micromark::{micromark, micromark_with_options, Constructs, Options};

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

    // Support other extensions that are not in CommonMark.
    println!(
        "{:?}",
        micromark_with_options(
            "---\ntitle: Neptune\n---\nSome stuff on the moons of Neptune.",
            &Options {
                constructs: Constructs {
                    frontmatter: true,
                    ..Constructs::default()
                },
                ..Options::default()
            }
        )?
    );

    Ok(())
}
