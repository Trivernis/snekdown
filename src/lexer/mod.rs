use crate::error::SnekdownResult;
use crate::lexer::token_parsers::{
    parse_header_start, parse_linebreak, parse_whitespace, parse_word,
};
use charred::input_reader::InputReader;
use charred::lexer::Lexer;
use charred::token::{Token, TokenCheckerFn};
use std::sync::Arc;
use tokio::io::AsyncBufRead;

mod token_parsers;
pub mod tokens;

/// Tokenizes a string
pub async fn tokenize<R: AsyncBufRead + Unpin + Send + 'static>(
    reader: R,
) -> SnekdownResult<Vec<Token>> {
    let input_reader = InputReader::new(reader);
    let checkers: Vec<TokenCheckerFn> = vec![
        Arc::new(|r| Box::pin(parse_header_start(r))),
        Arc::new(|r| Box::pin(parse_whitespace(r))),
        Arc::new(|r| Box::pin(parse_linebreak(r))),
        Arc::new(|r| Box::pin(parse_word(r))),
    ];
    let mut lexer = Lexer::new(input_reader, checkers);
    let tokens = lexer.scan().await?;

    Ok(tokens)
}
