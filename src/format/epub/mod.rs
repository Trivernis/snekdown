use crate::format::epub::epub_writer::EpubWriter;
use std::io;

pub mod epub_writer;

pub trait ToEpub {
    fn to_epub(&self, writer: &EpubWriter) -> io::Result<()>;
}
