use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct FormatSettings {
    pub bib_ref_display: String,
}

impl Default for FormatSettings {
    fn default() -> Self {
        Self {
            bib_ref_display: "{{number}}".to_string(),
        }
    }
}
