use epub_builder::{EpubBuilder, EpubContent, ReferenceType, Result, ZipLibrary};
use htmlescape::{encode_attribute, encode_minimal};
use std::cmp::max;
use std::collections::HashMap;
use std::io;
use std::io::{Read, Write};
use std::mem;

#[derive(Clone, Debug, PartialOrd, PartialEq)]
struct Buffer {
    pub inner: Vec<u8>,
}

impl Default for Buffer {
    fn default() -> Self {
        Self { inner: Vec::new() }
    }
}

impl Read for Buffer {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        let written = buf.write(&self.inner[self.position..])?;

        self.inner.reverse();
        self.inner.truncate(self.inner.len() - written);
        self.inner.reverse();

        Ok(written)
    }
}

pub struct EpubWriter {
    inner: Box<dyn Write>,
    builder: EpubBuilder<ZipLibrary>,
    content_begin: bool,
    sections: Vec<Box<EpubContent<Buffer>>>,
    content_buffer: Buffer,
    section_level: u8,
    section_count: HashMap<u8, usize>,
}

impl EpubWriter {
    pub fn new(writer: Box<dyn Write>) -> Result<Self> {
        Ok(Self {
            inner: writer,
            builder: EpubBuilder::new(ZipLibrary::new()?)?,
            content_begin: false,
            sections: Vec::new(),
            content_buffer: Buffer::default(),
            section_count: HashMap::new(),
            section_level: 0,
        })
    }

    /// Sets the metadata of the Epub File
    pub fn metadata(&mut self, key: &str, value: &str) -> Result<()> {
        self.builder.metadata(key, value)?;

        Ok(())
    }

    /// Sets the stylesheet of the epub
    pub fn stylesheet(&mut self, style: &String) -> Result<()> {
        self.builder.stylesheet(style.as_bytes())?;

        Ok(())
    }

    /// Adds a resource
    pub fn resource(&mut self, path: &str, resource: Vec<u8>, mimetype: &str) -> Result<()> {
        self.builder
            .add_resource(path, resource.as_slice(), mimetype)?;

        Ok(())
    }

    /// adds a section
    pub fn section(&mut self, level: u8, title: String) -> Result<()> {
        if self.section_level >= level {
            if let Some(mut section) = self.sections.pop() {
                section.content = mem::replace(&mut self.content_buffer, Buffer::default());
                self.builder.add_content(*section)?;
            }
        }
        let mut section = EpubContent::new(
            format!("{}.xhtml", self.next_section_name(level),),
            Buffer::default(),
        )
        .title(title)
        .level(level as i32);
        if !self.content_begin {
            section = section.reftype(ReferenceType::Text);
        }
        self.sections.push(Box::new(section));
        self.section_level = level;

        Ok(())
    }

    /// Adds (string) content to the epub file
    pub fn content(&mut self, content: String) {
        self.content_buffer
            .inner
            .append(&mut content.as_bytes().to_vec());
    }

    pub fn escaped_content(&mut self, content: String) {
        self.content(encode_minimal(content.as_str()));
    }

    pub fn escaped_attribute_content(&mut self, content: String) {
        self.content(encode_attribute(content.as_str()));
    }

    /// Finishes writing the epub
    pub fn finish(&mut self) -> Result<()> {
        self.builder.generate(&mut self.inner)?;

        Ok(())
    }

    /// Returns the next section name for a section level
    fn next_section_name(&mut self, level: u8) -> String {
        let count = *self.section_count.get(&level).unwrap_or(&1);
        self.section_count.insert(level, count);

        format!("s{}_{}", level, count)
    }
}
