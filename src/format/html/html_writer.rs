use std::io;
use std::io::Write;

pub struct HTMLWriter {
    inner: Box<dyn Write>,
}

impl HTMLWriter {
    /// Creates a new writer
    pub fn new(inner: Box<dyn Write>) -> Self {
        Self { inner }
    }

    /// Writes a raw string
    pub fn write(&mut self, html: String) -> io::Result<()> {
        self.inner.write_all(html.as_bytes())
    }

    /// Writes an escaped string
    pub fn write_escaped(&mut self, html: String) -> io::Result<()> {
        self.write(htmlescape::encode_minimal(html.as_str()))
    }

    /// Writes an escaped attribute
    pub fn write_attribute(&mut self, attribute_value: String) -> io::Result<()> {
        self.write(htmlescape::encode_attribute(attribute_value.as_str()))
    }
}
