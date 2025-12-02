use pretty::Arena;

use crate::{error::Error, parser::parse};

pub fn format_text<'src>(src: &'src str) -> Result<String, Error<'src>> {
    let tree = parse(src)?;
    let arena = Arena::<()>::new();

    let formatted = tree.to_doc(&arena).pretty(100).to_string();

    // TODO: This might also remove whitespace from multiline strings
    Ok(formatted
        .split('\n')
        .map(|s| s.trim_end())
        .collect::<Vec<_>>()
        .join("\n"))
}
