use std::fmt::Write;
use std::fs::File;
use std::io::Write as IoWrite;

use crate::error::{Error, Result as EzResult};
use crate::parse;
use crate::parse::Node;
use crate::util;

// /// The two output streams -- one stripped of documentation, one only for documentation -- of an
// /// ezlatexdoc run.
// pub struct DocWrite<S, D>
// where
// S: Write,
// D: Write,
// {
// pub src: S,
// pub doc: D,
// }

pub struct Process {
    src: String,
    doc: String,
    src_output: Option<File>,
    doc_output: Option<File>,
}

impl Default for Process {
    fn default() -> Self {
        Process {
            // 10 kb
            src: String::with_capacity(10_000),
            doc: String::with_capacity(10_000),
            src_output: None,
            doc_output: None,
        }
    }
}

impl Process {
    pub fn process_document<'input>(&mut self, input: &'input str) -> EzResult<'input, ()> {
        for node in parse::parse_document(input)? {
            self.process(node)?;
        }
        Ok(())
    }

    pub fn process<'input>(&mut self, node: Node<'input>) -> EzResult<'input, ()> {
        match node {
            Node::Source(src) => write!(self.src, "{}", src).map_err(Error::Format),
            Node::PreservedComment(c) => writeln!(self.src, "% {}", c).map_err(Error::Format),
            Node::Comment => writeln!(self.src, "%").map_err(Error::Format),
            Node::Documentation(doc) => write!(self.doc, "{}", doc).map_err(Error::Format),
            Node::Directives(d) => {
                if let Some(src_filename) = d.src_output {
                    self.src_output = Some(util::open_new(src_filename).map_err(Error::file_open)?);
                }
                if let Some(doc_filename) = d.doc_output {
                    self.doc_output = Some(util::open_new(doc_filename).map_err(Error::file_open)?);
                }
                Ok(())
            }
        }
    }

    pub fn finish<'s, 'a>(&'s self) -> EzResult<'a, ()> {
        write!(
            self.src_output.as_ref().ok_or(Error::NoOutput)?,
            "{}",
            self.src
        )
        .map_err(Error::write)?;
        write!(
            self.doc_output.as_ref().ok_or(Error::NoOutput)?,
            "{}",
            self.doc
        )
        .map_err(Error::write)
    }
}
