use nom::IResult;
use nom::{
    branch::alt,
    bytes::complete::tag,
    character::complete,
    character::complete::{anychar, none_of, not_line_ending, one_of},
    combinator::{map, not, recognize, value},
    multi::{many0, many1},
    sequence::{pair, preceded},
};

const DIRECTIVE_TAG: &str = "%%%";
const DOC_TAG: &str = "%%";
const PRESERVED_COMMENT_TAG: &str = "%!";

#[derive(Clone, Debug, PartialEq)]
pub enum Chunk<'a> {
    Comment(Comment<'a>),
    Source(&'a str),
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum CommentKind {
    Directive,
    Documentation,
    PreservedComment,
    EolComment,
}

#[derive(Clone, Debug, PartialEq)]
pub struct Comment<'a> {
    pub text: &'a str,
    pub kind: CommentKind,
}

/// Processes an escaped character; this may be part of or an entire control sequence.
fn escaped(input: &str) -> IResult<&str, &str> {
    recognize(pair(complete::char('\\'), anychar))(input)
}

fn _non_comment(input: &str) -> IResult<&str, &str> {
    alt((recognize(none_of("%\\")), recognize(escaped)))(input)
}

/// Parses the next non-comment sequence; this may include escaped percent signs
fn non_comment(input: &str) -> IResult<&str, &str> {
    recognize(many1(_non_comment))(input)
}

fn plain_eol_comment(input: &str) -> IResult<&str, Comment> {
    preceded(not(special_comment), eol_comment)(input)
}

fn eol_comment(input: &str) -> IResult<&str, Comment> {
    map(
        preceded(complete::char('%'), not_line_ending),
        |comment_text| Comment {
            text: comment_text,
            kind: CommentKind::EolComment,
        },
    )(input)
}

fn comment_tag(input: &str) -> IResult<&str, CommentKind> {
    alt((
        value(CommentKind::Directive, tag(DIRECTIVE_TAG)),
        value(CommentKind::Documentation, tag(DOC_TAG)),
        value(CommentKind::PreservedComment, tag(PRESERVED_COMMENT_TAG)),
    ))(input)
}

fn special_comment(input: &str) -> IResult<&str, Comment> {
    map(pair(comment_tag, not_line_ending), |(kind, text)| Comment {
        kind,
        text,
    })(input)
}

fn sol_comment(input: &str) -> IResult<&str, Comment> {
    preceded(
        pair(complete::char('\n'), one_of(" \t")),
        alt((
            special_comment,
            eol_comment, // An EOL comment at the start of the line.
        )),
    )(input)
}

fn any_comment(input: &str) -> IResult<&str, Chunk> {
    map(alt((sol_comment, eol_comment)), Chunk::Comment)(input)
}

pub fn parse<'input>(input: &'input str) -> IResult<&'input str, Vec<Chunk<'input>>> {
    many0(alt((map(non_comment, Chunk::Source), any_comment)))(input)
}

#[cfg(test)]
mod tests {
    use super::*;
    use pretty_assertions::{assert_eq, assert_ne};

    #[test]
    fn test_parse() {
        assert_eq!(Ok(("", vec![])), parse(""));
    }
}
