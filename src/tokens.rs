use std::ops::Sub;

pub enum Block {
    Section(Section),
    Paragraph(Paragraph),
    List(List),
    Table(Table),
}

pub enum Inline {
    Text(Text),
}

pub struct Document {
    elements: Vec<Block>,
}

pub struct Section {
    header: Header,
    elements: Vec<Block>,
}

pub struct Header {
    pub size: u8,
    pub line: Inline,
}

pub struct BlockQuote {
    paragraph: Paragraph,
}

pub struct Paragraph {
    pub elements: Vec<Inline>,
}

pub struct List {
    pub ordered: bool,
    pub items: Vec<ListItem>,
}

pub struct ListItem {
    text: Inline,
}

pub struct Table {
    header: Row,
    rows: Vec<Row>,
}

pub struct Row {
    text: Vec<Cell>,
}

pub struct Cell {
    text: Inline,
}

pub struct CodeBlock {
    language: String,
    code: String,
}

pub struct Code {
    code: String,
}

pub struct Text {
    subtext: Vec<SubText>,
}

pub enum SubText {
    Plain(PlainText),
    Code(Code),
    Bold(BoldText),
    Italic(ItalicText),
    Underlined(UnderlinedText),
    Striked(StrikedText),
}

pub struct PlainText {
    value: String,
}

pub struct BoldText {
    value: Box<SubText>,
}

pub struct ItalicText {
    value: Box<SubText>,
}

pub struct UnderlinedText {
    value: Box<SubText>,
}

pub struct StrikedText {
    value: Box<SubText>,
}

impl Document {
    pub fn new() -> Self {
        Self {
            elements: Vec::new(),
        }
    }

    pub fn add_element(&mut self, element: Block) {
        self.elements.push(element)
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

// TODO: Images, URIs
