use super::charstate::CharStateMachine;
use super::elements::*;
use super::parser::ParseError;
use super::tokens::*;
use crate::Parser;

pub(crate) trait ParseInline {
    fn parse_inline(&mut self) -> Result<Inline, ParseError>;
    fn parse_image(&mut self) -> Result<Image, ParseError>;
    fn parse_url(&mut self, short_syntax: bool) -> Result<Url, ParseError>;
    fn parse_bold(&mut self) -> Result<BoldText, ParseError>;
    fn parse_italic(&mut self) -> Result<ItalicText, ParseError>;
    fn parse_striked(&mut self) -> Result<StrikedText, ParseError>;
    fn parse_monospace(&mut self) -> Result<MonospaceText, ParseError>;
    fn parse_underlined(&mut self) -> Result<UnderlinedText, ParseError>;
    fn parse_superscript(&mut self) -> Result<SuperscriptText, ParseError>;
    fn parse_plain(&mut self) -> Result<PlainText, ParseError>;
    fn parse_surrounded(&mut self, surrounding: &char) -> Result<Inline, ParseError>;
}

impl ParseInline for Parser {
    /// parses Inline, the formatting parts of a line (Text)
    fn parse_inline(&mut self) -> Result<Inline, ParseError> {
        if self.check_special(&PIPE) || self.check_linebreak() {
            Err(ParseError::new(self.index))
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
        } else {
            Ok(Inline::Plain(self.parse_plain()?))
        }
    }

    /// parses an image url
    fn parse_image(&mut self) -> Result<Image, ParseError> {
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
    fn parse_url(&mut self, short_syntax: bool) -> Result<Url, ParseError> {
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

    /// parses bold text with must start with two asterisks
    fn parse_bold(&mut self) -> Result<BoldText, ParseError> {
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

    fn parse_italic(&mut self) -> Result<ItalicText, ParseError> {
        Ok(ItalicText {
            value: Box::new(self.parse_surrounded(&ITALIC)?),
        })
    }

    fn parse_striked(&mut self) -> Result<StrikedText, ParseError> {
        Ok(StrikedText {
            value: Box::new(self.parse_surrounded(&STRIKED)?),
        })
    }

    /// parses monospace text (inline-code) that isn't allowed to contain special characters
    fn parse_monospace(&mut self) -> Result<MonospaceText, ParseError> {
        let start_index = self.index;
        self.assert_special(&BACKTICK, start_index)?;
        self.skip_char();
        let content = self.get_string_until(&[BACKTICK, LB], &[])?;
        self.assert_special(&BACKTICK, start_index)?;
        self.skip_char();

        Ok(MonospaceText { value: content })
    }

    fn parse_underlined(&mut self) -> Result<UnderlinedText, ParseError> {
        Ok(UnderlinedText {
            value: Box::new(self.parse_surrounded(&UNDERLINED)?),
        })
    }

    fn parse_superscript(&mut self) -> Result<SuperscriptText, ParseError> {
        Ok(SuperscriptText {
            value: Box::new(self.parse_surrounded(&SUPER)?),
        })
    }

    /// parses plain text as a string until it encounters an unescaped special inline char
    fn parse_plain(&mut self) -> Result<PlainText, ParseError> {
        if self.check_linebreak() {
            return Err(ParseError::new(self.index));
        }
        let mut characters = String::new();
        characters.push(self.current_char);
        while let Some(ch) = self.next_char() {
            if self.check_special_group(&INLINE_SPECIAL_CHARS)
                || self.check_special_group(&self.inline_break_at)
            {
                break;
            }
            characters.push(ch)
        }

        if characters.len() > 0 {
            Ok(PlainText { value: characters })
        } else {
            Err(ParseError::new(self.index))
        }
    }

    /// parses Inline surrounded by characters
    fn parse_surrounded(&mut self, surrounding: &char) -> Result<Inline, ParseError> {
        let start_index = self.index;
        self.assert_special(surrounding, start_index)?;
        self.skip_char();
        let inline = self.parse_inline()?;
        self.assert_special(surrounding, start_index)?;
        self.skip_char();

        Ok(inline)
    }
}
