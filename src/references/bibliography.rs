use crate::elements::Metadata;
use crate::format::PlaceholderTemplate;
use crate::references::configuration::keys::{BIB_DISPLAY, BIB_HIDE_UNUSED};
use crate::references::configuration::{ConfigRefEntry, Configuration, Value};
use std::collections::HashMap;
use std::sync::{Arc, RwLock};

const B_NUMBER: &str = "number";
const B_AUTHOR: &str = "author";
const B_DATE: &str = "date";
const B_URL: &str = "url";
const B_TITLE: &str = "title";
const B_PUBLISHER: &str = "publisher";
const B_NOTES: &str = "notes";

#[derive(Clone, Debug)]
pub struct BibEntry {
    pub(crate) number: usize,
    pub(crate) ref_count: usize,
    pub key: String,
    pub author: Option<String>,
    pub date: Option<String>,
    pub url: Option<String>,
    pub title: Option<String>,
    pub publisher: Option<String>,
    pub notes: Option<String>,
    pub display: Option<ConfigRefEntry>,
    pub hide_unused: Option<ConfigRefEntry>,
}

#[derive(Clone, Debug)]
pub struct BibReference {
    pub(crate) key: String,
    pub(crate) reference_entry: Option<Arc<RwLock<BibEntry>>>,
    pub(crate) display: Option<ConfigRefEntry>,
}

#[derive(Clone, Debug)]
pub struct Bibliography {
    entries: HashMap<String, Arc<RwLock<BibEntry>>>,
    references: Vec<Arc<RwLock<BibReference>>>,
}

impl BibEntry {
    pub fn as_map(&self) -> HashMap<String, String> {
        let mut map = HashMap::new();
        map.insert(B_NUMBER.to_string(), format!("{}", self.number));
        map.insert("key".to_string(), self.key.clone());
        if let Some(author) = &self.author {
            map.insert(B_AUTHOR.to_string(), author.clone());
        }
        if let Some(date) = &self.date {
            map.insert(B_DATE.to_string(), date.clone());
        }
        if let Some(url) = &self.url {
            map.insert(B_URL.to_string(), url.clone());
        }
        if let Some(title) = &self.title {
            map.insert(B_TITLE.to_string(), title.clone());
        }
        if let Some(publisher) = &self.publisher {
            map.insert(B_PUBLISHER.to_string(), publisher.clone());
        }
        if let Some(notes) = &self.notes {
            map.insert(B_NOTES.to_string(), notes.clone());
        }

        map
    }

    pub fn from_metadata(key: String, data: Box<dyn Metadata>, config: &Configuration) -> Self {
        BibEntry {
            number: 0,
            ref_count: 0,
            key,
            author: data.get_string(B_AUTHOR),
            date: data.get_string(B_DATE),
            url: data.get_string(B_URL),
            title: data.get_string(B_TITLE),
            publisher: data.get_string(B_PUBLISHER),
            notes: data.get_string(B_NOTES),
            display: config.get_ref_entry(BIB_DISPLAY),
            hide_unused: config.get_ref_entry(BIB_HIDE_UNUSED),
        }
    }

    pub fn from_url(key: String, url: String, config: &Configuration) -> Self {
        BibEntry {
            number: 0,
            ref_count: 0,
            key,
            author: None,
            date: None,
            url: Some(url),
            title: None,
            publisher: None,
            notes: None,
            display: config.get_ref_entry(BIB_DISPLAY),
            hide_unused: config.get_ref_entry(BIB_HIDE_UNUSED),
        }
    }

    pub fn set_number(&mut self, number: usize) {
        self.number = number
    }

    pub fn set_ref_count(&mut self, number: usize) {
        self.ref_count = number
    }

    pub fn is_visible(&self) -> bool {
        if let Some(hide_cfg) = &self.hide_unused {
            let hide_cfg = hide_cfg.read().unwrap();
            if let Value::Bool(b) = hide_cfg.get() {
                if *b && self.ref_count == 0 {
                    return false;
                }
            }
        }

        true
    }
}

impl BibReference {
    pub fn new(key: String, display: Option<ConfigRefEntry>) -> Self {
        Self {
            key: key.to_string(),
            display,
            reference_entry: None,
        }
    }

    /// sets the reference to the bib entry
    pub(crate) fn set_entry(&mut self, entry: Arc<RwLock<BibEntry>>) {
        self.reference_entry = Some(entry)
    }

    pub(crate) fn get_formatted(&self) -> String {
        if let Some(entry) = &self.reference_entry {
            let entry = entry.read().unwrap();
            if let Some(display) = &self.display {
                let display = display.read().unwrap();
                let mut template = PlaceholderTemplate::new(display.get().as_string());
                template.set_replacements(entry.as_map());
                return template.render();
            }
            return format!("{}", entry.number);
        }

        return "citation needed".to_string();
    }
}

impl Bibliography {
    pub fn new() -> Self {
        Self {
            entries: HashMap::new(),
            references: Vec::new(),
        }
    }

    pub(crate) fn assign_entry_data(&mut self) {
        let mut count = 0;
        self.references.iter().for_each(|e| {
            let mut reference = e.write().unwrap();
            if let Some(entry) = self.entries.get(&reference.key) {
                {
                    let mut entry_raw = entry.write().unwrap();
                    let ref_count = entry_raw.ref_count;
                    entry_raw.set_ref_count(ref_count + 1);
                }
                reference.set_entry(Arc::clone(entry));
            }
        });
        self.entries.iter().for_each(|(_, e)| {
            let mut entry = e.write().unwrap();
            if entry.is_visible() {
                count += 1;
                entry.set_number(count)
            }
        });
    }

    pub fn add_ref_entry(&mut self, entry: Arc<RwLock<BibReference>>) {
        self.references.push(entry)
    }

    pub fn add_bib_entry(&mut self, entry: Arc<RwLock<BibEntry>>) {
        let key = entry.read().unwrap().key.clone();
        self.entries.insert(key, entry);
    }

    pub fn combine(&mut self, other: &mut Bibliography) {
        let other_entries = other.entries.clone();
        other.entries = HashMap::new();
        self.entries.extend(other_entries.into_iter());
        self.references.append(&mut other.references);
    }
}
