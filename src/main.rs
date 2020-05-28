use markdown_rs::parser::Parser;
use std::fs::{read_to_string, write};

fn main() {
    let mut parser = Parser::new(read_to_string("test/document.md").unwrap());
    let document = parser.parse();
    write("test/document.ast", format!("{:#?}", document)).unwrap();
}
