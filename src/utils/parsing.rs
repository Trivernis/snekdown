/*
 * Snekdown - Custom Markdown flavour and parser
 * Copyright (C) 2021  Trivernis
 * See LICENSE for more information.
 */

use regex::Regex;
#[macro_export]
macro_rules! parse {
    ($str:expr) => {
        Parser::new($str.to_string(), None).parse()
    };
}

/// Removes a single backslash from the given content
pub(crate) fn remove_single_backlslash<S: ToString>(content: S) -> String {
    let content = content.to_string();
    lazy_static::lazy_static! {static ref R: Regex = Regex::new(r"\\(?P<c>[^\\])").unwrap();}

    R.replace_all(&*content, "$c").to_string()
}
