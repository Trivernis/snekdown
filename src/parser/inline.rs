use super::{ParseError, ParseResult};
use crate::elements::tokens::*;
use crate::elements::*;
use crate::parser::block::ParseBlock;
use crate::references::bibliography::BibReference;
use crate::references::configuration::keys::BIB_REF_DISPLAY;
use crate::references::templates::{GetTemplateVariables, Template, TemplateVariable};
use crate::Parser;
use std::collections::HashMap;
use std::sync::{Arc, RwLock};

pub(crate) trait ParseInline {
    fn parse_surrounded(&mut self, surrounding: &char) -> ParseResult<Inline>;
    fn parse_inline(&mut self) -> ParseResult<Inline>;
    fn parse_image(&mut self) -> ParseResult<Image>;
    fn parse_url(&mut self, short_syntax: bool) -> ParseResult<Url>;
    fn parse_checkbox(&mut self) -> ParseResult<Checkbox>;
    fn parse_bold(&mut self) -> ParseResult<BoldText>;
    fn parse_italic(&mut self) -> ParseResult<ItalicText>;
    fn parse_striked(&mut self) -> ParseResult<StrikedText>;
    fn parse_monospace(&mut self) -> ParseResult<MonospaceText>;
    fn parse_underlined(&mut self) -> ParseResult<UnderlinedText>;
    fn parse_superscript(&mut self) -> ParseResult<SuperscriptText>;
    fn parse_emoji(&mut self) -> ParseResult<Emoji>;
    fn parse_colored(&mut self) -> ParseResult<Colored>;
    fn parse_bibref(&mut self) -> ParseResult<Arc<RwLock<BibReference>>>;
    fn parse_template_variable(&mut self) -> ParseResult<Arc<RwLock<TemplateVariable>>>;
    fn parse_plain(&mut self) -> ParseResult<PlainText>;
    fn parse_inline_metadata(&mut self) -> ParseResult<InlineMetadata>;
    fn parse_metadata_pair(&mut self) -> ParseResult<(String, MetadataValue)>;
    fn parse_placeholder(&mut self) -> ParseResult<Arc<RwLock<Placeholder>>>;
    fn parse_template(&mut self) -> ParseResult<Template>;
}

impl ParseInline for Parser {
    /// parses Inline surrounded by characters
    fn parse_surrounded(&mut self, surrounding: &char) -> ParseResult<Inline> {
        let start_index = self.ctm.get_index();
        self.ctm.assert_char(surrounding, Some(start_index))?;
        self.ctm.seek_one()?;
        let inline = self.parse_inline()?;
        self.ctm.assert_char(surrounding, Some(start_index))?;
        if !self.ctm.check_eof() {
            self.ctm.seek_one()?;
        }

        Ok(inline)
    }

    /// parses Inline, the formatting parts of a line (Text)
    fn parse_inline(&mut self) -> ParseResult<Inline> {
        if self.parse_variables {
            if let Ok(var) = self.parse_template_variable() {
                return Ok(Inline::TemplateVar(var));
            }
        }
        if self.ctm.check_char(&PIPE) || self.ctm.check_char(&LB) {
            Err(self.ctm.err())
        } else if self.ctm.check_eof() {
            Err(self.ctm.err())
        } else if let Ok(image) = self.parse_image() {
            Ok(Inline::Image(image))
        } else if let Ok(url) = self.parse_url(false) {
            Ok(Inline::Url(url))
        } else if let Ok(pholder) = self.parse_placeholder() {
            Ok(Inline::Placeholder(pholder))
        } else if let Ok(bold) = self.parse_bold() {
            Ok(Inline::Bold(bold))
        } else if let Ok(italic) = self.parse_italic() {
            Ok(Inline::Italic(italic))
        } else if let Ok(under) = self.parse_underlined() {
            Ok(Inline::Underlined(under))
        } else if let Ok(mono) = self.parse_monospace() {
            Ok(Inline::Monospace(mono))
        } else if let Ok(striked) = self.parse_striked() {
            Ok(Inline::Striked(striked))
        } else if let Ok(superscript) = self.parse_superscript() {
            Ok(Inline::Superscript(superscript))
        } else if let Ok(checkbox) = self.parse_checkbox() {
            Ok(Inline::Checkbox(checkbox))
        } else if let Ok(emoji) = self.parse_emoji() {
            Ok(Inline::Emoji(emoji))
        } else if let Ok(colored) = self.parse_colored() {
            Ok(Inline::Colored(colored))
        } else if let Ok(bibref) = self.parse_bibref() {
            Ok(Inline::BibReference(bibref))
        } else {
            Ok(Inline::Plain(self.parse_plain()?))
        }
    }

    /// parses an image url
    fn parse_image(&mut self) -> ParseResult<Image> {
        let start_index = self.ctm.get_index();
        self.ctm.seek_any(&INLINE_WHITESPACE)?;
        self.ctm.assert_char(&IMG_START, Some(start_index))?;
        self.ctm.seek_one()?;

        if let Ok(url) = self.parse_url(true) {
            let metadata = if let Ok(meta) = self.parse_inline_metadata() {
                Some(meta)
            } else {
                None
            };
            Ok(Image { url, metadata })
        } else {
            Err(self.ctm.rewind_with_error(start_index))
        }
    }

    // parses an url
    fn parse_url(&mut self, short_syntax: bool) -> ParseResult<Url> {
        let start_index = self.ctm.get_index();
        self.ctm.seek_any(&INLINE_WHITESPACE)?;

        let mut description = Vec::new();

        if self.ctm.check_char(&DESC_OPEN) {
            self.ctm.seek_one()?;
            self.inline_break_at.push(DESC_CLOSE);

            while let Ok(inline) = self.parse_inline() {
                description.push(inline);
                if self.ctm.check_char(&DESC_CLOSE) {
                    break;
                }
            }
            self.inline_break_at.pop();
        } else if !short_syntax {
            return Err(self.ctm.rewind_with_error(start_index));
        }
        self.ctm.seek_one()?;
        self.ctm.assert_char(&URL_OPEN, Some(start_index))?;
        self.ctm.seek_one()?;
        self.ctm.seek_any(&INLINE_WHITESPACE)?;

        let url = self
            .ctm
            .get_string_until_any_or_rewind(&[URL_CLOSE], &[LB], start_index)?;

        self.ctm.seek_one()?;

        if description.len() > 0 {
            Ok(Url::new(Some(description), url))
        } else {
            Ok(Url::new(None, url))
        }
    }

    /// parses a markdown checkbox
    fn parse_checkbox(&mut self) -> ParseResult<Checkbox> {
        let start_index = self.ctm.get_index();
        self.ctm.assert_char(&CHECK_OPEN, Some(start_index))?;
        self.ctm.seek_one()?;
        let checked = if self.ctm.check_char(&CHECK_CHECKED) {
            true
        } else if self.ctm.check_char(&SPACE) {
            false
        } else {
            return Err(self.ctm.rewind_with_error(start_index));
        };
        self.ctm.seek_one()?;
        self.ctm.assert_char(&CHECK_CLOSE, Some(start_index))?;
        self.ctm.seek_one()?;

        Ok(Checkbox { value: checked })
    }

    /// parses bold text with must start with two asterisks
    fn parse_bold(&mut self) -> ParseResult<BoldText> {
        let start_index = self.ctm.get_index();
        self.ctm.assert_sequence(&BOLD, Some(start_index))?;
        self.ctm.seek_one()?;
        let inline = self.parse_inline()?;
        self.ctm.assert_sequence(&BOLD, Some(start_index))?;
        self.ctm.seek_one()?;

        Ok(BoldText {
            value: Box::new(inline),
        })
    }

    fn parse_italic(&mut self) -> ParseResult<ItalicText> {
        Ok(ItalicText {
            value: Box::new(self.parse_surrounded(&ITALIC)?),
        })
    }

    fn parse_striked(&mut self) -> ParseResult<StrikedText> {
        Ok(StrikedText {
            value: Box::new(self.parse_surrounded(&STRIKED)?),
        })
    }

    /// parses monospace text (inline-code) that isn't allowed to contain special characters
    fn parse_monospace(&mut self) -> ParseResult<MonospaceText> {
        let start_index = self.ctm.get_index();
        self.ctm.assert_char(&BACKTICK, Some(start_index))?;
        self.ctm.seek_one()?;
        let content = self
            .ctm
            .get_string_until_any_or_rewind(&[BACKTICK, LB], &[], start_index)?;
        self.ctm.assert_char(&BACKTICK, Some(start_index))?;
        self.ctm.seek_one()?;

        Ok(MonospaceText { value: content })
    }

    fn parse_underlined(&mut self) -> ParseResult<UnderlinedText> {
        Ok(UnderlinedText {
            value: Box::new(self.parse_surrounded(&UNDERLINED)?),
        })
    }

    fn parse_superscript(&mut self) -> ParseResult<SuperscriptText> {
        Ok(SuperscriptText {
            value: Box::new(self.parse_surrounded(&SUPER)?),
        })
    }

    fn parse_emoji(&mut self) -> ParseResult<Emoji> {
        let start_index = self.ctm.get_index();
        self.ctm.assert_char(&EMOJI, Some(start_index))?;
        self.ctm.seek_one()?;
        let name = self
            .ctm
            .get_string_until_any_or_rewind(&[EMOJI], &[SPACE, LB], start_index)?;
        self.ctm.seek_one()?;
        if let Some(emoji) = gh_emoji::get(name.as_str()) {
            let emoji_char = *emoji.chars().collect::<Vec<char>>().first().unwrap();
            Ok(Emoji {
                value: emoji_char,
                name,
            })
        } else {
            Err(self.ctm.rewind_with_error(start_index))
        }
    }

    /// parses colored text
    fn parse_colored(&mut self) -> ParseResult<Colored> {
        let start_index = self.ctm.get_index();
        self.ctm
            .assert_sequence(&SQ_COLOR_START, Some(start_index))?;
        self.ctm.seek_one()?;
        let color = self.ctm.get_string_until_any_or_rewind(
            &[COLOR_CLOSE],
            &[SPACE, LB, SEMICOLON],
            start_index,
        )?;
        self.ctm.seek_one()?;
        if color.is_empty() {
            return Err(self.ctm.err());
        }
        Ok(Colored {
            value: Box::new(self.parse_inline()?),
            color,
        })
    }

    fn parse_bibref(&mut self) -> ParseResult<Arc<RwLock<BibReference>>> {
        let start_index = self.ctm.get_index();
        self.ctm
            .assert_sequence(&SQ_BIBREF_START, Some(start_index))?;
        self.ctm.seek_one()?;
        let key =
            self.ctm
                .get_string_until_any_or_rewind(&[BIBREF_CLOSE], &[SPACE, LB], start_index)?;
        self.ctm.seek_one()?;
        let ref_entry = Arc::new(RwLock::new(BibReference::new(
            key,
            self.document.config.get_ref_entry(BIB_REF_DISPLAY),
        )));
        self.document
            .bibliography
            .add_ref_entry(Arc::clone(&ref_entry));

        Ok(ref_entry)
    }

    /// parses a template variable {prefix{name}suffix}
    fn parse_template_variable(&mut self) -> ParseResult<Arc<RwLock<TemplateVariable>>> {
        let start_index = self.ctm.get_index();
        self.ctm.assert_char(&TEMP_VAR_OPEN, Some(start_index))?;
        self.ctm.seek_one()?;
        let prefix =
            self.ctm
                .get_string_until_any_or_rewind(&[TEMP_VAR_OPEN], &[LB], start_index)?;
        self.ctm.seek_one()?;
        let name =
            self.ctm
                .get_string_until_any_or_rewind(&[TEMP_VAR_CLOSE], &[LB], start_index)?;
        self.ctm.seek_one()?;
        let suffix =
            self.ctm
                .get_string_until_any_or_rewind(&[TEMP_VAR_CLOSE], &[LB], start_index)?;
        self.ctm.seek_one()?;
        Ok(Arc::new(RwLock::new(TemplateVariable {
            value: None,
            name,
            prefix,
            suffix,
        })))
    }

    /// parses plain text as a string until it encounters an unescaped special inline char
    fn parse_plain(&mut self) -> ParseResult<PlainText> {
        if self.ctm.check_char(&LB) {
            return Err(self.ctm.err());
        }
        let mut characters = String::new();
        characters.push(self.ctm.get_current());
        while let Some(ch) = self.ctm.next_char() {
            if self.ctm.check_any(&INLINE_SPECIAL_CHARS)
                || self.ctm.check_any(&self.inline_break_at)
                || (self.parse_variables && self.ctm.check_char(&TEMP_VAR_OPEN))
            {
                break;
            }
            characters.push(ch)
        }

        if characters.len() > 0 {
            Ok(PlainText { value: characters })
        } else {
            Err(self.ctm.err())
        }
    }

    /// Parses metadata
    fn parse_inline_metadata(&mut self) -> ParseResult<InlineMetadata> {
        let start_index = self.ctm.get_index();
        self.ctm.assert_char(&META_OPEN, Some(start_index))?;
        self.ctm.seek_one()?;

        let mut values = HashMap::new();
        while let Ok((key, value)) = self.parse_metadata_pair() {
            values.insert(key, value);
            if self.ctm.check_char(&META_CLOSE) || self.ctm.check_char(&LB) {
                // abort the parser of the inner content when encountering a closing tag or linebreak
                break;
            }
        }
        if self.ctm.check_char(&META_CLOSE) {
            self.ctm.seek_one()?;
        }
        if values.len() == 0 {
            // if there was a linebreak (the metadata wasn't closed) or there is no inner data
            // return an error
            return Err(self.ctm.rewind_with_error(start_index));
        }

        Ok(InlineMetadata { data: values })
    }

    /// parses a key-value metadata pair
    fn parse_metadata_pair(&mut self) -> Result<(String, MetadataValue), ParseError> {
        self.ctm.seek_any(&INLINE_WHITESPACE)?;
        let name = self
            .ctm
            .get_string_until_any(&[META_CLOSE, EQ, SPACE, LB], &[])?;

        self.ctm.seek_any(&INLINE_WHITESPACE)?;
        let mut value = MetadataValue::Bool(true);
        if self.ctm.check_char(&EQ) {
            self.ctm.seek_one()?;
            self.ctm.seek_any(&INLINE_WHITESPACE)?;
            if let Ok(ph) = self.parse_placeholder() {
                value = MetadataValue::Placeholder(ph);
            } else if let Ok(template) = self.parse_template() {
                value = MetadataValue::Template(template)
            } else {
                let quoted_string = self.ctm.check_any(&QUOTES);

                let parse_until = if quoted_string {
                    let quote_start = self.ctm.get_current();
                    self.ctm.seek_one()?;
                    vec![quote_start, META_CLOSE, LB]
                } else {
                    vec![META_CLOSE, LB, SPACE]
                };

                let raw_value = self.ctm.get_string_until_any(&parse_until, &[])?;

                if self.ctm.check_any(&QUOTES) {
                    self.ctm.seek_one()?;
                }
                self.ctm.seek_any(&INLINE_WHITESPACE)?;

                if self.ctm.check_char(&COMMA) {
                    self.ctm.seek_one()?;
                }

                value = if quoted_string {
                    MetadataValue::String(raw_value)
                } else if raw_value.to_lowercase().as_str() == "true" {
                    MetadataValue::Bool(true)
                } else if raw_value.to_lowercase().as_str() == "false" {
                    MetadataValue::Bool(false)
                } else if let Ok(num) = raw_value.parse::<i64>() {
                    MetadataValue::Integer(num)
                } else if let Ok(num) = raw_value.parse::<f64>() {
                    MetadataValue::Float(num)
                } else {
                    MetadataValue::String(raw_value)
                }
            }
        }

        Ok((name, value))
    }

    /// parses a placeholder element
    fn parse_placeholder(&mut self) -> ParseResult<Arc<RwLock<Placeholder>>> {
        let start_index = self.ctm.get_index();
        self.ctm.assert_sequence(&SQ_PHOLDER_START, None)?;
        self.ctm.seek_one()?;

        let name = if let Ok(name_str) = self
            .ctm
            .get_string_until_sequence(&[&SQ_PHOLDER_STOP], &[&[LB]])
        {
            name_str
        } else {
            return Err(self.ctm.rewind_with_error(start_index));
        };
        self.ctm.seek_one()?;

        let metadata = if let Ok(meta) = self.parse_inline_metadata() {
            Some(meta)
        } else {
            None
        };

        let placeholder = Arc::new(RwLock::new(Placeholder::new(name, metadata)));
        self.document.add_placeholder(Arc::clone(&placeholder));

        Ok(placeholder)
    }

    /// parses a template
    fn parse_template(&mut self) -> ParseResult<Template> {
        let start_index = self.ctm.get_index();

        self.ctm.assert_char(&TEMPLATE, None)?;
        self.ctm.seek_one()?;

        if self.ctm.check_char(&TEMPLATE) {
            return Err(self.ctm.rewind_with_error(start_index));
        }

        let mut elements = Vec::new();
        self.block_break_at.push(TEMPLATE);
        self.inline_break_at.push(TEMPLATE);
        self.parse_variables = true;

        while let Ok(e) = self.parse_block() {
            elements.push(Element::Block(Box::new(e)));
            if self.ctm.check_char(&TEMPLATE) {
                break;
            }
        }
        self.parse_variables = false;
        self.block_break_at.clear();
        self.inline_break_at.clear();
        self.ctm.assert_char(&TEMPLATE, Some(start_index))?;
        self.ctm.seek_one()?;

        let vars: HashMap<String, Arc<RwLock<TemplateVariable>>> = elements
            .iter()
            .map(|e| e.get_template_variables())
            .flatten()
            .map(|e: Arc<RwLock<TemplateVariable>>| {
                let name;
                {
                    name = e.read().unwrap().name.clone();
                };

                (name, e)
            })
            .collect();

        Ok(Template {
            text: elements,
            variables: vars,
        })
    }
}
