use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct FeatureSettings {
    pub embed_external: bool,
    pub smart_arrows: bool,
    pub include_mathjax: bool,
}

impl Default for FeatureSettings {
    fn default() -> Self {
        Self {
            embed_external: true,
            smart_arrows: true,
            include_mathjax: true,
        }
    }
}
