pub enum Block {
    Section(Section),
    Paragraph(Paragraph),
    List(List),
    Table(Table),
}

pub enum Inline {
    Text(Text),
    Code(Code),
}

pub struct Document {
    elements: Vec<Block>,
}

pub struct Section {
    header: Header,
    elements: Vec<Block>,
}

pub struct Header {
    pub(crate) size: u8,
    pub(crate) text: Text,
}

pub struct BlockQuote {
    paragraph: Paragraph,
}

pub struct Paragraph {
    elements: Vec<Inline>,
}

pub struct List {
    ordered: bool,
    items: Vec<ListItem>,
}

pub struct ListItem {
    text: Vec<Inline>,
}

pub struct Table {
    header: Row,
    rows: Vec<Row>,
}

pub struct Row {
    text: Vec<Cell>,
}

pub struct Cell {
    text: Vec<Inline>,
}

pub struct CodeBlock {
    language: String,
    code: String,
}

pub struct Code {
    code: String,
}

pub struct Text {
    bold: bool,
    italic: bool,
    underlined: bool,
    striked: bool,
    value: String,
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
