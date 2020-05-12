use crate::tokens::*;
use std::error::Error;
use std::fmt;
use std::fmt::{Display, Formatter};

#[derive(Debug)]
pub struct ParseError {
    index: usize,
}
impl Display for ParseError {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "parse error at index {}", self.index)
    }
}
impl Error for ParseError {}
impl ParseError {
    pub fn new(index: usize) -> Self {
        Self { index }
    }
}

pub struct Parser {
    index: usize,
    text: Vec<char>,
    current_char: char,
    section_nesting: u8,
}

impl Parser {
    pub fn next_char(&mut self) -> Option<char> {
        self.index += 1;

        self.current_char = self.text.get(self.index)?.clone();

        Some(self.current_char)
    }

    pub fn revert_to(&mut self, index: usize) -> Result<(), ParseError> {
        self.index = index;
        if let Some(_) = self.next_char() {
            Ok(())
        } else {
            Err(ParseError::new(index))
        }
    }

    pub fn seek_whitespace(&mut self) {
        if self.current_char.is_whitespace() {
            while let Some(next_char) = self.next_char() {
                if !next_char.is_whitespace() {
                    break;
                }
            }
        }
    }

    pub fn parse(&mut self) {
        let mut document = Document::new();
        while self.index < self.text.len() {
            if let Ok(token) = self.parse_block() {
                document.add_element(token);
            }
        }
    }

    pub fn parse_block(&mut self) -> Result<Block, ParseError> {
        if let Some(_) = self.next_char() {
            let token = if let Ok(section) = self.parse_section() {
                Block::Section(section)
            } else if let Ok(list) = self.parse_list() {
                Block::List(list)
            } else if let Ok(table) = self.parse_table() {
                Block::Table(table)
            } else if let Ok(paragraph) = self.parse_paragraph() {
                Block::Paragraph(paragraph)
            } else {
                return Err(ParseError::new(self.index));
            };

            Ok(token)
        } else {
            Err(ParseError::new(self.index))
        }
    }

    /// Parses a section that consists of a header and one or more blocks
    pub fn parse_section(&mut self) -> Result<Section, ParseError> {
        let start_index = self.index;
        if self.current_char == '#' {
            let mut size = 1;
            while let Some(next_char) = self.next_char() {
                if next_char == '#' {
                    size += 1;
                } else {
                    break;
                }
            }
            if size <= self.section_nesting || !self.current_char.is_whitespace() {
                let index = self.index;
                self.revert_to(start_index)?;
                return Err(ParseError::new(index));
            }
            self.seek_whitespace();
            let mut header = self.parse_header()?;
            header.size = size;
            self.section_nesting = size;
            let mut section = Section::new(header);

            while let Ok(block) = self.parse_block() {
                section.add_element(block);
            }

            Ok(section)
        } else {
            Err(ParseError::new(self.index))
        }
    }

    pub fn parse_paragraph(&mut self) -> Result<Paragraph, ParseError> {
        let mut paragraph = Paragraph::new();
        while let Ok(token) = self.parse_inline() {
            paragraph.add_element(token);
        }

        if paragraph.elements.len() > 0 {
            Ok(paragraph)
        } else {
            Err(ParseError::new(self.index))
        }
    }

    pub fn parse_list(&mut self) -> Result<List, ParseError> {
        unimplemented!()
    }

    pub fn parse_table(&mut self) -> Result<Table, ParseError> {
        unimplemented!()
    }

    pub fn parse_header(&mut self) -> Result<Header, ParseError> {
        Ok(Header {
            size: 0,
            line: self.parse_inline()?,
        })
    }

    pub fn parse_list_item(&mut self) -> Result<ListItem, ParseError> {
        unimplemented!()
    }

    pub fn parse_inline(&mut self) -> Result<Inline, ParseError> {
        unimplemented!()
    }

    pub fn parse_text(&mut self) -> Result<Text, ParseError> {
        unimplemented!()
    }
}
