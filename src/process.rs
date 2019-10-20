use crate::line::{Line, LineKind};
use std::io;
use std::io::Write;
use std::ops::Deref;

/// The two output streams -- one stripped of documentation, one only for documentation -- of an
/// ezlatexdoc run.
pub struct DocWrite<S, D>
where
    S: Write,
    D: Write,
{
    pub src: S,
    pub doc: D,
}

fn write_line<W: Write>(writer: &mut W, line: &Line) -> io::Result<()> {
    writer.write_all(line.processed.as_bytes())?;
    writer.write_all(b"\n")
}

pub fn process_lines<S, D>(
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
            LineKind::Documentation => write_line(&mut output.doc, &line)?,
            // Write commends and source to the stripped source stream.
            LineKind::Comment | LineKind::Source => {
                write_line(&mut output.src, &line)?
            }
        }
    }
    Ok(())
}
