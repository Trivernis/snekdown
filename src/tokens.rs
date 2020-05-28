#[derive(Clone, Debug)]
pub enum Block {
    Section(Section),
    Paragraph(Paragraph),
    List(List),
    Table(Table),
}

#[derive(Clone, Debug)]
pub enum Inline {
    Text(Text),
}

#[derive(Clone, Debug)]
pub struct Document {
    elements: Vec<Block>,
}

#[derive(Clone, Debug)]
pub struct Section {
    header: Header,
    elements: Vec<Block>,
}

#[derive(Clone, Debug)]
pub struct Header {
    pub size: u8,
    pub line: Inline,
}

#[derive(Clone, Debug)]
pub struct BlockQuote {
    paragraph: Paragraph,
}

#[derive(Clone, Debug)]
pub struct Paragraph {
    pub elements: Vec<Inline>,
}

#[derive(Clone, Debug)]
pub struct List {
    pub ordered: bool,
    pub items: Vec<ListItem>,
}

#[derive(Clone, Debug)]
pub struct ListItem {
    pub(crate) text: Inline,
}

#[derive(Clone, Debug)]
pub struct Table {
    header: Row,
    rows: Vec<Row>,
}

#[derive(Clone, Debug)]
pub struct Row {
    text: Vec<Cell>,
}

#[derive(Clone, Debug)]
pub struct Cell {
    text: Inline,
}

#[derive(Clone, Debug)]
pub struct CodeBlock {
    language: String,
    code: String,
}

#[derive(Clone, Debug)]
pub struct Code {
    code: String,
}

#[derive(Clone, Debug)]
pub struct Text {
    pub subtext: Vec<SubText>,
}

#[derive(Clone, Debug)]
pub enum SubText {
    Plain(PlainText),
    Code(Code),
    Bold(BoldText),
    Italic(ItalicText),
    Underlined(UnderlinedText),
    Striked(StrikedText),
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

// TODO: Images, URIs
