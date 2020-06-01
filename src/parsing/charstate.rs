use crate::parsing::parser::ParseError;
use crate::parsing::tokens::{LB, SPECIAL_ESCAPE};
use crate::Parser;

pub trait CharStateMachine {
    fn next_char(&mut self) -> Option<char>;
    fn skip_char(&mut self);
    fn revert_to(&mut self, index: usize) -> Result<(), ParseError>;
    fn revert_with_error(&mut self, index: usize) -> ParseError;
    fn seek_inline_whitespace(&mut self);
    fn seek_whitespace(&mut self);
    fn seek_until_linebreak(&mut self);
    fn check_seek_inline_whitespace(&mut self) -> bool;
    fn check_escaped(&self) -> bool;
    fn check_special(&self, character: &char) -> bool;
    fn check_special_group(&self, group: &[char]) -> bool;
    fn check_special_sequence(&mut self, sequence: &[char]) -> bool;
    fn check_special_sequence_group(&mut self, sequences: &[&[char]]) -> bool;
    fn check_linebreak(&self) -> bool;
    fn assert_special(&mut self, character: &char, revert_index: usize) -> Result<(), ParseError>;
    fn assert_special_group(
        &mut self,
        group: &[char],
        revert_index: usize,
    ) -> Result<(), ParseError>;
    fn assert_special_sequence(
        &mut self,
        sequence: &[char],
        revert_index: usize,
    ) -> Result<(), ParseError>;
    fn assert_special_sequence_group(
        &mut self,
        sequences: &[&[char]],
        revert_index: usize,
    ) -> Result<(), ParseError>;
    fn get_string_until(
        &mut self,
        break_at: &[char],
        err_at: &[char],
    ) -> Result<String, ParseError>;
    fn get_string_until_sequence(
        &mut self,
        break_at: &[&[char]],
        err_at: &[char],
    ) -> Result<String, ParseError>;
}

impl CharStateMachine for Parser {
    /// Increments the current index and returns the
    /// char at the indexes position
    fn next_char(&mut self) -> Option<char> {
        self.index += 1;

        self.current_char = *self.text.get(self.index)?;

        Some(self.current_char)
    }

    /// skips to the next char
    fn skip_char(&mut self) {
        let _ = self.next_char();
    }

    /// Returns to an index position
    fn revert_to(&mut self, index: usize) -> Result<(), ParseError> {
        if let Some(char) = self.text.get(index) {
            self.index = index;
            self.current_char = char.clone();
            Ok(())
        } else {
            Err(ParseError::new_with_message(index, "failed to revert"))
        }
    }

    /// reverts and returns a parse error
    fn revert_with_error(&mut self, index: usize) -> ParseError {
        let err = ParseError::new(self.index);

        if let Err(revert_err) = self.revert_to(index) {
            revert_err
        } else {
            err
        }
    }

    /// Skips characters until it encounters a character
    /// that isn't an inline whitespace character
    fn seek_inline_whitespace(&mut self) {
        if self.current_char.is_whitespace() && !self.check_linebreak() {
            while let Some(next_char) = self.next_char() {
                if !next_char.is_whitespace() || self.check_linebreak() {
                    break;
                }
            }
        }
    }

    /// Skips characters until it encounters a character
    /// that isn't a whitespace character
    fn seek_whitespace(&mut self) {
        if self.current_char.is_whitespace() {
            while let Some(next_char) = self.next_char() {
                if !next_char.is_whitespace() {
                    break;
                }
            }
        }
    }

    /// checks if the input character is escaped
    fn check_escaped(&self) -> bool {
        if self.index == 0 {
            return false;
        }
        if let Some(previous_char) = self.text.get(self.index - 1) {
            if previous_char == &SPECIAL_ESCAPE {
                return true;
            }
        }
        return false;
    }

    /// checks if the current character is the given input character and not escaped
    fn check_special(&self, character: &char) -> bool {
        self.current_char == *character && !self.check_escaped()
    }

    /// checks if the current character is part of the given group
    fn check_special_group(&self, chars: &[char]) -> bool {
        chars.contains(&self.current_char) && !self.check_escaped()
    }

    /// checks if the next chars are a special sequence
    fn check_special_sequence_group(&mut self, sequences: &[&[char]]) -> bool {
        for sequence in sequences {
            if self.check_special_sequence(*sequence) {
                return true;
            }
        }

        false
    }

    /// returns if the current character is a linebreak character
    /// Note: No one likes CRLF
    fn check_linebreak(&self) -> bool {
        self.current_char == LB && !self.check_escaped()
    }

    /// seeks inline whitespaces and returns if there
    /// were seeked whitespaces
    fn check_seek_inline_whitespace(&mut self) -> bool {
        let start_index = self.index;
        self.seek_inline_whitespace();
        self.index > start_index
    }

    /// checks if the next characters match a special sequence
    fn check_special_sequence(&mut self, sequence: &[char]) -> bool {
        let start_index = self.index;
        if self.check_escaped() {
            self.revert_to(start_index).unwrap();
            return false;
        }
        for sq_character in sequence {
            if self.current_char != *sq_character {
                let _ = self.revert_to(start_index);
                return false;
            }
            if self.next_char() == None {
                let _ = self.revert_to(start_index);
                return false;
            }
        }
        if self.index > 0 {
            self.revert_to(self.index - 1).unwrap();
        }

        true
    }

    /// returns the string until a specific
    fn get_string_until(
        &mut self,
        break_at: &[char],
        err_at: &[char],
    ) -> Result<String, ParseError> {
        let start_index = self.index;
        let mut result = String::new();
        if self.check_special_group(break_at) {
            return Ok(result);
        } else if self.check_special_group(err_at) {
            return Err(ParseError::new(self.index));
        }

        result.push(self.current_char);
        while let Some(ch) = self.next_char() {
            if self.check_special_group(break_at) || self.check_special_group(err_at) {
                break;
            }
            result.push(ch);
        }

        if self.check_special_group(err_at) {
            Err(self.revert_with_error(start_index))
        } else {
            Ok(result)
        }
    }

    /// Returns the string until a specific end sequence or an error character
    fn get_string_until_sequence(
        &mut self,
        break_at: &[&[char]],
        err_at: &[char],
    ) -> Result<String, ParseError> {
        let start_index = self.index;
        let mut result = String::new();
        if self.check_special_sequence_group(break_at) {
            return Ok(result);
        } else if self.check_special_group(err_at) {
            return Err(ParseError::new(self.index));
        }

        result.push(self.current_char);
        while let Some(ch) = self.next_char() {
            if self.check_special_sequence_group(break_at) || self.check_special_group(err_at) {
                break;
            }
            result.push(ch);
        }

        if self.check_special_group(err_at) {
            Err(self.revert_with_error(start_index))
        } else {
            Ok(result)
        }
    }

    fn assert_special(&mut self, character: &char, revert_index: usize) -> Result<(), ParseError> {
        if self.check_special(character) {
            Ok(())
        } else {
            Err(self.revert_with_error(revert_index))
        }
    }

    fn assert_special_group(
        &mut self,
        group: &[char],
        revert_index: usize,
    ) -> Result<(), ParseError> {
        if self.check_special_group(group) {
            Ok(())
        } else {
            Err(self.revert_with_error(revert_index))
        }
    }

    fn assert_special_sequence(
        &mut self,
        sequence: &[char],
        revert_index: usize,
    ) -> Result<(), ParseError> {
        if self.check_special_sequence(sequence) {
            Ok(())
        } else {
            Err(self.revert_with_error(revert_index))
        }
    }

    fn assert_special_sequence_group(
        &mut self,
        sequences: &[&[char]],
        revert_index: usize,
    ) -> Result<(), ParseError> {
        if self.check_special_sequence_group(sequences) {
            Ok(())
        } else {
            Err(self.revert_with_error(revert_index))
        }
    }

    /// seeks until it encounters a linebreak character
    fn seek_until_linebreak(&mut self) {
        if self.check_special(&LB) {
            self.skip_char();
            return;
        }
        while let Some(_) = self.next_char() {
            if self.check_special(&LB) {
                self.skip_char();
                return;
            }
        }
    }
}
