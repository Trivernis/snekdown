use markdown_rs::format::html::ToHtml;
use markdown_rs::parser::Parser;
use std::fs::{read_to_string, write};
use std::time::Instant;

fn main() {
    let start = Instant::now();
    let mut parser = Parser::new(
        read_to_string("/home/trivernis/Documents/Programming/Rust/markdown-rs/test/document.md")
            .unwrap(),
        Some("/home/trivernis/Documents/Programming/Rust/markdown-rs/test/document.md".to_string()),
    );
    let document = parser.parse();
    println!("Total duration: {:?}", start.elapsed());
    write("test/document.ast", format!("{:#?}", document)).unwrap();
    write("test/document.html", document.to_html()).unwrap()
}
