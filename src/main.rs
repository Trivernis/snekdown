use snekdown::format::html::ToHtml;
use snekdown::Parser;
use std::fs::write;
use std::time::Instant;

use colored::Colorize;
use std::path::PathBuf;
use structopt::StructOpt;

#[derive(StructOpt, Debug)]
struct Opt {
    #[structopt(parse(from_os_str))]
    input: PathBuf,
    #[structopt(parse(from_os_str))]
    output: PathBuf,
    #[structopt(short, long, default_value = "html")]
    format: String,
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
    let start = Instant::now();
    let mut parser = Parser::new_from_file(opt.input.to_str().unwrap().to_string()).unwrap();
    let document = parser.parse();
    println!(
        "{}",
        format!("Parsing took:     {:?}", start.elapsed()).italic()
    );
    let start_render = Instant::now();
    match opt.format.as_str() {
        "html" => write(opt.output.to_str().unwrap(), document.to_html()).unwrap(),
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
}
