use crate::elements::Document;
use crate::format::chromium_pdf::result::{PdfRenderingError, PdfRenderingResult};
use crate::format::html::html_writer::HTMLWriter;
use crate::format::html::to_html::ToHtml;
use crate::references::configuration::keys::INCLUDE_MATHJAX;
use crate::utils::downloads::get_cached_path;
use headless_chrome::protocol::page::PrintToPdfOptions;
use headless_chrome::{Browser, LaunchOptionsBuilder, Tab};
use std::error::Error;
use std::fs::{File, OpenOptions};
use std::io;
use std::io::BufWriter;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::{Duration, Instant};

pub mod result;

/// Renders the document to pdf and returns the resulting bytes
pub fn render_to_pdf(document: Document) -> PdfRenderingResult<Vec<u8>> {
    let mut file_path = PathBuf::from("tmp-document.html");
    file_path = get_cached_path(file_path).with_extension("html");
    let mut mathjax = false;

    if let Some(entry) = document.config.get_entry(INCLUDE_MATHJAX) {
        if entry.get().as_bool() == Some(true) {
            mathjax = true;
        }
    }

    let handle = thread::spawn({
        let file_path = file_path.clone();
        move || {
            log::debug!("Rendering html...");
            let writer = BufWriter::new(
                OpenOptions::new()
                    .create(true)
                    .write(true)
                    .open(file_path)?,
            );
            let mut html_writer = HTMLWriter::new(Box::new(writer));
            document.to_html(&mut html_writer);
            log::debug!("Successfully rendered temporary html file!");
            html_writer.flush()
        }
    });
    log::debug!("Starting browser...");
    let browser = Browser::new(LaunchOptionsBuilder::default().build().unwrap())?;
    let tab = browser.wait_for_initial_tab()?;
    handle.join().unwrap()?;
    tab.navigate_to(format!("file:///{}", file_path.to_string_lossy()).as_str())?;
    tab.wait_until_navigated()?;

    if mathjax {
        wait_for_mathjax(&tab, Duration::from_secs(60))?;
    }
    let result = tab.print_to_pdf(Some(get_pdf_options()))?;

    Ok(result)
}

/// Waits for mathjax to be finished
fn wait_for_mathjax(tab: &Tab, timeout: Duration) -> PdfRenderingResult<()> {
    let start = Instant::now();
    log::debug!("Waiting for mathjax...");
    loop {
        let result = tab
            .evaluate(
                "\
            if (window.MathJax)\
                !!window.MathJax;\
            else \
                false;\
            ",
                true,
            )?
            .value;
        match result {
            Some(value) => {
                if value.is_boolean() && value.as_bool().unwrap() == true {
                    break;
                } else {
                    if start.elapsed() >= timeout {
                        return Err(PdfRenderingError::Timeout);
                    }
                    thread::sleep(Duration::from_millis(10))
                }
            }
            None => {
                if start.elapsed() >= timeout {
                    return Err(PdfRenderingError::Timeout);
                }
                thread::sleep(Duration::from_millis(10))
            }
        }
    }
    Ok(())
}

fn get_pdf_options() -> PrintToPdfOptions {
    PrintToPdfOptions {
        landscape: None,
        display_header_footer: Some(false),
        print_background: Some(true),
        scale: None,
        paper_width: None,
        paper_height: None,
        margin_top: None,
        margin_bottom: None,
        margin_left: None,
        margin_right: None,
        page_ranges: None,
        ignore_invalid_page_ranges: None,
        header_template: None,
        footer_template: None,
        prefer_css_page_size: None,
    }
}
