/*
 * Snekdown - Custom Markdown flavour and parser
 * Copyright (C) 2021  Trivernis
 * See LICENSE for more information.
 */

use crate::elements::Document;
use crate::format::chromium_pdf::result::{PdfRenderingError, PdfRenderingResult};
use crate::format::html::html_writer::HTMLWriter;
use crate::format::html::to_html::ToHtml;
use crate::settings::Settings;
use crate::utils::caching::CacheStorage;
use bibliographix::Mutex;
use headless_chrome::protocol::page::PrintToPdfOptions;
use headless_chrome::{Browser, Tab};
use std::env;
use std::fs;
use std::fs::OpenOptions;
use std::io::BufWriter;
use std::path::PathBuf;
use std::sync::Arc;
use std::thread;
use std::time::{Duration, Instant};

pub mod result;

/// Renders the document to pdf and returns the resulting bytes
pub fn render_to_pdf(document: Document) -> PdfRenderingResult<Vec<u8>> {
    let cache = CacheStorage::new();
    let mut file_path = PathBuf::from("tmp-document.html");
    file_path = cache.get_file_path(&file_path);

    if !file_path.parent().map(|p| p.exists()).unwrap_or(false) {
        file_path = env::current_dir()?;
        file_path.push(PathBuf::from(".tmp-document.html"))
    }

    let config = document.config.clone();
    let mathjax = config.lock().features.include_mathjax;

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
            let mut html_writer =
                HTMLWriter::new(Box::new(writer), document.config.lock().style.theme.clone());
            document.to_html(&mut html_writer)?;
            log::info!("Successfully rendered temporary html file!");
            html_writer.flush()
        }
    });

    let browser = Browser::default()?;
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

fn get_pdf_options(config: Arc<Mutex<Settings>>) -> PrintToPdfOptions {
    let config = config.lock().pdf.clone();
    PrintToPdfOptions {
        landscape: None,
        display_header_footer: Some(config.display_header_footer),
        print_background: Some(true),
        scale: Some(config.page_scale),
        paper_width: config.page_width,
        paper_height: config.page_height,
        margin_top: config.margin.top,
        margin_bottom: config.margin.bottom,
        margin_left: config.margin.left,
        margin_right: config.margin.right,
        page_ranges: None,
        ignore_invalid_page_ranges: None,
        header_template: config.header_template,
        footer_template: config.footer_template,
        prefer_css_page_size: None,
    }
}
