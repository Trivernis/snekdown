use crate::elements::Inline;
use crate::elements::{BoldText, ItalicText, Line, List, ListItem, PlainText, TextLine};
use bibliographix::bibliography::bib_types::article::Article;
use bibliographix::bibliography::bib_types::BibliographyType;
use bibliographix::bibliography::bibliography_entry::{
    BibliographyEntry, BibliographyEntryReference,
};
use std::sync::MutexGuard;

macro_rules! plain_text {
    ($e:expr) => {
        Inline::Plain(PlainText { value: $e })
    };
}

macro_rules! bold_text {
    ($e:expr) => {
        Inline::Bold(BoldText {
            value: vec![Inline::Plain(PlainText { value: $e })],
        })
    };
}

macro_rules! italic_text {
    ($e:expr) => {
        Inline::Italic(ItalicText {
            value: vec![Inline::Plain(PlainText { value: $e })],
        })
    };
}

fn create_bib_list(entries: Vec<BibliographyEntryReference>) -> List {
    let mut list = List::new();

    for entry in entries {
        list.add_item(get_item_for_entry(entry));
    }

    list
}

fn get_item_for_entry(entry: BibliographyEntryReference) -> ListItem {
    let entry = entry.lock().unwrap();

    match &entry.bib_type {
        BibliographyType::Article(a) => get_item_for_article(&*entry, a),
        _ => unimplemented!(),
    }
}

/// Returns the formatted article bib entry
fn get_item_for_article(entry: &BibliographyEntry, a: &Article) -> ListItem {
    let mut text = TextLine::new();
    text.subtext.push(bold_text!(entry.key().clone()));
    text.subtext
        .push(plain_text!(format!(": {}.", a.author.clone())));
    text.subtext
        .push(plain_text!(format!("\"{}\"", a.title.clone()).to_string()));
    text.subtext.push(plain_text!("In: ".to_string()));
    text.subtext.push(italic_text!(a.journal.clone()));

    if let Some(volume) = a.volume.clone() {
        text.subtext
            .push(italic_text!(format!(", {}", volume).to_string()))
    }
    if let Some(number) = a.number.clone() {
        text.subtext
            .push(plain_text!(format!(", Number: {}", number)));
    }
    text.subtext
        .push(plain_text!(format!(", {}", a.date.format("%d.%m.%y"))));

    if let Some(pages) = a.pages.clone() {
        text.subtext
            .push(plain_text!(format!(", Pages: {}", pages).to_string()));
    }
    if let Some(url) = a.url.clone() {
        text.subtext
            .push(plain_text!(format!("URL: {}", url).to_string()));
    }
    ListItem::new(Line::Text(text), 0, true)
}
