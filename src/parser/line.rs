use crate::elements::tokens::*;
use crate::elements::{Cell, Centered, Header, Line, ListItem, Row, Ruler, TextLine};
use crate::parser::charstate::CharStateMachine;
use crate::parser::inline::ParseInline;
use crate::references::bibliography::BibEntry;
use crate::utils::parsing::{ParseError, ParseResult};
use crate::Parser;
use std::sync::{Arc, RwLock};

pub(crate) trait ParseLine {
    fn parse_line(&mut self) -> ParseResult<Line>;
    fn parse_header(&mut self) -> ParseResult<Header>;
    fn parse_list_item(&mut self) -> ParseResult<ListItem>;
    fn parse_row(&mut self) -> ParseResult<Row>;
    fn parse_centered(&mut self) -> ParseResult<Centered>;
    fn parse_ruler(&mut self) -> ParseResult<Ruler>;
    fn parse_text_line(&mut self) -> ParseResult<TextLine>;
    fn parse_bib_entry(&mut self) -> ParseResult<Arc<RwLock<BibEntry>>>;
}

impl ParseLine for Parser {
    /// parses inline definitions
    fn parse_line(&mut self) -> ParseResult<Line> {
        if self.index > self.text.len() {
            Err(ParseError::new(self.index))
        } else {
            if let Ok(ruler) = self.parse_ruler() {
                Ok(Line::Ruler(ruler))
            } else if let Ok(centered) = self.parse_centered() {
                Ok(Line::Centered(centered))
            } else if let Ok(bib) = self.parse_bib_entry() {
                Ok(Line::BibEntry(bib))
            } else if let Ok(text) = self.parse_text_line() {
                Ok(Line::Text(text))
            } else {
                Err(ParseError::new(self.index))
            }
        }
    }

    /// parses the header of a section
    fn parse_header(&mut self) -> ParseResult<Header> {
        let start_index = self.index;
        let line = self.parse_line()?;
        let mut anchor = String::new();
        self.text[start_index..self.index]
            .iter()
            .for_each(|e| anchor.push(*e));
        anchor.retain(|c| !c.is_whitespace());
        Ok(Header::new(line, anchor))
    }

    /// parses a single list item defined with -
    fn parse_list_item(&mut self) -> ParseResult<ListItem> {
        let start_index = self.index;
        self.seek_inline_whitespace();
        let level = self.index - start_index;
        self.assert_special_group(&LIST_SPECIAL_CHARS, start_index)?;
        let ordered = self.current_char.is_numeric();
        self.skip_char();
        if self.check_special(&DOT) {
            self.skip_char();
        }
        if !self.check_seek_inline_whitespace() {
            return Err(self.revert_with_error(start_index));
        }
        self.seek_inline_whitespace();
        if self.check_special(&MINUS) {
            return Err(self.revert_with_error(start_index));
        }

        let item = ListItem::new(self.parse_line()?, level as u16, ordered);

        Ok(item)
    }

    /// parses a table row/head
    fn parse_row(&mut self) -> ParseResult<Row> {
        let start_index = self.index;
        self.seek_inline_whitespace();
        self.assert_special(&PIPE, start_index)?;
        self.skip_char();
        if self.check_special(&PIPE) {
            return Err(self.revert_with_error(start_index));
        }
        self.inline_break_at.push(PIPE);

        self.seek_inline_whitespace();
        let mut row = Row::new();
        loop {
            let mut element = TextLine::new();
            while let Ok(inline) = self.parse_inline() {
                element.subtext.push(inline);
                if self.check_linebreak() || self.check_special(&PIPE) || self.check_eof() {
                    break;
                }
            }
            row.add_cell(Cell {
                text: Line::Text(element),
            });
            if self.check_special(&PIPE) {
                self.skip_char();
            }
            if self.check_linebreak() || self.check_eof() {
                break;
            }
            self.seek_inline_whitespace();
        }
        self.inline_break_at.clear();
        if self.check_special(&PIPE) {
            self.skip_char();
            self.skip_char();
        } else {
            self.skip_char();
        }

        if row.cells.len() > 0 {
            Ok(row)
        } else {
            return Err(self.revert_with_error(start_index));
        }
    }

    fn parse_bib_entry(&mut self) -> ParseResult<Arc<RwLock<BibEntry>>> {
        let start_index = self.index;
        self.seek_inline_whitespace();
        self.assert_special(&BIB_KEY_OPEN, start_index)?;
        self.skip_char();
        let key = self.get_string_until_or_revert(&[BIB_KEY_CLOSE], &[LB, SPACE], start_index)?;
        self.skip_char();
        self.assert_special(&BIB_DATA_START, start_index)?;
        self.skip_char();
        self.seek_inline_whitespace();
        let entry = if let Ok(meta) = self.parse_inline_metadata() {
            BibEntry::from_metadata(key, Box::new(meta), &self.document.config)
        } else {
            let url = self.get_string_until_or_revert(&[LB], &[], start_index)?;
            BibEntry::from_url(key, url, &self.document.config)
        };
        let entry_ref = Arc::new(RwLock::new(entry));
        self.document
            .bibliography
            .add_bib_entry(Arc::clone(&entry_ref));

        Ok(entry_ref)
    }

    /// parses centered text
    fn parse_centered(&mut self) -> ParseResult<Centered> {
        let start_index = self.index;
        self.assert_special_sequence(&SQ_CENTERED_START, start_index)?;
        self.skip_char();
        let line = self.parse_text_line()?;

        Ok(Centered { line })
    }

    /// parses a ruler
    fn parse_ruler(&mut self) -> ParseResult<Ruler> {
        let start_index = self.index;
        self.seek_inline_whitespace();
        self.assert_special_sequence(&SQ_RULER, start_index)?;
        self.seek_until_linebreak();
        Ok(Ruler {})
    }

    /// Parses a line of text
    fn parse_text_line(&mut self) -> Result<TextLine, ParseError> {
        let mut text = TextLine::new();
        while let Ok(subtext) = self.parse_inline() {
            text.add_subtext(subtext);
            if self.check_eof() || self.check_special_group(&self.inline_break_at) {
                break;
            }
        }

        if self.check_linebreak() {
            self.skip_char();
        }

        if text.subtext.len() > 0 || !self.check_eof() {
            Ok(text)
        } else {
            Err(ParseError::eof(self.index))
        }
    }
}
