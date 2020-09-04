use crate::elements::Inline;
use crate::elements::{BoldText, ItalicText, Line, List, ListItem, PlainText, TextLine};
use bibliographix::bibliography::bib_types::article::Article;
use bibliographix::bibliography::bib_types::book::Book;
use bibliographix::bibliography::bib_types::booklet::Booklet;
use bibliographix::bibliography::bib_types::in_book::InBook;
use bibliographix::bibliography::bib_types::in_collection::InCollection;
use bibliographix::bibliography::bib_types::manual::Manual;
use bibliographix::bibliography::bib_types::misc::Misc;
use bibliographix::bibliography::bib_types::repository::Repository;
use bibliographix::bibliography::bib_types::tech_report::TechReport;
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
        BibliographyType::Book(b) => get_item_for_book(&*entry, b),
        BibliographyType::Booklet(b) => get_item_for_booklet(&*entry, b),
        BibliographyType::InBook(ib) => get_item_for_in_book(&*entry, ib),
        BibliographyType::InCollection(ic) => get_item_for_in_collection(&*entry, ic),
        BibliographyType::Manual(m) => get_item_for_manual(&*entry, m),
        BibliographyType::Misc(m) => get_item_for_misc(&*entry, m),
        BibliographyType::Repository(r) => get_item_for_repository(&*entry, r),
        BibliographyType::TechReport(tr) => get_item_for_tech_report(&*entry, tr),
        _ => unimplemented!(),
    }
}

/// Returns the formatted article bib entry
fn get_item_for_article(entry: &BibliographyEntry, a: &Article) -> ListItem {
    let mut text = TextLine::new();
    text.subtext
        .push(bold_text!(format!("{}: ", entry.key().clone())));
    text.subtext
        .push(plain_text!(format!("{}.", a.author.clone())));
    text.subtext
        .push(plain_text!(format!("\"{}\"", a.title.clone())));
    text.subtext.push(plain_text!("In: ".to_string()));
    text.subtext.push(italic_text!(a.journal.clone()));

    if let Some(volume) = a.volume.clone() {
        text.subtext.push(italic_text!(format!(", {}", volume)))
    }
    if let Some(number) = a.number.clone() {
        text.subtext
            .push(plain_text!(format!(", Number: {}", number)));
    }
    text.subtext
        .push(plain_text!(format!(", {}", a.date.format("%d.%m.%y"))));

    if let Some(pages) = a.pages.clone() {
        text.subtext
            .push(plain_text!(format!(", Pages: {}", pages)));
    }
    if let Some(url) = a.url.clone() {
        text.subtext.push(plain_text!(format!(", URL: {}", url)));
    }
    ListItem::new(Line::Text(text), 0, true)
}

/// Returns a list item for a book entry
fn get_item_for_book(entry: &BibliographyEntry, b: &Book) -> ListItem {
    let mut text = TextLine::new();
    text.subtext
        .push(bold_text!(format!("{}: ", entry.key().clone())));
    text.subtext
        .push(plain_text!(format!("{}.", b.author.clone())));
    text.subtext
        .push(plain_text!(format!("\"{}\"", b.title.clone())));

    if let Some(volume) = b.volume.clone() {
        text.subtext.push(plain_text!(format!(", {}", volume)))
    }
    if let Some(edition) = b.edition.clone() {
        text.subtext.push(plain_text!(format!(", {}", edition)))
    }
    if let Some(series) = b.series.clone() {
        text.subtext.push(plain_text!(format!("In: ")));
        text.subtext.push(italic_text!(series))
    }
    text.subtext.push(plain_text!(format!(
        "Published By: {}",
        b.publisher.clone()
    )
    .to_string()));
    text.subtext
        .push(plain_text!(format!("on {}", b.date.format("%d.%m.%y"))));
    if let Some(url) = b.url.clone() {
        text.subtext.push(plain_text!(format!("URL: {}", url)))
    }

    ListItem::new(Line::Text(text), 0, true)
}

/// Returns the list item for a booklet
fn get_item_for_booklet(entry: &BibliographyEntry, b: &Booklet) -> ListItem {
    let mut text = TextLine::new();
    text.subtext
        .push(bold_text!(format!("{}: ", entry.key().clone())));
    if let Some(author) = b.author.clone() {
        text.subtext.push(plain_text!(format!("{}.", author)))
    }
    text.subtext
        .push(plain_text!(format!("\"{}\", Published ", b.title.clone())));
    if let Some(how_pub) = b.how_published.clone() {
        text.subtext.push(plain_text!(format!("as {} ", how_pub)))
    }
    if let Some(date) = b.date {
        text.subtext
            .push(plain_text!(format!("on {}", date.format("%d.%m.%y"))))
    }

    ListItem::new(Line::Text(text), 0, true)
}

/// Returns the list item for an in book bib entry
fn get_item_for_in_book(entry: &BibliographyEntry, ib: &InBook) -> ListItem {
    let mut text = TextLine::new();
    text.subtext
        .push(bold_text!(format!("{}: ", entry.key().clone())));
    text.subtext
        .push(plain_text!(format!("{}.", ib.author.clone())));
    text.subtext
        .push(plain_text!(format!("\"{}\"", ib.title.clone())));
    text.subtext
        .push(plain_text!(format!(" ({})", ib.position.clone())));

    if let Some(volume) = ib.volume.clone() {
        text.subtext.push(plain_text!(format!(", {}", volume)))
    }
    if let Some(edition) = ib.edition.clone() {
        text.subtext.push(plain_text!(format!(", {}", edition)))
    }
    if let Some(series) = ib.series.clone() {
        text.subtext.push(plain_text!("In: ".to_string()));
        text.subtext.push(italic_text!(series))
    }
    text.subtext.push(plain_text!(format!(
        ", Published By: {}",
        ib.publisher.clone()
    )));

    ListItem::new(Line::Text(text), 0, true)
}

/// Returns the list item for an InCollection bib entry
fn get_item_for_in_collection(entry: &BibliographyEntry, ic: &InCollection) -> ListItem {
    let mut text = TextLine::new();
    text.subtext
        .push(bold_text!(format!("{}: ", entry.key().clone())));
    text.subtext
        .push(plain_text!(format!("{}.", ic.author.clone())));

    if let Some(editor) = ic.editor.clone() {
        text.subtext
            .push(plain_text!(format!("(Editor: {})", editor)))
    }
    text.subtext
        .push(plain_text!(format!("\"{}\"", ic.title.clone())));

    if let Some(position) = ic.position.clone() {
        text.subtext.push(plain_text!(format!(" ({})", position)));
    }

    if let Some(volume) = ic.volume.clone() {
        text.subtext.push(plain_text!(format!(", {}", volume)))
    }
    if let Some(edition) = ic.edition.clone() {
        text.subtext.push(plain_text!(format!(", {}", edition)))
    }
    if let Some(series) = ic.series.clone() {
        text.subtext.push(plain_text!("In: ".to_string()));
        text.subtext.push(italic_text!(series))
    }

    ListItem::new(Line::Text(text), 0, true)
}

/// Returns the list item for a manual
fn get_item_for_manual(entry: &BibliographyEntry, m: &Manual) -> ListItem {
    let mut text = TextLine::new();
    text.subtext
        .push(bold_text!(format!("{}: ", entry.key().clone())));

    if let Some(author) = m.author.clone() {
        text.subtext.push(plain_text!(format!("{}.", author)));
    }
    text.subtext
        .push(plain_text!(format!("\"{}\"", m.title.clone())));

    if let Some(edition) = m.edition.clone() {
        text.subtext.push(plain_text!(format!(", {}", edition)));
    }
    if let Some(organization) = m.organization.clone() {
        text.subtext
            .push(plain_text!(format!(", by {}", organization)))
    }
    if let Some(date) = m.date {
        text.subtext
            .push(plain_text!(format!(" on {}", date.format("%d.%m.%y"))))
    }

    ListItem::new(Line::Text(text), 0, true)
}

/// Returns the list item for a misc bib entry
fn get_item_for_misc(entry: &BibliographyEntry, m: &Misc) -> ListItem {
    let mut text = TextLine::new();
    text.subtext
        .push(bold_text!(format!("{}: ", entry.key().clone())));

    if let Some(author) = m.author.clone() {
        text.subtext.push(plain_text!(format!("{}.", author)));
    }
    if let Some(title) = m.title.clone() {
        text.subtext.push(plain_text!(format!("\"{}\"", title)));
    }
    if let Some(how_pub) = m.how_published.clone() {
        text.subtext.push(plain_text!(format!("as {} ", how_pub)))
    }
    if let Some(date) = m.date {
        text.subtext
            .push(plain_text!(format!("on {}", date.format("%d.%m.%y"))))
    }
    if let Some(url) = m.url.clone() {
        text.subtext.push(plain_text!(format!(", URL: {}", url)));
    }

    ListItem::new(Line::Text(text), 0, true)
}

/// Returns a list item for a repository bib entry
fn get_item_for_repository(entry: &BibliographyEntry, r: &Repository) -> ListItem {
    let mut text = TextLine::new();
    text.subtext
        .push(bold_text!(format!("{}: ", entry.key().clone())));

    text.subtext.push(italic_text!(r.title.clone()));
    text.subtext
        .push(plain_text!(format!(" by {}", r.author.clone())));

    if let Some(url) = r.url.clone() {
        text.subtext.push(plain_text!(format!(", URL: {}", url)))
    }
    if let Some(accessed) = r.accessed_at.clone() {
        text.subtext.push(plain_text!(format!(
            "(accessed: {})",
            accessed.format("%d.%m.%y")
        )))
    }
    if let Some(license) = r.license.clone() {
        text.subtext
            .push(plain_text!(format!(", License: {}", license)))
    }

    ListItem::new(Line::Text(text), 0, true)
}

fn get_item_for_tech_report(entry: &BibliographyEntry, tr: &TechReport) -> ListItem {
    let mut text = TextLine::new();
    text.subtext
        .push(bold_text!(format!("{}: ", entry.key().clone())));
    text.subtext
        .push(plain_text!(format!("{}.", tr.author.clone())));
    text.subtext
        .push(plain_text!(format!("\"{}\"", tr.title.clone())));
    text.subtext
        .push(plain_text!(format!("by {}", tr.institution.clone())));
    text.subtext
        .push(plain_text!(format!(" on {}", tr.date.format("%d.%m.%y"))));

    ListItem::new(Line::Text(text), 0, true)
}
