fn main() -> Result<(), markdown::message::Message> {
    println!(
        "{:?}",
        markdown::to_mdast("# Hi *Earth*!", &markdown::ParseOptions::default())?
    );

    Ok(())
}
