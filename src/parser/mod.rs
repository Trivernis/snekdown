pub(crate) mod block;
pub(crate) mod inline;
pub(crate) mod line;

use self::block::ParseBlock;
use crate::elements::{Document, ImportAnchor};
use crate::references::configuration::keys::{
    IMP_BIBLIOGRAPHY, IMP_CONFIGS, IMP_IGNORE, IMP_STYLESHEETS,
};
use crate::references::configuration::{Configuration, Value};
use bibliographix::bib_manager::BibManager;
use charred::tapemachine::{CharTapeMachine, TapeError, TapeResult};
use colored::*;
use crossbeam_utils::sync::WaitGroup;
use regex::Regex;
use std::collections::HashMap;
use std::fs::{read_to_string, File};
use std::io;
use std::io::{BufRead, BufReader, Cursor};
use std::path::PathBuf;
use std::sync::{Arc, Mutex, RwLock};
use std::thread;

pub type ParseResult<T> = TapeResult<T>;
pub type ParseError = TapeError;

const DEFAULT_IMPORTS: &'static [(&str, &str)] = &[
    ("snekdown.toml", "manifest"),
    ("manifest.toml", "manifest"),
    ("bibliography.toml", "bibliography"),
    ("bibliography2.bib.toml", "bibliography"),
    ("style.css", "stylesheet"),
];

pub struct Parser {
    pub(crate) ctm: CharTapeMachine,
    section_nesting: u8,
    sections: Vec<u8>,
    section_return: Option<u8>,
    path: Option<PathBuf>,
    paths: Arc<Mutex<Vec<PathBuf>>>,
    wg: WaitGroup,
    is_child: bool,
    pub(crate) block_break_at: Vec<char>,
    pub(crate) inline_break_at: Vec<char>,
    pub(crate) document: Document,
    pub(crate) parse_variables: bool,
}

impl Parser {
    /// Creates a new parser from a path
    pub fn new_from_file(path: PathBuf) -> Result<Self, io::Error> {
        let f = File::open(&path)?;
        Ok(Self::create(
            Some(PathBuf::from(path)),
            Arc::new(Mutex::new(Vec::new())),
            false,
            Box::new(BufReader::new(f)),
            BibManager::new(),
        ))
    }

    /// Creates a new parser with text being the markdown text
    pub fn new(text: String, path: Option<PathBuf>) -> Self {
        let text_bytes = text.as_bytes();
        let path = if let Some(inner_path) = path {
            Some(PathBuf::from(inner_path))
        } else {
            None
        };
        Parser::create(
            path,
            Arc::new(Mutex::new(Vec::new())),
            false,
            Box::new(Cursor::new(text_bytes.to_vec())),
            BibManager::new(),
        )
    }

    /// Creates a child parser from string text
    pub fn child(
        text: String,
        path: PathBuf,
        paths: Arc<Mutex<Vec<PathBuf>>>,
        bib_manager: BibManager,
    ) -> Self {
        let text_bytes = text.as_bytes();
        Self::create(
            Some(PathBuf::from(path)),
            paths,
            true,
            Box::new(Cursor::new(text_bytes.to_vec())),
            bib_manager,
        )
    }

    /// Creates a child parser from a file
    pub fn child_from_file(
        path: PathBuf,
        paths: Arc<Mutex<Vec<PathBuf>>>,
        bib_manager: BibManager,
    ) -> Result<Self, io::Error> {
        let f = File::open(&path)?;
        Ok(Self::create(
            Some(PathBuf::from(path)),
            paths,
            true,
            Box::new(BufReader::new(f)),
            bib_manager,
        ))
    }

    fn create(
        path: Option<PathBuf>,
        paths: Arc<Mutex<Vec<PathBuf>>>,
        is_child: bool,
        mut reader: Box<dyn BufRead>,
        bib_manager: BibManager,
    ) -> Self {
        if let Some(path) = path.clone() {
            paths.lock().unwrap().push(path.clone())
        }
        let mut text = String::new();
        reader
            .read_to_string(&mut text)
            .expect("Failed to read file");
        if text.chars().last() != Some('\n') {
            text.push('\n');
        }

        let document = Document::new_with_manager(!is_child, bib_manager);
        Self {
            sections: Vec::new(),
            section_nesting: 0,
            section_return: None,
            path,
            paths,
            wg: WaitGroup::new(),
            is_child,
            ctm: CharTapeMachine::new(text.chars().collect()),
            inline_break_at: Vec::new(),
            block_break_at: Vec::new(),
            document,
            parse_variables: false,
        }
    }

    pub fn set_config(&mut self, config: Configuration) {
        self.document.config = config;
    }

    /// Returns the import paths of the parser
    pub fn get_paths(&self) -> Vec<PathBuf> {
        self.paths.lock().unwrap().clone()
    }

    /// transform an import path to be relative to the current parsers file
    fn transform_path(&mut self, path: String) -> PathBuf {
        let mut path = PathBuf::from(path);

        if !path.is_absolute() {
            if let Some(selfpath) = &self.path {
                if let Some(dir) = selfpath.parent() {
                    path = PathBuf::new().join(dir).join(path);
                }
            }
        }

        path
    }

    /// starts up a new thread to parse the imported document
    fn import_document(&mut self, path: PathBuf) -> ParseResult<Arc<RwLock<ImportAnchor>>> {
        if !path.exists() || !path.is_file() {
            println!(
                "{}",
                format!(
                    "Import of \"{}\" failed: The file doesn't exist.",
                    path.to_str().unwrap()
                )
                .red()
            );
            eprintln!("file {} does not exist", path.to_str().unwrap());
            return Err(self.ctm.assert_error(None));
        }
        {
            let mut paths = self.paths.lock().unwrap();
            if paths.iter().find(|item| **item == path) != None {
                eprintln!(
                    "{}",
                    format!(
                        "Import of \"{}\" failed: Cyclic import.",
                        path.to_str().unwrap()
                    )
                    .yellow()
                );
                return Err(self.ctm.assert_error(None));
            }
            paths.push(path.clone());
        }
        let anchor = Arc::new(RwLock::new(ImportAnchor::new()));
        let anchor_clone = Arc::clone(&anchor);
        let wg = self.wg.clone();
        let paths = Arc::clone(&self.paths);
        let config = self.document.config.clone();
        let bibliography = self.document.bibliography.create_child();

        let _ = thread::spawn(move || {
            let mut parser = Parser::child_from_file(path, paths, bibliography).unwrap();
            parser.set_config(config);
            let document = parser.parse();
            anchor_clone.write().unwrap().set_document(document);

            drop(wg);
        });

        Ok(anchor)
    }

    /// Imports a bibliography toml file
    fn import_bib(&mut self, path: PathBuf) -> ParseResult<()> {
        let f = File::open(path).map_err(|_| self.ctm.err())?;
        self.document
            .bibliography
            .read_bib_file(&mut BufReader::new(f))
            .map_err(|_| self.ctm.err())?;

        Ok(())
    }

    /// Returns the text of an imported text file
    fn import_text_file(&self, path: PathBuf) -> ParseResult<String> {
        read_to_string(path).map_err(|_| self.ctm.err())
    }

    fn import_stylesheet(&mut self, path: PathBuf) -> ParseResult<()> {
        let content = self.import_text_file(path)?;
        self.document.stylesheets.push(content);

        Ok(())
    }

    fn import_manifest(&mut self, path: PathBuf) -> ParseResult<()> {
        let contents = self.import_text_file(path)?;
        let value = contents
            .parse::<toml::Value>()
            .map_err(|_| self.ctm.err())?;
        self.document.config.set_from_toml(&value);

        Ok(())
    }

    /// Imports a path
    fn import(&mut self, path: String, args: &HashMap<String, Value>) -> ImportType {
        let path = self.transform_path(path);
        if let Some(fname) = path
            .file_name()
            .and_then(|f| Some(f.to_str().unwrap().to_string()))
        {
            if let Some(Value::Array(ignore)) = self
                .document
                .config
                .get_entry(IMP_IGNORE)
                .and_then(|e| Some(e.get().clone()))
            {
                let ignore = ignore
                    .iter()
                    .map(|v| v.as_string())
                    .collect::<Vec<String>>();
                if ignore.contains(&fname) {
                    return ImportType::None;
                }
            }
        }
        match args.get("type").map(|e| e.as_string().to_lowercase()) {
            Some(s) if s == "stylesheet".to_string() => {
                ImportType::Stylesheet(self.import_stylesheet(path))
            }
            Some(s) if s == "document".to_string() => {
                ImportType::Document(self.import_document(path))
            }
            Some(s) if s == "bibliography".to_string() => {
                ImportType::Bibliography(self.import_bib(path))
            }
            Some(s) if s == "manifest".to_string() || s == "config" => {
                ImportType::Manifest(self.import_manifest(path))
            }
            _ => {
                lazy_static::lazy_static! {
                    static ref BIB_NAME: Regex = Regex::new(r".*\.bib\.toml$").unwrap();
                }
                if let Some(fname) = path.file_name().and_then(|f| Some(f.to_str().unwrap())) {
                    if BIB_NAME.is_match(fname) {
                        return ImportType::Bibliography(self.import_bib(path));
                    }
                }
                match path.extension().map(|e| e.to_str().unwrap().to_lowercase()) {
                    Some(e) if e == "css" => ImportType::Stylesheet(self.import_stylesheet(path)),
                    Some(e) if e == "toml" => ImportType::Manifest(self.import_manifest(path)),
                    _ => ImportType::Document(self.import_document(path)),
                }
            }
        }
    }

    /// parses the given text into a document
    pub fn parse(&mut self) -> Document {
        self.document.path = if let Some(path) = &self.path {
            Some(path.canonicalize().unwrap().to_str().unwrap().to_string())
        } else {
            None
        };

        while !self.ctm.check_eof() {
            match self.parse_block() {
                Ok(block) => self.document.add_element(block),
                Err(err) => {
                    if self.ctm.check_eof() {
                        break;
                    }
                    eprintln!("{}", err);
                    break;
                }
            }
        }

        let wg = self.wg.clone();
        self.wg = WaitGroup::new();
        if !self.is_child {
            for (path, file_type) in DEFAULT_IMPORTS {
                self.import(
                    path.to_string(),
                    &maplit::hashmap! {"type".to_string() => Value::String(file_type.to_string())},
                );
            }
        }
        wg.wait();
        if !self.is_child {
            self.import_from_config();
        }
        self.document.post_process();
        let document = self.document.clone();
        self.document = Document::new(!self.is_child);

        document
    }

    /// Imports files from the configs import values
    fn import_from_config(&mut self) {
        if let Some(Value::Array(mut imp)) = self
            .document
            .config
            .get_entry(IMP_STYLESHEETS)
            .and_then(|e| Some(e.get().clone()))
        {
            let args =
                maplit::hashmap! {"type".to_string() => Value::String("stylesheet".to_string())};
            while let Some(Value::String(s)) = imp.pop() {
                self.import(s, &args);
            }
        }
        if let Some(Value::Array(mut imp)) = self
            .document
            .config
            .get_entry(IMP_CONFIGS)
            .and_then(|e| Some(e.get().clone()))
        {
            let args = maplit::hashmap! {"type".to_string() => Value::String("config".to_string())};
            while let Some(Value::String(s)) = imp.pop() {
                self.import(s, &args);
            }
        }
        if let Some(Value::Array(mut imp)) = self
            .document
            .config
            .get_entry(IMP_BIBLIOGRAPHY)
            .and_then(|e| Some(e.get().clone()))
        {
            let args =
                maplit::hashmap! {"type".to_string() => Value::String("bibliography".to_string())};
            while let Some(Value::String(s)) = imp.pop() {
                self.import(s, &args);
            }
        }
    }
}

pub(crate) enum ImportType {
    Document(ParseResult<Arc<RwLock<ImportAnchor>>>),
    Stylesheet(ParseResult<()>),
    Bibliography(ParseResult<()>),
    Manifest(ParseResult<()>),
    None,
}
