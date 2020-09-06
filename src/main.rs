use colored::Colorize;
use env_logger::Env;
use log::{Level, LevelFilter};
use notify::{watcher, RecursiveMode, Watcher};
use snekdown::format::html::html_writer::HTMLWriter;
use snekdown::format::html::to_html::ToHtml;
use snekdown::Parser;
use std::fs::OpenOptions;
use std::io::BufWriter;
use std::path::PathBuf;
use std::sync::mpsc::channel;
use std::time::{Duration, Instant};
use structopt::StructOpt;

#[derive(StructOpt, Debug)]
struct Opt {
    /// Path to the input file
    #[structopt(parse(from_os_str))]
    input: PathBuf,

    /// Path for the output file
    #[structopt(parse(from_os_str))]
    output: PathBuf,

    /// the output format
    #[structopt(short, long, default_value = "html")]
    format: String,

    #[structopt(subcommand)]
    sub_command: Option<SubCommand>,
}

#[derive(StructOpt, Debug)]
#[structopt()]
enum SubCommand {
    /// Watch the document and its imports and render on change.
    Watch,

    /// Default. Parse and render the document.
    Render,
}

fn main() {
    let opt: Opt = Opt::from_args();
    env_logger::Builder::from_env(Env::default().filter_or("SNEKDOWN_LOG", "info"))
        .filter_module("reqwest", LevelFilter::Warn)
        .filter_module("hyper", LevelFilter::Warn)
        .filter_module("mio", LevelFilter::Warn)
        .filter_module("want", LevelFilter::Warn)
        .format(|buf, record| {
            use std::io::Write;
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
    if !opt.input.exists() {
        log::error!(
            "The input file {} could not be found",
            opt.input.to_str().unwrap()
        );
        return;
    }

    match &opt.sub_command {
        Some(SubCommand::Render) | None => {
            let _ = render(&opt);
        }
        Some(SubCommand::Watch) => watch(&opt),
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

/// Watches a file with all of its imports and renders on change
fn watch(opt: &Opt) {
    let parser = render(opt);
    let (tx, rx) = channel();
    let mut watcher = watcher(tx, Duration::from_millis(250)).unwrap();
    for path in parser.get_paths() {
        watcher.watch(path, RecursiveMode::NonRecursive).unwrap();
    }
    while let Ok(_) = rx.recv() {
        println!("---");
        let parser = render(opt);
        for path in parser.get_paths() {
            watcher.watch(path, RecursiveMode::NonRecursive).unwrap();
        }
    }
}

/// Renders the document to the output path
fn render(opt: &Opt) -> Parser {
    let start = Instant::now();
    let mut parser = Parser::new_from_file(opt.input.clone()).unwrap();
    let document = parser.parse();

    log::info!("Parsing took:     {:?}", start.elapsed());
    let start_render = Instant::now();
    let file = OpenOptions::new()
        .read(true)
        .write(true)
        .truncate(true)
        .create(true)
        .open(&opt.output)
        .unwrap();
    let writer = BufWriter::new(file);
    match opt.format.as_str() {
        "html" => {
            let mut writer = HTMLWriter::new(Box::new(writer));
            document.to_html(&mut writer).unwrap();
            writer.flush().unwrap();
        }
        _ => log::error!("Unknown format {}", opt.format),
    }
    log::info!("Rendering took:   {:?}", start_render.elapsed());
    log::info!("Total:            {:?}", start.elapsed());

    parser
}
