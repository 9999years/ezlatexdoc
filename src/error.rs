use std::error;
use std::fmt;
use std::io;

use nom;
use thiserror::{private::AsDynError, Error};
use toml::de::Error as TomlError;

#[derive(Error, Debug, PartialEq)]
pub enum Error<'a> {
    // #[snafu(display("Unable to read configuration from {}: {}", path.display(), source))]
    // ReadConfiguration { source: io::Error, path: PathBuf },

    // #[snafu(display("Unable to write result to {}: {}", path.display(), source))]
    // WriteResult { source: io::Error, path: PathBuf },
    #[error("Failed to parse document: {0:#?}")]
    Lex(NomError<'a>),

    #[error("Failed to parse directives as TOML: {0:#?}")]
    DirectivesParseToml(#[from] TomlError),

    #[error("Failed to open file: {0:#?}")]
    FileOpen(#[source] Box<IoError>),

    #[error("Failed to write file: {0:#?}")]
    Write(#[source] Box<IoError>),

    #[error("Failed to format: {0:#?}")]
    Format(#[from] fmt::Error),

    #[error("No src_output or doc_output files provided")]
    NoOutput,
}

impl<'a> Error<'a> {
    pub fn file_open(err: io::Error) -> Error<'a> {
        Error::FileOpen(Box::new(IoError(err)))
    }

    pub fn write(err: io::Error) -> Error<'a> {
        Error::Write(Box::new(IoError(err)))
    }
}

pub type Result<'a, T, E = Error<'a>> = std::result::Result<T, E>;

pub type NomError<'input> = nom::Err<(&'input str, nom::error::ErrorKind)>;

#[derive(Debug)]
pub struct IoError(io::Error);

impl IoError {
    fn kind(&self) -> io::ErrorKind {
        self.0.kind()
    }
}

impl PartialEq for IoError {
    fn eq(&self, rhs: &Self) -> bool {
        self.kind() == rhs.kind()
    }
}

impl AsDynError for IoError {
    fn as_dyn_error(&self) -> &(dyn error::Error + 'static) {
        &self.0
    }
}
