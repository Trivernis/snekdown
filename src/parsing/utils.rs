use colored::*;
use std::error::Error;
use std::fmt;
use std::fmt::{Display, Formatter};

pub type ParseResult<T> = Result<T, ParseError>;

#[derive(Debug)]
pub struct ParseError {
    index: usize,
    message: Option<String>,
}
impl Display for ParseError {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        if let Some(message) = &self.message {
            write!(
                f,
                "{}",
                format!("Parse Error at index {}: {}", self.index, message).red()
            )
        } else {
            write!(
                f,
                "{}",
                format!("Parse Error at index {}", self.index).red()
            )
        }
    }
}
impl Error for ParseError {}
impl ParseError {
    pub fn new(index: usize) -> Self {
        Self {
            index,
            message: None,
        }
    }

    pub fn new_with_message(index: usize, message: &str) -> Self {
        Self {
            index,
            message: Some(message.to_string()),
        }
    }

    pub fn set_message(&mut self, message: &str) {
        self.message = Some(message.to_string());
    }

    pub fn get_position(&self, content: &str) -> Option<(usize, usize)> {
        if content.len() <= self.index {
            return None;
        }
        let split_content = content.split_at(self.index);
        let line_number = split_content.0.matches("\n").count() as usize;
        let overshoot_position = self.index as isize - split_content.0.len() as isize;

        if let Some(line) = split_content.0.lines().last() {
            let inline_position = (line.len() as isize + overshoot_position) as usize;

            Some((line_number, inline_position))
        } else {
            None
        }
    }
}
