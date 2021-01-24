/*
 * Snekdown - Custom Markdown flavour and parser
 * Copyright (C) 2021  Trivernis
 * See LICENSE for more information.
 */

use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct PDFSettings {
    pub display_header_footer: bool,
    pub header_template: Option<String>,
    pub footer_template: Option<String>,
    pub page_height: Option<f32>,
    pub page_width: Option<f32>,
    pub page_scale: f32,
    pub margin: PDFMarginSettings,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct PDFMarginSettings {
    pub top: Option<f32>,
    pub bottom: Option<f32>,
    pub left: Option<f32>,
    pub right: Option<f32>,
}

impl Default for PDFMarginSettings {
    fn default() -> Self {
        Self {
            top: Some(0.5),
            bottom: Some(0.5),
            left: None,
            right: None,
        }
    }
}

impl Default for PDFSettings {
    fn default() -> Self {
        Self {
            display_header_footer: true,
            header_template: Some("<div></div>".to_string()),
            footer_template: Some(
                include_str!("../format/chromium_pdf/assets/default-footer-template.html")
                    .to_string(),
            ),
            page_height: None,
            page_width: None,
            page_scale: 1.0,
            margin: Default::default(),
        }
    }
}
