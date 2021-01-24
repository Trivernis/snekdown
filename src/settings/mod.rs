/*
 * Snekdown - Custom Markdown flavour and parser
 * Copyright (C) 2021  Trivernis
 * See LICENSE for more information.
 */

use crate::elements::{Metadata, MetadataValue};
use crate::settings::feature_settings::FeatureSettings;
use crate::settings::image_settings::ImageSettings;
use crate::settings::import_settings::ImportSettings;
use crate::settings::metadata_settings::MetadataSettings;
use crate::settings::pdf_settings::PDFSettings;
use crate::settings::style_settings::StyleSettings;
use config::{ConfigError, Source};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::error::Error;
use std::fmt::{self, Display};
use std::io;
use std::mem;
use std::path::PathBuf;

pub mod feature_settings;
pub mod image_settings;
pub mod import_settings;
pub mod metadata_settings;
pub mod pdf_settings;
pub mod style_settings;

pub type SettingsResult<T> = Result<T, SettingsError>;

#[derive(Debug)]
pub enum SettingsError {
    IoError(io::Error),
    ConfigError(ConfigError),
    TomlError(toml::ser::Error),
}

impl Display for SettingsError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::IoError(e) => write!(f, "IO Error: {}", e),
            Self::ConfigError(e) => write!(f, "Config Error: {}", e),
            Self::TomlError(e) => write!(f, "Toml Error: {}", e),
        }
    }
}

impl Error for SettingsError {}

impl From<io::Error> for SettingsError {
    fn from(e: io::Error) -> Self {
        Self::IoError(e)
    }
}

impl From<ConfigError> for SettingsError {
    fn from(e: ConfigError) -> Self {
        Self::ConfigError(e)
    }
}

impl From<toml::ser::Error> for SettingsError {
    fn from(e: toml::ser::Error) -> Self {
        Self::TomlError(e)
    }
}

#[derive(Serialize, Deserialize, Clone, Debug, Default)]
pub struct Settings {
    pub metadata: MetadataSettings,
    pub features: FeatureSettings,
    pub imports: ImportSettings,
    pub pdf: PDFSettings,
    pub images: ImageSettings,
    pub style: StyleSettings,
    pub custom_attributes: HashMap<String, String>,
}

impl Source for Settings {
    fn clone_into_box(&self) -> Box<dyn Source + Send + Sync> {
        Box::new(self.clone())
    }

    fn collect(&self) -> Result<HashMap<String, config::Value>, config::ConfigError> {
        let source_str =
            toml::to_string(&self).map_err(|e| config::ConfigError::Foreign(Box::new(e)))?;
        let result = toml::de::from_str(&source_str)
            .map_err(|e| config::ConfigError::Foreign(Box::new(e)))?;

        Ok(result)
    }
}

impl Settings {
    /// Loads the settings from the specified path
    pub fn load(path: PathBuf) -> SettingsResult<Self> {
        let mut settings = config::Config::default();
        settings
            .merge(Self::default())?
            .merge(config::File::from(path))?;
        let settings: Self = settings.try_into()?;

        Ok(settings)
    }

    /// Merges the current settings with the settings from the given path
    /// returning updated settings
    pub fn merge(&mut self, path: PathBuf) -> SettingsResult<()> {
        let mut settings = config::Config::default();
        settings
            .merge(self.clone())?
            .merge(config::File::from(path))?;
        let mut settings: Self = settings.try_into()?;
        mem::swap(self, &mut settings); // replace the old settings with the new ones

        Ok(())
    }

    pub fn append_metadata<M: Metadata>(&mut self, metadata: M) {
        let entries = metadata.get_string_map();
        for (key, value) in entries {
            self.custom_attributes.insert(key, value);
        }
    }

    pub fn set_from_meta(&mut self, key: &str, value: MetadataValue) {
        self.custom_attributes
            .insert(key.to_string(), value.to_string());
    }
}
