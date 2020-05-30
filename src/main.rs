use markdown_rs::format::html::ToHtml;
use markdown_rs::parser::Parser;
use std::fs::write;
use std::time::Instant;

use std::path::PathBuf;
use structopt::StructOpt;

#[derive(StructOpt, Debug)]
struct Opt {
    #[structopt(parse(from_os_str))]
    input: PathBuf,
    #[structopt(parse(from_os_str))]
    output: PathBuf,
    #[structopt(short, long)]
    format: String,
}

fn main() {
    let opt: Opt = Opt::from_args();
    let start = Instant::now();
    let mut parser = Parser::new_from_file(opt.input.to_str().unwrap().to_string()).unwrap();
    let document = parser.parse();
    println!("Total duration: {:?}", start.elapsed());
    match opt.format.as_str() {
        "html" => write(opt.output.to_str().unwrap(), document.to_html()).unwrap(),
        _ => println!("Unknown format {}", opt.format),
    }
}
