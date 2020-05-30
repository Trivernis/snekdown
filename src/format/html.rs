use crate::elements::*;

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

impl ToHtml for Inline {
    fn to_html(&self) -> String {
        match self {
            Inline::Text(text) => text.to_html(),
            Inline::Ruler(ruler) => ruler.to_html(),
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
            _ => "".to_string(),
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
        }
    }
}

impl ToHtml for Document {
    fn to_html(&self) -> String {
        let inner = self
            .elements
            .iter()
            .fold("".to_string(), |a, b| format!("{}{}", a, b.to_html()));
        if self.is_root {
            let style = std::include_str!("assets/style.css");
            format!(
                "<html><head><style>{}</style></head><body><div class='content'>{}</div></body></html>",
                style, inner
            )
        } else {
            format!(
                "<div class='documentImport' document-import=true>{}</div>",
                inner
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
        format!("<h{0}>{1}</h{0}>", self.size, self.line.to_html())
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
        format!("<table><tr>{}<tr>{}</table>", head, body)
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

impl ToHtml for CodeBlock {
    fn to_html(&self) -> String {
        format!(
            "<div><code lang='{}'><pre>{}</pre></code></div>",
            self.language.clone(),
            self.code.clone()
        )
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
                "<div><blockquote>{}</blockquote><span>- {}</span></div>",
                text, meta.data
            )
        } else {
            format!("<blockquote>{}</blockquote>", text)
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
            format!(
                "<div class='figure'>\
                 <a href={0}>\
                 <img src='{0}' alt='{1}'/>\
                 </a>\
                 <label class='imageDescription'>{1}</label>\
                 </div>",
                self.url.url.clone(),
                description
            )
        } else {
            format!("<a href={0}><img src='{0}'/></a>", self.url.url.clone(),)
        }
    }
}

impl ToHtml for BoldText {
    fn to_html(&self) -> String {
        format!("<u>{}</u>", self.value.to_html())
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
        format!("<code>{}</code>", self.value.to_html())
    }
}

impl ToHtml for Url {
    fn to_html(&self) -> String {
        if let Some(description) = self.description.clone() {
            format!("<a href='{}'>{}</a>", self.url.clone(), description)
        } else {
            format!("<a href='{}'>{}</a>", self.url.clone(), self.url.clone())
        }
    }
}

impl ToHtml for PlainText {
    fn to_html(&self) -> String {
        self.value.clone()
    }
}
