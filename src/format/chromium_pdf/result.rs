/*
 * Snekdown - Custom Markdown flavour and parser
 * Copyright (C) 2021  Trivernis
 * See LICENSE for more information.
 */

use std::error::Error;
use std::fmt::{self, Display};
use std::io;

pub type PdfRenderingResult<T> = Result<T, PdfRenderingError>;

#[derive(Debug)]
pub enum PdfRenderingError {
    IoError(io::Error),
    ChromiumError(failure::Error),
    Timeout,
    HtmlRenderingError,
}

impl Display for PdfRenderingError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            PdfRenderingError::IoError(e) => write!(f, "IO Error: {}", e),
            PdfRenderingError::Timeout => write!(f, "Rendering timed out"),
            PdfRenderingError::ChromiumError(e) => write!(f, "Chromium Error: {}", e),
            PdfRenderingError::HtmlRenderingError => write!(f, "Failed to render html"),
        }
    }
}

impl Error for PdfRenderingError {}

impl From<failure::Error> for PdfRenderingError {
    fn from(other: failure::Error) -> Self {
        Self::ChromiumError(other)
    }
}

impl From<io::Error> for PdfRenderingError {
    fn from(other: io::Error) -> Self {
        Self::IoError(other)
    }
}
