//! Regenerate CJK emphasis character ranges with:
//! `cargo run --manifest-path generate/Cargo.toml --bin cjk`.
//!
//! Unicode inputs are pinned to 17.0.0 and cached below `target/`.

use std::{error::Error, fmt::Write as _, fs, path::Path};

const VERSION: &str = "17.0.0";
const LIMIT: usize = 0x110000;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let mut wide = vec![false; LIMIT];
    let mut ambiguous = vec![false; LIMIT];
    let mut hangul = vec![false; LIMIT];
    let mut emoji = vec![false; LIMIT];
    let eaw = load("EastAsianWidth.txt").await?;
    fill(&eaw, &["W", "F", "H"], &mut wide);
    fill(&eaw, &["A"], &mut ambiguous);
    fill(&load("Scripts.txt").await?, &["Hangul"], &mut hangul);
    fill(
        &load("emoji/emoji-data.txt").await?,
        &["Emoji_Presentation"],
        &mut emoji,
    );
    let cjk: Vec<bool> = (0..LIMIT)
        .map(|cp| (wide[cp] && !emoji[cp]) || hangul[cp])
        .collect();
    let mut output = format!(
        "//! Generated Unicode {VERSION} data for CJK-friendly emphasis.\n\
         //! Regenerate with `cargo run --manifest-path generate/Cargo.toml --bin cjk`.\n\
         //! Sources: <https://www.unicode.org/Public/{VERSION}/ucd/>\n\
         //! Unicode data is used under <https://www.unicode.org/license.txt>.\n\n\
         /// East Asian W/F/H characters excluding `Emoji_Presentation`, plus Hangul.\n\
         pub const CJK: &[(u32, u32)] = &[\n"
    );
    let mut start = None;
    for (cp, is_cjk) in cjk
        .iter()
        .copied()
        .chain(std::iter::once(false))
        .enumerate()
    {
        if is_cjk {
            start.get_or_insert(cp);
        } else if let Some(first) = start.take() {
            writeln!(output, "    (0x{first:X}, 0x{:X}),", cp - 1)?;
        }
    }
    output.push_str(
        "];

/// Ambiguous-width punctuation with a standardized fullwidth variation.
pub const CJK_AMBIGUOUS_PUNCTUATION: &[(u32, u32)] = &[\n",
    );
    let mut variants = vec![];
    for line in load("StandardizedVariants.txt").await?.lines() {
        let fields: Vec<_> = line.split('#').next().unwrap().split(';').collect();
        if fields.len() < 2
            || (!fields[1].contains("fullwidth form") || fields[1].contains("non-fullwidth form"))
        {
            continue;
        }
        let cps: Vec<_> = fields[0].split_whitespace().collect();
        let base = u32::from_str_radix(cps[0], 16)?;
        let selector = u32::from_str_radix(cps[1], 16)?;
        if ambiguous[base as usize] {
            variants.push((base, selector));
        }
    }
    variants.sort_unstable();
    variants.dedup();
    for (base, selector) in variants {
        writeln!(output, "    (0x{base:X}, 0x{selector:X}),")?;
    }
    output.push_str("];\n");
    fs::write("src/util/unicode_cjk.rs", output)?;
    Ok(())
}

/// Load a versioned Unicode source, reusing its local cache for offline runs.
async fn load(name: &str) -> Result<String, Box<dyn Error>> {
    let path = Path::new("target")
        .join(format!("cjk-unicode-{VERSION}"))
        .join(name);
    if path.exists() {
        return Ok(fs::read_to_string(path)?);
    }
    let value = reqwest::get(format!(
        "https://www.unicode.org/Public/{VERSION}/ucd/{name}"
    ))
    .await?
    .error_for_status()?
    .text()
    .await?;
    fs::create_dir_all(path.parent().unwrap())?;
    fs::write(path, &value)?;
    Ok(value)
}

/// Expand the selected properties, including explicitly listed unassigned ranges.
fn fill(data: &str, properties: &[&str], values: &mut [bool]) {
    for line in data.lines() {
        let fields: Vec<_> = line.split('#').next().unwrap().split(';').collect();
        if fields.len() < 2 || !properties.contains(&fields[1].trim()) {
            continue;
        }
        let bounds: Vec<_> = fields[0].trim().split("..").collect();
        let start = usize::from_str_radix(bounds[0], 16).unwrap();
        let end = usize::from_str_radix(bounds.last().unwrap(), 16).unwrap();
        values[start..=end].fill(true);
    }
}
