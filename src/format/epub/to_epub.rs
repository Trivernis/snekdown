use crate::elements::{
    Block, Document, Element, Header, Inline, Line, List, ListItem, Paragraph, Section,
};
use crate::format::epub::epub_writer::EpubWriter;
use std::io::Result;
use std::mem;

pub trait ToEpub {
    fn to_epub(&self, writer: &mut EpubWriter) -> Result<()>;
}

impl ToEpub for Element {
    fn to_epub(&self, writer: &mut EpubWriter) -> Result<()> {
        match self {
            Element::Inline(inline) => inline.to_epub(writer),
            Element::Block(block) => block.to_epub(writer),
            Element::Line(line) => line.to_epub(writer),
        }
    }
}

impl ToEpub for Block {
    fn to_epub(&self, writer: &mut EpubWriter) -> Result<()> {
        match self {
            Block::Null => Ok(()),
            Block::Section(s) => s.to_epub(writer),
            Block::List(l) => l.to_epub(writer),
            Block::Paragraph(p) => p.to_epub(writer),
        }
    }
}

impl ToEpub for Line {
    fn to_epub(&self, writer: &mut EpubWriter) -> Result<()> {
        unimplemented!()
    }
}

impl ToEpub for Inline {
    fn to_epub(&self, writer: &mut EpubWriter) -> Result<()> {
        unimplemented!()
    }
}

impl ToEpub for Document {
    fn to_epub(&self, writer: &mut EpubWriter) -> Result<()> {
        self.downloads.lock().unwrap().download_all();

        let style = minify(std::include_str!("assets/style.css"));
        writer.stylesheet(style)?;

        for mut stylesheet in self.stylesheets {
            let mut sheet = stylesheet.lock().unwrap();
            let data = mem::take(&mut sheet.data);

            if let Some(data) = data {
                let sheet_data = String::from_utf8(data)?;
                writer.stylesheet(&sheet_data);
            }
        }

        Ok(())
    }
}

impl ToEpub for Section {
    fn to_epub(&self, writer: &mut EpubWriter) -> Result<()> {
        writer.section(self.header.size, self.header.plain.clone())?;
        writer.content("<!DOCTYPE html>".to_string());
        writer.content("<html xmlns=\"http://www.w3.org/1999/xhtml\"><body>".to_string());
        self.header.to_epub(writer)?;
        for element in &self.elements {
            element.to_epub(writer)?;
        }
        writer.content("</body></html>".to_string());

        Ok(())
    }
}

impl ToEpub for Header {
    fn to_epub(&self, writer: &mut EpubWriter) -> Result<()> {
        writer.content(format!("<h{} id=\"", self.size));
        writer.escaped_attribute_content(self.anchor.clone());
        writer.content("\">".to_string());
        self.line.to_epub(writer);
        writer.content("</h1>".to_string());

        Ok(())
    }
}

impl ToEpub for List {
    fn to_epub(&self, writer: &mut EpubWriter) -> Result<()> {
        if self.ordered {
            writer.content("<ol>".to_string());
            for item in self.items {
                item.to_epub(writer);
            }
            writer.content("</ol>".to_string());
        } else {
            writer.content("<ul>".to_string());
            for item in self.items {
                item.to_epub(writer);
            }
            writer.content("</ul>".to_string());
        }

        Ok(())
    }
}

impl ToEpub for ListItem {
    fn to_epub(&self, writer: &mut EpubWriter) -> Result<()> {
        writer.content("<li>".to_string());
        self.text.to_epub(writer)?;

        if let Some(first) = self.children.first() {
            if first.ordered {
                writer.content("<ol>".to_string());
                for item in &self.children {
                    item.to_epub(writer)?;
                }
                writer.content("</ol>".to_string());
            } else {
                writer.content("<ul>".to_string());
                for item in &self.children {
                    item.to_epub(writer)?;
                }
                writer.content("</ul>".to_string());
            }
        }
        writer.content("</li>".to_string());

        Ok(())
    }
}

impl ToEpub for Paragraph {
    fn to_epub(&self, writer: &mut EpubWriter) -> Result<()> {
        writer.content("<div class=\"paragraph\"".to_string());
        if let Some(first) = self.elements.first() {
            first.to_epub(writer)?;
        }
        if self.elements.len() > 1 {
            for element in &self.elements[1..] {
                writer.content("<br/>".to_string());
                element.to_epub(writer)?;
            }
        }

        Ok(())
    }
}
