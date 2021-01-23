use super::ParseResult;
use crate::elements::tokens::*;
use crate::elements::Inline::LineBreak;
use crate::elements::{BibEntry, Metadata};
use crate::elements::{Cell, Centered, Header, Line, ListItem, Row, Ruler, TextLine};
use crate::parser::inline::ParseInline;
use crate::Parser;
use bibliographix::bibliography::bibliography_entry::BibliographyEntry;
use bibliographix::bibliography::keys::{K_KEY, K_TYPE, K_URL, T_WEBSITE};
use bibliographix::bibliography::FromHashMap;
use std::collections::HashMap;

pub(crate) trait ParseLine {
    fn parse_line(&mut self) -> ParseResult<Line>;
    fn parse_header(&mut self) -> ParseResult<Header>;
    fn parse_list_item(&mut self) -> ParseResult<ListItem>;
    fn parse_row(&mut self) -> ParseResult<Row>;
    fn parse_centered(&mut self) -> ParseResult<Centered>;
    fn parse_ruler(&mut self) -> ParseResult<Ruler>;
    fn parse_paragraph_break(&mut self) -> ParseResult<TextLine>;
    fn parse_text_line(&mut self) -> ParseResult<TextLine>;
    fn parse_bib_entry(&mut self) -> ParseResult<BibEntry>;
}

impl ParseLine for Parser {
    /// parses inline definitions
    fn parse_line(&mut self) -> ParseResult<Line> {
        if self.ctm.check_eof() {
            log::trace!("EOF");
            Err(self.ctm.err().into())
        } else {
            if let Ok(ruler) = self.parse_ruler() {
                log::trace!("Line::Ruler");
                Ok(Line::Ruler(ruler))
            } else if let Ok(centered) = self.parse_centered() {
                log::trace!("Line::Centered");
                Ok(Line::Centered(centered))
            } else if let Ok(bib) = self.parse_bib_entry() {
                log::trace!("Line::BibEntry");
                Ok(Line::BibEntry(bib))
            } else if let Ok(text) = self.parse_paragraph_break() {
                log::trace!("Line::LineBreak");
                Ok(Line::Text(text))
            } else if let Ok(text) = self.parse_text_line() {
                log::trace!("Line::Text");
                Ok(Line::Text(text))
            } else {
                Err(self.ctm.err().into())
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
        if let Some(last) = self.section_anchors.last() {
            anchor = format!("{}-{}", last, anchor);
        }
        anchor.retain(|c| !c.is_whitespace());
        log::trace!("Line::Header");
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
        if ordered {
            while self.ctm.get_current().is_numeric() {
                self.ctm.seek_one()?;
            }
            self.ctm.assert_char(&DOT, Some(start_index))?;
            self.ctm.seek_one()?;
        }

        if !self.ctm.check_any(&INLINE_WHITESPACE) {
            return Err(self.ctm.rewind_with_error(start_index).into());
        }
        self.ctm.seek_any(&INLINE_WHITESPACE)?;
        if self.ctm.check_char(&MINUS) {
            return Err(self.ctm.rewind_with_error(start_index).into());
        }

        let item = ListItem::new(self.parse_line()?, level as u16, ordered);
        log::trace!("Line::ListItem");

        Ok(item)
    }

    /// parses a table row/head
    fn parse_row(&mut self) -> ParseResult<Row> {
        let start_index = self.ctm.get_index();
        self.ctm.seek_any(&INLINE_WHITESPACE)?;
        self.ctm.assert_char(&PIPE, Some(start_index))?;
        self.ctm.seek_one()?;
        if self.ctm.check_char(&PIPE) {
            return Err(self.ctm.rewind_with_error(start_index).into());
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
        if !self.ctm.check_eof() {
            let _ = self.ctm.seek_one();
        }

        if row.cells.len() > 0 {
            log::trace!("Line::TableRow");
            Ok(row)
        } else {
            return Err(self.ctm.rewind_with_error(start_index).into());
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
        let start_index = self.ctm.get_index();

        while let Ok(subtext) = self.parse_inline() {
            text.add_subtext(subtext);
            if self.ctm.check_eof() || self.ctm.check_any(&self.inline_break_at) {
                break;
            }
        }

        // add a linebreak when encountering \n\n
        if self.ctm.check_char(&LB) {
            self.ctm.try_seek();

            if self.ctm.check_char(&LB) {
                text.add_subtext(LineBreak);

                self.ctm.try_seek();
            }
        }

        if text.subtext.len() > 0 {
            Ok(text)
        } else {
            Err(self.ctm.rewind_with_error(start_index).into())
        }
    }

    /// Parses a paragraph break
    fn parse_paragraph_break(&mut self) -> ParseResult<TextLine> {
        let start_index = self.ctm.get_index();
        self.ctm.assert_char(&LB, Some(start_index))?;
        self.ctm.seek_one()?;

        let mut line = TextLine::new();
        line.subtext.push(LineBreak);

        Ok(line)
    }

    fn parse_bib_entry(&mut self) -> ParseResult<BibEntry> {
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
            let mut string_map = meta.get_string_map();
            string_map.insert(K_KEY.to_string(), key.clone());

            match BibliographyEntry::from_hash_map(&string_map) {
                Ok(entry) => *entry,
                Err(msg) => {
                    log::error!(
                        "Failed to parse bib entry with key '{}': {}\n\t--> {}\n",
                        key,
                        msg,
                        self.get_position_string()
                    );
                    return Err(self.ctm.rewind_with_error(start_index).into());
                }
            }
        } else {
            let url = self
                .ctm
                .get_string_until_any_or_rewind(&[LB], &[], start_index)?;
            let mut map = HashMap::new();
            map.insert(K_TYPE.to_string(), T_WEBSITE.to_string());
            map.insert(K_URL.to_string(), url);
            map.insert(K_KEY.to_string(), key.clone());

            match BibliographyEntry::from_hash_map(&map) {
                Ok(entry) => *entry,
                Err(msg) => {
                    log::error!(
                        "Failed to parse bib entry with key '{}': {}\n\t--> {}\n",
                        key,
                        msg,
                        self.get_position_string()
                    );
                    return Err(self.ctm.rewind_with_error(start_index).into());
                }
            }
        };
        self.ctm.seek_whitespace();

        self.options
            .document
            .bibliography
            .entry_dictionary()
            .lock()
            .insert(entry);

        Ok(BibEntry {
            entry: self
                .options
                .document
                .bibliography
                .entry_dictionary()
                .lock()
                .get(&key)
                .unwrap(),
            key,
        })
    }
}
