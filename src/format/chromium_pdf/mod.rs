use crate::elements::Document;
use crate::format::chromium_pdf::result::{PdfRenderingError, PdfRenderingResult};
use crate::format::html::html_writer::HTMLWriter;
use crate::format::html::to_html::ToHtml;
use crate::references::configuration::keys::{
    INCLUDE_MATHJAX, PDF_DISPLAY_HEADER_FOOTER, PDF_FOOTER_TEMPLATE, PDF_HEADER_TEMPLATE,
    PDF_MARGIN_BOTTOM, PDF_MARGIN_LEFT, PDF_MARGIN_RIGHT, PDF_MARGIN_TOP, PDF_PAGE_HEIGHT,
    PDF_PAGE_SCALE, PDF_PAGE_WIDTH,
};
use crate::references::configuration::Configuration;
use crate::utils::downloads::get_cached_path;
use headless_chrome::protocol::page::PrintToPdfOptions;
use headless_chrome::{Browser, LaunchOptionsBuilder, Tab};
use std::fs;
use std::fs::OpenOptions;
use std::io::BufWriter;
use std::path::PathBuf;
use std::thread;
use std::time::{Duration, Instant};

pub mod result;

/// Renders the document to pdf and returns the resulting bytes
pub fn render_to_pdf(document: Document) -> PdfRenderingResult<Vec<u8>> {
    let mut file_path = PathBuf::from(format!("tmp-document.html"));
    file_path = get_cached_path(file_path).with_extension("html");
    let mut mathjax = false;

    if let Some(entry) = document.config.get_entry(INCLUDE_MATHJAX) {
        if entry.get().as_bool() == Some(true) {
            mathjax = true;
        }
    }
    let config = document.config.clone();

    let handle = thread::spawn({
        let file_path = file_path.clone();
        move || {
            log::info!("Rendering html...");
            let writer = BufWriter::new(
                OpenOptions::new()
                    .create(true)
                    .write(true)
                    .truncate(true)
                    .open(file_path)?,
            );
            let mut html_writer = HTMLWriter::new(Box::new(writer));
            document.to_html(&mut html_writer)?;
            log::info!("Successfully rendered temporary html file!");
            html_writer.flush()
        }
    });

    let browser = Browser::new(LaunchOptionsBuilder::default().build().unwrap())?;
    let tab = browser.wait_for_initial_tab()?;
    handle.join().unwrap()?;
    tab.navigate_to(format!("file:///{}", file_path.to_string_lossy()).as_str())?;
    tab.wait_until_navigated()?;

    if mathjax {
        wait_for_mathjax(&tab, Duration::from_secs(60))?;
    }
    log::info!("Rendering pdf...");
    let result = tab.print_to_pdf(Some(get_pdf_options(config)))?;
    log::info!("Removing temporary html...");
    fs::remove_file(file_path)?;

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

fn get_pdf_options(config: Configuration) -> PrintToPdfOptions {
    PrintToPdfOptions {
        landscape: None,
        display_header_footer: config
            .get_entry(PDF_DISPLAY_HEADER_FOOTER)
            .and_then(|value| value.get().as_bool()),
        print_background: Some(true),
        scale: config
            .get_entry(PDF_PAGE_SCALE)
            .and_then(|value| value.get().as_float())
            .map(|value| value as f32),
        paper_width: config
            .get_entry(PDF_PAGE_WIDTH)
            .and_then(|value| value.get().as_float())
            .map(|value| value as f32),
        paper_height: config
            .get_entry(PDF_PAGE_HEIGHT)
            .and_then(|value| value.get().as_float())
            .map(|value| value as f32),
        margin_top: config
            .get_entry(PDF_MARGIN_TOP)
            .and_then(|value| value.get().as_float())
            .map(|f| f as f32),
        margin_bottom: config
            .get_entry(PDF_MARGIN_BOTTOM)
            .and_then(|value| value.get().as_float())
            .map(|f| f as f32),
        margin_left: config
            .get_entry(PDF_MARGIN_LEFT)
            .and_then(|value| value.get().as_float())
            .map(|f| f as f32),
        margin_right: config
            .get_entry(PDF_MARGIN_RIGHT)
            .and_then(|value| value.get().as_float())
            .map(|f| f as f32),
        page_ranges: None,
        ignore_invalid_page_ranges: None,
        header_template: config
            .get_entry(PDF_HEADER_TEMPLATE)
            .map(|value| value.get().as_string()),
        footer_template: config
            .get_entry(PDF_FOOTER_TEMPLATE)
            .map(|value| value.get().as_string()),
        prefer_css_page_size: None,
    }
}
