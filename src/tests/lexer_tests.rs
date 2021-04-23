use crate::lexer::tokenize;
use crate::lexer::tokens::{HeaderStartToken, LinebreakToken, WhitespaceToken, WordToken};
use charred::token::UnknownToken;
use std::io::Cursor;

#[tokio::test]
async fn it_tokenizes_everything() {
    let input = r#"
# A Snekdown Document
With multiple  lines
<[import.md]
And some whitespaces

| tables | exist |
|--------|-------|
    "#;
    let tokens = tokenize(Cursor::new(input)).await.unwrap();
    let mut tokens = tokens.into_iter();
    assert!(tokens.next().unwrap().is::<LinebreakToken>());
    assert!(tokens.next().unwrap().is::<HeaderStartToken>());
    assert!(tokens.next().unwrap().is::<WhitespaceToken>());
    assert!(tokens.next().unwrap().is::<WordToken>());
    assert!(tokens.all(|t| !t.is::<UnknownToken>()));
}
