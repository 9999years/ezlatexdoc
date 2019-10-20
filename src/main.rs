use std::error::Error;
use std::ffi::OsString;
use std::fs::{File, OpenOptions};
use std::io;
use std::path::Path;

use clap::{clap_app, App, ArgMatches};
use quick_error::quick_error;

use ezlatexdoc::process::{process_lines, DocWrite};
mod util;

fn clap_app<'a, 'b>() -> App<'a, 'b> {
    clap_app!(ezlatexdoc =>
        (version: "0.0.1")
        (author: "Rebecca Turner <rbt@sent.as>")
        (about: "A user-friendly alternative to the LaTeX docstrip program.")
        (@arg OUTPUT: -o --output +takes_value "File for doc output. Defaults to STDOUT.")
        (@arg SRC_OUTPUT: -s --source +takes_value
         "File for processed source output. Defaults to the input file with .tex instead of its extension.")
        (@arg INPUT: +required "TeX file to strip of doc comments.")
    )
}

quick_error! {
    #[derive(Debug)]
    enum MainError {
        InvalidPath {
            display("Invalid path; a Unicode error somewhere. You shouldn't see this.")
        }
        NoInput {
            display("No input file. You shouldn't see this.")
        }
        AlreadyExists(path: OsString) {
            display("Output file {:?} already exists", path)
        }
        Io(err: io::Error) {}
    }
}

fn source_output_file(matches: &ArgMatches<'_>) -> Result<File, MainError> {
    matches
        .value_of("SRC_OUTPUT")
        .map(|s| s.into())
        .ok_or(())
        .or_else(|_| {
            let input = matches.value_of("INPUT").unwrap();
            let as_tex = Path::new(input).with_extension("tex");
            if as_tex.exists() {
                Err(MainError::AlreadyExists(as_tex.into_os_string()))
            } else {
                as_tex
                    .to_str()
                    .map(|s| s.into())
                    .ok_or(MainError::InvalidPath)
            }
        })
        .and_then(|s: String| util::open_new(s).map_err(MainError::Io))
}

fn doc_output_file(matches: &ArgMatches<'_>) -> Result<util::Writer, io::Error> {
    matches
        .value_of("OUTPUT")
        .map(|s| OpenOptions::new().write(true).create_new(true).open(s))
        .transpose()
        .map(|o| o.map(|f| util::Writer::File(f)))
        .map(|f| f.unwrap_or_else(|| util::Writer::Stdout(io::stdout())))
}

fn main() -> Result<(), Box<dyn Error>> {
    let matches = clap_app().get_matches();
    let mut doc_write = DocWrite {
        src: source_output_file(&matches)?,
        doc: doc_output_file(&matches)?,
    };
    let lines = util::file_lines(matches.value_of("INPUT").ok_or(MainError::NoInput)?)?;
    process_lines(lines, &mut doc_write)?;
    Ok(())
}
