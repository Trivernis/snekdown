use crate::lexer::tokens::{HeaderStartToken, LinebreakToken, WhitespaceToken, WordToken};
use charred::error::TapeResult;
use charred::input_reader::InputReader;
use charred::token::Token;

/// Parses a whitespace token
pub async fn parse_whitespace(reader: &mut InputReader) -> TapeResult<Option<Token>> {
    let check_whitespace = |c: char| c.is_whitespace() && c != '\n';
    let mut count = 0;

    while !reader.check_eof().await && check_whitespace(reader.peek().await?) {
        reader.consume().await?;
        count += 1;
    }

    if count > 0 {
        Ok(Some(Token::new(WhitespaceToken)))
    } else {
        Ok(None)
    }
}

/// Parses a linebreak token
pub async fn parse_linebreak(reader: &mut InputReader) -> TapeResult<Option<Token>> {
    if !reader.check_eof().await && reader.peek().await? == '\r' {
        reader.consume().await?;
    }
    if reader.check_eof().await || reader.peek().await? != '\n' {
        return Ok(None);
    }
    reader.consume().await?;

    Ok(Some(Token::new(LinebreakToken)))
}

/// Parses a word token
pub async fn parse_word(reader: &mut InputReader) -> TapeResult<Option<Token>> {
    let mut text = String::new();
    let check_word = |c: char| !c.is_whitespace();

    while !reader.check_eof().await && check_word(reader.peek().await?) {
        text.push(reader.consume().await?)
    }

    if text.len() > 0 {
        Ok(Some(Token::new(WordToken(text))))
    } else {
        Ok(None)
    }
}

/// Parses a markdown header start
pub async fn parse_header_start(reader: &mut InputReader) -> TapeResult<Option<Token>> {
    let mut size = 0u8;
    let previous = reader.previous().await;
    if previous.is_some() && previous.unwrap() != '\n' {
        return Ok(None);
    }
    while !reader.check_eof().await && reader.peek().await? == '#' {
        reader.consume().await?;
        size += 1;
    }

    if size > 0 {
        Ok(Some(Token::new(HeaderStartToken(size))))
    } else {
        Ok(None)
    }
}
