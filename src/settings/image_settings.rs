/*
 * Snekdown - Custom Markdown flavour and parser
 * Copyright (C) 2021  Trivernis
 * See LICENSE for more information.
 */

use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct ImageSettings {
    pub format: Option<String>,
    pub max_width: Option<u32>,
    pub max_height: Option<u32>,
}

impl Default for ImageSettings {
    fn default() -> Self {
        Self {
            format: None,
            max_height: None,
            max_width: None,
        }
    }
}
