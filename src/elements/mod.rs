pub mod tokens;

use crate::format::PlaceholderTemplate;
use crate::references::configuration::{ConfigRefEntry, Configuration, Value};
use crate::references::placeholders::ProcessPlaceholders;
use crate::references::templates::{Template, TemplateVariable};
use crate::utils::downloads::{DownloadManager, PendingDownload};
use asciimath_rs::elements::special::Expression;
use bibliographix::bib_manager::BibManager;
use bibliographix::bibliography::bibliography_entry::BibliographyEntryReference;
use bibliographix::references::bib_reference::BibRefAnchor;
use std::collections::HashMap;
use std::iter::FromIterator;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex, RwLock};

pub const SECTION: &str = "section";
pub const PARAGRAPH: &str = "paragraph";
pub const LIST: &str = "list";
pub const TABLE: &str = "table";
pub const CODE_BLOCK: &str = "code_block";
pub const QUOTE: &str = "quote";
pub const IMPORT: &str = "import";

#[derive(Clone, Debug)]
pub enum MetadataValue {
    String(String),
    Integer(i64),
    Float(f64),
    Bool(bool),
    Placeholder(Arc<RwLock<Placeholder>>),
    Template(Template),
}

#[derive(Clone, Debug)]
pub enum Element {
    Block(Box<Block>),
    Line(Box<Line>),
    Inline(Box<Inline>),
}

#[derive(Clone, Debug)]
pub enum Block {
    Section(Section),
    Paragraph(Paragraph),
    List(List),
    Table(Table),
    CodeBlock(CodeBlock),
    MathBlock(MathBlock),
    Quote(Quote),
    Import(Import),
    Placeholder(Arc<RwLock<Placeholder>>),
    Null,
}

#[derive(Clone, Debug)]
pub enum Line {
    Text(TextLine),
    Ruler(Ruler),
    RefLink(RefLink),
    Anchor(Anchor),
    Centered(Centered),
    BibEntry(BibEntry),
}

#[derive(Clone, Debug)]
pub struct Document {
    pub elements: Vec<Block>,
    pub(crate) is_root: bool,
    pub(crate) path: Option<String>,
    pub(crate) placeholders: Vec<Arc<RwLock<Placeholder>>>,
    pub config: Configuration,
    pub bibliography: BibManager,
    pub downloads: Arc<Mutex<DownloadManager>>,
    pub stylesheets: Vec<Arc<Mutex<PendingDownload>>>,
}

#[derive(Clone, Debug)]
pub struct Section {
    pub(crate) header: Header,
    pub(crate) elements: Vec<Block>,
    pub(crate) metadata: Option<InlineMetadata>,
}

#[derive(Clone, Debug)]
pub struct Header {
    pub(crate) size: u8,
    pub(crate) line: Line,
    pub(crate) anchor: String,
}

#[derive(Clone, Debug)]
pub struct Paragraph {
    pub(crate) elements: Vec<Line>,
}

#[derive(Clone, Debug)]
pub struct List {
    pub(crate) ordered: bool,
    pub items: Vec<ListItem>,
}

#[derive(Clone, Debug)]
pub struct ListItem {
    pub(crate) text: Line,
    pub(crate) level: u16,
    pub(crate) ordered: bool,
    pub(crate) children: Vec<ListItem>,
}

#[derive(Clone, Debug)]
pub struct Table {
    pub(crate) header: Row,
    pub(crate) rows: Vec<Row>,
}

#[derive(Clone, Debug)]
pub struct Row {
    pub(crate) cells: Vec<Cell>,
}

#[derive(Clone, Debug)]
pub struct Cell {
    pub(crate) text: Line,
}

#[derive(Clone, Debug)]
pub struct CodeBlock {
    pub(crate) language: String,
    pub(crate) code: String,
}

#[derive(Clone, Debug)]
pub struct Quote {
    pub(crate) metadata: Option<InlineMetadata>,
    pub(crate) text: Vec<TextLine>,
}

#[derive(Clone, Debug)]
pub struct Import {
    pub(crate) path: String,
    pub(crate) anchor: Arc<RwLock<ImportAnchor>>,
}

#[derive(Clone, Debug)]
pub struct ImportAnchor {
    pub(crate) document: Option<Document>,
}

#[derive(Clone, Debug)]
pub struct InlineMetadata {
    pub(crate) data: HashMap<String, MetadataValue>,
}

#[derive(Clone, Debug)]
pub struct Ruler {}

#[derive(Clone, Debug)]
pub struct TextLine {
    pub subtext: Vec<Inline>,
}

#[derive(Clone, Debug)]
pub enum Inline {
    Plain(PlainText),
    Bold(BoldText),
    Italic(ItalicText),
    Underlined(UnderlinedText),
    Striked(StrikedText),
    Monospace(MonospaceText),
    Superscript(SuperscriptText),
    Url(Url),
    Image(Image),
    Placeholder(Arc<RwLock<Placeholder>>),
    Checkbox(Checkbox),
    Emoji(Emoji),
    Colored(Colored),
    Math(Math),
    BibReference(Arc<RwLock<BibReference>>),
    TemplateVar(Arc<RwLock<TemplateVariable>>),
    CharacterCode(CharacterCode),
    LineBreak,
}

#[derive(Clone, Debug)]
pub struct PlainText {
    pub(crate) value: String,
}

#[derive(Clone, Debug)]
pub struct BoldText {
    pub(crate) value: Vec<Inline>,
}

#[derive(Clone, Debug)]
pub struct ItalicText {
    pub(crate) value: Vec<Inline>,
}

#[derive(Clone, Debug)]
pub struct UnderlinedText {
    pub(crate) value: Vec<Inline>,
}

#[derive(Clone, Debug)]
pub struct StrikedText {
    pub(crate) value: Vec<Inline>,
}

#[derive(Clone, Debug)]
pub struct MonospaceText {
    pub(crate) value: String,
}

#[derive(Clone, Debug)]
pub struct SuperscriptText {
    pub(crate) value: Vec<Inline>,
}

#[derive(Clone, Debug)]
pub struct Checkbox {
    pub(crate) value: bool,
}

#[derive(Clone, Debug)]
pub struct Url {
    pub description: Option<Vec<Inline>>,
    pub url: String,
}

#[derive(Clone, Debug)]
pub struct Image {
    pub(crate) url: Url,
    pub(crate) metadata: Option<InlineMetadata>,
    pub(crate) download: Arc<Mutex<PendingDownload>>,
}

#[derive(Clone, Debug)]
pub struct Placeholder {
    pub(crate) name: String,
    pub(crate) value: Option<Element>,
    pub(crate) metadata: Option<InlineMetadata>,
}

#[derive(Clone, Debug)]
pub struct RefLink {
    pub(crate) description: Box<Line>,
    pub(crate) reference: String,
}

#[derive(Clone, Debug)]
pub struct Anchor {
    pub(crate) inner: Box<Line>,
    pub(crate) key: String,
}

#[derive(Clone, Debug)]
pub struct Centered {
    pub(crate) line: TextLine,
}

#[derive(Clone, Debug)]
pub struct Emoji {
    pub(crate) value: char,
    pub(crate) name: String,
}

#[derive(Clone, Debug)]
pub struct Colored {
    pub(crate) value: Box<Inline>,
    pub(crate) color: String,
}

#[derive(Clone, Debug)]
pub struct Math {
    pub(crate) expression: Expression,
}

#[derive(Clone, Debug)]
pub struct MathBlock {
    pub(crate) expression: Expression,
}

#[derive(Clone, Debug)]
pub struct CharacterCode {
    pub(crate) code: String,
}

// implementations

impl Document {
    /// Creates a new parent document
    pub fn new() -> Self {
        Self {
            elements: Vec::new(),
            is_root: true,
            path: None,
            placeholders: Vec::new(),
            config: Configuration::default(),
            bibliography: BibManager::new(),
            stylesheets: Vec::new(),
            downloads: Arc::new(Mutex::new(DownloadManager::new())),
        }
    }

    /// Creates a new child document
    pub fn create_child(&self) -> Self {
        Self {
            elements: Vec::new(),
            is_root: false,
            path: None,
            placeholders: Vec::new(),
            config: self.config.clone(),
            bibliography: self.bibliography.create_child(),
            stylesheets: Vec::new(),
            downloads: Arc::clone(&self.downloads),
        }
    }

    pub fn add_element(&mut self, element: Block) {
        self.elements.push(element)
    }

    pub fn add_placeholder(&mut self, placeholder: Arc<RwLock<Placeholder>>) {
        self.placeholders.push(placeholder);
    }

    pub fn create_toc(&self, ordered: bool) -> List {
        let mut list = List::new();
        list.ordered = ordered;
        self.elements.iter().for_each(|e| match e {
            Block::Section(sec) => {
                if !sec.get_hide_in_toc() {
                    let mut item =
                        ListItem::new(Line::RefLink(sec.header.get_anchor()), 1, ordered);
                    item.children.append(&mut sec.get_toc_list(ordered).items);
                    list.add_item(item);
                }
            }
            Block::Import(imp) => {
                let anchor = imp.anchor.read().unwrap();
                if let Some(doc) = &anchor.document {
                    list.items.append(&mut doc.create_toc(ordered).items)
                }
            }
            _ => {}
        });

        list
    }

    /// Processes section and import elements
    ///
    /// if it encounters a section it checks if the sections is of smaller order than the previous one
    /// if thats the case it grabs the previous one and adds the section to its children
    ///
    /// if it encounters an import, it loads the imports top elements to its own
    pub fn postprocess_imports(&mut self) {
        let mut new_order: Vec<Block> = Vec::with_capacity(self.elements.len());
        self.elements.reverse();
        let mut count: usize = 0;
        let mut last_section: Option<(u8, usize)> = None;

        while let Some(element) = self.elements.pop() {
            match element {
                Block::Section(sec) => {
                    if let Some((last_size, last_pos)) = last_section {
                        if sec.header.size > last_size {
                            let last = new_order.get_mut(last_pos).unwrap();
                            if let Block::Section(p_sec) = last {
                                p_sec.add_section(sec);
                                continue;
                            }
                        }
                    }
                    last_section = Some((sec.header.size, count));
                    new_order.push(Block::Section(sec));
                }
                Block::Import(imp) => {
                    let arc_anchor = Arc::clone(&imp.anchor);
                    let anchor = &mut arc_anchor.write().unwrap();

                    if let Some(doc) = &mut anchor.document {
                        self.placeholders.append(&mut doc.placeholders);
                        doc.elements.reverse();
                        self.elements.append(&mut doc.elements);
                        anchor.document = None;
                        continue;
                    } else {
                        new_order.push(Block::Import(imp));
                    }
                }
                _ => {
                    if let Some((_, last_pos)) = last_section {
                        let last = new_order.get_mut(last_pos).unwrap();
                        if let Block::Section(p_sec) = last {
                            p_sec.add_element(element);
                            continue;
                        }
                    }
                    new_order.push(element)
                }
            }
            count += 1;
        }
        self.elements = new_order;
    }

    pub fn post_process(&mut self) {
        self.postprocess_imports();
        if self.is_root {
            self.process_definitions();
            self.bibliography.assign_entries_to_references();
            self.process_placeholders();
        }
    }
}

impl Section {
    pub fn new(header: Header) -> Self {
        Self {
            header,
            elements: Vec::new(),
            metadata: None,
        }
    }

    pub fn add_element(&mut self, element: Block) {
        self.elements.push(element)
    }

    pub fn get_toc_list(&self, ordered: bool) -> List {
        let mut list = List::new();
        self.elements.iter().for_each(|e| {
            if let Block::Section(sec) = e {
                if !sec.get_hide_in_toc() {
                    let mut item =
                        ListItem::new(Line::RefLink(sec.header.get_anchor()), 1, ordered);
                    item.children.append(&mut sec.get_toc_list(ordered).items);
                    list.add_item(item);
                }
            }
        });

        list
    }

    pub(crate) fn get_hide_in_toc(&self) -> bool {
        if let Some(meta) = &self.metadata {
            meta.get_bool("toc-hidden")
        } else {
            false
        }
    }

    /// adds a child section to the section
    /// It either adds it directly to its elements or iterates through its children to
    /// add it to the fitting one
    pub(crate) fn add_section(&mut self, section: Section) {
        if section.header.size == self.header.size + 1 {
            self.elements.push(Block::Section(section))
        } else {
            let has_parent = RwLock::new(AtomicBool::new(true));
            let iterator = self.elements.iter_mut().rev().filter(|e| {
                if let Block::Section(sec) = e {
                    if sec.header.size > section.header.size {
                        has_parent.write().unwrap().store(true, Ordering::Relaxed);
                        true
                    } else {
                        false
                    }
                } else {
                    false
                }
            });

            if has_parent.read().unwrap().load(Ordering::Relaxed) {
                for sec in iterator {
                    if let Block::Section(sec) = sec {
                        if sec.header.size < section.header.size {
                            sec.add_section(section);
                            break;
                        }
                    }
                }
            } else {
                self.elements.push(Block::Section(section));
            }
        }
    }
}

impl Header {
    pub fn new(content: Line, anchor: String) -> Self {
        Self {
            size: 0,
            anchor,
            line: content,
        }
    }

    pub fn get_anchor(&self) -> RefLink {
        RefLink {
            description: Box::new(self.line.clone()),
            reference: self.anchor.clone(),
        }
    }
}

impl Paragraph {
    pub fn new() -> Self {
        Self {
            elements: Vec::new(),
        }
    }

    pub fn add_element(&mut self, element: Line) {
        self.elements.push(element)
    }
}

impl List {
    pub fn new() -> Self {
        Self {
            ordered: false,
            items: Vec::new(),
        }
    }

    pub fn add_item(&mut self, item: ListItem) {
        self.items.push(item)
    }
}

impl ListItem {
    pub fn new(text: Line, level: u16, ordered: bool) -> Self {
        Self {
            text,
            level,
            ordered,
            children: Vec::new(),
        }
    }

    pub fn add_child(&mut self, child: ListItem) {
        self.children.push(child)
    }
}

impl TextLine {
    pub fn new() -> Self {
        Self {
            subtext: Vec::new(),
        }
    }

    pub fn add_subtext(&mut self, subtext: Inline) {
        self.subtext.push(subtext)
    }
}

impl Table {
    pub fn new(header: Row) -> Self {
        Self {
            header,
            rows: Vec::new(),
        }
    }

    pub fn add_row(&mut self, row: Row) {
        self.rows.push(row)
    }
}

impl Row {
    pub fn new() -> Self {
        Self { cells: Vec::new() }
    }

    pub fn add_cell(&mut self, cell: Cell) {
        self.cells.push(cell)
    }
}

impl Url {
    pub fn new(description: Option<Vec<Inline>>, url: String) -> Self {
        Self { description, url }
    }
}

impl Quote {
    pub fn new(metadata: Option<InlineMetadata>) -> Self {
        Self {
            metadata,
            text: Vec::new(),
        }
    }

    pub fn add_text(&mut self, text: TextLine) {
        self.text.push(text)
    }
}

impl ImportAnchor {
    pub fn new() -> Self {
        Self { document: None }
    }

    pub fn set_document(&mut self, document: Document) {
        self.document = Some(document);
    }
}

impl PartialEq for Import {
    fn eq(&self, other: &Self) -> bool {
        self.path == other.path
    }
}

impl Placeholder {
    pub fn new(name: String, metadata: Option<InlineMetadata>) -> Self {
        Self {
            name,
            value: None,
            metadata,
        }
    }

    pub fn set_value(&mut self, value: Element) {
        self.value = Some(value);
    }
}

pub trait Metadata {
    fn get_bool(&self, key: &str) -> bool;
    fn get_string(&self, key: &str) -> Option<String>;
    fn get_string_map(&self) -> HashMap<String, String>;
}

impl Metadata for InlineMetadata {
    fn get_bool(&self, key: &str) -> bool {
        if let Some(MetadataValue::Bool(value)) = self.data.get(key) {
            *value
        } else {
            false
        }
    }

    fn get_string(&self, key: &str) -> Option<String> {
        if let Some(MetadataValue::String(value)) = self.data.get(key) {
            Some(value.clone())
        } else {
            None
        }
    }

    fn get_string_map(&self) -> HashMap<String, String> {
        let mut string_map = HashMap::new();
        for (k, v) in &self.data {
            match v {
                MetadataValue::String(s) => string_map.insert(k.clone(), s.clone()),
                MetadataValue::Bool(b) => string_map.insert(k.clone(), b.to_string()),
                MetadataValue::Float(f) => string_map.insert(k.clone(), f.to_string()),
                MetadataValue::Integer(i) => string_map.insert(k.clone(), i.to_string()),
                _ => None,
            };
        }

        string_map
    }
}

impl Into<HashMap<String, Value>> for InlineMetadata {
    fn into(self) -> HashMap<String, Value> {
        HashMap::from_iter(self.data.iter().filter_map(|(k, v)| match v {
            MetadataValue::String(s) => Some((k.clone(), Value::String(s.clone()))),
            MetadataValue::Bool(b) => Some((k.clone(), Value::Bool(*b))),
            MetadataValue::Integer(i) => Some((k.clone(), Value::Integer(*i))),
            MetadataValue::Float(f) => Some((k.clone(), Value::Float(*f))),
            MetadataValue::Template(t) => Some((k.clone(), Value::Template(t.clone()))),
            _ => None,
        }))
    }
}

impl Image {
    pub fn get_content(&self) -> Option<Vec<u8>> {
        let mut data = None;
        std::mem::swap(&mut data, &mut self.download.lock().unwrap().data);

        data
    }
}

#[derive(Clone, Debug)]
pub struct BibEntry {
    pub key: String,
    pub entry: BibliographyEntryReference,
}

#[derive(Clone, Debug)]
pub struct BibReference {
    pub(crate) key: String,
    pub(crate) entry_anchor: Arc<Mutex<BibRefAnchor>>,
    pub(crate) display: Option<ConfigRefEntry>,
}

impl BibReference {
    pub fn new(
        key: String,
        display: Option<ConfigRefEntry>,
        anchor: Arc<Mutex<BibRefAnchor>>,
    ) -> Self {
        Self {
            key: key.to_string(),
            display,
            entry_anchor: anchor,
        }
    }

    pub(crate) fn get_formatted(&self) -> String {
        if let Some(entry) = &self.entry_anchor.lock().unwrap().entry {
            let entry = entry.lock().unwrap();

            if let Some(display) = &self.display {
                let display = display.read().unwrap();
                let mut template = PlaceholderTemplate::new(display.get().as_string());
                let mut value_map = HashMap::new();
                value_map.insert("key".to_string(), entry.key());

                if let Some(ord) = entry.raw_fields.get("ord") {
                    value_map.insert("number".to_string(), ord.clone());
                }
                template.set_replacements(value_map);
                return template.render();
            }
            return format!("{}", entry.key());
        }

        return "citation needed".to_string();
    }
}

impl MetadataValue {
    pub fn to_string(&self) -> String {
        match self {
            MetadataValue::String(s) => s.clone(),
            MetadataValue::Placeholder(_) => "".to_string(),
            MetadataValue::Integer(i) => i.to_string(),
            MetadataValue::Float(f) => f.to_string(),
            MetadataValue::Bool(b) => b.to_string(),
            MetadataValue::Template(_) => "".to_string(),
        }
    }
}
