/*
 * Snekdown - Custom Markdown flavour and parser
 * Copyright (C) 2021  Trivernis
 * See LICENSE for more information.
 */

pub mod elements;
pub mod format;
pub mod parser;
pub mod references;
pub mod settings;
pub mod utils;

pub use parser::Parser;
pub use utils::parsing;
