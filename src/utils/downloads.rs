/*
 * Snekdown - Custom Markdown flavour and parser
 * Copyright (C) 2021  Trivernis
 * See LICENSE for more information.
 */

use crate::utils::caching::CacheStorage;
use indicatif::{ProgressBar, ProgressStyle};
use parking_lot::Mutex;
use rayon::prelude::*;
use std::fs::read;
use std::path::PathBuf;
use std::sync::Arc;

/// A manager for downloading urls in parallel
#[derive(Clone, Debug)]
pub struct DownloadManager {
    downloads: Vec<Arc<Mutex<PendingDownload>>>,
}

impl DownloadManager {
    /// Creates a new download manager
    pub fn new() -> Self {
        Self {
            downloads: Vec::new(),
        }
    }

    /// Adds a new pending download
    pub fn add_download(&mut self, path: String) -> Arc<Mutex<PendingDownload>> {
        let download = PendingDownload::new(path.clone());
        let pending = Arc::new(Mutex::new(download));
        self.downloads.push(Arc::clone(&pending));
        log::debug!("Added download {}", path);

        pending
    }

    /// Downloads all download entries
    pub fn download_all(&self) {
        let pb = Arc::new(Mutex::new(ProgressBar::new(self.downloads.len() as u64)));
        pb.lock().set_style(
            ProgressStyle::default_bar()
                .template("Fetching Embeds: [{bar:40.cyan/blue}]")
                .progress_chars("=> "),
        );
        let pb_cloned = Arc::clone(&pb);

        self.downloads.par_iter().for_each_with(pb_cloned, |pb, d| {
            d.lock().download();
            pb.lock().inc(1);
        });
        pb.lock().finish_and_clear();
    }
}

/// A pending download entry.
/// Download does not necessarily mean that it's not a local file
#[derive(Clone, Debug)]
pub struct PendingDownload {
    pub(crate) path: String,
    pub(crate) data: Option<Vec<u8>>,
    pub(crate) use_cache: bool,
    cache: CacheStorage,
}

impl PendingDownload {
    pub fn new(path: String) -> Self {
        Self {
            path,
            data: None,
            use_cache: true,
            cache: CacheStorage::new(),
        }
    }

    /// Downloads the file and writes the content to the content field
    pub fn download(&mut self) {
        self.data = self.read_content();
    }

    /// Reads the fiels content or downloads it if it doesn't exist in the filesystem
    fn read_content(&self) -> Option<Vec<u8>> {
        let path = PathBuf::from(&self.path);

        if path.exists() {
            read(path).ok()
        } else if let Some(contents) = self.read_from_cache() {
            log::debug!("Read {} from cache.", self.path.clone());
            Some(contents)
        } else {
            if let Some(data) = self.download_content() {
                self.store_to_cache(&data);
                Some(data)
            } else {
                None
            }
        }
    }

    /// Stores the data to a cache file to retrieve it later
    fn store_to_cache(&self, data: &Vec<u8>) {
        if self.use_cache {
            let path = PathBuf::from(&self.path);
            self.cache
                .write(&path, data.clone())
                .unwrap_or_else(|_| log::warn!("Failed to write file to cache: {}", self.path));
        }
    }

    fn read_from_cache(&self) -> Option<Vec<u8>> {
        let path = PathBuf::from(&self.path);

        if self.cache.has_file(&path) && self.use_cache {
            self.cache.read(&path).ok()
        } else {
            None
        }
    }

    /// Downloads the content from the given url
    fn download_content(&self) -> Option<Vec<u8>> {
        download_path(self.path.clone())
    }
}

pub fn download_path(path: String) -> Option<Vec<u8>> {
    reqwest::blocking::get(&path)
        .ok()
        .map(|c| c.bytes())
        .and_then(|b| b.ok())
        .map(|b| b.to_vec())
}
