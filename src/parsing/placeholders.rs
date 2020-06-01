use super::elements::*;
use chrono::prelude::*;
use regex::Regex;
use std::sync::{Arc, Mutex, MutexGuard};

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
    fn add_bib_entry(&mut self, key: String, value: BibEntry) -> Arc<Mutex<BibEntry>>;
    fn get_bib_entry(&self, ph: &MutexGuard<Placeholder>, key: &str) -> BibEntry;
}

const B_AUTHOR: &str = "author";
const B_DATE: &str = "date";
const B_URL: &str = "url";
const B_TITLE: &str = "title";
const B_PUBLISHER: &str = "publisher";
const B_NOTES: &str = "notes";

#[derive(Clone, Debug)]
pub struct BibEntry {
    key: String,
    author: Option<String>,
    date: Option<String>,
    url: Option<String>,
    title: Option<String>,
    publisher: Option<String>,
    notes: Option<String>,
    display: Option<Arc<Mutex<ConfigValue>>>,
}

impl BibEntry {
    pub fn get_formatted(&self) -> String {
        if let Some(display) = &self.display {
            let value = display.lock().unwrap();
            if let MetadataValue::String(format) = &value.value {
                let mut format = format.clone();
                if let Some(author) = &self.author {
                    format = format.replace("author", author);
                }
                if let Some(date) = &self.date {
                    format = format.replace("date", date);
                }
                if let Some(url) = &self.url {
                    format = format.replace("url", url)
                }
                if let Some(title) = &self.title {
                    format = format.replace("title", title);
                }
                if let Some(publisher) = &self.publisher {
                    format = format.replace("publisher", publisher);
                }
                if let Some(notes) = &self.notes {
                    format = format.replace("notes", notes);
                }

                format
            } else {
                format!("'Invalid formatter!' {:?}", self)
            }
        } else {
            format!("{:?}", self)
        }
    }
}

const S_VALUE: &str = "value";

const P_TOC: &str = "toc";
const P_DATE: &str = "date";
const P_TIME: &str = "time";
const P_DATETIME: &str = "datetime";

impl ProcessPlaceholders for Document {
    /// parses all placeholders and assigns values to them
    fn process_placeholders(&mut self) {
        self.process_definitions();
        lazy_static::lazy_static! {static ref RE_REF: Regex = Regex::new(r"^ref:(.*)$").unwrap();}
        self.placeholders.iter().for_each(|p| {
            let mut pholder = p.lock().unwrap();
            if let Some(cap) = RE_REF.captures(&pholder.name) {
                if let Some(key) = cap.get(1) {
                    if let Some(entry) = self.bib_entries.get(key.as_str()) {
                        pholder.value = Some(inline!(Inline::Reference(Reference {
                            value: Some(RefValue::BibEntry(entry.clone())),
                            metadata: pholder.metadata.clone()
                        })))
                    } else {
                        pholder.value = Some(inline!(Inline::Reference(Reference {
                            value: None,
                            metadata: pholder.metadata.clone()
                        })))
                    }
                }
            }
            match pholder.name.to_ascii_lowercase().as_str() {
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
                _ => {}
            }
        })
    }

    fn process_definitions(&mut self) {
        lazy_static::lazy_static! {
            static ref RE_BIB: Regex = Regex::new(r"^bib:(.*)$").unwrap();
        }

        lazy_static::lazy_static! {
            static ref RE_SET: Regex = Regex::new(r"^set:(.*)$").unwrap();
        }

        let placeholders = self.placeholders.clone();
        placeholders.iter().for_each(|p| {
            let mut pholder = p.lock().unwrap();

            let name = pholder.name.clone();
            if let Some(cap) = RE_BIB.captures(&name) {
                if let Some(key) = cap.get(1) {
                    let key: &str = key.as_str();
                    let entry = self.get_bib_entry(&pholder, key);
                    let entry = self.add_bib_entry(key.to_string(), entry);
                    pholder.value = Some(Element::Line(Box::new(Line::ReferenceEntry(
                        ReferenceEntry {
                            value: Some(RefValue::BibEntry(entry)),
                        },
                    ))));
                }
                return;
            }

            if let Some(cap) = RE_SET.captures(&name) {
                if let Some(key) = cap.get(1) {
                    let key: &str = key.as_str();
                    pholder.value = Some(inline!(Inline::Plain(PlainText {
                        value: "".to_string()
                    })));
                    if let Some(meta) = &pholder.metadata {
                        if let Some(value) = meta.data.get(S_VALUE) {
                            self.set_config_param(
                                key.to_string(),
                                ConfigValue {
                                    value: value.clone(),
                                },
                            );
                        }
                    }
                }
                return;
            }
        });
    }

    fn add_bib_entry(&mut self, key: String, value: BibEntry) -> Arc<Mutex<BibEntry>> {
        let arc_entry = Arc::new(Mutex::new(value));
        self.bib_entries.insert(key, Arc::clone(&arc_entry));

        arc_entry
    }

    fn get_bib_entry(&self, ph: &MutexGuard<Placeholder>, key: &str) -> BibEntry {
        if let Some(meta) = &ph.metadata {
            BibEntry {
                key: key.to_string(),
                author: meta.get_string(B_AUTHOR),
                date: meta.get_string(B_DATE),
                url: meta.get_string(B_URL),
                title: meta.get_string(B_TITLE),
                publisher: meta.get_string(B_PUBLISHER),
                notes: meta.get_string(B_NOTES),
                display: self.get_config_param("bib-display"),
            }
        } else {
            BibEntry {
                key: key.to_string(),
                author: None,
                date: None,
                url: None,
                title: None,
                publisher: None,
                notes: None,
                display: self.get_config_param("bib-display"),
            }
        }
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
