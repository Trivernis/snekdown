#[macro_export]
macro_rules! parse {
    ($str:expr) => {
        Parser::new($str.to_string(), None).parse()
    };
}
