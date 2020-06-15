use crate::elements::MetadataValue;
use crate::references::configuration::config::RootConfig;
use crate::references::configuration::keys::{
    BIB_DISPLAY, BIB_HIDE_UNUSED, BIB_REF_DISPLAY, META_AUTHOR, META_DATE, META_TITLE,
};
use crate::references::templates::Template;
use std::collections::HashMap;
use std::sync::{Arc, RwLock};

pub mod config;
pub(crate) mod keys;

#[derive(Clone, Debug)]
pub enum Value {
    String(String),
    Bool(bool),
    Float(f64),
    Integer(i64),
    Template(Template),
}

#[derive(Clone, Debug)]
pub struct ConfigEntry {
    inner: Value,
}

pub type ConfigRefEntry = Arc<RwLock<ConfigEntry>>;

#[derive(Clone, Debug)]
pub struct Configuration {
    config: Arc<RwLock<HashMap<String, ConfigRefEntry>>>,
}

impl Value {
    pub fn as_string(&self) -> String {
        match self {
            Value::String(string) => string.clone(),
            Value::Integer(int) => format!("{}", int),
            Value::Float(f) => format!("{:02}", f),
            Value::Bool(b) => format!("{}", b),
            _ => "".to_string(),
        }
    }
}

impl ConfigEntry {
    pub fn new(value: Value) -> Self {
        Self { inner: value }
    }

    pub fn set(&mut self, value: Value) {
        self.inner = value;
    }

    pub fn get(&self) -> &Value {
        &self.inner
    }
}

impl Configuration {
    pub fn new() -> Self {
        Self {
            config: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    pub fn default() -> Self {
        let mut self_config = Self::new();
        lazy_static::lazy_static! { static ref CONFIG: RootConfig = toml::from_str(std::include_str!("default.toml")).unwrap();}
        self_config.assign_config(&CONFIG);

        self_config
    }

    pub fn assign_config(&mut self, config: &RootConfig) {
        if let Some(bib) = &config.bibliography {
            if let Some(cfg) = &bib.entry_display {
                self.set(BIB_DISPLAY, Value::String(cfg.clone()))
            }
            if let Some(cfg) = &bib.reference_display {
                self.set(BIB_REF_DISPLAY, Value::String(cfg.clone()))
            }
            if let Some(cfg) = &bib.hide_unused {
                self.set(BIB_HIDE_UNUSED, Value::Bool(*cfg));
            }
        }
        if let Some(meta) = &config.metadata {
            if let Some(cfg) = &meta.author {
                self.set(META_AUTHOR, Value::String(cfg.clone()))
            }
            if let Some(cfg) = &meta.date {
                self.set(META_DATE, Value::String(cfg.clone()))
            }
            if let Some(cfg) = &meta.title {
                self.set(META_TITLE, Value::String(cfg.clone()))
            }
        }
    }

    /// returns the value of a config entry
    pub fn get_entry(&self, key: &str) -> Option<ConfigEntry> {
        let config = self.config.read().unwrap();
        if let Some(entry) = config.get(key) {
            let value = entry.read().unwrap();
            Some(value.clone())
        } else {
            None
        }
    }

    /// returns a config entry that is a reference to a value
    pub fn get_ref_entry(&self, key: &str) -> Option<ConfigRefEntry> {
        let config = self.config.read().unwrap();
        if let Some(entry) = config.get(&key.to_string()) {
            Some(Arc::clone(entry))
        } else {
            None
        }
    }

    /// Sets a config parameter
    pub fn set(&mut self, key: &str, value: Value) {
        let mut config = self.config.write().unwrap();
        if let Some(entry) = config.get(&key.to_string()) {
            entry.write().unwrap().set(value)
        } else {
            config.insert(
                key.to_string(),
                Arc::new(RwLock::new(ConfigEntry::new(value))),
            );
        }
    }

    /// Sets a config value based on a metadata value
    pub fn set_from_meta(&mut self, key: &str, value: MetadataValue) {
        match value {
            MetadataValue::String(string) => self.set(key, Value::String(string)),
            MetadataValue::Bool(bool) => self.set(key, Value::Bool(bool)),
            MetadataValue::Float(f) => self.set(key, Value::Float(f)),
            MetadataValue::Integer(i) => self.set(key, Value::Integer(i)),
            MetadataValue::Template(t) => self.set(key, Value::Template(t)),
            _ => {}
        }
    }
}
