use colored::Colorize;
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
    if !opt.input.exists() {
        println!(
            "{}",
            format!(
                "The input file {} could not be found",
                opt.input.to_str().unwrap()
            )
            .red()
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
    println!(
        "{}",
        format!("Parsing took:     {:?}", start.elapsed()).italic()
    );
    let start_render = Instant::now();
    let file = OpenOptions::new()
        .write(true)
        .read(true)
        .open(&opt.output)
        .unwrap();
    let writer = BufWriter::new(file);
    match opt.format.as_str() {
        "html" => {
            let mut writer = HTMLWriter::new(Box::new(writer));
            document.to_html(&mut writer).unwrap()
        }
        _ => println!("Unknown format {}", opt.format),
    }
    println!(
        "{}",
        format!("Rendering took:   {:?}", start_render.elapsed()).italic()
    );
    println!(
        "{}",
        format!("Total:            {:?}", start.elapsed()).italic()
    );

    parser
}
