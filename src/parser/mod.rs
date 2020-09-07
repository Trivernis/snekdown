pub(crate) mod block;
pub(crate) mod inline;
pub(crate) mod line;

use self::block::ParseBlock;
use crate::elements::tokens::LB;
use crate::elements::{Document, ImportAnchor};
use crate::references::configuration::keys::{
    IMP_BIBLIOGRAPHY, IMP_CONFIGS, IMP_GLOSSARY, IMP_IGNORE, IMP_STYLESHEETS,
};
use crate::references::configuration::Value;
use charred::tapemachine::{CharTapeMachine, TapeError, TapeResult};
use crossbeam_utils::sync::WaitGroup;
use regex::Regex;
use std::collections::HashMap;
use std::fs::{read_to_string, File};
use std::io::BufReader;
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
    ("glossary.toml", "glossary"),
    ("style.css", "stylesheet"),
];

#[derive(Clone, Debug)]
pub struct ParserOptions {
    pub path: Option<PathBuf>,
    pub paths: Arc<Mutex<Vec<PathBuf>>>,
    pub document: Document,
    pub is_child: bool,
}

impl Default for ParserOptions {
    fn default() -> Self {
        Self {
            path: None,
            paths: Arc::new(Mutex::new(Vec::new())),
            document: Document::new(),
            is_child: false,
        }
    }
}

impl ParserOptions {
    /// Adds a path to the parser options
    pub fn add_path(mut self, path: PathBuf) -> Self {
        self.path = Some(path.clone());
        self.paths.lock().unwrap().push(path);

        self
    }

    /// If external sources should be cached when after downloaded
    pub fn use_cache(self, value: bool) -> Self {
        self.document.downloads.lock().unwrap().use_cache = value;

        self
    }
}

pub struct Parser {
    pub(crate) options: ParserOptions,
    pub(crate) ctm: CharTapeMachine,
    section_nesting: u8,
    sections: Vec<u8>,
    section_return: Option<u8>,
    wg: WaitGroup,
    pub(crate) block_break_at: Vec<char>,
    pub(crate) inline_break_at: Vec<char>,
    pub(crate) parse_variables: bool,
}

impl Parser {
    /// Creates a new parser with the default values given
    pub fn with_defaults(options: ParserOptions) -> Self {
        let text = if let Some(path) = &options.path {
            let mut text = read_to_string(&path).unwrap();
            if text.chars().last() != Some('\n') {
                text.push('\n');
            }

            text
        } else {
            "".to_string()
        };
        Self {
            options,
            sections: Vec::new(),
            section_nesting: 0,
            section_return: None,
            wg: WaitGroup::new(),
            ctm: CharTapeMachine::new(text.chars().collect()),
            inline_break_at: Vec::new(),
            block_break_at: Vec::new(),
            parse_variables: false,
        }
    }

    /// Creates a new child parser
    fn create_child(&self, path: PathBuf) -> Self {
        let mut options = self.options.clone().add_path(path.clone());
        options.document = self.options.document.create_child();
        options.document.path = Some(path.to_str().unwrap().to_string());
        options.is_child = true;

        Self::with_defaults(options)
    }

    /// Returns a string of the current position in the file
    pub(crate) fn get_position_string(&self) -> String {
        let char_index = self.ctm.get_index();
        let text = self.ctm.get_text();
        let mut text_unil = text[..char_index].to_vec();
        let line_number = text_unil.iter().filter(|c| c == &&LB).count();
        text_unil.reverse();
        let mut inline_pos = 0;

        while text_unil[inline_pos] != LB {
            inline_pos += 1;
        }
        if let Some(path) = &self.options.path {
            format!("{}:{}:{}", path.to_str().unwrap(), line_number, inline_pos)
        } else {
            format!("{}:{}", line_number, inline_pos)
        }
    }

    /// transform an import path to be relative to the current parsers file
    fn transform_path(&mut self, path: String) -> PathBuf {
        let mut path = PathBuf::from(path);

        if !path.is_absolute() {
            if let Some(selfpath) = &self.options.path {
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
            log::error!(
                "Import of \"{}\" failed: The file doesn't exist.\n\t--> {}\n",
                path.to_str().unwrap(),
                self.get_position_string(),
            );
            return Err(self.ctm.assert_error(None));
        }
        {
            let mut paths = self.options.paths.lock().unwrap();
            if paths.iter().find(|item| **item == path) != None {
                log::warn!(
                    "Import of \"{}\" failed: Already imported.\n\t--> {}\n",
                    path.to_str().unwrap(),
                    self.get_position_string(),
                );
                return Err(self.ctm.assert_error(None));
            }
            paths.push(path.clone());
        }
        let anchor = Arc::new(RwLock::new(ImportAnchor::new()));
        let anchor_clone = Arc::clone(&anchor);
        let wg = self.wg.clone();
        let mut chid_parser = self.create_child(path.clone());

        let _ = thread::spawn(move || {
            let document = chid_parser.parse();
            anchor_clone.write().unwrap().set_document(document);

            drop(wg);
        });

        Ok(anchor)
    }

    /// Imports a bibliography toml file
    fn import_bib(&mut self, path: PathBuf) -> ParseResult<()> {
        let f = File::open(path).map_err(|_| self.ctm.err())?;
        self.options
            .document
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
        self.options.document.stylesheets.push(
            self.options
                .document
                .downloads
                .lock()
                .unwrap()
                .add_download(path.to_str().unwrap().to_string()),
        );

        Ok(())
    }

    fn import_manifest(&mut self, path: PathBuf) -> ParseResult<()> {
        let contents = self.import_text_file(path)?;
        let value = contents
            .parse::<toml::Value>()
            .map_err(|_| self.ctm.err())?;
        self.options.document.config.set_from_toml(&value);

        Ok(())
    }

    /// Imports a glossary
    fn import_glossary(&self, path: PathBuf) -> ParseResult<()> {
        let contents = self.import_text_file(path)?;
        let value = contents
            .parse::<toml::Value>()
            .map_err(|_| self.ctm.err())?;
        self.options
            .document
            .glossary
            .lock()
            .unwrap()
            .assign_from_toml(value)
            .unwrap_or_else(|e| log::error!("{}", e));

        Ok(())
    }

    /// Imports a path
    fn import(&mut self, path: String, args: &HashMap<String, Value>) -> ImportType {
        log::debug!(
            "Importing file {}\n\t--> {}\n",
            path,
            self.get_position_string()
        );
        let path = self.transform_path(path);
        if !path.exists() {
            log::error!(
                "Import of \"{}\" failed: The file doesn't exist.\n\t--> {}\n",
                path.to_str().unwrap(),
                self.get_position_string(),
            );
            return ImportType::None;
        }
        if let Some(fname) = path
            .file_name()
            .and_then(|f| Some(f.to_str().unwrap().to_string()))
        {
            if let Some(Value::Array(ignore)) = self
                .options
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
            Some(s) if s == "glossary".to_string() => {
                ImportType::Glossary(self.import_glossary(path))
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
        self.options.document.path = if let Some(path) = &self.options.path {
            Some(path.canonicalize().unwrap().to_str().unwrap().to_string())
        } else {
            None
        };

        while !self.ctm.check_eof() {
            match self.parse_block() {
                Ok(block) => self.options.document.add_element(block),
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
        if !self.options.is_child {
            for (path, file_type) in DEFAULT_IMPORTS {
                if self.transform_path(path.to_string()).exists() {
                    self.import(
                        path.to_string(),
                        &maplit::hashmap! {"type".to_string() => Value::String(file_type.to_string())},
                    );
                }
            }
        }
        wg.wait();
        if !self.options.is_child {
            self.import_from_config();
        }
        self.options.document.post_process();
        let document = std::mem::replace(&mut self.options.document, Document::new());

        document
    }

    pub fn get_paths(&self) -> Vec<PathBuf> {
        self.options.paths.lock().unwrap().clone()
    }

    /// Imports files from the configs import values
    fn import_from_config(&mut self) {
        if let Some(Value::Array(mut imp)) = self
            .options
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
            .options
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
            .options
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

        if let Some(Value::Array(mut imp)) = self
            .options
            .document
            .config
            .get_entry(IMP_GLOSSARY)
            .and_then(|e| Some(e.get().clone()))
        {
            let args =
                maplit::hashmap! {"type".to_string() => Value::String("glossary".to_string())};
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
    Glossary(ParseResult<()>),
    None,
}
