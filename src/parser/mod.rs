/*
 * Snekdown - Custom Markdown flavour and parser
 * Copyright (C) 2021  Trivernis
 * See LICENSE for more information.
 */

pub(crate) mod block;
pub(crate) mod inline;
pub(crate) mod line;

use self::block::ParseBlock;
use crate::elements::tokens::LB;
use crate::elements::{Document, ImportAnchor};
use crate::settings::SettingsError;
use charred::tapemachine::{CharTapeMachine, TapeError};
use crossbeam_utils::sync::WaitGroup;
use regex::Regex;
use std::collections::HashMap;
use std::fmt;
use std::fs::{read_to_string, File};
use std::io::{self, BufReader};
use std::path::PathBuf;
use std::sync::{Arc, Mutex, RwLock};
use std::thread;

pub type ParseResult<T> = Result<T, ParseError>;

#[derive(Debug)]
pub enum ParseError {
    TapeError(TapeError),
    SettingsError(SettingsError),
    IoError(io::Error),
}

impl fmt::Display for ParseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ParseError::TapeError(e) => write!(f, "{}", e),
            ParseError::SettingsError(e) => write!(f, "{}", e),
            ParseError::IoError(e) => write!(f, "IO Error: {}", e),
        }
    }
}

impl From<TapeError> for ParseError {
    fn from(e: TapeError) -> Self {
        Self::TapeError(e)
    }
}

impl From<SettingsError> for ParseError {
    fn from(e: SettingsError) -> Self {
        Self::SettingsError(e)
    }
}

impl From<io::Error> for ParseError {
    fn from(e: io::Error) -> Self {
        Self::IoError(e)
    }
}

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
}

pub struct Parser {
    pub(crate) options: ParserOptions,
    pub(crate) ctm: CharTapeMachine,
    section_nesting: u8,
    sections: Vec<u8>,
    section_anchors: Vec<String>,
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
            text = text.replace("\r\n", "\n");
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
            section_anchors: Vec::new(),
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
        self.get_position_string_for_index(char_index)
    }

    /// Returns a string of the given index position in the file
    fn get_position_string_for_index(&self, char_index: usize) -> String {
        let text = self.ctm.get_text();
        let mut text_unil = text[..char_index].to_vec();
        let line_number = text_unil.iter().filter(|c| c == &&LB).count();
        text_unil.reverse();
        let mut inline_pos = 0;

        while inline_pos < text_unil.len() && text_unil[inline_pos] != LB {
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
            return Err(self.ctm.assert_error(None).into());
        }
        let anchor = Arc::new(RwLock::new(ImportAnchor::new()));
        let anchor_clone = Arc::clone(&anchor);
        let wg = self.wg.clone();
        let mut child_parser = self.create_child(path.clone());

        let _ = thread::spawn(move || {
            let document = child_parser.parse();
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
        read_to_string(path).map_err(ParseError::from)
    }

    fn import_stylesheet(&mut self, path: PathBuf) -> ParseResult<()> {
        self.options.document.stylesheets.push(
            self.options
                .document
                .downloads
                .lock()
                .add_download(path.to_str().unwrap().to_string()),
        );

        Ok(())
    }

    fn import_manifest(&mut self, path: PathBuf) -> ParseResult<()> {
        self.options
            .document
            .config
            .lock()
            .merge(path)
            .map_err(ParseError::from)
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
            .assign_from_toml(value)
            .unwrap_or_else(|e| log::error!("{}", e));

        Ok(())
    }

    /// Imports a path
    fn import(&mut self, path: String, args: &HashMap<String, String>) -> ImportType {
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
            let ignore = &self.options.document.config.lock().imports.ignored_imports;
            if ignore.contains(&fname) {
                return ImportType::None;
            }
        }
        {
            let mut paths = self.options.paths.lock().unwrap();
            if paths.iter().find(|item| **item == path).is_some() {
                log::warn!(
                    "Import of \"{}\" failed: Already imported.\n\t--> {}\n",
                    path.to_str().unwrap(),
                    self.get_position_string(),
                );
                return ImportType::None;
            }
            paths.push(path.clone());
        }
        match args.get("type").cloned() {
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
                    match err {
                        ParseError::TapeError(t) => {
                            log::error!(
                                "Parse Error: {}\n\t--> {}\n",
                                t,
                                self.get_position_string_for_index(t.get_index())
                            )
                        }
                        _ => {
                            log::error!("{}", err)
                        }
                    }
                    break;
                }
            }
        }

        let wg = self.wg.clone();
        self.wg = WaitGroup::new();
        if !self.options.is_child {
            self.import(
                "Manifest.toml".to_string(),
                &maplit::hashmap! {"type".to_string() => "manifest".to_string()},
            );
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
        let config = Arc::clone(&self.options.document.config);

        let mut stylesheets = config.lock().imports.included_stylesheets.clone();
        let args = maplit::hashmap! {"type".to_string() => "stylesheet".to_string()};
        while let Some(s) = stylesheets.pop() {
            self.import(s, &args);
        }

        let mut bibliography = config.lock().imports.included_bibliography.clone();
        let args = maplit::hashmap! {"type".to_string() => "bibliography".to_string()};
        while let Some(s) = bibliography.pop() {
            self.import(s, &args);
        }

        let mut glossaries = config.lock().imports.included_glossaries.clone();

        let args = maplit::hashmap! {"type".to_string() =>"glossary".to_string()};
        while let Some(s) = glossaries.pop() {
            self.import(s, &args);
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
