use std::iter;
use std::vec;

use nom::IResult;
use nom::{
    branch::alt,
    bytes::complete::tag,
    character::complete,
    character::complete::{anychar, none_of, not_line_ending, one_of},
    combinator::{map, not, recognize, value},
    multi::{many0, many1, separated_nonempty_list},
    sequence::{pair, preceded},
};

const DIRECTIVE_TAG: &str = "%%%";
const DOC_TAG: &str = "%%";
const PRESERVED_COMMENT_TAG: &str = "%!";

#[derive(Clone, Debug, PartialEq)]
pub enum Chunk<'a> {
    Comment(Comment<String>),
    Source(&'a str),
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum CommentKind {
    Directive,
    Documentation,
    Preserved,
    Eol,
}

#[derive(Clone, Debug, PartialEq)]
pub struct Comment<T: Clone + PartialEq> {
    pub text: T,
    pub kind: CommentKind,
}

impl Comment<&str> {
    pub fn to_owned(&self) -> Comment<String> {
        Comment {
            text: String::from(self.text),
            kind: self.kind,
        }
    }
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

// fn plain_eol_comment(input: &str) -> IResult<&str, Comment<&str>> {
// preceded(not(special_comment), eol_comment)(input)
// }

fn eol_comment(input: &str) -> IResult<&str, Comment<&str>> {
    map(
        preceded(complete::char('%'), not_line_ending),
        |comment_text| Comment {
            text: comment_text,
            kind: CommentKind::Eol,
        },
    )(input)
}

fn comment_tag(input: &str) -> IResult<&str, CommentKind> {
    alt((
        value(CommentKind::Directive, tag(DIRECTIVE_TAG)),
        value(CommentKind::Documentation, tag(DOC_TAG)),
        value(CommentKind::Preserved, tag(PRESERVED_COMMENT_TAG)),
    ))(input)
}

fn special_comment(input: &str) -> IResult<&str, Comment<&str>> {
    map(pair(comment_tag, not_line_ending), |(kind, text)| Comment {
        kind,
        text,
    })(input)
}

fn collapse_comments(comments: Vec<Comment<&str>>) -> Vec<Comment<String>> {
    if comments.is_empty() {
        return Vec::with_capacity(0);
    }

    let mut ret = Vec::with_capacity(comments.len());
    // Safety: `comments` is non-empty
    let mut last = comments.first().unwrap().to_owned();
    for comment in comments.iter().skip(1) {
        if comment.kind == last.kind {
            last.text.push_str(comment.text);
        } else {
            ret.push(last);
            last = comment.to_owned();
        }
    }
    ret.push(last);
    ret
}

fn special_comment_block(input: &str) -> IResult<&str, Vec<Comment<String>>> {
    map(
        separated_nonempty_list(complete::char('\n'), special_comment),
        collapse_comments,
    )(input)
}

fn eol_comment_block(input: &str) -> IResult<&str, Vec<Comment<String>>> {
    map(
        separated_nonempty_list(complete::char('\n'), eol_comment),
        collapse_comments,
    )(input)
}

fn sol_comment_block(input: &str) -> IResult<&str, Vec<Comment<String>>> {
    preceded(
        pair(complete::char('\n'), one_of(" \t")),
        alt((
            special_comment_block,
            eol_comment_block, // An EOL comment at the start of the line.
        )),
    )(input)
}

fn any_comment(input: &str) -> IResult<&str, Vec<Chunk>> {
    map(alt((sol_comment_block, eol_comment_block)), |comments| {
        comments.iter().cloned().map(Chunk::Comment).collect()
    })(input)
}

pub fn parse<'input>(input: &'input str) -> IResult<&'input str, Vec<Chunk<'input>>> {
    enum OneOrMore<'a> {
        One(iter::Once<Chunk<'a>>),
        More(vec::IntoIter<Chunk<'a>>),
    };

    impl<'a> Iterator for OneOrMore<'a> {
        type Item = Chunk<'a>;

        fn next(&mut self) -> Option<Self::Item> {
            match self {
                OneOrMore::One(chunk) => chunk.next(),
                OneOrMore::More(chunks) => chunks.next(),
            }
        }
    }

    map(
        many0(alt((
            map(non_comment, |nc| {
                OneOrMore::One(iter::once(Chunk::Source(nc)))
            }),
            map(any_comment, |comments| {
                OneOrMore::More(comments.into_iter())
            }),
        ))),
        |mut chunks_2d| chunks_2d.iter_mut().flatten().collect(),
    )(input)
}

#[cfg(test)]
mod tests {
    use super::*;
    use pretty_assertions::assert_eq;

    #[test]
    fn parse_empty() {
        assert_eq!(Ok(("", vec![])), parse(""));
    }

    #[test]
    fn parse_source_simple() {
        assert_eq!(
            Ok(("", vec![Chunk::Source("lorem ipsum dolor...")])),
            parse("lorem ipsum dolor...")
        );
    }

    #[test]
    fn parse_eol_comment_simple() {
        assert_eq!(
            Ok((
                "",
                vec![
                    Chunk::Source("lorem ipsum dolor..."),
                    Chunk::Comment(Comment {
                        kind: CommentKind::Eol,
                        text: " eol comment (thrown away)".to_string(),
                    }),
                ]
            )),
            parse("lorem ipsum dolor...% eol comment (thrown away)")
        );
    }
}
