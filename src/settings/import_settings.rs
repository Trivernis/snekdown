use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct ImportSettings {
    pub ignored_imports: Vec<String>,
    pub included_stylesheets: Vec<String>,
    pub included_bibliography: Vec<String>,
    pub included_glossaries: Vec<String>,
}

impl Default for ImportSettings {
    fn default() -> Self {
        Self {
            ignored_imports: Vec::with_capacity(0),
            included_stylesheets: vec!["style.css".to_string()],
            included_bibliography: vec!["Bibliography.toml".to_string()],
            included_glossaries: vec!["Glossary.toml".to_string()],
        }
    }
}
