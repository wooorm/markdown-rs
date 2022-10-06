extern crate micromark;
use micromark::{
    micromark, micromark_to_mdast, micromark_with_options, CompileOptions, Options, ParseOptions,
};

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
                compile: CompileOptions {
                    allow_dangerous_html: true,
                    allow_dangerous_protocol: true,
                    ..CompileOptions::default()
                },
                ..Options::default()
            }
        )
    );

    // Support GFM extensions.
    println!(
        "{}",
        micromark_with_options(
            "* [x] contact@example.com ~~strikethrough~~",
            &Options::gfm()
        )?
    );

    // Access syntax tree and support MDX extensions:
    println!(
        "{:?}",
        micromark_to_mdast("# <HelloMessage />, {username}!", &ParseOptions::mdx())?
    );

    Ok(())
}
