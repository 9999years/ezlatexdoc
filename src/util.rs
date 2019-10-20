use std::fs::{File, OpenOptions};
use std::io;
use std::io::{BufRead, BufReader, Write};
use std::path::Path;

// Returns an Iterator<Item = io::Result<String>>
pub fn file_lines<P>(path: P) -> io::Result<io::Lines<BufReader<File>>>
where
    P: AsRef<Path>,
{
    Ok(BufReader::new(File::open(path)?).lines())
}

/// Surely there must be an easier way of saying 'either a File or an io::Stdout but the important
/// thing is they both impl Write'???
pub enum Writer {
    File(File),
    Stdout(io::Stdout),
}

impl Write for Writer {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        match self {
            Writer::File(f) => f.write(buf),
            Writer::Stdout(s) => s.write(buf),
        }
    }

    fn flush(&mut self) -> io::Result<()> {
        match self {
            Writer::File(f) => f.flush(),
            Writer::Stdout(s) => s.flush(),
        }
    }
}

pub fn open_new<P>(path: P) -> io::Result<File>
where
    P: AsRef<Path>,
{
    OpenOptions::new().write(true).create_new(true).open(path)
}
