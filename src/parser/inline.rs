use super::{ParseError, ParseResult};
use crate::elements::tokens::*;
use crate::elements::BibReference;
use crate::elements::*;
use crate::parser::block::ParseBlock;
use crate::references::configuration::keys::{BIB_REF_DISPLAY, SMART_ARROWS};
use crate::references::glossary::GlossaryDisplay;
use crate::references::glossary::GlossaryReference;
use crate::references::templates::{GetTemplateVariables, Template, TemplateVariable};
use crate::Parser;
use bibliographix::references::bib_reference::BibRef;
use std::collections::HashMap;
use std::sync::{Arc, Mutex, RwLock};

pub(crate) trait ParseInline {
    fn parse_surrounded(&mut self, surrounding: &char) -> ParseResult<Vec<Inline>>;
    fn parse_inline(&mut self) -> ParseResult<Inline>;
    fn parse_image(&mut self) -> ParseResult<Image>;
    fn parse_url(&mut self, short_syntax: bool) -> ParseResult<Url>;
    fn parse_checkbox(&mut self) -> ParseResult<Checkbox>;
    fn parse_bold(&mut self) -> ParseResult<BoldText>;
    fn parse_italic(&mut self) -> ParseResult<ItalicText>;
    fn parse_striked(&mut self) -> ParseResult<StrikedText>;
    fn parse_math(&mut self) -> ParseResult<Math>;
    fn parse_monospace(&mut self) -> ParseResult<MonospaceText>;
    fn parse_underlined(&mut self) -> ParseResult<UnderlinedText>;
    fn parse_superscript(&mut self) -> ParseResult<SuperscriptText>;
    fn parse_emoji(&mut self) -> ParseResult<Emoji>;
    fn parse_colored(&mut self) -> ParseResult<Colored>;
    fn parse_bibref(&mut self) -> ParseResult<Arc<RwLock<BibReference>>>;
    fn parse_template_variable(&mut self) -> ParseResult<Arc<RwLock<TemplateVariable>>>;
    fn parse_glossary_reference(&mut self) -> ParseResult<Arc<Mutex<GlossaryReference>>>;
    fn parse_plain(&mut self) -> ParseResult<PlainText>;
    fn parse_inline_metadata(&mut self) -> ParseResult<InlineMetadata>;
    fn parse_metadata_pair(&mut self) -> ParseResult<(String, MetadataValue)>;
    fn parse_placeholder(&mut self) -> ParseResult<Arc<RwLock<Placeholder>>>;
    fn parse_template(&mut self) -> ParseResult<Template>;
    fn parse_character_code(&mut self) -> ParseResult<CharacterCode>;
    fn parse_arrow(&mut self) -> ParseResult<Arrow>;
}

impl ParseInline for Parser {
    /// parses Inline surrounded by characters
    fn parse_surrounded(&mut self, surrounding: &char) -> ParseResult<Vec<Inline>> {
        let start_index = self.ctm.get_index();
        self.ctm.assert_char(surrounding, Some(start_index))?;
        self.ctm.seek_one()?;
        let mut inline = vec![self.parse_inline()?];
        while !self.ctm.check_char(surrounding) {
            if let Ok(result) = self.parse_inline() {
                inline.push(result)
            } else {
                return Err(self.ctm.rewind_with_error(start_index));
            }
        }
        if !self.ctm.check_eof() {
            self.ctm.seek_one()?;
        }

        Ok(inline)
    }

    /// parses Inline, the formatting parts of a line (Text)
    fn parse_inline(&mut self) -> ParseResult<Inline> {
        if self.parse_variables {
            if let Ok(var) = self.parse_template_variable() {
                log::trace!("Inline::TemplateVar");
                return Ok(Inline::TemplateVar(var));
            }
        }
        if self.ctm.check_char(&PIPE) || self.ctm.check_char(&LB) {
            Err(self.ctm.err())
        } else if self.ctm.check_eof() {
            log::trace!("EOF");
            Err(self.ctm.err())
        } else if let Ok(image) = self.parse_image() {
            log::trace!("Inline::Image {:?}", image);
            Ok(Inline::Image(image))
        } else if let Ok(url) = self.parse_url(false) {
            log::trace!("Inline::Url {:?}", url);
            Ok(Inline::Url(url))
        } else if let Ok(pholder) = self.parse_placeholder() {
            log::trace!("Inline::Placeholder {:?}", pholder);
            Ok(Inline::Placeholder(pholder))
        } else if let Ok(bold) = self.parse_bold() {
            log::trace!("Inline::Bold");
            Ok(Inline::Bold(bold))
        } else if let Ok(italic) = self.parse_italic() {
            log::trace!("Inline::Italic");
            Ok(Inline::Italic(italic))
        } else if let Ok(under) = self.parse_underlined() {
            log::trace!("Inline::Underlined");
            Ok(Inline::Underlined(under))
        } else if let Ok(mono) = self.parse_monospace() {
            log::trace!("Inline::Monospace {}", mono.value);
            Ok(Inline::Monospace(mono))
        } else if let Ok(striked) = self.parse_striked() {
            log::trace!("Inline::Striked");
            Ok(Inline::Striked(striked))
        } else if let Ok(gloss) = self.parse_glossary_reference() {
            log::trace!("Inline::GlossaryReference {}", gloss.lock().unwrap().short);
            Ok(Inline::GlossaryReference(gloss))
        } else if let Ok(superscript) = self.parse_superscript() {
            log::trace!("Inline::Superscript");
            Ok(Inline::Superscript(superscript))
        } else if let Ok(checkbox) = self.parse_checkbox() {
            log::trace!("Inline::Checkbox {}", checkbox.value);
            Ok(Inline::Checkbox(checkbox))
        } else if let Ok(emoji) = self.parse_emoji() {
            log::trace!("Inline::Emoji {} -> {}", emoji.name, emoji.value);
            Ok(Inline::Emoji(emoji))
        } else if let Ok(colored) = self.parse_colored() {
            log::trace!("Inline::Colored");
            Ok(Inline::Colored(colored))
        } else if let Ok(bibref) = self.parse_bibref() {
            log::trace!("Inline::BibReference {:?}", bibref);
            Ok(Inline::BibReference(bibref))
        } else if let Ok(math) = self.parse_math() {
            log::trace!("Inline::Math");
            Ok(Inline::Math(math))
        } else if let Ok(char_code) = self.parse_character_code() {
            log::trace!("Inline::CharacterCode {}", char_code.code);
            Ok(Inline::CharacterCode(char_code))
        } else if let Ok(arrow) = self.parse_arrow() {
            log::trace!("Inline::Arrow {:?}", arrow);
            Ok(Inline::Arrow(arrow))
        } else {
            let plain = self.parse_plain()?;
            log::trace!("Inline::Plain {}", plain.value);
            Ok(Inline::Plain(plain))
        }
    }

    /// parses an image url
    fn parse_image(&mut self) -> ParseResult<Image> {
        let start_index = self.ctm.get_index();
        self.ctm.assert_char(&IMG_START, Some(start_index))?;
        self.ctm.seek_one()?;

        if let Ok(url) = self.parse_url(true) {
            let metadata = if let Ok(meta) = self.parse_inline_metadata() {
                Some(meta)
            } else {
                None
            };
            let path = url.url.clone();
            Ok(Image {
                url,
                metadata,
                download: self
                    .options
                    .document
                    .downloads
                    .lock()
                    .unwrap()
                    .add_download(path),
            })
        } else {
            Err(self.ctm.rewind_with_error(start_index))
        }
    }

    // parses an url
    fn parse_url(&mut self, short_syntax: bool) -> ParseResult<Url> {
        let start_index = self.ctm.get_index();
        let mut description = Vec::new();

        if self.ctm.check_char(&DESC_OPEN) {
            self.ctm.seek_one()?;
            self.inline_break_at.push(DESC_CLOSE);

            // only parse the description as inline if there is a description
            if !self.ctm.check_char(&DESC_CLOSE) {
                while let Ok(inline) = self.parse_inline() {
                    description.push(inline);
                    if self.ctm.check_char(&DESC_CLOSE) {
                        break;
                    }
                }
            }
            self.inline_break_at.pop();
            self.ctm.seek_one()?;
        } else if !short_syntax {
            return Err(self.ctm.rewind_with_error(start_index));
        }
        self.ctm.assert_char(&URL_OPEN, Some(start_index))?;
        self.ctm.seek_one()?;
        self.ctm.seek_any(&INLINE_WHITESPACE)?;

        let mut url = self
            .ctm
            .get_string_until_any_or_rewind(&[URL_CLOSE], &[LB], start_index)?;

        self.ctm.seek_one()?;
        let url_path = self.transform_path(url.clone());
        if url_path.exists() {
            url = url_path.to_str().unwrap().to_string();
        }

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
        let mut inline = vec![self.parse_inline()?];
        while !self.ctm.check_sequence(&BOLD) {
            if let Ok(result) = self.parse_inline() {
                inline.push(result);
            } else {
                return Err(self.ctm.rewind_with_error(start_index));
            }
        }
        self.ctm.seek_one()?;

        Ok(BoldText { value: inline })
    }

    fn parse_italic(&mut self) -> ParseResult<ItalicText> {
        Ok(ItalicText {
            value: self.parse_surrounded(&ITALIC)?,
        })
    }

    fn parse_striked(&mut self) -> ParseResult<StrikedText> {
        let start_index = self.ctm.get_index();
        self.ctm.assert_sequence(&STRIKED, Some(start_index))?;
        self.ctm.seek_one()?;
        let mut inline = vec![self.parse_inline()?];

        while !self.ctm.check_sequence(&STRIKED) {
            if let Ok(result) = self.parse_inline() {
                inline.push(result);
            } else {
                return Err(self.ctm.rewind_with_error(start_index));
            }
        }
        self.ctm.rewind(self.ctm.get_index() - STRIKED.len());
        if self.ctm.check_any(WHITESPACE) {
            return Err(self.ctm.rewind_with_error(start_index));
        }
        for _ in 0..(STRIKED.len() + 1) {
            self.ctm.seek_one()?;
        }

        Ok(StrikedText { value: inline })
    }

    fn parse_math(&mut self) -> ParseResult<Math> {
        let start_index = self.ctm.get_index();
        self.ctm.assert_sequence(&MATH_INLINE, Some(start_index))?;
        self.ctm.seek_one()?;
        let content = self
            .ctm
            .get_string_until_sequence(&[MATH_INLINE, &[LB]], &[])?;
        self.ctm.seek_one()?;

        Ok(Math {
            expression: asciimath_rs::parse(content),
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
            value: self.parse_surrounded(&UNDERLINED)?,
        })
    }

    fn parse_superscript(&mut self) -> ParseResult<SuperscriptText> {
        Ok(SuperscriptText {
            value: self.parse_surrounded(&SUPER)?,
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
        let bib_ref = BibRef::new(key.clone());
        let ref_entry = Arc::new(RwLock::new(BibReference::new(
            key,
            self.options.document.config.get_ref_entry(BIB_REF_DISPLAY),
            bib_ref.anchor(),
        )));
        self.options
            .document
            .bibliography
            .root_ref_anchor()
            .lock()
            .unwrap()
            .insert(bib_ref);

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

    /// Parses a reference to a glossary entry
    fn parse_glossary_reference(&mut self) -> ParseResult<Arc<Mutex<GlossaryReference>>> {
        let start_index = self.ctm.get_index();
        self.ctm
            .assert_char(&GLOSSARY_REF_START, Some(start_index))?;
        self.ctm.seek_one()?;

        let display = if self.ctm.check_char(&GLOSSARY_REF_START) {
            self.ctm.seek_one()?;
            GlossaryDisplay::Long
        } else {
            GlossaryDisplay::Short
        };
        let mut key =
            self.ctm
                .get_string_until_any_or_rewind(&WHITESPACE, &[TILDE], start_index)?;
        if key.is_empty() {
            return Err(self.ctm.rewind_with_error(start_index));
        }
        while !key.is_empty() && !key.chars().last().unwrap().is_alphabetic() {
            self.ctm.rewind(self.ctm.get_index() - 1);
            key = key[..key.len() - 1].to_string();
        }
        let reference = GlossaryReference::with_display(key, display);

        Ok(self
            .options
            .document
            .glossary
            .lock()
            .unwrap()
            .add_reference(reference))
    }

    /// parses plain text as a string until it encounters an unescaped special inline char
    fn parse_plain(&mut self) -> ParseResult<PlainText> {
        if self.ctm.check_char(&LB) {
            return Err(self.ctm.err());
        }
        let mut characters = String::new();
        if !self.ctm.check_char(&SPECIAL_ESCAPE) {
            characters.push(self.ctm.get_current());
        }

        while let Some(ch) = self.ctm.next_char() {
            let index = self.ctm.get_index();
            if self.ctm.check_any(&INLINE_SPECIAL_CHARS)
                || self.ctm.check_any(&self.inline_break_at)
                || self.ctm.check_any_sequence(&INLINE_SPECIAL_SEQUENCES)
                || (self.parse_variables && self.ctm.check_char(&TEMP_VAR_OPEN))
            {
                self.ctm.rewind(index);
                break;
            }
            if !self.ctm.check_char(&SPECIAL_ESCAPE) {
                characters.push(ch)
            }
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
        if !self.ctm.check_eof() {
            self.ctm.seek_one()?;
        }

        let metadata = self.parse_inline_metadata().ok();

        let placeholder = Arc::new(RwLock::new(Placeholder::new(name, metadata)));
        self.options
            .document
            .add_placeholder(Arc::clone(&placeholder));

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

    /// parses a character code &code; like a html character code
    fn parse_character_code(&mut self) -> ParseResult<CharacterCode> {
        let start_index = self.ctm.get_index();
        self.ctm.assert_char(&CHARACTER_START, None)?;
        self.ctm.seek_one()?;
        let code =
            self.ctm
                .get_string_until_any_or_rewind(&[CHARACTER_STOP], &[LB], start_index)?;
        self.ctm.seek_one()?;

        Ok(CharacterCode { code })
    }

    /// Parses an arrow
    fn parse_arrow(&mut self) -> ParseResult<Arrow> {
        if !self
            .options
            .document
            .config
            .get_entry(SMART_ARROWS)
            .and_then(|e| e.get().as_bool())
            .unwrap_or(true)
        {
            Err(self.ctm.err())
        } else if self.ctm.check_sequence(A_LEFT_RIGHT_ARROW) {
            self.ctm.seek_one()?;
            Ok(Arrow::LeftRightArrow)
        } else if self.ctm.check_sequence(A_RIGHT_ARROW) {
            self.ctm.seek_one()?;
            Ok(Arrow::RightArrow)
        } else if self.ctm.check_sequence(A_LEFT_ARROW) {
            self.ctm.seek_one()?;
            Ok(Arrow::LeftArrow)
        } else if self.ctm.check_sequence(A_BIG_LEFT_RIGHT_ARROW) {
            self.ctm.seek_one()?;
            Ok(Arrow::BigLeftRightArrow)
        } else if self.ctm.check_sequence(A_BIG_RIGHT_ARROW) {
            self.ctm.seek_one()?;
            Ok(Arrow::BigRightArrow)
        } else if self.ctm.check_sequence(A_BIG_LEFT_ARROW) {
            self.ctm.seek_one()?;
            Ok(Arrow::BigLeftArrow)
        } else {
            Err(self.ctm.err())
        }
    }
}
