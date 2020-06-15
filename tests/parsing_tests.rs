use snekdown::parse;
use snekdown::parser::elements::Block;
use snekdown::Parser;

macro_rules! count_block_elements {
    ($document:expr, $filter:expr) => {
        $document
            .elements
            .iter()
            .filter($filter)
            .collect::<Vec<&Block>>()
            .len()
    };
}

#[test]
fn it_inits() {
    let _ = Parser::new("".to_string(), None);
}

#[test]
fn it_parses_sections() {
    let document = parse!("# Section\n## Subsection\n# Section");
    assert_eq!(
        count_block_elements!(document, |e| if let Block::Section(_) = e {
            true
        } else {
            false
        }),
        2
    )
}

#[test]
fn it_parses_tables() {
    let document = parse!("|header|header|\n|---|---|\n|col|col|");
    assert_eq!(
        count_block_elements!(document, |e| if let Block::Table(_) = e {
            true
        } else {
            false
        }),
        1
    )
}

#[test]
fn it_parses_paragraphs() {
    let document = parse!("**Bold***Italic*_Underline_`Monospace`^super^~strike~");
    assert_eq!(
        count_block_elements!(document, |e| if let Block::Paragraph(_) = e {
            true
        } else {
            false
        }),
        1
    )
}

#[test]
fn it_parses_lists() {
    let document = parse!("- item1\n- item2\n\n* item\n+ item\n\no item\n1. item");
    assert_eq!(
        count_block_elements!(document, |e| if let Block::List(l) = e {
            l.items.len() == 2
        } else {
            false
        }),
        3
    )
}

#[test]
fn it_parses_code_blocks() {
    let document = parse!("```\ncode\n```\n```rust\ncode\n``````");
    assert_eq!(
        count_block_elements!(document, |e| if let Block::CodeBlock(_) = e {
            true
        } else {
            false
        }),
        2
    )
}

#[test]
fn it_parses_quotes() {
    let document = parse!("> quote\n\n[meta]> quote\n>hm");
    assert_eq!(
        count_block_elements!(document, |e| if let Block::Quote(_) = e {
            true
        } else {
            false
        }),
        2
    )
}
