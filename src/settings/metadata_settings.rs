use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct MetadataSettings {
    pub title: Option<String>,
    pub author: Option<String>,
    pub language: String,
}

impl Default for MetadataSettings {
    fn default() -> Self {
        Self {
            title: None,
            author: None,
            language: "en".to_string(),
        }
    }
}
