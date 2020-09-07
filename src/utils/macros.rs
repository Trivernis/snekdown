#[macro_export]
macro_rules! plain_text {
    ($e:expr) => {
        Inline::Plain(PlainText { value: $e })
    };
}

#[macro_export]
macro_rules! bold_text {
    ($e:expr) => {
        Inline::Bold(BoldText {
            value: vec![Inline::Plain(PlainText { value: $e })],
        })
    };
}

#[macro_export]
macro_rules! italic_text {
    ($e:expr) => {
        Inline::Italic(ItalicText {
            value: vec![Inline::Plain(PlainText { value: $e })],
        })
    };
}

#[macro_export]
macro_rules! url_text {
    ($e:expr) => {
        Inline::Url(Url {
            url: $e,
            description: None,
        })
    };
}

#[macro_export]
macro_rules! list_item {
    ($e:expr, $k:expr) => {
        ListItem::new(
            Line::Anchor(Anchor {
                inner: Box::new(Line::Text($e)),
                key: $k,
            }),
            0,
            true,
        )
    };
}
