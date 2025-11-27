use crate::{doc::DocExt, parser::parse};

pub fn format_text(src: &str) -> Result<String, ()> {
    let tree = parse(src).unwrap();
    let formatted = tree.to_doc().pretty(120).to_string();
    Ok(formatted)
}
