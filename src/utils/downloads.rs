use indicatif::{MultiProgress, ProgressBar, ProgressStyle};
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
        let pending = Arc::new(Mutex::new(PendingDownload::new(path.clone())));
        self.downloads.push(Arc::clone(&pending));
        log::debug!("Added download {}", path);

        pending
    }

    /// Downloads all download entries
    pub fn download_all(&self) {
        let pb = Arc::new(Mutex::new(ProgressBar::new(self.downloads.len() as u64)));
        pb.lock().unwrap().set_style(
            ProgressStyle::default_bar()
                .template("Reading Imports: [{bar:40.cyan/blue}]")
                .progress_chars("=> "),
        );
        let pb_cloned = Arc::clone(&pb);

        self.downloads.par_iter().for_each_with(pb_cloned, |pb, d| {
            d.lock().unwrap().download();
            pb.lock().unwrap().inc(1);
        });
        pb.lock()
            .unwrap()
            .finish_with_message("All downloads finished!");
    }
}

/// A pending download entry.
/// Download does not necessarily mean that it's not a local file
#[derive(Clone, Debug)]
pub struct PendingDownload {
    pub(crate) path: String,
    pub(crate) data: Option<Vec<u8>>,
}

impl PendingDownload {
    pub fn new(path: String) -> Self {
        Self { path, data: None }
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
