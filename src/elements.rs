#[derive(Clone, Debug)]
pub enum Block {
    Section(Section),
    Paragraph(Paragraph),
    List(List),
    Table(Table),
    CodeBlock(CodeBlock),
    Quote(Quote),
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
    text: Inline,
    pub(crate) level: u16,
    pub(crate) children: Vec<ListItem>,
}

#[derive(Clone, Debug)]
pub struct Table {
    header: Row,
    pub rows: Vec<Row>,
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
pub struct Code {
    code: String,
}

#[derive(Clone, Debug)]
pub struct Quote {
    pub(crate) metadata: Option<InlineMetadata>,
    pub(crate) text: Vec<Text>,
}

#[derive(Clone, Debug)]
pub struct InlineMetadata {
    pub(crate) data: String,
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
    Monospace(MonospaceText),
    Url(Url),
    Image(Image),
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
    description: Option<String>,
    url: String,
}

#[derive(Clone, Debug)]
pub struct Image {
    pub(crate) url: Url,
    pub(crate) metadata: Option<InlineMetadata>,
}

// implementations

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

impl ListItem {
    pub fn new(text: Inline, level: u16) -> Self {
        Self {
            text,
            level,
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

// TODO: Images, URIs
