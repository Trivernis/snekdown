/*
 * Snekdown - Custom Markdown flavour and parser
 * Copyright (C) 2021  Trivernis
 * See LICENSE for more information.
 */

use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct StyleSettings {
    pub bib_ref_display: String,
    pub theme: Theme,
}

impl Default for StyleSettings {
    fn default() -> Self {
        Self {
            bib_ref_display: "{{number}}".to_string(),
            theme: Theme::GitHub,
        }
    }
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub enum Theme {
    GitHub,
    SolarizedDark,
    SolarizedLight,
    OceanDark,
    OceanLight,
    MagicDark,
}
