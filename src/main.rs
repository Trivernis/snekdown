use colored::Colorize;
use env_logger::Env;
use log::{Level, LevelFilter};
use notify::{watcher, RecursiveMode, Watcher};
use snekdown::elements::Document;
use snekdown::format::html::html_writer::HTMLWriter;
use snekdown::format::html::to_html::ToHtml;
use snekdown::parser::ParserOptions;
use snekdown::settings::Settings;
use snekdown::utils::caching::CacheStorage;
use snekdown::Parser;
use std::fs::{File, OpenOptions};
use std::io::{BufWriter, Write};
use std::path::PathBuf;
use std::process::exit;
use std::sync::mpsc::channel;
use std::time::{Duration, Instant};
use structopt::StructOpt;

#[derive(StructOpt, Debug, Clone)]
struct Opt {
    #[structopt(subcommand)]
    sub_command: SubCommand,
}

#[derive(StructOpt, Debug, Clone)]
#[structopt()]
enum SubCommand {
    /// Watch the document and its imports and render on change.
    Watch(WatchOptions),

    /// Parse and render the document.
    Render(RenderOptions),

    /// Initializes the project with default settings
    Init,

    /// Clears the cache directory
    ClearCache,
}

#[derive(StructOpt, Debug, Clone)]
#[structopt()]
struct RenderOptions {
    /// Path to the input file
    #[structopt(parse(from_os_str))]
    input: PathBuf,

    /// Path for the output file
    #[structopt(parse(from_os_str))]
    output: PathBuf,

    /// the output format
    #[structopt(short, long, default_value = "html")]
    format: String,
}

#[derive(StructOpt, Debug, Clone)]
#[structopt()]
struct WatchOptions {
    /// The amount of time in milliseconds to wait after changes before rendering
    #[structopt(long, default_value = "500")]
    debounce: u64,

    #[structopt(flatten)]
    render_options: RenderOptions,
}

fn main() {
    let opt: Opt = Opt::from_args();
    env_logger::Builder::from_env(Env::default().filter_or("SNEKDOWN_LOG", "info"))
        .filter_module("reqwest", LevelFilter::Warn)
        .filter_module("hyper", LevelFilter::Warn)
        .filter_module("mio", LevelFilter::Warn)
        .filter_module("want", LevelFilter::Warn)
        .format(|buf, record| {
            let color = get_level_style(record.level());
            writeln!(
                buf,
                "{}: {}",
                record
                    .level()
                    .to_string()
                    .to_lowercase()
                    .as_str()
                    .color(color),
                record.args()
            )
        })
        .init();

    match &opt.sub_command {
        SubCommand::Render(opt) => {
            let _ = render(&opt);
        }
        SubCommand::Watch(opt) => watch(&opt),
        SubCommand::ClearCache => {
            let cache = CacheStorage::new();
            cache.clear().expect("Failed to clear cache");
        }
        SubCommand::Init => init(),
    };
}

fn get_level_style(level: Level) -> colored::Color {
    match level {
        Level::Trace => colored::Color::Magenta,
        Level::Debug => colored::Color::Blue,
        Level::Info => colored::Color::Green,
        Level::Warn => colored::Color::Yellow,
        Level::Error => colored::Color::Red,
    }
}

fn init() {
    let settings = Settings::default();
    let settings_string = toml::to_string_pretty(&settings).unwrap();
    let manifest_path = PathBuf::from("Manifest.toml");
    let bibliography_path = PathBuf::from("Bibliography.toml");
    let glossary_path = PathBuf::from("Glossary.toml");

    if !manifest_path.exists() {
        let mut file = OpenOptions::new()
            .create(true)
            .write(true)
            .truncate(true)
            .open("Manifest.toml")
            .unwrap();
        file.write_all(settings_string.as_bytes()).unwrap();
        file.flush().unwrap();
    }
    if !bibliography_path.exists() {
        File::create("Bibliography.toml".to_string()).unwrap();
    }
    if !glossary_path.exists() {
        File::create("Glossary.toml".to_string()).unwrap();
    }
}

/// Watches a file with all of its imports and renders on change
fn watch(opt: &WatchOptions) {
    let parser = render(&opt.render_options);
    let (tx, rx) = channel();
    let mut watcher = watcher(tx, Duration::from_millis(opt.debounce)).unwrap();

    for path in parser.get_paths() {
        watcher.watch(path, RecursiveMode::NonRecursive).unwrap();
    }
    while let Ok(_) = rx.recv() {
        println!("---");
        let parser = render(&opt.render_options);
        for path in parser.get_paths() {
            watcher.watch(path, RecursiveMode::NonRecursive).unwrap();
        }
    }
}

/// Renders the document to the output path
fn render(opt: &RenderOptions) -> Parser {
    if !opt.input.exists() {
        log::error!(
            "The input file {} could not be found",
            opt.input.to_str().unwrap()
        );

        exit(1)
    }
    let start = Instant::now();

    let mut parser = Parser::with_defaults(ParserOptions::default().add_path(opt.input.clone()));
    let document = parser.parse();

    log::info!("Parsing + Processing took: {:?}", start.elapsed());
    let start_render = Instant::now();

    let file = OpenOptions::new()
        .read(true)
        .write(true)
        .truncate(true)
        .create(true)
        .open(&opt.output)
        .unwrap();
    let writer = BufWriter::new(file);

    render_format(opt, document, writer);
    log::info!("Rendering took: {:?}", start_render.elapsed());
    log::info!("Total: {:?}", start.elapsed());

    parser
}

#[cfg(not(feature = "pdf"))]
fn render_format(opt: &RenderOptions, document: Document, writer: BufWriter<File>) {
    match opt.format.as_str() {
        "html" => render_html(document, writer),
        _ => log::error!("Unknown format {}", opt.format),
    }
}

#[cfg(feature = "pdf")]
fn render_format(opt: &RenderOptions, document: Document, writer: BufWriter<File>) {
    match opt.format.as_str() {
        "html" => render_html(document, writer),
        "pdf" => render_pdf(document, writer),
        _ => log::error!("Unknown format {}", opt.format),
    }
}

fn render_html(document: Document, writer: BufWriter<File>) {
    let mut writer = HTMLWriter::new(Box::new(writer));
    document.to_html(&mut writer).unwrap();
    writer.flush().unwrap();
}

#[cfg(feature = "pdf")]
fn render_pdf(document: Document, mut writer: BufWriter<File>) {
    use snekdown::format::chromium_pdf::render_to_pdf;

    let result = render_to_pdf(document).expect("Failed to render pdf!");
    writer.write_all(&result).unwrap();
    writer.flush().unwrap();
}
