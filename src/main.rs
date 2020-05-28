use markdown_rs::parser::Parser;
use std::fs::{read_to_string, write};
use std::time::Instant;

fn main() {
    let start = Instant::now();
    let mut parser = Parser::new(read_to_string("test/document.md").unwrap());
    let document = parser.parse();
    println!("Total duration: {:?}", start.elapsed());
    write("test/document.ast", format!("{:#?}", document)).unwrap();
}
