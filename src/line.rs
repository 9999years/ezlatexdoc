// /// A line in a source file, possibly stripped or otherwise processed.
// #[derive(Debug, Clone, PartialEq)]
// pub struct SourceChunks<'a> {
// chunks: Vec<&'a str>,
// kind: Kind,
// }

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Kind {
    Directive,
    Documentation,
    PreservedComment,
    Comment,
    Source,
}
