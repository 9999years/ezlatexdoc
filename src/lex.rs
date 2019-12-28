use std::fmt;
use std::fmt::{Display, Formatter};
use std::iter;
use std::vec;

use nom;
use nom::IResult;
use nom::{
    branch::alt,
    bytes::complete::tag,
    character::complete,
    character::complete::{anychar, line_ending, multispace0, none_of, not_line_ending},
    combinator::{complete, map, not, opt, recognize, value},
    multi::{fold_many0, many1, separated_nonempty_list},
    sequence::{pair, preceded, terminated},
};

use unindent::unindent;

use itertools::Itertools;

const DIRECTIVE_TAG: &str = "%%%";
const DOC_TAG: &str = "%%";
const PRESERVED_COMMENT_TAG: &str = "%!";
const EOL_COMMENT_TAG: char = '%';

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
pub struct Comment<T> {
    pub text: T,
    pub kind: CommentKind,
}

impl<T: Display> Display for Comment<T> {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), fmt::Error> {
        writeln!(f, "{}", self.text)
    }
}

impl Comment<&str> {
    /// Not using the ToOwned trait because it isn't letting me return Comment<String> instead of
    /// Comment<&str> :(
    pub fn to_owned(&self) -> Comment<String> {
        Comment {
            text: String::from(self.text),
            kind: self.kind,
        }
    }

    /// Like `to_owned`, but prepends '\n' to self.text. Why? `unindent` ignores the first line.
    /// (FML).
    pub fn after_newline(&self) -> Comment<String> {
        let mut text_new = String::with_capacity(self.text.len() + 1);
        text_new.push('\n');
        text_new.push_str(self.text);
        Comment {
            text: text_new,
            kind: self.kind,
        }
    }

    pub fn trimmed(&self) -> Comment<String> {
        Comment {
            text: self.text.trim_start().to_string(),
            kind: self.kind,
        }
    }
}

enum Chunks<'a> {
    One(Chunk<'a>),
    More(Vec<Chunk<'a>>),
}

impl<'a> IntoIterator for Chunks<'a> {
    type IntoIter = ChunksIter<'a>;
    type Item = Chunk<'a>; //Self::Iter::Item;

    fn into_iter(self) -> Self::IntoIter {
        match self {
            Chunks::One(c) => ChunksIter::One(iter::once(c)),
            Chunks::More(cs) => ChunksIter::More(cs.into_iter()),
        }
    }
}

enum ChunksIter<'a> {
    One(iter::Once<Chunk<'a>>),
    More(vec::IntoIter<Chunk<'a>>),
}

impl<'a> Iterator for ChunksIter<'a> {
    type Item = Chunk<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        match self {
            ChunksIter::One(chunk) => chunk.next(),
            ChunksIter::More(chunks) => chunks.next(),
        }
    }
}

/// Succeeds if the parser is at the end of input. Otherwise, returns an error.
fn eof(input: &str) -> IResult<&str, ()> {
    not(anychar)(input)
}

fn line_ending_or_eof(input: &str) -> IResult<&str, ()> {
    alt((value((), line_ending), eof))(input)
}

/// Recognizes an escaped character /\\./; this may be part of or an entire control sequence.
/// (Note that /./ in the example regex does not include \n.)
fn escaped(input: &str) -> IResult<&str, &str> {
    recognize(pair(complete::char('\\'), none_of("\r\n")))(input)
}

/// Recognizes as long a sequence of non-comment source code as possible (either a character
/// /[^\\%\n]/, or an escape). Stops parsing when it finds a comment or a newline.
fn non_comment(input: &str) -> IResult<&str, &str> {
    recognize(many1(alt((
        recognize(none_of("%\\\r\n")),
        recognize(escaped),
    ))))(input)
}

/// non_comment wrapped in a chunk.
fn non_comment_chunk<'input>(input: &'input str) -> IResult<&'input str, Chunk<'input>> {
    map(
        recognize(separated_nonempty_list(line_ending, non_comment)),
        Chunk::Source,
    )(input)
}

fn directive_tag(input: &str) -> IResult<&str, CommentKind> {
    value(CommentKind::Directive, tag(DIRECTIVE_TAG))(input)
}

fn documentation_tag(input: &str) -> IResult<&str, CommentKind> {
    value(CommentKind::Documentation, tag(DOC_TAG))(input)
}

fn preserved_tag(input: &str) -> IResult<&str, CommentKind> {
    value(CommentKind::Preserved, tag(PRESERVED_COMMENT_TAG))(input)
}

fn eol_tag(input: &str) -> IResult<&str, CommentKind> {
    value(CommentKind::Eol, complete::char(EOL_COMMENT_TAG))(input)
}

/// Parses a comment tag valid for an inline commennt; this includes preserved and eol tags.
fn inline_comment_tag(input: &str) -> IResult<&str, CommentKind> {
    alt((preserved_tag, eol_tag))(input)
}

/// Parses a comment tag valid *only* at the start of a line; doesn't include tags that are valid
/// both for inline and start-of-line comments.
fn only_sol_comment_tag(input: &str) -> IResult<&str, CommentKind> {
    alt((directive_tag, documentation_tag))(input)
}

/// Parses any comment tag.
fn any_comment_tag(input: &str) -> IResult<&str, CommentKind> {
    alt((only_sol_comment_tag, inline_comment_tag))(input)
}

/// An EOL-comment. Doesn't recognize special comments (e.g. directives or documentation), but will
/// recognize preserved comments.
fn inline_comment(input: &str) -> IResult<&str, Comment<&str>> {
    preceded(
        not(only_sol_comment_tag),
        map(pair(inline_comment_tag, not_line_ending), |(kind, text)| {
            Comment { text, kind }
        }),
    )(input)
}

fn inline_comment_chunk<'input>(input: &'input str) -> IResult<&'input str, Chunk<'input>> {
    map(inline_comment, |c| Chunk::Comment(c.trimmed()))(input)
}

/// Parses any comment.
fn any_comment(input: &str) -> IResult<&str, Comment<&str>> {
    map(pair(any_comment_tag, not_line_ending), |(kind, text)| {
        Comment { kind, text }
    })(input)
}

/// A block of comments. Comment tags may be indented any amount, but non-comment source code is
/// not allowed.
fn any_comment_block(input: &str) -> IResult<&str, Vec<Comment<String>>> {
    map(
        separated_nonempty_list(pair(line_ending, multispace0), any_comment),
        collapse_comments,
    )(input)
}

/// any_comment_block wrapped in `Chunk`s.
fn any_comment_chunk<'input>(input: &'input str) -> IResult<&'input str, Vec<Chunk<'input>>> {
    map(any_comment_block, |comments| {
        comments.iter().cloned().map(Chunk::Comment).collect()
    })(input)
}

/// Collapses adjacent comments of the same `kind` into one comment with all the text concatenated
/// and unindented.
fn collapse_comments(comments: Vec<Comment<&str>>) -> Vec<Comment<String>> {
    match comments.len() {
        0 => Vec::with_capacity(0),
        1 => vec![comments[0].trimmed()],
        _ => comments
            .into_iter()
            .group_by(|c| c.kind)
            .into_iter()
            .map(|(kind, mut group)| Comment {
                kind,
                text: unindent(&format!("\n{}", &group.join(""))),
            })
            .collect(),
    }
}

fn parse_document_fragment<'input>(
    input: &'input str,
) -> IResult<&'input str, (Chunks<'input>, Option<Chunks<'input>>)> {
    let non_comment = map(non_comment_chunk, Chunks::One);
    let inline_comment = map(inline_comment_chunk, Chunks::One);
    let any_comments = map(any_comment_chunk, Chunks::More);

    alt((
        // Non-comment source followed by an optional inline comment and a line-end.
        terminated(pair(non_comment, opt(inline_comment)), line_ending_or_eof),
        // A block of sol-comments; we map it to a tuple to satisfy the type-constraint from the
        // other branch.
        map(terminated(any_comments, line_ending_or_eof), |comments| {
            (comments, None)
        }),
    ))(input)
}

fn parse_document_greedy<'input>(input: &'input str) -> IResult<&'input str, Vec<Chunk<'input>>> {
    fold_many0(
        parse_document_fragment,
        Vec::<Chunk<'input>>::new(),
        |mut acc, (chunks, maybe_chunk)| {
            acc.extend(chunks);
            if let Some(chunk) = maybe_chunk {
                acc.extend(chunk);
            }
            acc
        },
    )(input)
}

pub fn parse_document<'input>(
    input: &'input str,
) -> Result<Vec<Chunk<'input>>, nom::Err<(&'input str, nom::error::ErrorKind)>> {
    Ok(complete(parse_document_greedy)(input)?.1)
}

#[cfg(test)]
mod tests {
    use super::*;
    use indoc::indoc;
    use pretty_assertions::assert_eq;
    use unindent::unindent;

    /// Utility function for creating a chunk of source code.
    fn src(source: &str) -> Chunk<'_> {
        Chunk::Source(source)
    }

    fn _comment(text: &str, kind: CommentKind) -> Chunk<'_> {
        Chunk::Comment(Comment {
            text: unindent(text),
            kind,
        })
    }

    /// Utility function for creating a directive comment chunk.
    fn dir(text: &str) -> Chunk<'_> {
        _comment(text, CommentKind::Directive)
    }

    /// Utility function for creating a documentation comment chunk.
    fn doc(text: &str) -> Chunk<'_> {
        _comment(text, CommentKind::Documentation)
    }

    /// Utility function for creating a preserved comment comment chunk.
    fn preserved(text: &str) -> Chunk<'_> {
        _comment(text, CommentKind::Preserved)
    }

    /// Utility function for creating an EOL comment chunk.
    fn eol(text: &str) -> Chunk<'_> {
        _comment(text, CommentKind::Eol)
    }

    #[test]
    fn parse_empty() {
        assert_eq!(Ok(vec![]), parse_document(""));
    }

    #[test]
    fn parse_source_simple() {
        assert_eq!(
            Ok(vec![src(indoc!(
                "lorem ipsum dolor...
                 foo bar baz"
            ))]),
            parse_document(indoc!(
                "lorem ipsum dolor...
                 foo bar baz"
            ))
        );

        assert_eq!(
            Ok(vec![src("lorem ipsum dolor...")]),
            parse_document("lorem ipsum dolor...")
        );
    }

    #[test]
    fn parse_eol_comment_simple() {
        assert_eq!(
            Ok(vec![
                src("lorem ipsum dolor..."),
                eol("eol comment (thrown away)"),
            ]),
            parse_document("lorem ipsum dolor...% eol comment (thrown away)")
        );
    }

    #[test]
    fn parse_directives() {
        assert_eq!(
            Ok(vec![
                dir("ezlatexdoc directives
                    all come in blocks where each line starts with '%%%'
                    whitespace before the markers is optional."),
                eol("this plain comment will be thrown out..."),
                dir("...but it breaks up the directive blocks into two."),
            ]),
            parse_document(indoc!(
                "
                %%% ezlatexdoc directives
                %%% all come in blocks where each line starts with '%%%'
                %%% whitespace before the markers is optional.
                % this plain comment will be thrown out...
                %%% ...but it breaks up the directive blocks into two.
                "
            )),
        );
    }

    #[test]
    fn parse_mixed() {
        assert_eq!(
            Ok(vec![
                src(indoc!(
                    "foo bar
                     foo bar baz
                     foo bar"
                )),
                eol("eol"),
                src("foo baz"),
                preserved("preserved"),
                src("baz qux"),
                dir("directives\ndirectives"),
                doc("documentation...
                    ...goes here, and doesn't even need to be in TeX"),
                eol("impl note"),
                src(indoc!(
                    "more source...
                     foo bar baz
                     bux boz"
                )),
            ]),
            parse_document(indoc!(
                "foo bar
                foo bar baz
                foo bar% eol
                foo baz%! preserved
                baz qux
                %%% directives
                %%% directives
                %% documentation...
                %% ...goes here, and doesn't even need to be in TeX
                % impl note
                more source...
                foo bar baz
                bux boz
                "
            ))
        );
    }
}
