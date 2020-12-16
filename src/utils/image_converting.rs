use crate::elements::Metadata;
use crate::utils::caching::CacheStorage;
use crate::utils::downloads::download_path;
use image::imageops::FilterType;
use image::io::Reader as ImageReader;
use image::{GenericImageView, ImageFormat, ImageResult};
use indicatif::{ProgressBar, ProgressStyle};
use mime::Mime;
use parking_lot::Mutex;
use rayon::prelude::*;
use std::io;
use std::io::Cursor;
use std::path::PathBuf;
use std::sync::Arc;

#[derive(Clone, Debug)]
pub struct ImageConverter {
    images: Vec<Arc<Mutex<PendingImage>>>,
    target_format: Option<ImageFormat>,
    target_size: Option<(u32, u32)>,
}

impl ImageConverter {
    pub fn new() -> Self {
        Self {
            images: Vec::new(),
            target_format: None,
            target_size: None,
        }
    }

    pub fn set_target_size(&mut self, target_size: (u32, u32)) {
        self.target_size = Some(target_size)
    }

    pub fn set_target_format(&mut self, target_format: ImageFormat) {
        self.target_format = Some(target_format);
    }

    /// Adds an image to convert
    pub fn add_image(&mut self, path: PathBuf) -> Arc<Mutex<PendingImage>> {
        let image = Arc::new(Mutex::new(PendingImage::new(path)));
        self.images.push(image.clone());

        image
    }

    /// Converts all images
    pub fn convert_all(&mut self) {
        let pb = Arc::new(Mutex::new(ProgressBar::new(self.images.len() as u64)));
        pb.lock().set_style(
            ProgressStyle::default_bar()
                .template("Processing images: [{bar:40.cyan/blue}]")
                .progress_chars("=> "),
        );
        self.images.par_iter().for_each(|image| {
            let mut image = image.lock();
            if let Err(e) = image.convert(self.target_format.clone(), self.target_size.clone()) {
                log::error!("Failed to embed image {:?}: {}", image.path, e)
            }
            pb.lock().tick();
        });
        pb.lock().finish();
    }
}

#[derive(Clone, Debug)]
pub struct PendingImage {
    pub path: PathBuf,
    pub data: Option<Vec<u8>>,
    cache: CacheStorage,
    pub mime: Mime,
    brightness: Option<i32>,
    contrast: Option<f32>,
    grayscale: bool,
    invert: bool,
}

impl PendingImage {
    pub fn new(path: PathBuf) -> Self {
        let mime = get_mime(&path);

        Self {
            path,
            data: None,
            cache: CacheStorage::new(),
            mime,
            brightness: None,
            contrast: None,
            grayscale: false,
            invert: false,
        }
    }

    pub fn assign_from_meta<M: Metadata>(&mut self, meta: &M) {
        if let Some(brightness) = meta.get_integer("brightness") {
            self.brightness = Some(brightness as i32);
        }
        if let Some(contrast) = meta.get_float("contrast") {
            self.contrast = Some(contrast as f32);
        }
        self.grayscale = meta.get_bool("grayscale");
        self.invert = meta.get_bool("invert");
    }

    /// Converts the image to the specified target format (specified by target_extension)
    pub fn convert(
        &mut self,
        target_format: Option<ImageFormat>,
        target_size: Option<(u32, u32)>,
    ) -> ImageResult<()> {
        let format = target_format
            .or_else(|| {
                self.path
                    .extension()
                    .and_then(|extension| ImageFormat::from_extension(extension))
            })
            .unwrap_or(ImageFormat::Png);

        let output_path = self.get_output_path(format, target_size);
        self.mime = get_mime(&output_path);

        if self.cache.has_file(&output_path) {
            self.data = Some(self.cache.read(&output_path)?)
        } else {
            self.convert_image(format, target_size)?;

            if let Some(data) = &self.data {
                self.cache.write(&output_path, data)?;
            }
        }

        Ok(())
    }

    /// Converts the image
    fn convert_image(
        &mut self,
        format: ImageFormat,
        target_size: Option<(u32, u32)>,
    ) -> ImageResult<()> {
        let mut image = ImageReader::open(self.get_path()?)?.decode()?;

        if let Some((width, height)) = target_size {
            let dimensions = image.dimensions();

            if dimensions.0 > width || dimensions.1 > height {
                image = image.resize(width, height, FilterType::Lanczos3);
            }
        }

        if let Some(brightness) = self.brightness {
            image = image.brighten(brightness);
        }

        if let Some(contrast) = self.contrast {
            image = image.adjust_contrast(contrast);
        }
        if self.grayscale {
            image = image.grayscale();
        }
        if self.invert {
            image.invert();
        }

        let data = Vec::new();
        let mut writer = Cursor::new(data);

        image.write_to(&mut writer, format)?;
        self.data = Some(writer.into_inner());

        Ok(())
    }

    /// Returns the path of the file
    fn get_path(&self) -> io::Result<PathBuf> {
        if !self.path.exists() {
            if self.cache.has_file(&self.path) {
                return Ok(self.cache.get_file_path(&self.path));
            }
            if let Some(data) = download_path(self.path.to_string_lossy().to_string()) {
                self.cache.write(&self.path, data)?;
                return Ok(self.cache.get_file_path(&self.path));
            }
        }
        Ok(self.path.clone())
    }

    /// Returns the output file name after converting the image
    fn get_output_path(
        &self,
        target_format: ImageFormat,
        target_size: Option<(u32, u32)>,
    ) -> PathBuf {
        let mut path = self.path.clone();
        let mut file_name = path.file_stem().unwrap().to_string_lossy().to_string();
        let extension = target_format.extensions_str()[0];
        let type_name = format!("{:?}", target_format);

        if let Some(target_size) = target_size {
            file_name += &*format!("-{}-{}", target_size.0, target_size.1);
        }
        if let Some(b) = self.brightness {
            file_name += &*format!("-{}", b);
        }
        if let Some(c) = self.contrast {
            file_name += &*format!("-{}", c);
        }
        file_name += &*format!("{}-{}", self.invert, self.grayscale);

        file_name += format!("-{}", type_name).as_str();
        path.set_file_name(file_name);
        path.set_extension(extension);

        path
    }
}

fn get_mime(path: &PathBuf) -> Mime {
    let mime = mime_guess::from_ext(
        path.clone()
            .extension()
            .and_then(|e| e.to_str())
            .unwrap_or("png"),
    )
    .first()
    .unwrap_or(mime::IMAGE_PNG);
    mime
}
