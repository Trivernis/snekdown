use std::sync::{Arc, Mutex};

pub const SECTION: &str = "section";
pub const PARAGRAPH: &str = "paragraph";
pub const LIST: &str = "list";
pub const TABLE: &str = "table";
pub const CODE_BLOCK: &str = "code_block";
pub const QUOTE: &str = "quote";
pub const IMPORT: &str = "import";

macro_rules! test_block {
    ($block:expr, $block_type:expr) => {
        match $block {
            Block::Section(_) if $block_type == SECTION => true,
            Block::List(_) if $block_type == LIST => true,
            Block::Table(_) if $block_type == TABLE => true,
            Block::Paragraph(_) if $block_type == PARAGRAPH => true,
            Block::CodeBlock(_) if $block_type == CODE_BLOCK => true,
            Block::Quote(_) if $block_type == QUOTE => true,
            Block::Import(_) if $block_type == IMPORT => true,
            _ => false,
        }
    };
}

#[derive(Clone, Debug)]
pub enum Element {
    Block(Box<Block>),
    Inline(Box<Inline>),
    SubText(Box<SubText>),
}

#[derive(Clone, Debug)]
pub enum Block {
    Section(Section),
    Paragraph(Paragraph),
    List(List),
    Table(Table),
    CodeBlock(CodeBlock),
    Quote(Quote),
    Import(Import),
    Placeholder(Arc<Mutex<Placeholder>>),
}

#[derive(Clone, Debug)]
pub enum Inline {
    Text(Text),
    Ruler(Ruler),
}

#[derive(Clone, Debug)]
pub struct Document {
    pub(crate) elements: Vec<Block>,
    pub(crate) is_root: bool,
    pub(crate) path: Option<String>,
    pub(crate) placeholders: Vec<Arc<Mutex<Placeholder>>>,
}

#[derive(Clone, Debug)]
pub struct Section {
    pub(crate) header: Header,
    pub(crate) elements: Vec<Block>,
}

#[derive(Clone, Debug)]
pub struct Header {
    pub(crate) size: u8,
    pub(crate) line: Inline,
}

#[derive(Clone, Debug)]
pub struct Paragraph {
    pub(crate) elements: Vec<Inline>,
}

#[derive(Clone, Debug)]
pub struct List {
    pub(crate) ordered: bool,
    pub(crate) items: Vec<ListItem>,
}

#[derive(Clone, Debug)]
pub struct ListItem {
    pub(crate) text: Inline,
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
    pub(crate) text: Inline,
}

#[derive(Clone, Debug)]
pub struct CodeBlock {
    pub(crate) language: String,
    pub(crate) code: String,
}

#[derive(Clone, Debug)]
pub struct Quote {
    pub(crate) metadata: Option<InlineMetadata>,
    pub(crate) text: Vec<Text>,
}

#[derive(Clone, Debug)]
pub struct Import {
    pub(crate) path: String,
    pub(crate) anchor: Arc<Mutex<ImportAnchor>>,
}

#[derive(Clone, Debug)]
pub struct ImportAnchor {
    pub(crate) document: Option<Document>,
}

#[derive(Clone, Debug)]
pub struct InlineMetadata {
    pub(crate) data: String,
}

#[derive(Clone, Debug)]
pub struct Ruler {}

#[derive(Clone, Debug)]
pub struct Text {
    pub subtext: Vec<SubText>,
}

#[derive(Clone, Debug)]
pub enum SubText {
    Plain(PlainText),
    Bold(BoldText),
    Italic(ItalicText),
    Underlined(UnderlinedText),
    Striked(StrikedText),
    Monospace(MonospaceText),
    Url(Url),
    Image(Image),
    Placeholder(Arc<Mutex<Placeholder>>),
}

#[derive(Clone, Debug)]
pub struct PlainText {
    pub(crate) value: String,
}

#[derive(Clone, Debug)]
pub struct BoldText {
    pub(crate) value: Box<SubText>,
}

#[derive(Clone, Debug)]
pub struct ItalicText {
    pub(crate) value: Box<SubText>,
}

#[derive(Clone, Debug)]
pub struct UnderlinedText {
    pub(crate) value: Box<SubText>,
}

#[derive(Clone, Debug)]
pub struct StrikedText {
    pub(crate) value: Box<SubText>,
}

#[derive(Clone, Debug)]
pub struct MonospaceText {
    pub(crate) value: PlainText,
}

#[derive(Clone, Debug)]
pub struct Url {
    pub description: Option<String>,
    pub url: String,
}

#[derive(Clone, Debug)]
pub struct Image {
    pub(crate) url: Url,
    pub(crate) metadata: Option<InlineMetadata>,
}

#[derive(Clone, Debug)]
pub struct Placeholder {
    pub(crate) name: String,
    pub(crate) value: Option<Element>,
}

// implementations

impl Document {
    pub fn new(is_root: bool) -> Self {
        Self {
            elements: Vec::new(),
            is_root,
            path: None,
            placeholders: Vec::new(),
        }
    }

    pub fn add_element(&mut self, element: Block) {
        self.elements.push(element)
    }

    pub fn add_placeholder(&mut self, placeholder: Arc<Mutex<Placeholder>>) {
        self.placeholders.push(placeholder);
    }

    pub fn find(&self, block_type: &str, nested: bool) -> Vec<&Block> {
        let mut found = Vec::new();
        let mut found_self = self
            .elements
            .iter()
            .filter(|e| {
                if nested {
                    match e {
                        Block::Section(sec) => found.append(&mut sec.find(block_type, nested)),
                        _ => {}
                    }
                }

                test_block!(e, block_type)
            })
            .collect();
        found.append(&mut found_self);

        found
    }

    pub fn create_toc(&self) -> List {
        let mut list = List::new();
        list.ordered = true;
        self.elements.iter().for_each(|e| match e {
            Block::Section(sec) => {
                let mut item = ListItem::new(sec.header.line.clone(), 1, true);
                item.children.append(&mut sec.get_toc_list().items);
                list.add_item(item);
            }
            Block::Import(imp) => {
                let anchor = imp.anchor.lock().unwrap();
                if let Some(doc) = &anchor.document {
                    list.items.append(&mut doc.create_toc().items)
                }
            }
            _ => {}
        });

        list
    }
}

impl Section {
    pub fn new(header: Header) -> Self {
        Self {
            header,
            elements: Vec::new(),
        }
    }

    pub fn add_element(&mut self, element: Block) {
        self.elements.push(element)
    }

    pub fn find(&self, block_type: &str, nested: bool) -> Vec<&Block> {
        let mut found = Vec::new();
        let mut found_self = self
            .elements
            .iter()
            .filter(|e| {
                if nested {
                    match e {
                        Block::Section(sec) => found.append(&mut sec.find(block_type, nested)),
                        _ => {}
                    }
                }
                test_block!(e, block_type)
            })
            .collect();
        found.append(&mut found_self);

        found
    }

    pub fn get_toc_list(&self) -> List {
        let mut list = List::new();
        self.elements.iter().for_each(|e| {
            if let Block::Section(sec) = e {
                let mut item = ListItem::new(sec.header.line.clone(), 1, true);
                item.children.append(&mut sec.get_toc_list().items);
                list.add_item(item);
            }
        });

        list
    }
}

impl Paragraph {
    pub fn new() -> Self {
        Self {
            elements: Vec::new(),
        }
    }

    pub fn add_element(&mut self, element: Inline) {
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
    pub fn new(text: Inline, level: u16, ordered: bool) -> Self {
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

impl Text {
    pub fn new() -> Self {
        Self {
            subtext: Vec::new(),
        }
    }

    pub fn add_subtext(&mut self, subtext: SubText) {
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
    pub fn new(description: Option<String>, url: String) -> Self {
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

    pub fn add_text(&mut self, text: Text) {
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
    pub fn new(name: String) -> Self {
        Self { name, value: None }
    }

    pub fn set_value(&mut self, value: Element) {
        self.value = Some(value);
    }
}
