use crate::line::{Line, LineKind};
use line;
use std::fs::File;
use std::io;
use std::io::{BufRead, BufReader, Write};
use std::ops::Deref;
use std::path::Path;

// Returns an Iterator<Item = io::Result<String>>
pub fn file_lines<P>(path: P) -> io::Result<io::Lines<BufReader<File>>>
where
    P: AsRef<Path>,
{
    Ok(BufReader::new(File::open(path)?).lines())
}

/// An interface for coupling the two output streams -- one stripped of documentation, one only
/// for documentation -- of an ezlatexdoc run.
pub struct DocWrite<S, D>
where
    S: Write,
    D: Write,
{
    pub src: S,
    pub doc: D,
}

pub fn process<S, D>(
    input: impl Iterator<Item = io::Result<String>>,
    output: &mut DocWrite<S, D>,
) -> io::Result<()>
where
    S: Write,
    D: Write,
{
    for line_string in input {
        // We need a binding for this to prevent it from being dropped.
        let ok_line_string = line_string?;
        let line: Line = ok_line_string.deref().into();
        if line.should_discard() {
            continue;
        }
        match line.kind {
            // Ignore directives.
            LineKind::Directive => {}
            // Write documentation to the doc stream.
            LineKind::Documentation => output.doc.write_all(line.processed.as_bytes())?,
            // Write commends and source to the stripped source stream.
            LineKind::Comment | LineKind::Source => {
                output.src.write_all(line.processed.as_bytes())?
            }
        }
    }
    Ok(())
}
