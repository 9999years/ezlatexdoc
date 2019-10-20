use std::fs::File;
use std::io;
use std::io::{BufRead, BufReader,};
use std::path::Path;
use clap::{clap_app, App, ArgMatches};
use indoc::indoc;

// Returns an Iterator<Item = io::Result<String>>
fn file_lines<P>(path: P) -> io::Result<io::Lines<BufReader<File>>>
where
    P: AsRef<Path>,
{
    Ok(BufReader::new(File::open(path)?).lines())
}

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

// fn source_output_file<'a>(matches: ArgMatches<'a>) -> &'a str {
    // matches.value_of("SRC_OUTPUT").unwrap_or("")
// }

fn main() {
    let matches = clap_app().get_matches();
    dbg!(matches);
}
