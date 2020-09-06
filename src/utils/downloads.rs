use crossbeam_utils::sync::WaitGroup;
use rayon::prelude::*;
use std::fs::read;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};

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
        let pending = Arc::new(Mutex::new(PendingDownload::new(path)));
        self.downloads.push(Arc::clone(&pending));

        pending
    }

    /// Downloads all download entries
    pub fn download_all(&self) {
        self.downloads
            .par_iter()
            .for_each(|d| d.lock().unwrap().download())
    }
}

/// A pending download entry.
/// Download does not necessarily mean that it's not a local file
#[derive(Clone, Debug)]
pub struct PendingDownload {
    path: String,
    pub(crate) data: Option<Vec<u8>>,
    pub(crate) wg: Option<WaitGroup>,
}

impl PendingDownload {
    pub fn new(path: String) -> Self {
        Self {
            path,
            data: None,
            wg: Some(WaitGroup::new()),
        }
    }

    /// Downloads the file and writes the content to the content field
    pub fn download(&mut self) {
        let wg = std::mem::replace(&mut self.wg, None);
        if let Some(wg) = wg {
            log::debug!("Reading {}...", self.path);
            self.data = self.read_content();
            log::debug!("{} read!", self.path);
            drop(wg);
        }
    }

    /// Reads the fiels content or downloads it if it doesn't exist in the filesystem
    fn read_content(&self) -> Option<Vec<u8>> {
        let path = PathBuf::from(&self.path);
        if path.exists() {
            read(path).ok()
        } else {
            self.download_content()
        }
    }

    /// Downloads the content from the given url
    fn download_content(&self) -> Option<Vec<u8>> {
        reqwest::blocking::get(&self.path)
            .ok()
            .map(|c| c.bytes())
            .and_then(|b| b.ok())
            .map(|b| b.to_vec())
    }
}
