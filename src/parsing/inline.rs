use super::charstate::CharStateMachine;
use super::elements::*;
use super::tokens::*;
use crate::parsing::bibliography::BibReference;
use crate::parsing::configuration::keys::BIB_REF_DISPLAY;
use crate::parsing::utils::{ParseError, ParseResult};
use crate::Parser;
use std::sync::{Arc, Mutex};

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
    fn parse_bibref(&mut self) -> ParseResult<Arc<Mutex<BibReference>>>;
    fn parse_template_variable(&mut self) -> ParseResult<Arc<Mutex<TemplateVariable>>>;
    fn parse_plain(&mut self) -> ParseResult<PlainText>;
}

impl ParseInline for Parser {
    /// parses Inline surrounded by characters
    fn parse_surrounded(&mut self, surrounding: &char) -> ParseResult<Inline> {
        let start_index = self.index;
        self.assert_special(surrounding, start_index)?;
        self.skip_char();
        let inline = self.parse_inline()?;
        self.assert_special(surrounding, start_index)?;
        self.skip_char();

        Ok(inline)
    }

    /// parses Inline, the formatting parts of a line (Text)
    fn parse_inline(&mut self) -> ParseResult<Inline> {
        if self.parse_variables {
            if let Ok(var) = self.parse_template_variable() {
                return Ok(Inline::TemplateVar(var));
            }
        }
        if self.check_special(&PIPE) || self.check_linebreak() {
            Err(ParseError::new(self.index))
        } else if self.check_eof() {
            Err(ParseError::eof(self.index))
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
        let start_index = self.index;
        self.seek_inline_whitespace();
        self.assert_special(&IMG_START, start_index)?;
        self.skip_char();

        if let Ok(url) = self.parse_url(true) {
            let metadata = if let Ok(meta) = self.parse_inline_metadata() {
                Some(meta)
            } else {
                None
            };
            Ok(Image { url, metadata })
        } else {
            Err(self.revert_with_error(start_index))
        }
    }

    // parses an url
    fn parse_url(&mut self, short_syntax: bool) -> ParseResult<Url> {
        let start_index = self.index;
        self.seek_inline_whitespace();

        let mut description = String::new();
        if self.check_special(&DESC_OPEN) {
            self.skip_char();
            description = if let Ok(desc) = self.get_string_until(&[DESC_CLOSE], &[LB]) {
                desc
            } else {
                return Err(self.revert_with_error(start_index));
            };
        } else if !short_syntax {
            return Err(self.revert_with_error(start_index));
        }
        self.skip_char();
        self.assert_special(&URL_OPEN, start_index)?;
        self.skip_char();
        self.seek_inline_whitespace();

        let url = if let Ok(url_str) = self.get_string_until(&[URL_CLOSE], &[LB]) {
            url_str
        } else {
            return Err(self.revert_with_error(start_index));
        };
        self.skip_char();

        if description.is_empty() {
            Ok(Url::new(None, url))
        } else {
            Ok(Url::new(Some(description), url))
        }
    }

    /// parses a markdown checkbox
    fn parse_checkbox(&mut self) -> ParseResult<Checkbox> {
        let start_index = self.index;
        self.assert_special(&CHECK_OPEN, start_index)?;
        self.skip_char();
        let checked = if self.check_special(&CHECK_CHECKED) {
            true
        } else if self.check_special(&SPACE) {
            false
        } else {
            return Err(self.revert_with_error(start_index));
        };
        self.skip_char();
        self.assert_special(&CHECK_CLOSE, start_index)?;
        self.skip_char();

        Ok(Checkbox { value: checked })
    }

    /// parses bold text with must start with two asterisks
    fn parse_bold(&mut self) -> ParseResult<BoldText> {
        let start_index = self.index;
        self.assert_special_sequence(&BOLD, start_index)?;
        self.skip_char();
        let inline = self.parse_inline()?;
        self.assert_special_sequence(&BOLD, start_index)?;
        self.skip_char();

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
        let start_index = self.index;
        self.assert_special(&BACKTICK, start_index)?;
        self.skip_char();
        let content = self.get_string_until(&[BACKTICK, LB], &[])?;
        self.assert_special(&BACKTICK, start_index)?;
        self.skip_char();

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
        let start_index = self.index;
        self.assert_special(&EMOJI, start_index)?;
        self.skip_char();
        let name = self.get_string_until_or_revert(&[EMOJI], &[SPACE, LB], start_index)?;
        self.skip_char();
        if let Some(emoji) = gh_emoji::get(name.as_str()) {
            let emoji_char = *emoji.chars().collect::<Vec<char>>().first().unwrap();
            Ok(Emoji {
                value: emoji_char,
                name,
            })
        } else {
            Err(self.revert_with_error(start_index))
        }
    }

    /// parses colored text
    fn parse_colored(&mut self) -> ParseResult<Colored> {
        let start_index = self.index;
        self.assert_special_sequence(&SQ_COLOR_START, start_index)?;
        self.skip_char();
        let color =
            self.get_string_until_or_revert(&[COLOR_CLOSE], &[SPACE, LB, SEMICOLON], start_index)?;
        self.skip_char();
        if color.is_empty() {
            return Err(ParseError::new(self.index));
        }
        Ok(Colored {
            value: Box::new(self.parse_inline()?),
            color,
        })
    }

    fn parse_bibref(&mut self) -> ParseResult<Arc<Mutex<BibReference>>> {
        let start_index = self.index;
        self.assert_special_sequence(&SQ_BIBREF_START, start_index)?;
        self.skip_char();
        let key = self.get_string_until_or_revert(&[BIBREF_CLOSE], &[SPACE, LB], start_index)?;
        self.skip_char();
        let ref_entry = Arc::new(Mutex::new(BibReference::new(
            key,
            self.document.config.get_ref_entry(BIB_REF_DISPLAY),
        )));
        self.document
            .bibliography
            .add_ref_entry(Arc::clone(&ref_entry));

        Ok(ref_entry)
    }

    /// parses a template variable {prefix{name}suffix}
    fn parse_template_variable(&mut self) -> ParseResult<Arc<Mutex<TemplateVariable>>> {
        let start_index = self.index;
        self.assert_special(&TEMP_VAR_OPEN, start_index)?;
        self.skip_char();
        let prefix = self.get_string_until_or_revert(&[TEMP_VAR_OPEN], &[LB], start_index)?;
        self.skip_char();
        let name = self.get_string_until_or_revert(&[TEMP_VAR_CLOSE], &[LB], start_index)?;
        self.skip_char();
        let suffix = self.get_string_until_or_revert(&[TEMP_VAR_CLOSE], &[LB], start_index)?;
        self.skip_char();
        Ok(Arc::new(Mutex::new(TemplateVariable {
            value: None,
            name,
            prefix,
            suffix,
        })))
    }

    /// parses plain text as a string until it encounters an unescaped special inline char
    fn parse_plain(&mut self) -> ParseResult<PlainText> {
        if self.check_linebreak() {
            return Err(ParseError::new(self.index));
        }
        let mut characters = String::new();
        characters.push(self.current_char);
        while let Some(ch) = self.next_char() {
            if self.check_special_group(&INLINE_SPECIAL_CHARS)
                || self.check_special_group(&self.inline_break_at)
                || (self.parse_variables && self.check_special(&TEMP_VAR_OPEN))
            {
                break;
            }
            characters.push(ch)
        }

        if characters.len() > 0 {
            Ok(PlainText { value: characters })
        } else {
            Err(ParseError::new_with_message(
                self.index,
                "no plaintext characters parsed",
            ))
        }
    }
}
