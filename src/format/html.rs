use crate::parsing::elements::*;
use htmlescape::{encode_attribute, encode_minimal};
use minify::html::minify;
use std::cell::RefCell;
use syntect::highlighting::ThemeSet;
use syntect::html::highlighted_html_for_string;
use syntect::parsing::SyntaxSet;

macro_rules! combine_with_lb {
    ($a:expr, $b:expr) => {
        if $a.len() > 0 {
            format!("{}<br>{}", $a, $b.to_html())
        } else {
            $b.to_html()
        }
    };
}

pub trait ToHtml {
    fn to_html(&self) -> String;
}

impl ToHtml for Element {
    fn to_html(&self) -> String {
        match self {
            Element::Block(block) => block.to_html(),
            Element::Inline(inline) => inline.to_html(),
            Element::SubText(sub) => sub.to_html(),
        }
    }
}

impl ToHtml for Inline {
    fn to_html(&self) -> String {
        match self {
            Inline::Text(text) => text.to_html(),
            Inline::Ruler(ruler) => ruler.to_html(),
            Inline::Anchor(anchor) => anchor.to_html(),
        }
    }
}

impl ToHtml for SubText {
    fn to_html(&self) -> String {
        match self {
            SubText::Url(url) => url.to_html(),
            SubText::Monospace(mono) => mono.to_html(),
            SubText::Striked(striked) => striked.to_html(),
            SubText::Plain(plain) => plain.to_html(),
            SubText::Italic(italic) => italic.to_html(),
            SubText::Underlined(under) => under.to_html(),
            SubText::Bold(bold) => bold.to_html(),
            SubText::Image(img) => img.to_html(),
            SubText::Placeholder(placeholder) => placeholder.lock().unwrap().to_html(),
        }
    }
}

impl ToHtml for Block {
    fn to_html(&self) -> String {
        match self {
            Block::Paragraph(para) => para.to_html(),
            Block::List(list) => list.to_html(),
            Block::Table(table) => table.to_html(),
            Block::CodeBlock(code) => code.to_html(),
            Block::Quote(quote) => quote.to_html(),
            Block::Section(section) => section.to_html(),
            Block::Import(import) => import.to_html(),
            Block::Placeholder(placeholder) => placeholder.lock().unwrap().to_html(),
        }
    }
}

impl ToHtml for Document {
    fn to_html(&self) -> String {
        let inner = self
            .elements
            .iter()
            .fold("".to_string(), |a, b| format!("{}{}", a, b.to_html()));
        let path = if let Some(path) = &self.path {
            format!("path='{}'", encode_attribute(path.as_str()))
        } else {
            "".to_string()
        };
        if self.is_root {
            let style = minify(std::include_str!("assets/style.css"));
            format!(
                "<!DOCTYPE html>\n<html><head {}><style>{}</style></head><body><div class='content'>{}</div></body></html>",
                path, style, inner
            )
        } else {
            format!(
                "<div class='documentImport' document-import=true {}>{}</div>",
                path, inner
            )
        }
    }
}

impl ToHtml for Import {
    fn to_html(&self) -> String {
        let anchor = self.anchor.lock().unwrap();
        if let Some(document) = &anchor.document {
            document.to_html()
        } else {
            "".to_string()
        }
    }
}

impl ToHtml for Section {
    fn to_html(&self) -> String {
        let inner = self
            .elements
            .iter()
            .fold("".to_string(), |a, b| format!("{}{}", a, b.to_html()));
        format!("<section>{}{}</section>", self.header.to_html(), inner)
    }
}

impl ToHtml for Header {
    fn to_html(&self) -> String {
        format!(
            "<h{0} id='{1}'>{2}</h{0}>",
            self.size,
            encode_attribute(self.anchor.as_str()),
            self.line.to_html()
        )
    }
}

impl ToHtml for Paragraph {
    fn to_html(&self) -> String {
        let inner = self
            .elements
            .iter()
            .fold("".to_string(), |a, b| combine_with_lb!(a, b));
        format!("<p>{}</p>", inner)
    }
}

impl ToHtml for List {
    fn to_html(&self) -> String {
        let inner = self
            .items
            .iter()
            .fold("".to_string(), |a, b| format!("{}{}", a, b.to_html()));
        if self.ordered {
            format!("<ol>{}</ol>", inner)
        } else {
            format!("<ul>{}</ul>", inner)
        }
    }
}

impl ToHtml for ListItem {
    fn to_html(&self) -> String {
        let inner = self
            .children
            .iter()
            .fold("".to_string(), |a, b| format!("{}{}", a, b.to_html()));
        if let Some(first) = self.children.first() {
            if first.ordered {
                format!("<li>{}<ol>{}</ol></li>", self.text.to_html(), inner)
            } else {
                format!("<li>{}<ul>{}</ul></li>", self.text.to_html(), inner)
            }
        } else {
            format!("<li>{}</li>", self.text.to_html())
        }
    }
}

impl ToHtml for Table {
    fn to_html(&self) -> String {
        let head = self.header.cells.iter().fold("".to_string(), |a, b| {
            format!("{}<th>{}</th>", a, b.text.to_html())
        });
        let body = self
            .rows
            .iter()
            .fold("".to_string(), |a, b| format!("{}{}", a, b.to_html()));
        format!(
            "<div class='tableWrapper'><table><tr>{}<tr>{}</table></div>",
            head, body
        )
    }
}

impl ToHtml for Row {
    fn to_html(&self) -> String {
        let inner = self
            .cells
            .iter()
            .fold("".to_string(), |a, b| format!("{}{}", a, b.to_html()));
        format!("<tr>{}</tr>", inner)
    }
}

impl ToHtml for Cell {
    fn to_html(&self) -> String {
        format!("<td>{}</td>", self.text.to_html())
    }
}

thread_local! {static PS: RefCell<SyntaxSet> = RefCell::new(SyntaxSet::load_defaults_nonewlines());}
thread_local! {static TS: RefCell<ThemeSet> = RefCell::new(ThemeSet::load_defaults());}

impl ToHtml for CodeBlock {
    fn to_html(&self) -> String {
        if self.language.len() > 0 {
            PS.with(|ps_cell| {
                let ps = ps_cell.borrow();
                if let Some(syntax) = ps.find_syntax_by_token(self.language.as_str()) {
                    TS.with(|ts_cell| {
                        let ts = ts_cell.borrow();
                        format!(
                            "<div><code lang='{}'>{}</code></div>",
                            encode_attribute(self.language.clone().as_str()),
                            highlighted_html_for_string(
                                self.code.as_str(),
                                &ps,
                                syntax,
                                &ts.themes["InspiredGitHub"]
                            )
                        )
                    })
                } else {
                    format!(
                        "<div><code lang='{}'><pre>{}</pre></code></div>",
                        encode_attribute(self.language.clone().as_str()),
                        encode_minimal(self.code.as_str())
                    )
                }
            })
        } else {
            format!(
                "<div><code><pre>{}</pre></code></div>",
                encode_minimal(self.code.as_str())
            )
        }
    }
}

impl ToHtml for Quote {
    fn to_html(&self) -> String {
        let text = self
            .text
            .iter()
            .fold("".to_string(), |a, b| combine_with_lb!(a, b));
        if let Some(meta) = self.metadata.clone() {
            format!(
                "<div class='quote'><blockquote>{}</blockquote><span class='metadata'>{}</span></div>",
                text, encode_minimal(meta.data.as_str())
            )
        } else {
            format!("<div class='quote'><blockquote>{}</blockquote></div>", text)
        }
    }
}

impl ToHtml for Ruler {
    fn to_html(&self) -> String {
        "<hr>".to_string()
    }
}

impl ToHtml for Text {
    fn to_html(&self) -> String {
        self.subtext
            .iter()
            .fold("".to_string(), |a, b| format!("{}{}", a, b.to_html()))
    }
}

impl ToHtml for Image {
    fn to_html(&self) -> String {
        if let Some(description) = self.url.description.clone() {
            minify(
                format!(
                    "<div class='figure'>\
                     <a href={0}>\
                     <img src='{0}' alt='{1}'/>\
                     </a>\
                     <label class='imageDescription'>{1}</label>\
                     </div>",
                    encode_attribute(self.url.url.clone().as_str()),
                    encode_attribute(description.as_str())
                )
                .as_str(),
            )
        } else {
            format!("<a href={0}><img src='{0}'/></a>", self.url.url.clone(),)
        }
    }
}

impl ToHtml for BoldText {
    fn to_html(&self) -> String {
        format!("<b>{}</b>", self.value.to_html())
    }
}

impl ToHtml for UnderlinedText {
    fn to_html(&self) -> String {
        format!("<u>{}</u>", self.value.to_html())
    }
}

impl ToHtml for ItalicText {
    fn to_html(&self) -> String {
        format!("<i>{}</i>", self.value.to_html())
    }
}

impl ToHtml for StrikedText {
    fn to_html(&self) -> String {
        format!("<del>{}</del>", self.value.to_html())
    }
}

impl ToHtml for MonospaceText {
    fn to_html(&self) -> String {
        format!("<code class='inlineCode'>{}</code>", self.value.to_html())
    }
}

impl ToHtml for Url {
    fn to_html(&self) -> String {
        if let Some(description) = self.description.clone() {
            format!(
                "<a href='{}'>{}</a>",
                self.url.clone(),
                encode_minimal(description.as_str())
            )
        } else {
            format!(
                "<a href='{}'>{}</a>",
                self.url.clone(),
                encode_minimal(self.url.clone().as_str())
            )
        }
    }
}

impl ToHtml for PlainText {
    fn to_html(&self) -> String {
        encode_minimal(self.value.clone().as_str())
    }
}

impl ToHtml for Placeholder {
    fn to_html(&self) -> String {
        if let Some(value) = &self.value {
            value.to_html()
        } else {
            format!("Unknown placeholder '{}'!", encode_minimal(&self.name))
        }
    }
}

impl ToHtml for Anchor {
    fn to_html(&self) -> String {
        format!(
            "<a href='#{}'>{}</a>",
            encode_attribute(self.reference.as_str()),
            self.description.to_html()
        )
    }
}
