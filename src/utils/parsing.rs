use charred::tapemachine::{TapeError, TapeResult};
use colored::*;
use std::error::Error;
use std::fmt;
use std::fmt::{Display, Formatter};

#[macro_export]
macro_rules! parse {
    ($str:expr) => {
        Parser::new($str.to_string(), None).parse()
    };
}

pub type ParseResult<T> = TapeResult<T>;
pub type ParseError = TapeError;

/*
#[derive(Debug)]
pub struct ParseError {
    index: usize,
    message: Option<String>,
    pub(crate) eof: bool,
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
            eof: false,
        }
    }

    pub fn new_with_message(index: usize, message: &str) -> Self {
        Self {
            index,
            message: Some(message.to_string()),
            eof: false,
        }
    }

    pub fn eof(index: usize) -> Self {
        Self {
            index,
            message: Some("EOF".to_string()),
            eof: true,
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
        let line_number = split_content.0.lines().count() as usize;

        if let Some(line) = split_content.0.lines().last() {
            let inline_position = line.len();

            Some((line_number, inline_position))
        } else {
            None
        }
    }
}
*/
