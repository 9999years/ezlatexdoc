use serde_derive::Deserialize;
use toml;

use crate::error::{Error, Result as EzResult};
use crate::lex::{lex_document, Chunk, CommentKind};

#[derive(Debug, Clone)]
pub enum Node<'a> {
    Source(&'a str),
    Directives(Directives),
    Documentation(String),
    PreservedComment(String),
    Comment,
}

#[derive(Deserialize, Debug, Clone)]
pub struct Directives {
    pub src_output: Option<String>,
    pub doc_output: Option<String>,
}

pub fn parse_document<'input>(input: &'input str) -> EzResult<Vec<Node<'input>>> {
    let chunks = lex_document(input)?;
    let mut ret = Vec::with_capacity(chunks.len());

    for chunk in chunks {
        ret.push(match chunk {
            Chunk::Source(src) => Node::Source(src),
            Chunk::Comment(comment) => match comment.kind {
                CommentKind::Directive => Node::Directives(
                    toml::from_str(&comment.text).map_err(Error::DirectivesParseToml)?,
                ),
                CommentKind::Documentation => Node::Documentation(comment.text),
                CommentKind::Preserved => Node::PreservedComment(comment.text),
                CommentKind::Eol => Node::Comment,
            },
        });
    }

    Ok(ret)
}
