use crate::elements::{Anchor, BoldText, ItalicText, Line, List, ListItem, PlainText, TextLine};
use crate::elements::{Inline, Url};
use bibliographix::bibliography::bib_types::article::Article;
use bibliographix::bibliography::bib_types::book::Book;
use bibliographix::bibliography::bib_types::booklet::Booklet;
use bibliographix::bibliography::bib_types::in_book::InBook;
use bibliographix::bibliography::bib_types::in_collection::InCollection;
use bibliographix::bibliography::bib_types::manual::Manual;
use bibliographix::bibliography::bib_types::misc::Misc;
use bibliographix::bibliography::bib_types::repository::Repository;
use bibliographix::bibliography::bib_types::tech_report::TechReport;
use bibliographix::bibliography::bib_types::thesis::Thesis;
use bibliographix::bibliography::bib_types::unpublished::Unpublished;
use bibliographix::bibliography::bib_types::website::Website;
use bibliographix::bibliography::bib_types::BibliographyType;
use bibliographix::bibliography::bibliography_entry::{
    BibliographyEntry, BibliographyEntryReference,
};

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

macro_rules! url_text {
    ($e:expr) => {
        Inline::Url(Url {
            url: $e,
            description: None,
        })
    };
}

macro_rules! list_item {
    ($e:expr, $k:expr) => {
        ListItem::new(
            Line::Anchor(Anchor {
                inner: Box::new(Line::Text($e)),
                key: $k,
            }),
            0,
            true,
        )
    };
}

const DATE_FORMAT: &str = "%d.%m.%Y";

/// Creates a list from a list of bib items
pub fn create_bib_list(entries: Vec<BibliographyEntryReference>) -> List {
    let mut list = List::new();
    list.ordered = true;

    let mut count = 1;
    for entry in entries {
        entry
            .lock()
            .unwrap()
            .raw_fields
            .insert("ord".to_string(), count.to_string());
        list.add_item(get_item_for_entry(entry));
        count += 1;
    }

    list
}

/// Returns the list item for a bib entry
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
        BibliographyType::Thesis(t) => get_item_for_thesis(&*entry, t),
        BibliographyType::Unpublished(u) => get_item_for_unpublished(&*entry, u),
        BibliographyType::Website(w) => get_item_for_website(&*entry, w),
    }
}

/// Returns the formatted article bib entry
fn get_item_for_article(entry: &BibliographyEntry, a: &Article) -> ListItem {
    let mut text = TextLine::new();
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
        .push(plain_text!(format!(", {}", a.date.format(DATE_FORMAT))));

    if let Some(pages) = a.pages.clone() {
        text.subtext
            .push(plain_text!(format!(", Pages: {}", pages)));
    }
    if let Some(url) = a.url.clone() {
        text.subtext.push(plain_text!(", URL: ".to_string()));
        text.subtext.push(url_text!(url));
    }
    if let Some(notes) = entry.note.clone() {
        text.subtext.push(plain_text!(notes))
    }

    list_item!(text, entry.key())
}

/// Returns a list item for a book entry
fn get_item_for_book(entry: &BibliographyEntry, b: &Book) -> ListItem {
    let mut text = TextLine::new();
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
        .push(plain_text!(format!("on {}", b.date.format(DATE_FORMAT))));
    if let Some(url) = b.url.clone() {
        text.subtext.push(plain_text!(", URL: ".to_string()));
        text.subtext.push(url_text!(url));
    }
    if let Some(notes) = entry.note.clone() {
        text.subtext.push(plain_text!(notes))
    }

    list_item!(text, entry.key())
}

/// Returns the list item for a booklet
fn get_item_for_booklet(entry: &BibliographyEntry, b: &Booklet) -> ListItem {
    let mut text = TextLine::new();
    if let Some(author) = b.author.clone() {
        text.subtext.push(plain_text!(format!("{}. ", author)))
    }
    text.subtext
        .push(plain_text!(format!("\"{}\", Published ", b.title.clone())));
    if let Some(how_pub) = b.how_published.clone() {
        text.subtext.push(plain_text!(format!("as {} ", how_pub)))
    }
    if let Some(date) = b.date {
        text.subtext
            .push(plain_text!(format!("on {}", date.format(DATE_FORMAT))))
    }
    if let Some(notes) = entry.note.clone() {
        text.subtext.push(plain_text!(notes))
    }

    list_item!(text, entry.key())
}

/// Returns the list item for an in book bib entry
fn get_item_for_in_book(entry: &BibliographyEntry, ib: &InBook) -> ListItem {
    let mut text = TextLine::new();
    text.subtext
        .push(plain_text!(format!("{}. ", ib.author.clone())));
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
    if let Some(notes) = entry.note.clone() {
        text.subtext.push(plain_text!(notes))
    }

    list_item!(text, entry.key())
}

/// Returns the list item for an InCollection bib entry
fn get_item_for_in_collection(entry: &BibliographyEntry, ic: &InCollection) -> ListItem {
    let mut text = TextLine::new();
    text.subtext
        .push(plain_text!(format!("{}. ", ic.author.clone())));

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
    if let Some(notes) = entry.note.clone() {
        text.subtext.push(plain_text!(notes))
    }

    list_item!(text, entry.key())
}

/// Returns the list item for a manual
fn get_item_for_manual(entry: &BibliographyEntry, m: &Manual) -> ListItem {
    let mut text = TextLine::new();

    if let Some(author) = m.author.clone() {
        text.subtext.push(plain_text!(format!("{}. ", author)));
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
            .push(plain_text!(format!(" on {}", date.format(DATE_FORMAT))))
    }
    if let Some(notes) = entry.note.clone() {
        text.subtext.push(plain_text!(notes))
    }

    list_item!(text, entry.key())
}

/// Returns the list item for a misc bib entry
fn get_item_for_misc(entry: &BibliographyEntry, m: &Misc) -> ListItem {
    let mut text = TextLine::new();

    if let Some(author) = m.author.clone() {
        text.subtext.push(plain_text!(format!("{}. ", author)));
    }
    if let Some(title) = m.title.clone() {
        text.subtext.push(plain_text!(format!("\"{}\"", title)));
    }
    if let Some(how_pub) = m.how_published.clone() {
        text.subtext.push(plain_text!(format!("as {} ", how_pub)))
    }
    if let Some(date) = m.date {
        text.subtext
            .push(plain_text!(format!("on {}", date.format(DATE_FORMAT))))
    }
    if let Some(url) = m.url.clone() {
        text.subtext.push(plain_text!(format!(", URL: {}", url)));
    }
    if let Some(notes) = entry.note.clone() {
        text.subtext.push(plain_text!(notes))
    }

    list_item!(text, entry.key())
}

/// Returns a list item for a repository bib entry
fn get_item_for_repository(entry: &BibliographyEntry, r: &Repository) -> ListItem {
    let mut text = TextLine::new();

    text.subtext.push(italic_text!(r.title.clone()));
    text.subtext
        .push(plain_text!(format!(" by {}", r.author.clone())));

    if let Some(url) = r.url.clone() {
        text.subtext.push(plain_text!(", URL: ".to_string()));
        text.subtext.push(url_text!(url));
    }
    if let Some(accessed) = r.accessed_at.clone() {
        text.subtext.push(plain_text!(format!(
            "(accessed: {})",
            accessed.format(DATE_FORMAT)
        )))
    }
    if let Some(license) = r.license.clone() {
        text.subtext
            .push(plain_text!(format!(", License: {}", license)))
    }
    if let Some(notes) = entry.note.clone() {
        text.subtext.push(plain_text!(notes))
    }

    list_item!(text, entry.key())
}

/// Returns the list item for the tech report type
fn get_item_for_tech_report(entry: &BibliographyEntry, tr: &TechReport) -> ListItem {
    let mut text = TextLine::new();

    text.subtext
        .push(plain_text!(format!("{}. ", tr.author.clone())));
    text.subtext
        .push(plain_text!(format!("\"{}\"", tr.title.clone())));
    text.subtext
        .push(plain_text!(format!(" by {}", tr.institution.clone())));
    text.subtext
        .push(plain_text!(format!(" on {}", tr.date.format(DATE_FORMAT))));
    if let Some(notes) = entry.note.clone() {
        text.subtext.push(plain_text!(notes))
    }

    list_item!(text, entry.key())
}

/// Returns a list item for a thesis
fn get_item_for_thesis(entry: &BibliographyEntry, t: &Thesis) -> ListItem {
    let mut text = TextLine::new();

    text.subtext
        .push(plain_text!(format!("{}. ", t.author.clone())));
    text.subtext
        .push(plain_text!(format!("\"{}\" ", t.title.clone())));
    text.subtext
        .push(plain_text!(format!("at {}", t.school.clone())));
    text.subtext
        .push(plain_text!(format!(" on {}", t.date.format(DATE_FORMAT))));
    if let Some(notes) = entry.note.clone() {
        text.subtext.push(plain_text!(notes))
    }

    list_item!(text, entry.key())
}

/// Returns the list item for an unpublished bib type
fn get_item_for_unpublished(entry: &BibliographyEntry, u: &Unpublished) -> ListItem {
    let mut text = TextLine::new();

    text.subtext
        .push(plain_text!(format!("{}.", u.author.clone())));
    text.subtext
        .push(plain_text!(format!("\"{}\"", u.title.clone())));
    if let Some(date) = u.date.clone() {
        text.subtext
            .push(plain_text!(format!(" on {}", date.format(DATE_FORMAT))));
    }
    if let Some(notes) = entry.note.clone() {
        text.subtext.push(plain_text!(notes))
    }

    list_item!(text, entry.key())
}

fn get_item_for_website(entry: &BibliographyEntry, w: &Website) -> ListItem {
    let mut text = TextLine::new();

    if let Some(title) = w.title.clone() {
        text.subtext.push(italic_text!(format!("{} - ", title)));
    }
    text.subtext.push(url_text!(w.url.clone()));
    if let Some(author) = w.author.clone() {
        text.subtext.push(bold_text!(format!(" by {}", author)))
    }
    if let Some(accessed) = w.accessed_at.clone() {
        text.subtext.push(plain_text!(format!(
            "(accessed: {})",
            accessed.format(DATE_FORMAT)
        )))
    }
    if let Some(date) = w.date.clone() {
        text.subtext.push(plain_text!(format!(
            ", Published On: {}",
            date.format(DATE_FORMAT)
        )))
    }
    if let Some(notes) = entry.note.clone() {
        text.subtext.push(plain_text!(notes))
    }

    list_item!(text, entry.key())
}
