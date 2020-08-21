use serde_derive::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct RootConfig {
    pub(crate) bibliography: Option<BibConfig>,
    pub(crate) metadata: Option<MetaConfig>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct BibConfig {
    pub(crate) entry_display: Option<String>,
    pub(crate) reference_display: Option<String>,
    pub(crate) hide_unused: Option<bool>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct MetaConfig {
    pub(crate) author: Option<String>,
    pub(crate) date: Option<String>,
    pub(crate) title: Option<String>,
    pub(crate) language: Option<String>,
}
