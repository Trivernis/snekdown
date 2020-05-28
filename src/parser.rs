use crate::tokens::*;
use std::error::Error;
use std::fmt;
use std::fmt::{Display, Formatter};

macro_rules! parse_option {
    ($option:expr, $index:expr) => {
        if let Some(_) = $option {
        } else {
            return Err(ParseError::new($index));
        }
    };
}

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
    section_return: Option<u8>,
}

impl Parser {
    pub fn new(text: String) -> Self {
        let text: Vec<char> = text.chars().collect();
        let current_char = text.get(0).unwrap().clone();

        Self {
            index: 0,
            text,
            current_char,
            section_nesting: 0,
            section_return: None,
        }
    }

    /// Increments the current index and returns the
    /// char at the indexes position
    pub fn next_char(&mut self) -> Option<char> {
        self.index += 1;

        self.current_char = self.text.get(self.index)?.clone();

        Some(self.current_char)
    }

    /// Returns to an index position
    pub fn revert_to(&mut self, index: usize) -> Result<(), ParseError> {
        self.index = index - 1;
        if let Some(_) = self.next_char() {
            Ok(())
        } else {
            Err(ParseError::new(index))
        }
    }

    /// Skips characters until it encounters a character
    /// that isn't an inline whitespace character
    pub fn seek_inline_whitespace(&mut self) {
        if self.current_char.is_whitespace() && self.current_char != '\n' {
            while let Some(next_char) = self.next_char() {
                if !next_char.is_whitespace() || self.current_char == '\n' {
                    break;
                }
            }
        }
    }

    /// Skips characters until it encounters a character
    /// that isn't a whitespace character
    pub fn seek_whitespace(&mut self) {
        if self.current_char.is_whitespace() {
            while let Some(next_char) = self.next_char() {
                if !next_char.is_whitespace() {
                    break;
                }
            }
        }
    }

    pub fn parse(&mut self) -> Document {
        let mut document = Document::new();
        while self.index < self.text.len() {
            if let Ok(token) = self.parse_block() {
                document.add_element(token);
            }
        }

        document
    }

    /// Parses a block Token
    pub fn parse_block(&mut self) -> Result<Block, ParseError> {
        if let Some(section) = self.section_return {
            if section <= self.section_nesting {
                return Err(ParseError::new(self.index));
            } else {
                self.section_return = None;
            }
        }
        let token = if let Ok(section) = self.parse_section() {
            Block::Section(section)
        } else if let Some(_) = self.section_return {
            return Err(ParseError::new(self.index));
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
    }

    /// Parses a section that consists of a header and one or more blocks
    pub fn parse_section(&mut self) -> Result<Section, ParseError> {
        let start_index = self.index;
        self.seek_whitespace();
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
                if size <= self.section_nesting {
                    self.section_return = Some(size);
                }
                self.revert_to(start_index)?;
                return Err(ParseError::new(index));
            }
            self.seek_inline_whitespace();
            let mut header = self.parse_header()?;
            header.size = size;
            self.section_nesting = size;
            let mut section = Section::new(header);

            while let Ok(block) = self.parse_block() {
                section.add_element(block);
            }

            self.section_nesting -= 1;
            Ok(section)
        } else {
            let error_index = self.index;
            self.revert_to(start_index)?;
            Err(ParseError::new(error_index))
        }
    }

    /// Parses a paragraph
    pub fn parse_paragraph(&mut self) -> Result<Paragraph, ParseError> {
        let mut paragraph = Paragraph::new();
        while let Ok(token) = self.parse_inline() {
            paragraph.add_element(token);
            if ['-', '#', '`', '|'].contains(&self.current_char) {
                break;
            }
        }

        if paragraph.elements.len() > 0 {
            Ok(paragraph)
        } else {
            Err(ParseError::new(self.index))
        }
    }

    /// parses a list which consists of one or more list items
    pub fn parse_list(&mut self) -> Result<List, ParseError> {
        let mut list = List::new();
        let start_index = self.index;
        self.seek_whitespace();
        while let Ok(token) = self.parse_list_item() {
            list.add_item(token);
        }

        if list.items.len() > 0 {
            Ok(list)
        } else {
            self.revert_to(start_index)?;
            Err(ParseError::new(self.index))
        }
    }

    pub fn parse_table(&mut self) -> Result<Table, ParseError> {
        Err(ParseError::new(self.index))
    }

    pub fn parse_header(&mut self) -> Result<Header, ParseError> {
        Ok(Header {
            size: 0,
            line: self.parse_inline()?,
        })
    }

    /// parses a single list item defined with -
    pub fn parse_list_item(&mut self) -> Result<ListItem, ParseError> {
        let start_index = self.index;
        self.seek_inline_whitespace();

        if self.current_char != '-' {
            let err = ParseError::new(self.index);
            self.revert_to(start_index)?;
            return Err(err);
        }
        self.seek_inline_whitespace();
        let item = ListItem {
            text: self.parse_inline()?,
        };

        Ok(item)
    }

    pub fn parse_inline(&mut self) -> Result<Inline, ParseError> {
        if self.index > self.text.len() {
            Err(ParseError::new(self.index))
        } else {
            Ok(Inline::Text(self.parse_text()?))
        }
    }

    /// Parses a line of text
    pub fn parse_text(&mut self) -> Result<Text, ParseError> {
        let mut text = Text::new();
        while let Ok(subtext) = self.parse_subtext() {
            text.add_subtext(subtext);
            let current_index = self.index;
            if self.next_char() == None {
                break;
            }
            self.revert_to(current_index)?;
        }

        if self.current_char == '\n' {
            parse_option!(self.next_char(), self.index);
        }

        Ok(text)
    }

    pub fn parse_subtext(&mut self) -> Result<SubText, ParseError> {
        match self.current_char {
            '*' => {
                parse_option!(self.next_char(), self.index);

                if self.current_char == '*' {
                    parse_option!(self.next_char(), self.index);
                    let subtext = self.parse_subtext()?;
                    if self.current_char == '*' {
                        parse_option!(self.next_char(), self.index);
                        if self.current_char == '*' {
                            parse_option!(self.next_char(), self.index);
                        }
                    }
                    Ok(SubText::Bold(BoldText {
                        value: Box::new(subtext),
                    }))
                } else {
                    let subtext = self.parse_subtext()?;
                    parse_option!(self.next_char(), self.index);
                    Ok(SubText::Italic(ItalicText {
                        value: Box::new(subtext),
                    }))
                }
            }
            '_' => {
                parse_option!(self.next_char(), self.index);
                let subtext = self.parse_subtext()?;
                parse_option!(self.next_char(), self.index);
                Ok(SubText::Underlined(UnderlinedText {
                    value: Box::new(subtext),
                }))
            }
            '~' => {
                parse_option!(self.next_char(), self.index);
                let subtext = self.parse_subtext()?;
                if self.current_char == '~' {
                    parse_option!(self.next_char(), self.index);
                }
                Ok(SubText::Striked(StrikedText {
                    value: Box::new(subtext),
                }))
            }
            '\n' => Err(ParseError::new(self.index)),
            _ => Ok(SubText::Plain(self.parse_plain_text()?)),
        }
    }

    pub fn parse_plain_text(&mut self) -> Result<PlainText, ParseError> {
        let mut current_char = self.current_char;
        let mut characters = String::new();
        loop {
            match current_char {
                '\n' | '*' | '_' | '~' => break,
                _ => characters.push(current_char),
            }
            if let Some(character) = self.next_char() {
                current_char = character;
            } else {
                break;
            }
        }

        if characters.len() > 0 {
            Ok(PlainText { value: characters })
        } else {
            Err(ParseError::new(self.index))
        }
    }
}
