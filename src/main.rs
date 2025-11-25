use std::io::{self, Read};

use crate::format::format_text;

mod doc;
mod format;
mod kind;
mod node;
mod parser;

fn read_stdin() -> Result<String, io::Error> {
    let mut buf = String::new();
    io::stdin().read_to_string(&mut buf)?;
    Ok(buf)
}

fn main() {
    let input = read_stdin().unwrap_or_else(|e| panic!("Unable to read input from stdin: {}", e));
    let formatted = format_text(&input).unwrap();
    print!("{}", formatted);
}
