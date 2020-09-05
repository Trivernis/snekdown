use super::ParseResult;
use crate::elements::tokens::*;
use crate::elements::{
    Block, CodeBlock, Import, List, ListItem, MathBlock, Paragraph, Quote, Section, Table,
};
use crate::parser::inline::ParseInline;
use crate::parser::line::ParseLine;
use crate::parser::ImportType;
use crate::Parser;
use std::collections::HashMap;

pub(crate) trait ParseBlock {
    fn parse_block(&mut self) -> ParseResult<Block>;
    fn parse_section(&mut self) -> ParseResult<Section>;
    fn parse_code_block(&mut self) -> ParseResult<CodeBlock>;
    fn parse_math_block(&mut self) -> ParseResult<MathBlock>;
    fn parse_quote(&mut self) -> ParseResult<Quote>;
    fn parse_paragraph(&mut self) -> ParseResult<Paragraph>;
    fn parse_list(&mut self) -> ParseResult<List>;
    fn parse_table(&mut self) -> ParseResult<Table>;
    fn parse_import(&mut self) -> ParseResult<Option<Import>>;
}

impl ParseBlock for Parser {
    /// Parses a block Token
    fn parse_block(&mut self) -> ParseResult<Block> {
        if let Some(section) = self.section_return {
            if section <= self.section_nesting && (self.section_nesting > 0) {
                return Err(self.ctm.assert_error(None));
            } else {
                self.section_return = None;
            }
        }
        let token = if let Ok(section) = self.parse_section() {
            Block::Section(section)
        } else if let Some(_) = self.section_return {
            return Err(self.ctm.err());
        } else if let Ok(list) = self.parse_list() {
            Block::List(list)
        } else if let Ok(table) = self.parse_table() {
            Block::Table(table)
        } else if let Ok(code_block) = self.parse_code_block() {
            Block::CodeBlock(code_block)
        } else if let Ok(math_block) = self.parse_math_block() {
            Block::MathBlock(math_block)
        } else if let Ok(quote) = self.parse_quote() {
            Block::Quote(quote)
        } else if let Ok(import) = self.parse_import() {
            if let Some(import) = import {
                Block::Import(import)
            } else {
                Block::Null
            }
        } else if let Some(_) = self.section_return {
            return Err(self.ctm.err());
        } else if let Ok(pholder) = self.parse_placeholder() {
            Block::Placeholder(pholder)
        } else if let Ok(paragraph) = self.parse_paragraph() {
            Block::Paragraph(paragraph)
        } else {
            return Err(self.ctm.err());
        };

        Ok(token)
    }

    /// Parses a section that consists of a header and one or more blocks
    fn parse_section(&mut self) -> ParseResult<Section> {
        let start_index = self.ctm.get_index();
        self.ctm.seek_whitespace();
        if self.ctm.check_char(&HASH) {
            let mut size = 1;
            while let Some(_) = self.ctm.next_char() {
                if !self.ctm.check_char(&HASH) {
                    break;
                }
                size += 1;
            }
            let mut metadata = None;
            if let Ok(meta) = self.parse_inline_metadata() {
                metadata = Some(meta);
            }
            if size <= self.section_nesting || !self.ctm.get_current().is_whitespace() {
                if size <= self.section_nesting {
                    self.section_return = Some(size);
                }
                return Err(self.ctm.rewind_with_error(start_index));
            }
            self.ctm.seek_any(&INLINE_WHITESPACE)?;
            let mut header = self.parse_header()?;
            header.size = size;
            self.section_nesting = size;
            self.sections.push(size);
            let mut section = Section::new(header);
            section.metadata = metadata;
            self.ctm.seek_whitespace();

            while let Ok(block) = self.parse_block() {
                section.add_element(block);
            }

            self.sections.pop();
            if let Some(sec) = self.sections.last() {
                self.section_nesting = *sec
            } else {
                self.section_nesting = 0;
            }
            Ok(section)
        } else {
            return Err(self.ctm.rewind_with_error(start_index));
        }
    }

    /// parses a code block
    fn parse_code_block(&mut self) -> ParseResult<CodeBlock> {
        let start_index = self.ctm.get_index();
        self.ctm.seek_whitespace();
        self.ctm
            .assert_sequence(&SQ_CODE_BLOCK, Some(start_index))?;
        self.ctm.seek_one()?;
        let language = self.ctm.get_string_until_any(&[LB], &[])?;
        self.ctm.seek_one()?;
        let text = self.ctm.get_string_until_sequence(&[&SQ_CODE_BLOCK], &[])?;
        for _ in 0..2 {
            self.ctm.seek_one()?;
        }

        Ok(CodeBlock {
            language,
            code: text,
        })
    }

    /// parses a math block
    fn parse_math_block(&mut self) -> ParseResult<MathBlock> {
        let start_index = self.ctm.get_index();
        self.ctm.seek_whitespace();
        self.ctm.assert_sequence(SQ_MATH, Some(start_index))?;
        self.ctm.seek_one()?;
        let text = self.ctm.get_string_until_sequence(&[SQ_MATH], &[])?;
        for _ in 0..1 {
            self.ctm.seek_one()?;
        }
        Ok(MathBlock {
            expression: asciimath_rs::parse(text),
        })
    }

    /// parses a quote
    fn parse_quote(&mut self) -> ParseResult<Quote> {
        let start_index = self.ctm.get_index();
        self.ctm.seek_whitespace();
        let metadata = if let Ok(meta) = self.parse_inline_metadata() {
            Some(meta)
        } else {
            None
        };
        if self.ctm.check_char(&META_CLOSE) {
            if self.ctm.next_char() == None {
                return Err(self.ctm.rewind_with_error(start_index));
            }
        }
        let mut quote = Quote::new(metadata);

        while self.ctm.check_char(&QUOTE_START)
            && self.ctm.next_char() != None
            && (self.ctm.check_any(&WHITESPACE))
        {
            self.ctm.seek_any(&INLINE_WHITESPACE)?;
            if let Ok(text) = self.parse_text_line() {
                if text.subtext.len() > 0 {
                    quote.add_text(text);
                }
            } else {
                break;
            }
        }
        if quote.text.len() == 0 {
            return Err(self.ctm.rewind_with_error(start_index));
        }

        Ok(quote)
    }

    /// Parses a paragraph
    fn parse_paragraph(&mut self) -> ParseResult<Paragraph> {
        self.ctm.seek_whitespace();
        let mut paragraph = Paragraph::new();
        while let Ok(token) = self.parse_line() {
            paragraph.add_element(token);
            let start_index = self.ctm.get_index();
            if self.ctm.check_any_sequence(&BLOCK_SPECIAL_CHARS)
                || self.ctm.check_any(&self.block_break_at)
            {
                self.ctm.rewind(start_index);
                break;
            }
            if !self.ctm.check_eof() {
                self.ctm.rewind(start_index);
            }
        }

        if paragraph.elements.len() > 0 {
            Ok(paragraph)
        } else {
            Err(self.ctm.err())
        }
    }

    /// parses a list which consists of one or more list items
    /// The parsing is done iterative to resolve nested items
    fn parse_list(&mut self) -> ParseResult<List> {
        let mut list = List::new();
        let start_index = self.ctm.get_index();
        self.ctm.seek_whitespace();

        let ordered = self.ctm.get_current().is_numeric();
        list.ordered = ordered;
        let mut list_hierarchy: Vec<ListItem> = Vec::new();
        while let Ok(mut item) = self.parse_list_item() {
            while let Some(parent_item) = list_hierarchy.pop() {
                if parent_item.level < item.level {
                    // the parent item is the actual parent of the next item
                    list_hierarchy.push(parent_item);
                    break;
                } else if parent_item.level == item.level {
                    // the parent item is a sibling and has to be appended to a parent
                    if list_hierarchy.is_empty() {
                        list.add_item(parent_item);
                    } else {
                        let mut parent_parent = list_hierarchy.pop().unwrap();
                        parent_parent.add_child(parent_item);
                        list_hierarchy.push(parent_parent);
                    }
                    break;
                } else {
                    // the parent item is a child of a sibling of the current item
                    if list_hierarchy.is_empty() {
                        item.add_child(parent_item);
                    } else {
                        let mut parent_parent = list_hierarchy.pop().unwrap();
                        parent_parent.add_child(parent_item);
                        list_hierarchy.push(parent_parent);
                    }
                }
            }
            list_hierarchy.push(item);
        }

        // the remaining items in the hierarchy need to be combined
        while let Some(item) = list_hierarchy.pop() {
            if !list_hierarchy.is_empty() {
                let mut parent_item = list_hierarchy.pop().unwrap();
                parent_item.add_child(item);
                list_hierarchy.push(parent_item);
            } else {
                list_hierarchy.push(item);
                break;
            }
        }
        list.items.append(&mut list_hierarchy);

        if list.items.len() > 0 {
            Ok(list)
        } else {
            return Err(self.ctm.rewind_with_error(start_index));
        }
    }

    /// parses a markdown table
    fn parse_table(&mut self) -> ParseResult<Table> {
        let header = self.parse_row()?;
        if self.ctm.check_char(&LB) {
            self.ctm.seek_one()?;
        }
        let seek_index = self.ctm.get_index();
        let mut table = Table::new(header);
        while let Ok(_) = self.ctm.seek_one() {
            self.ctm.seek_any(&INLINE_WHITESPACE)?;
            if !self.ctm.check_any(&[MINUS, PIPE]) || self.ctm.check_char(&LB) {
                break;
            }
        }

        if !self.ctm.check_char(&LB) {
            self.ctm.rewind(seek_index);
            return Ok(table);
        }

        self.ctm.seek_whitespace();
        while let Ok(row) = self.parse_row() {
            table.add_row(row);
        }

        Ok(table)
    }

    /// parses an import and starts a new task to parse the document of the import
    fn parse_import(&mut self) -> ParseResult<Option<Import>> {
        let start_index = self.ctm.get_index();
        self.ctm.seek_whitespace();
        self.ctm
            .assert_any_sequence(&[&[IMPORT_START, IMPORT_OPEN]], Some(start_index))?;
        let mut path = String::new();
        while let Some(character) = self.ctm.next_char() {
            if self.ctm.check_char(&LB) || self.ctm.check_char(&IMPORT_CLOSE) {
                break;
            }
            path.push(character);
        }
        if self.ctm.check_char(&LB) || path.is_empty() {
            return Err(self.ctm.rewind_with_error(start_index));
        }
        if self.ctm.check_char(&IMPORT_CLOSE) {
            self.ctm.seek_one()?;
        }
        // parser success

        if self.section_nesting > 0 {
            self.section_return = Some(0);
            return Err(self.ctm.rewind_with_error(start_index));
        }
        let metadata = self
            .parse_inline_metadata()
            .ok()
            .map(|m| m.into())
            .unwrap_or(HashMap::new());

        self.ctm.seek_whitespace();

        match self.import(path.clone(), &metadata) {
            ImportType::Document(Ok(anchor)) => Ok(Some(Import { path, anchor })),
            ImportType::Stylesheet(_) => Ok(None),
            ImportType::Bibliography(_) => Ok(None),
            ImportType::Manifest(_) => Ok(None),
            _ => Err(self.ctm.err()),
        }
    }
}
