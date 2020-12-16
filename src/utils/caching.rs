use platform_dirs::{AppDirs, AppUI};
use std::collections::hash_map::DefaultHasher;
use std::fs;
use std::hash::{Hash, Hasher};
use std::io;
use std::path::PathBuf;

#[derive(Clone, Debug)]
pub struct CacheStorage {
    location: PathBuf,
}

impl CacheStorage {
    pub fn new() -> Self {
        lazy_static::lazy_static! {
            static ref APP_DIRS: AppDirs = AppDirs::new(Some("snekdown"), AppUI::CommandLine).unwrap();
        }

        Self {
            location: APP_DIRS.cache_dir.clone(),
        }
    }

    /// Returns the cache path for a given file
    pub fn get_file_path(&self, path: &PathBuf) -> PathBuf {
        let mut hasher = DefaultHasher::new();
        path.hash(&mut hasher);
        let mut file_name = PathBuf::from(format!("{:x}", hasher.finish()));

        if let Some(extension) = path.extension() {
            file_name.set_extension(extension);
        }

        return self.location.join(PathBuf::from(file_name));
    }

    /// Returns if the given file exists in the cache
    pub fn has_file(&self, path: &PathBuf) -> bool {
        let cache_path = self.get_file_path(path);

        cache_path.exists()
    }

    /// Writes into the corresponding cache file
    pub fn read(&self, path: &PathBuf) -> io::Result<Vec<u8>> {
        let cache_path = self.get_file_path(path);

        fs::read(cache_path)
    }

    /// Reads the corresponding cache file
    pub fn write<R: AsRef<[u8]>>(&self, path: &PathBuf, contents: R) -> io::Result<()> {
        let cache_path = self.get_file_path(path);

        fs::write(cache_path, contents)
    }

    /// Clears the cache directory by deleting and recreating it
    pub fn clear(&self) -> io::Result<()> {
        fs::remove_dir_all(&self.location)?;
        fs::create_dir(&self.location)
    }
}
