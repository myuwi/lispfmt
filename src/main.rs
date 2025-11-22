use std::io::{self, Read};

mod lexer;
mod token;

fn read_stdin() -> Result<String, io::Error> {
    let mut buf = String::new();
    io::stdin().read_to_string(&mut buf)?;
    Ok(buf)
}

fn main() {
    let input = read_stdin().unwrap_or_else(|e| panic!("Unable to read input from stdin: {}", e));
    let tokens = lexer::tokenize(&input);
    dbg!(tokens);
}
