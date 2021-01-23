use crate::elements::*;
use crate::format::html::html_writer::HTMLWriter;
use crate::format::style::{get_code_theme_for_theme, get_css_for_theme};
use crate::format::PlaceholderTemplate;
use crate::references::glossary::{GlossaryDisplay, GlossaryReference};
use crate::references::templates::{Template, TemplateVariable};
use asciimath_rs::format::mathml::ToMathML;
use htmlescape::encode_attribute;
use minify::html::minify;
use std::io;
use syntect::html::highlighted_html_for_string;

const MATHJAX_URL: &str = "https://cdn.jsdelivr.net/npm/mathjax@3/es5/tex-mml-chtml.js";

pub trait ToHtml {
    fn to_html(&self, writer: &mut HTMLWriter) -> io::Result<()>;
}

impl ToHtml for Element {
    fn to_html(&self, writer: &mut HTMLWriter) -> io::Result<()> {
        match self {
            Element::Block(block) => block.to_html(writer),
            Element::Inline(inline) => inline.to_html(writer),
            Element::Line(line) => line.to_html(writer),
        }
    }
}

impl ToHtml for Line {
    fn to_html(&self, writer: &mut HTMLWriter) -> io::Result<()> {
        match self {
            Line::Text(text) => text.to_html(writer),
            Line::Ruler(ruler) => ruler.to_html(writer),
            Line::RefLink(anchor) => anchor.to_html(writer),
            Line::Centered(centered) => centered.to_html(writer),
            Line::Anchor(a) => a.to_html(writer),
            _ => Ok(()),
        }
    }
}

impl ToHtml for Inline {
    fn to_html(&self, writer: &mut HTMLWriter) -> io::Result<()> {
        match self {
            Inline::Url(url) => url.to_html(writer),
            Inline::Monospace(mono) => mono.to_html(writer),
            Inline::Striked(striked) => striked.to_html(writer),
            Inline::Plain(plain) => plain.to_html(writer),
            Inline::Italic(italic) => italic.to_html(writer),
            Inline::Underlined(under) => under.to_html(writer),
            Inline::Bold(bold) => bold.to_html(writer),
            Inline::Image(img) => img.to_html(writer),
            Inline::Placeholder(placeholder) => placeholder.read().unwrap().to_html(writer),
            Inline::Superscript(superscript) => superscript.to_html(writer),
            Inline::Checkbox(checkbox) => checkbox.to_html(writer),
            Inline::Emoji(emoji) => emoji.to_html(writer),
            Inline::Colored(colored) => colored.to_html(writer),
            Inline::BibReference(bibref) => bibref.read().unwrap().to_html(writer),
            Inline::TemplateVar(var) => var.read().unwrap().to_html(writer),
            Inline::Math(m) => m.to_html(writer),
            Inline::LineBreak => writer.write("<br>".to_string()),
            Inline::CharacterCode(code) => code.to_html(writer),
            Inline::GlossaryReference(gloss) => gloss.lock().to_html(writer),
            Inline::Arrow(a) => a.to_html(writer),
        }
    }
}

impl ToHtml for Block {
    fn to_html(&self, writer: &mut HTMLWriter) -> io::Result<()> {
        match self {
            Block::Paragraph(para) => para.to_html(writer),
            Block::List(list) => list.to_html(writer),
            Block::Table(table) => table.to_html(writer),
            Block::CodeBlock(code) => code.to_html(writer),
            Block::Quote(quote) => quote.to_html(writer),
            Block::Section(section) => section.to_html(writer),
            Block::Import(import) => import.to_html(writer),
            Block::Placeholder(placeholder) => placeholder.read().unwrap().to_html(writer),
            Block::MathBlock(m) => m.to_html(writer),
            _ => Ok(()),
        }
    }
}

impl ToHtml for MetadataValue {
    fn to_html(&self, writer: &mut HTMLWriter) -> io::Result<()> {
        match self {
            MetadataValue::String(string) => writer.write_escaped(string.clone()),
            MetadataValue::Integer(num) => writer.write(num.to_string()),
            MetadataValue::Placeholder(ph) => ph.read().unwrap().to_html(writer),
            MetadataValue::Bool(b) => writer.write(b.to_string()),
            MetadataValue::Float(f) => writer.write(f.to_string()),
            MetadataValue::Template(t) => t.to_html(writer),
        }
    }
}

impl ToHtml for Document {
    fn to_html(&self, writer: &mut HTMLWriter) -> io::Result<()> {
        let path = if let Some(path) = &self.path {
            format!("path=\"{}\"", encode_attribute(path.as_str()))
        } else {
            "".to_string()
        };

        if self.is_root {
            let metadata = self.config.lock().metadata.clone();

            let style = minify(get_css_for_theme(writer.get_theme()).as_str());
            writer.write("<!DOCTYPE html>".to_string())?;
            writer.write("<html lang=\"".to_string())?;
            writer.write_attribute(metadata.language)?;
            writer.write("\"><head>".to_string())?;
            writer.write("<meta charset=\"UTF-8\">".to_string())?;

            if let Some(author) = metadata.author {
                writer.write("<meta name=\"author\" content=\"".to_string())?;
                writer.write_attribute(author)?;
                writer.write("\">".to_string())?;
            }

            if let Some(title) = metadata.title {
                writer.write("<title>".to_string())?;
                writer.write_escaped(title.clone())?;
                writer.write("</title>".to_string())?;
                writer.write("<meta name=\"title\" content=\"".to_string())?;
                writer.write_attribute(title)?;
                writer.write("\">".to_string())?;
            }

            if let Some(description) = metadata.description {
                writer.write("<meta name=\"description\" content=\"".to_string())?;
                writer.write_attribute(description)?;
                writer.write("\">".to_string())?;
            }

            if !metadata.keywords.is_empty() {
                writer.write("<meta name=\"keywords\" content=\"".to_string())?;
                writer.write_attribute(
                    metadata
                        .keywords
                        .iter()
                        .fold("".to_string(), |a, b| format!("{}, {}", a, b))
                        .trim_start_matches(", ")
                        .to_string(),
                )?;
                writer.write("\">".to_string())?;
            }

            writer.write("<style>".to_string())?;
            writer.write(style)?;
            writer.write("</style>".to_string())?;

            if self.config.lock().features.include_mathjax {
                writer.write(format!(
                    "<script id=\"MathJax-script\" type=\"text/javascript\" async src={}></script>",
                    MATHJAX_URL
                ))?;
            }

            for stylesheet in &self.stylesheets {
                let mut stylesheet = stylesheet.lock();
                let data = std::mem::replace(&mut stylesheet.data, None);
                if let Some(data) = data {
                    writer.write("<style>".to_string())?;
                    writer.write(minify(String::from_utf8(data).unwrap().as_str()))?;
                    writer.write("</style>".to_string())?;
                } else {
                    writer.write("<link rel=\"stylsheet\" href=\"".to_string())?;
                    writer.write_attribute(stylesheet.path.clone())?;
                    writer.write("\">".to_string())?;
                }
            }
            writer.write("</head><body><div class=\"content\">".to_string())?;
            for element in &self.elements {
                element.to_html(writer)?;
            }
            writer.write("</div></body></html>".to_string())?;
        } else {
            writer.write("<div class=\"documentImport\" document-import=\"true\" ".to_string())?;
            writer.write(path)?;
            writer.write(">".to_string())?;

            for element in &self.elements {
                element.to_html(writer)?;
            }
            writer.write("</div>".to_string())?;
        }

        Ok(())
    }
}

impl ToHtml for Math {
    fn to_html(&self, writer: &mut HTMLWriter) -> io::Result<()> {
        writer.write("<math xmlns='http://www.w3.org/1998/Math/MathML'>".to_string())?;
        writer.write(self.expression.to_mathml())?;

        writer.write("</math>".to_string())
    }
}

impl ToHtml for MathBlock {
    fn to_html(&self, writer: &mut HTMLWriter) -> io::Result<()> {
        writer.write(
            "<math xmlns='http://www.w3.org/1998/Math/MathML' display='block'>".to_string(),
        )?;
        writer.write(self.expression.to_mathml())?;

        writer.write("</math>".to_string())
    }
}

impl ToHtml for Import {
    fn to_html(&self, _writer: &mut HTMLWriter) -> io::Result<()> {
        Ok(())
    }
}

impl ToHtml for Section {
    fn to_html(&self, writer: &mut HTMLWriter) -> io::Result<()> {
        writer.write("<section>".to_string())?;
        self.header.to_html(writer)?;
        for element in &self.elements {
            element.to_html(writer)?;
        }
        writer.write("</section>".to_string())
    }
}

impl ToHtml for Header {
    fn to_html(&self, writer: &mut HTMLWriter) -> io::Result<()> {
        writer.write(format!("<h{}", self.size))?;
        writer.write(" id=\"".to_string())?;
        writer.write_attribute(self.anchor.clone())?;
        writer.write("\">".to_string())?;
        self.line.to_html(writer)?;

        writer.write(format!("</h{}>", self.size))
    }
}

impl ToHtml for Paragraph {
    fn to_html(&self, writer: &mut HTMLWriter) -> io::Result<()> {
        writer.write("<div class=\"paragraph\">".to_string())?;

        if let Some(first) = self.elements.first() {
            first.to_html(writer)?;
        }
        if self.elements.len() > 1 {
            for element in &self.elements[1..] {
                writer.write(" ".to_string())?;
                element.to_html(writer)?;
            }
        }

        writer.write("</div>".to_string())
    }
}

impl ToHtml for List {
    fn to_html(&self, writer: &mut HTMLWriter) -> io::Result<()> {
        if self.ordered {
            writer.write("<ol>".to_string())?;
            for item in &self.items {
                item.to_html(writer)?;
            }
            writer.write("</ol>".to_string())
        } else {
            writer.write("<ul>".to_string())?;
            for item in &self.items {
                item.to_html(writer)?;
            }

            writer.write("</ul>".to_string())
        }
    }
}

impl ToHtml for ListItem {
    fn to_html(&self, writer: &mut HTMLWriter) -> io::Result<()> {
        writer.write("<li>".to_string())?;
        self.text.to_html(writer)?;

        if let Some(first) = self.children.first() {
            if first.ordered {
                writer.write("<ol>".to_string())?;
                for item in &self.children {
                    item.to_html(writer)?;
                }
                writer.write("</ol>".to_string())?;
            } else {
                writer.write("<ul>".to_string())?;
                for item in &self.children {
                    item.to_html(writer)?;
                }
                writer.write("</ul>".to_string())?;
            }
        }

        writer.write("</li>".to_string())
    }
}

impl ToHtml for Table {
    fn to_html(&self, writer: &mut HTMLWriter) -> io::Result<()> {
        writer.write("<div class=\"tableWrapper\"><table><tr>".to_string())?;

        for cell in &self.header.cells {
            writer.write("<th>".to_string())?;
            cell.text.to_html(writer)?;
            writer.write("</th>".to_string())?;
        }
        writer.write("</tr>".to_string())?;
        for row in &self.rows {
            row.to_html(writer)?;
        }

        writer.write("</table></div>".to_string())
    }
}

impl ToHtml for Row {
    fn to_html(&self, writer: &mut HTMLWriter) -> io::Result<()> {
        writer.write("<tr>".to_string())?;

        for cell in &self.cells {
            cell.to_html(writer)?;
        }

        writer.write("</tr>".to_string())
    }
}

impl ToHtml for Cell {
    fn to_html(&self, writer: &mut HTMLWriter) -> io::Result<()> {
        writer.write("<td>".to_string())?;
        self.text.to_html(writer)?;

        writer.write("</td>".to_string())
    }
}

impl ToHtml for CodeBlock {
    fn to_html(&self, writer: &mut HTMLWriter) -> io::Result<()> {
        writer.write("<div><code".to_string())?;

        if self.language.len() > 0 {
            writer.write(" lang=\"".to_string())?;
            writer.write_attribute(self.language.clone())?;
            writer.write("\">".to_string())?;
            let (theme, syntax_set) = get_code_theme_for_theme(writer.get_theme());

            if let Some(syntax) = syntax_set.find_syntax_by_token(self.language.as_str()) {
                writer.write(highlighted_html_for_string(
                    self.code.as_str(),
                    &syntax_set,
                    syntax,
                    &theme,
                ))?;
            } else {
                writer.write("<pre>".to_string())?;
                writer.write_escaped(self.code.clone())?;
                writer.write("</pre>".to_string())?;
            }
        } else {
            writer.write("><pre>".to_string())?;
            writer.write_escaped(self.code.clone())?;
            writer.write("</pre>".to_string())?;
        }

        writer.write("</code></div>".to_string())
    }
}

impl ToHtml for Quote {
    fn to_html(&self, writer: &mut HTMLWriter) -> io::Result<()> {
        writer.write("<div class=\"quote\"><blockquote>".to_string())?;
        for line in &self.text {
            line.to_html(writer)?;
            writer.write("<br/>".to_string())?;
        }
        if let Some(meta) = self.metadata.clone() {
            writer.write("<span class=\"metadata\">".to_string())?;
            meta.to_html(writer)?;
            writer.write("</span>".to_string())?;
        }
        writer.write("</blockquote></div>".to_string())
    }
}

impl ToHtml for Ruler {
    fn to_html(&self, writer: &mut HTMLWriter) -> io::Result<()> {
        writer.write("<hr/>".to_string())
    }
}

impl ToHtml for TextLine {
    fn to_html(&self, writer: &mut HTMLWriter) -> io::Result<()> {
        for text in &self.subtext {
            text.to_html(writer)?;
        }

        Ok(())
    }
}

impl ToHtml for Image {
    fn to_html(&self, writer: &mut HTMLWriter) -> io::Result<()> {
        let mut style = String::new();

        let url = if let Some(content) = self.get_content() {
            let mime_type = self.get_mime_type();
            format!(
                "data:{};base64,{}",
                mime_type.to_string(),
                base64::encode(content)
            )
        } else {
            encode_attribute(self.url.url.as_str())
        };
        if let Some(meta) = &self.metadata {
            if let Some(width) = meta.get_string("width") {
                style = format!("{}width: {};", style, width)
            }
            if let Some(height) = meta.get_string("height") {
                style = format!("{}height: {};", style, height)
            }
        }
        if let Some(description) = self.url.description.clone() {
            writer.write("<div class=\"figure\"><a href=\"".to_string())?;
            writer.write_attribute(url.clone())?;
            writer.write("\"><img src=\"".to_string())?;
            writer.write(url)?;
            writer.write("\" style=\"".to_string())?;
            writer.write(style)?;
            writer.write("\"/></a><br><label class=\"imageDescripton\">".to_string())?;
            for item in description {
                item.to_html(writer)?;
                writer.write("&#32;".to_string())?;
            }
            writer.write("</label></div>".to_string())?;
        } else {
            writer.write("<a href=\"".to_string())?;
            writer.write(url.clone())?;
            writer.write("\"><img src=\"".to_string())?;
            writer.write(url)?;
            writer.write("\" style=\"".to_string())?;
            writer.write(style)?;
            writer.write("\"/></a>".to_string())?;
        }

        Ok(())
    }
}

impl ToHtml for BoldText {
    fn to_html(&self, writer: &mut HTMLWriter) -> io::Result<()> {
        writer.write("<b>".to_string())?;
        for element in &self.value {
            element.to_html(writer)?;
        }
        writer.write("</b>".to_string())
    }
}

impl ToHtml for UnderlinedText {
    fn to_html(&self, writer: &mut HTMLWriter) -> io::Result<()> {
        writer.write("<u>".to_string())?;
        for element in &self.value {
            element.to_html(writer)?;
        }
        writer.write("</u>".to_string())
    }
}

impl ToHtml for ItalicText {
    fn to_html(&self, writer: &mut HTMLWriter) -> io::Result<()> {
        writer.write("<i>".to_string())?;
        for element in &self.value {
            element.to_html(writer)?;
        }
        writer.write("</i>".to_string())
    }
}

impl ToHtml for StrikedText {
    fn to_html(&self, writer: &mut HTMLWriter) -> io::Result<()> {
        writer.write("<del>".to_string())?;
        for element in &self.value {
            element.to_html(writer)?;
        }
        writer.write("</del>".to_string())
    }
}

impl ToHtml for SuperscriptText {
    fn to_html(&self, writer: &mut HTMLWriter) -> io::Result<()> {
        writer.write("<sup>".to_string())?;
        for element in &self.value {
            element.to_html(writer)?;
        }
        writer.write("</sup>".to_string())
    }
}

impl ToHtml for MonospaceText {
    fn to_html(&self, writer: &mut HTMLWriter) -> io::Result<()> {
        writer.write("<code class=\"inlineCode\">".to_string())?;
        writer.write_escaped(self.value.clone())?;

        writer.write("</code>".to_string())
    }
}

impl ToHtml for Url {
    fn to_html(&self, writer: &mut HTMLWriter) -> io::Result<()> {
        writer.write("<a href=\"".to_string())?;
        writer.write(self.url.clone())?;
        writer.write("\">".to_string())?;
        if let Some(description) = self.description.clone() {
            for desc in description {
                desc.to_html(writer)?;
            }
        } else {
            writer.write_escaped(self.url.clone())?;
        }

        writer.write("</a>".to_string())
    }
}

impl ToHtml for PlainText {
    fn to_html(&self, writer: &mut HTMLWriter) -> io::Result<()> {
        writer.write_escaped(self.value.clone())
    }
}

impl ToHtml for Placeholder {
    fn to_html(&self, writer: &mut HTMLWriter) -> io::Result<()> {
        if let Some(value) = &self.value {
            value.to_html(writer)
        } else {
            log::debug!("Unknown placeholder [[{}]]", self.name.clone());
            writer.write_escaped(format!("[[{}]]", self.name.clone()))?;
            writer.write("<!--Unknown placeholder \"".to_string())?;
            writer.write_escaped(self.name.clone())?;
            writer.write("\"-->".to_string())
        }
    }
}

impl ToHtml for RefLink {
    fn to_html(&self, writer: &mut HTMLWriter) -> io::Result<()> {
        writer.write("<a href=\"#".to_string())?;
        writer.write_escaped(self.reference.clone())?;
        writer.write("\">".to_string())?;
        self.description.to_html(writer)?;

        writer.write("</a>".to_string())
    }
}

impl ToHtml for InlineMetadata {
    fn to_html(&self, writer: &mut HTMLWriter) -> io::Result<()> {
        if let Some(MetadataValue::String(format)) = self.data.get("display") {
            let mut template = PlaceholderTemplate::new(format.clone());
            self.data
                .iter()
                .for_each(|(k, v)| template.add_replacement(k, &v.to_string()));

            writer.write(template.render())?;
        } else {
            for (k, v) in &self.data {
                writer.write_escaped(format!("{}={},", k, v.to_string()))?;
            }
        }
        Ok(())
    }
}

impl ToHtml for Centered {
    fn to_html(&self, writer: &mut HTMLWriter) -> io::Result<()> {
        writer.write("<div class=\"centered\">".to_string())?;
        self.line.to_html(writer)?;

        writer.write("</div>".to_string())
    }
}

impl ToHtml for Checkbox {
    fn to_html(&self, writer: &mut HTMLWriter) -> io::Result<()> {
        writer.write("<input type=\"checkbox\" disabled ".to_string())?;
        if self.value {
            writer.write("checked".to_string())?;
        }

        writer.write("/>".to_string())
    }
}

impl ToHtml for Emoji {
    fn to_html(&self, writer: &mut HTMLWriter) -> io::Result<()> {
        writer.write("<span class=\"emoji\" emoji-name=\"".to_string())?;
        writer.write_attribute(self.name.clone())?;
        writer.write("\">".to_string())?;
        writer.write(self.value.to_string())?;

        writer.write("</span>".to_string())
    }
}

impl ToHtml for Colored {
    fn to_html(&self, writer: &mut HTMLWriter) -> io::Result<()> {
        writer.write("<span class=\"colored\" style=\"color:".to_string())?;
        writer.write(self.color.clone())?;
        writer.write(";\">".to_string())?;
        self.value.to_html(writer)?;

        writer.write("</span>".to_string())
    }
}

impl ToHtml for BibReference {
    fn to_html(&self, writer: &mut HTMLWriter) -> io::Result<()> {
        writer.write("<sup><a href=\"#".to_string())?;
        writer.write_attribute(self.key.clone())?;
        writer.write("\">".to_string())?;
        writer.write(self.get_formatted())?;

        writer.write("</a></sup>".to_string())
    }
}

impl ToHtml for Template {
    fn to_html(&self, writer: &mut HTMLWriter) -> io::Result<()> {
        for element in &self.text {
            element.to_html(writer)?;
        }

        Ok(())
    }
}

impl ToHtml for TemplateVariable {
    fn to_html(&self, writer: &mut HTMLWriter) -> io::Result<()> {
        if let Some(value) = &self.value {
            writer.write_escaped(self.prefix.clone())?;
            value.to_html(writer)?;
            writer.write_escaped(self.suffix.clone())?;
        }

        Ok(())
    }
}

impl ToHtml for CharacterCode {
    fn to_html(&self, writer: &mut HTMLWriter) -> io::Result<()> {
        writer.write("&".to_string())?;
        writer.write_escaped(self.code.clone())?;

        writer.write(";".to_string())
    }
}

impl ToHtml for Anchor {
    fn to_html(&self, writer: &mut HTMLWriter) -> io::Result<()> {
        writer.write("<div id=\"".to_string())?;
        writer.write_attribute(self.key.clone())?;
        writer.write("\">".to_string())?;
        self.inner.to_html(writer)?;

        writer.write("</div>".to_string())
    }
}

impl ToHtml for GlossaryReference {
    fn to_html(&self, writer: &mut HTMLWriter) -> io::Result<()> {
        if let Some(entry) = &self.entry {
            let entry = entry.lock();
            writer.write("<a class=\"glossaryReference\" href=\"#".to_string())?;
            writer.write_attribute(self.short.clone())?;
            writer.write("\">".to_string())?;
            match self.display {
                GlossaryDisplay::Short => writer.write_escaped(entry.short.clone())?,
                GlossaryDisplay::Long => writer.write_escaped(entry.long.clone())?,
            }
            writer.write("</a>".to_string())?;
        } else {
            writer.write_escaped(format!("~{}", self.short.clone()))?;
        }

        Ok(())
    }
}

impl ToHtml for Arrow {
    fn to_html(&self, writer: &mut HTMLWriter) -> io::Result<()> {
        writer.write("<span class=\"arrow\">".to_string())?;
        match self {
            Arrow::RightArrow => writer.write("&xrarr;".to_string()),
            Arrow::LeftArrow => writer.write("&xlarr;".to_string()),
            Arrow::LeftRightArrow => writer.write("&xharr;".to_string()),
            Arrow::BigRightArrow => writer.write("&xrArr;".to_string()),
            Arrow::BigLeftArrow => writer.write("&xlArr;".to_string()),
            Arrow::BigLeftRightArrow => writer.write("&xhArr;".to_string()),
        }?;

        writer.write("</span>".to_string())
    }
}
