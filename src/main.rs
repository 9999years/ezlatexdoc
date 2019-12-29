use std::error::Error;
use std::ffi::OsString;
use std::fs::File;
use std::io;
use std::io::Read;
use std::path::{Path, PathBuf};

use clap::{clap_app, App, ArgMatches};
use structopt::StructOpt;

use ezlatexdoc::{parse, process, util};

#[derive(StructOpt, Debug)]
#[structopt(
    name = "ezlatexdoc",
    about = "A user-friendly alternative to the LaTeX docstrip program.",
    author = "Rebecca Turner <rbt@sent.as>",
    version = "0.0.1"
)]
struct Opt {
    /// Input file
    #[structopt(parse(from_os_str))]
    input: PathBuf,
}

// quick_error! {
// #[derive(Debug)]
// enum MainError {
// InvalidPath {
// display("Invalid path; a Unicode error somewhere. You shouldn't see this.")
// }
// NoInput {
// display("No input file name found. You shouldn't see this.")
// }
// NoOutput {
// display("No stripped input file name found. You shouldn't see this.")
// }
// AlreadyExists(path: OsString) {
// display("Output file {:?} already exists", path)
// }
// Io(err: io::Error) {}
// }
// }

// fn doc_output_file(matches: &ArgMatches<'_>) -> Result<File, MainError> {
// matches
// .value_of("DOC_OUTPUT")
// .map(|s| s.into())
// .ok_or(())
// .or_else(|_| {
// let input = matches.value_of("INPUT").unwrap();
// let as_tex = Path::new(input).with_extension("tex");
// if as_tex.exists() {
// Err(MainError::AlreadyExists(as_tex.into_os_string()))
// } else {
// as_tex
// .to_str()
// .map(|s| s.into())
// .ok_or(MainError::InvalidPath)
// }
// })
// .and_then(|s: String| util::open_new(s).map_err(MainError::Io))
// }

// fn src_output_file(matches: &ArgMatches<'_>) -> Result<File, MainError> {
// matches
// .value_of("SRC_OUTPUT")
// .ok_or(MainError::NoOutput)
// .and_then(|s| util::open_new(s).map_err(MainError::Io))
// }

// fn main() -> Result<(), Box<dyn Error>> {
// let matches = clap_app().get_matches();
// let mut doc_write = DocWrite {
// src: src_output_file(&matches)?,
// doc: doc_output_file(&matches)?,
// };
// let lines = util::file_lines(matches.value_of("INPUT").ok_or(MainError::NoInput)?)?;
// process_lines(lines, &mut doc_write)?;
// Ok(())
// }

fn main() {
    let opt = Opt::from_args();
    let mut reader = util::reader(opt.input).unwrap();
    let mut process = process::Process::default();
    let mut input = String::with_capacity(10_000);
    reader.read_to_string(&mut input).unwrap();
    for node in parse::parse_document(&input).unwrap() {
        process.process(dbg!(node)).unwrap();
    }
    process.finish();
}
