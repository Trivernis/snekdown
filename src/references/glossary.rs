use crate::elements::{
    Anchor, BoldText, Inline, ItalicText, Line, List, ListItem, PlainText, TextLine,
};
use parking_lot::Mutex;
use std::cmp::Ordering;
use std::collections::HashMap;
use std::sync::Arc;

use crate::bold_text;
use crate::italic_text;
use crate::plain_text;

const K_LONG: &str = "long";
const K_DESCRIPTION: &str = "description";

/// A glossary manager responsible for handling glossary entries and references to those entries
#[derive(Clone, Debug)]
pub struct GlossaryManager {
    entries: HashMap<String, Arc<Mutex<GlossaryEntry>>>,
    references: Vec<Arc<Mutex<GlossaryReference>>>,
}

/// A single glossary entry
#[derive(Clone, Debug)]
pub struct GlossaryEntry {
    pub short: String,
    pub long: String,
    pub description: String,
    pub is_assigned: bool,
}

/// A single glossary reference
#[derive(Clone, Debug)]
pub struct GlossaryReference {
    pub short: String,
    pub display: GlossaryDisplay,
    pub entry: Option<Arc<Mutex<GlossaryEntry>>>,
}

/// A glossary display value that determines which value
/// of a glossary entry will be rendered
#[derive(Clone, Debug)]
pub enum GlossaryDisplay {
    Short,
    Long,
}

impl GlossaryManager {
    /// Creates a new glossary manager
    pub fn new() -> Self {
        Self {
            entries: HashMap::new(),
            references: Vec::new(),
        }
    }

    /// Adds a new glossary entry to the manager
    pub fn add_entry(&mut self, entry: GlossaryEntry) -> Arc<Mutex<GlossaryEntry>> {
        let key = entry.short.clone();
        let entry = Arc::new(Mutex::new(entry));
        self.entries.insert(key.clone(), Arc::clone(&entry));
        log::debug!("Added glossary entry {}", key);

        entry
    }

    /// Adds a new glossary reference to the manager
    pub fn add_reference(&mut self, reference: GlossaryReference) -> Arc<Mutex<GlossaryReference>> {
        let reference = Arc::new(Mutex::new(reference));
        self.references.push(Arc::clone(&reference));

        reference
    }

    /// Assignes bibliography entries from toml
    pub fn assign_from_toml(&mut self, value: toml::Value) -> Result<(), String> {
        let table = value.as_table().ok_or("Failed to parse toml".to_string())?;

        log::debug!("Assigning glossary entries from toml...");
        for (key, value) in table {
            let long = value.get(K_LONG).and_then(|l| l.as_str());
            let description = value.get(K_DESCRIPTION).and_then(|d| d.as_str());
            if let Some(long) = long {
                if let Some(description) = description {
                    let entry = GlossaryEntry {
                        description: description.to_string(),
                        long: long.to_string(),
                        short: key.clone(),
                        is_assigned: false,
                    };
                    self.add_entry(entry);
                } else {
                    log::warn!(
                        "Failed to parse glossary entry {}: Missing field '{}'",
                        key,
                        K_DESCRIPTION
                    );
                }
            } else {
                log::warn!(
                    "Failed to parse glossary entry {}: Missing field '{}'",
                    key,
                    K_LONG
                );
            }
        }

        Ok(())
    }

    /// Assignes entries to references
    pub fn assign_entries_to_references(&self) {
        for reference in &self.references {
            let mut reference = reference.lock();

            if let Some(entry) = self.entries.get(&reference.short) {
                reference.entry = Some(Arc::clone(entry));
                let mut entry = entry.lock();

                if !entry.is_assigned {
                    entry.is_assigned = true;
                    reference.display = GlossaryDisplay::Long;
                }
            }
        }
    }

    /// Creates a sorted glossary list from the glossary entries
    pub fn create_glossary_list(&self) -> List {
        let mut list = List::new();
        let mut entries = self
            .entries
            .values()
            .filter(|e| e.lock().is_assigned)
            .cloned()
            .collect::<Vec<Arc<Mutex<GlossaryEntry>>>>();

        entries.sort_by(|a, b| {
            let a = a.lock();
            let b = b.lock();
            if a.short > b.short {
                Ordering::Greater
            } else if a.short < b.short {
                Ordering::Less
            } else {
                Ordering::Equal
            }
        });
        for entry in &entries {
            let entry = entry.lock();
            let mut line = TextLine::new();
            line.subtext.push(bold_text!(entry.short.clone()));
            line.subtext.push(plain_text!(" - ".to_string()));
            line.subtext.push(italic_text!(entry.long.clone()));
            line.subtext.push(plain_text!(" - ".to_string()));
            line.subtext.push(plain_text!(entry.description.clone()));
            list.add_item(ListItem::new(
                Line::Anchor(Anchor {
                    inner: Box::new(Line::Text(line)),
                    key: entry.short.clone(),
                }),
                0,
                false,
            ));
        }

        list
    }
}

impl GlossaryReference {
    /// Creates a new glossary reference
    pub fn new(key: String) -> Self {
        Self {
            short: key,
            display: GlossaryDisplay::Short,
            entry: None,
        }
    }

    /// Creates a new glossary reference with a given display parameter
    pub fn with_display(key: String, display: GlossaryDisplay) -> Self {
        Self {
            short: key,
            display,
            entry: None,
        }
    }
}
