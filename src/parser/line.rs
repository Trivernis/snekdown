use super::ParseResult;
use crate::elements::tokens::*;
use crate::elements::{Cell, Centered, Header, Inline, Line, ListItem, Row, Ruler, TextLine};
use crate::parser::inline::ParseInline;
use crate::references::bibliography::BibEntry;
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
        if self.ctm.check_eof() {
            Err(self.ctm.err())
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
                Err(self.ctm.err())
            }
        }
    }

    /// parses the header of a section
    fn parse_header(&mut self) -> ParseResult<Header> {
        let start_index = self.ctm.get_index();
        let line = self.parse_line()?;
        let mut anchor = String::new();
        self.ctm.get_text()[start_index..self.ctm.get_index()]
            .iter()
            .for_each(|e| anchor.push(*e));
        anchor.retain(|c| !c.is_whitespace());
        Ok(Header::new(line, anchor))
    }

    /// parses a single list item defined with -
    fn parse_list_item(&mut self) -> ParseResult<ListItem> {
        let start_index = self.ctm.get_index();
        self.ctm.seek_any(&INLINE_WHITESPACE)?;
        let level = self.ctm.get_index() - start_index;
        self.ctm
            .assert_any(&LIST_SPECIAL_CHARS, Some(start_index))?;
        let ordered = self.ctm.get_current().is_numeric();
        self.ctm.seek_one()?;

        if self.ctm.check_char(&DOT) {
            self.ctm.seek_one()?;
        }
        if !self.ctm.check_any(&INLINE_WHITESPACE) {
            return Err(self.ctm.rewind_with_error(start_index));
        }
        self.ctm.seek_any(&INLINE_WHITESPACE)?;
        if self.ctm.check_char(&MINUS) {
            return Err(self.ctm.rewind_with_error(start_index));
        }

        let item = ListItem::new(self.parse_line()?, level as u16, ordered);

        Ok(item)
    }

    /// parses a table row/head
    fn parse_row(&mut self) -> ParseResult<Row> {
        let start_index = self.ctm.get_index();
        self.ctm.seek_any(&INLINE_WHITESPACE)?;
        self.ctm.assert_char(&PIPE, Some(start_index))?;
        self.ctm.seek_one()?;
        if self.ctm.check_char(&PIPE) {
            return Err(self.ctm.rewind_with_error(start_index));
        }
        self.inline_break_at.push(PIPE);

        self.ctm.seek_any(&INLINE_WHITESPACE)?;
        let mut row = Row::new();
        loop {
            let mut element = TextLine::new();
            while let Ok(inline) = self.parse_inline() {
                element.subtext.push(inline);
                if self.ctm.check_char(&LB) || self.ctm.check_char(&PIPE) || self.ctm.check_eof() {
                    break;
                }
            }
            row.add_cell(Cell {
                text: Line::Text(element),
            });
            if self.ctm.check_char(&PIPE) {
                self.ctm.seek_one()?;
            }
            if self.ctm.check_char(&LB) || self.ctm.check_eof() {
                break;
            }
            self.ctm.seek_any(&INLINE_WHITESPACE)?;
        }
        self.inline_break_at.clear();

        if self.ctm.check_char(&PIPE) {
            self.ctm.seek_one()?;
        }
        self.ctm.seek_one()?;

        if row.cells.len() > 0 {
            Ok(row)
        } else {
            return Err(self.ctm.rewind_with_error(start_index));
        }
    }

    /// parses centered text
    fn parse_centered(&mut self) -> ParseResult<Centered> {
        let start_index = self.ctm.get_index();
        self.ctm
            .assert_sequence(&SQ_CENTERED_START, Some(start_index))?;
        self.ctm.seek_one()?;
        let line = self.parse_text_line()?;

        Ok(Centered { line })
    }

    /// parses a ruler
    fn parse_ruler(&mut self) -> ParseResult<Ruler> {
        let start_index = self.ctm.get_index();
        self.ctm.seek_any(&INLINE_WHITESPACE)?;
        self.ctm.assert_sequence(&SQ_RULER, Some(start_index))?;
        while !self.ctm.check_char(&LB) {
            self.ctm.seek_one()?;
        }
        Ok(Ruler {})
    }

    /// Parses a line of text
    fn parse_text_line(&mut self) -> ParseResult<TextLine> {
        let mut text = TextLine::new();
        while let Ok(subtext) = self.parse_inline() {
            text.add_subtext(subtext);
            if self.ctm.check_eof() || self.ctm.check_any(&self.inline_break_at) {
                break;
            }
        }

        if self.ctm.check_char(&LB) {
            if let Ok(_) = self.ctm.seek_one() {
                if self.ctm.check_char(&LB) {
                    text.add_subtext(Inline::LineBreak)
                }
            }
        }

        if text.subtext.len() > 0 {
            Ok(text)
        } else {
            Err(self.ctm.err())
        }
    }

    fn parse_bib_entry(&mut self) -> ParseResult<Arc<RwLock<BibEntry>>> {
        let start_index = self.ctm.get_index();
        self.ctm.seek_any(&INLINE_WHITESPACE)?;
        self.ctm.assert_char(&BIB_KEY_OPEN, Some(start_index))?;
        self.ctm.seek_one()?;
        let key =
            self.ctm
                .get_string_until_any_or_rewind(&[BIB_KEY_CLOSE], &[LB, SPACE], start_index)?;
        self.ctm.seek_one()?;
        self.ctm.assert_char(&BIB_DATA_START, Some(start_index))?;
        self.ctm.seek_one()?;
        self.ctm.seek_any(&INLINE_WHITESPACE)?;

        let entry = if let Ok(meta) = self.parse_inline_metadata() {
            BibEntry::from_metadata(key, Box::new(meta), &self.document.config)
        } else {
            let url = self
                .ctm
                .get_string_until_any_or_rewind(&[LB], &[], start_index)?;
            BibEntry::from_url(key, url, &self.document.config)
        };
        let entry_ref = Arc::new(RwLock::new(entry));
        self.document
            .bibliography
            .add_bib_entry(Arc::clone(&entry_ref));

        Ok(entry_ref)
    }
}
