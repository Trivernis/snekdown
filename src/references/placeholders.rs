use crate::elements::*;
use chrono::prelude::*;
use regex::Regex;

macro_rules! block {
    ($inner:expr) => {
        Element::Block(Box::new($inner))
    };
}

#[allow(unused)]
macro_rules! line {
    ($inner:expr) => {
        Element::Line(Box::new($inner))
    };
}

macro_rules! inline {
    ($inner:expr) => {
        Element::Inline(Box::new($inner))
    };
}

pub(crate) trait ProcessPlaceholders {
    fn process_placeholders(&mut self);
    fn process_definitions(&mut self);
}

const S_VALUE: &str = "value";

const P_TOC: &str = "toc";
const P_DATE: &str = "date";
const P_TIME: &str = "time";
const P_DATETIME: &str = "datetime";

impl ProcessPlaceholders for Document {
    /// parses all placeholders and assigns values to them
    fn process_placeholders(&mut self) {
        self.placeholders.iter().for_each(|p| {
            let mut pholder = p.write().unwrap();
            match pholder.name.to_lowercase().as_str() {
                P_TOC => {
                    let ordered = if let Some(meta) = &pholder.metadata {
                        meta.get_bool("ordered")
                    } else {
                        false
                    };
                    pholder.set_value(block!(Block::List(self.create_toc(ordered))))
                }
                P_DATE => pholder.set_value(inline!(Inline::Plain(PlainText {
                    value: get_date_string()
                }))),
                P_TIME => pholder.set_value(inline!(Inline::Plain(PlainText {
                    value: get_time_string()
                }))),
                P_DATETIME => pholder.set_value(inline!(Inline::Plain(PlainText {
                    value: format!("{} {}", get_date_string(), get_time_string())
                }))),
                _ => {
                    if let Some(entry) = self.config.get_entry(pholder.name.to_lowercase().as_str())
                    {
                        let value = entry.get().as_string();
                        pholder.set_value(inline!(Inline::Plain(PlainText { value })))
                    }
                }
            }
        })
    }

    fn process_definitions(&mut self) {
        lazy_static::lazy_static! {
            static ref RE_SET: Regex = Regex::new(r"^set:(.*)$").unwrap();
        }

        let placeholders = self.placeholders.clone();
        placeholders.iter().for_each(|p| {
            let mut pholder = p.write().unwrap();
            let name = pholder.name.clone();

            if let Some(cap) = RE_SET.captures(&name) {
                if let Some(key) = cap.get(1) {
                    let key: &str = key.as_str();
                    pholder.value = Some(inline!(Inline::Plain(PlainText {
                        value: "".to_string()
                    })));
                    if let Some(meta) = &pholder.metadata {
                        if let Some(value) = meta.data.get(S_VALUE) {
                            self.config.set_from_meta(key, value.clone())
                        }
                    }
                }
                return;
            }
        });
    }
}

fn get_time_string() -> String {
    let now = Local::now();
    format!("{:02}:{:02}:{:02}", now.hour(), now.minute(), now.second())
}

fn get_date_string() -> String {
    let now = Local::now();
    format!("{:02}.{:02}.{:04}", now.day(), now.month(), now.year())
}
