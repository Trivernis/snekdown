use regex::Regex;
use std::collections::HashMap;

pub mod html;

pub struct Template {
    value: String,
    replacements: HashMap<String, String>,
}

impl Template {
    pub fn empty() -> Self {
        Self::new(String::new())
    }

    pub fn new(value: String) -> Self {
        Self {
            value,
            replacements: HashMap::new(),
        }
    }

    pub fn set_value(&mut self, value: String) {
        self.value = value;
    }

    pub fn add_replacement(&mut self, name: &str, val: &str) {
        self.replacements.insert(name.to_string(), val.to_string());
    }

    pub fn set_replacements(&mut self, replacements: HashMap<String, String>) {
        self.replacements = replacements;
    }

    pub fn render(&self) -> String {
        lazy_static::lazy_static! { static ref RE_REP: Regex = Regex::new(r"\{\{([^}]*)}}").unwrap(); }
        let mut ret_string = self.value.clone();
        RE_REP.find_iter(&self.value).for_each(|m| {
            let full_match = m.as_str();
            let name = &full_match[2..full_match.len() - 2];
            if let Some(val) = self.replacements.get(name) {
                ret_string = ret_string.replace(full_match, val)
            } else {
                ret_string = ret_string.replace(full_match, "")
            }
        });

        ret_string
    }
}
