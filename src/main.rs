use std::{
    io::{self, Read},
    process::exit,
};

use crate::format::format_text;

mod doc;
mod doc_ext;
mod error;
mod format;
mod kind;
mod lexer;
mod node;
mod parser;
mod peekable_ext;

fn read_stdin() -> Result<String, io::Error> {
    let mut buf = String::new();
    io::stdin().read_to_string(&mut buf)?;
    Ok(buf)
}

fn main() {
    let input = read_stdin().unwrap_or_else(|e| panic!("Unable to read input from stdin: {}", e));

    match format_text(&input) {
        Ok(formatted) => print!("{}", formatted),
        Err(error) => {
            error.print(&input);
            exit(1);
        }
    }
}
