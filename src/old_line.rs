/// A line in a source file, possibly stripped or otherwise processed.
#[derive(Debug, Clone, PartialEq)]
pub struct Line<'a> {
    /// The original line, directly from the source file, not including line-ending characters.
    pub orig: &'a str,
    /// The processed line.
    pub processed: &'a str,
    pub kind: LineKind,
}

impl<'a> Line<'a> {
    pub fn should_discard(&self) -> bool {
        match self.kind {
            LineKind::Directive => true,
            LineKind::Comment => self.processed.is_empty(),
            _ => false,
        }
    }

    fn trim_trailing_comment(&mut self) {
        // Only meaningful for LineKind::Source
        if let LineKind::Source = self.kind {
            for (i, _) in self.processed.match_indices('%') {
                // Grab the char before the '%' sign.
                let char_before = self.processed.get(i - 1..i);
                // The end byte-index (inclusive) to trim self.processed to.
                let mut trim_to = None;
                if char_before
                    .and_then(|s| s.chars().next())
                    .map_or(false, is_space)
                {
                    // Space before comment; trim out the comment, incl. '%'
                    // E.g. "xyz... % comment" -> "xyz... "
                    trim_to = Some(i);
                } else if char_before.map_or(false, |s| s != "\\") {
                    // The match is a '%' not preceded by a '\', i.e. a true EOL-comment.
                    // Keep the comment marker, to avoid adding a spurious space.
                    // E.g. "100\% of profits.% comment" -> "100\% of profits.%"
                    trim_to = Some(i + 1);
                }
                // Finally, overwite self.processed.
                if let Some(trim_to_) = trim_to {
                    if let Some(s) = self.processed.get(..trim_to_) {
                        self.processed = s.trim_end_matches(is_space);
                    }
                }
            }
            // Finally, trim spaces again.
            self.processed = self.processed.trim_end_matches(is_space);
        }
    }
}

impl<'a> From<&'a str> for Line<'a> {
    /// Converts and cleans up a source line.
    ///
    /// ```
    /// # use pretty_assertions::{assert_eq, assert_ne};
    /// # use ezlatexdoc::line::{Line, LineKind};
    /// // Whitespace is trimmed from the end of source lines:
    /// assert_eq!(Line {
    ///     orig: "  xyz ",
    ///     processed: "  xyz",
    ///     kind: LineKind::Source,
    /// }, "  xyz ".into());
    ///
    /// // Comments are stripped too:
    /// assert_eq!(Line {
    ///     orig: "  xyz % Comment...",
    ///     processed: "  xyz",
    ///     kind: LineKind::Source,
    /// }, "  xyz % Comment...".into());
    ///
    /// // Unless they don't have spaces:
    /// assert_eq!(Line {
    ///     orig: "  xyz% Comment...",
    ///     processed: "  xyz%",
    ///     kind: LineKind::Source,
    /// }, "  xyz% Comment...".into());
    ///
    /// // Percent-signs are skipped:
    /// assert_eq!(Line {
    ///     orig: "xyz 100\\% not a comment",
    ///     processed: "xyz 100\\% not a comment",
    ///     kind: LineKind::Source,
    /// }, "xyz 100\\% not a comment".into());
    ///
    /// assert_eq!(Line {
    ///     orig: "%% Stays in source...",
    ///     processed: "% Stays in source...",
    ///     kind: LineKind::Comment,
    /// }, "%% Stays in source...".into());
    ///
    /// assert_eq!(Line {
    ///     orig: "%%  Stays in source...",
    ///     processed: "%  Stays in source...",
    ///     kind: LineKind::Comment,
    /// }, "%%  Stays in source...".into());
    ///
    /// assert_eq!(Line {
    ///     orig: "%%% Directives are ignored.",
    ///     processed: "Directives are ignored.",
    ///     kind: LineKind::Directive,
    /// }, "%%% Directives are ignored.".into());
    ///
    /// assert_eq!(Line {
    ///     orig: "% Documentation.",
    ///     processed: "Documentation.",
    ///     kind: LineKind::Documentation,
    /// }, "% Documentation.".into());
    ///
    /// // At most one whitespace token after the prefix token is trimmed:
    /// assert_eq!(Line {
    ///     orig: "%  Documentation.",
    ///     processed: " Documentation.",
    ///     kind: LineKind::Documentation,
    /// }, "%  Documentation.".into());
    /// ```
    fn from(orig: &'a str) -> Line {
        let (processed, kind) = determine_kind(orig);
        let mut line = Line {
            orig,
            processed,
            kind,
        };
        line.trim_trailing_comment();
        line
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum LineKind {
    /// An ezlatexdoc directive.
    Directive,
    /// A true comment, kept in the processed source.
    Comment,
    /// A documentation comment, written to the documentation source.
    Documentation,
    /// Source code, kept in the processed source.
    Source,
}

/// The line-prefix token for a given LineKind, not including leading whitespace.
fn prefix_token(kind: LineKind) -> &'static str {
    match kind {
        LineKind::Directive => "%%%",
        LineKind::Comment => "%%",
        LineKind::Documentation => "%",
        LineKind::Source => "",
    }
}

/// The length of the line-prefix token for a given LineKind.
fn prefix_bytes(kind: LineKind) -> usize {
    match kind {
        // We want Comment-lines to be written out with *one* '%', rather than two, so we actually
        // only want to trim *part* of the prefix.
        LineKind::Comment => "%".len(),
        _ => prefix_token(kind).len(),
    }
}

/// Trims the prefix token for the given LineKind from the line, otherwise returns None.
fn maybe_trim<'a>(orig_trimmed: &'a str, kind: LineKind) -> Option<(&'a str, LineKind)> {
    if orig_trimmed.starts_with(prefix_token(kind)) {
        let mut no_prefix = orig_trimmed.split_at(prefix_bytes(kind)).1;
        // Trim at most one space.
        if no_prefix.starts_with(is_space) {
            no_prefix = no_prefix.get(1..).unwrap_or(no_prefix);
        }
        Some((no_prefix, kind))
    } else {
        None
    }
}

/// Determines a line's Kind and trims it of its prefix.
fn determine_kind<'a>(orig: &'a str) -> (&'a str, LineKind) {
    let orig_trimmed = orig.trim_matches(is_space);
    let test = |kind| move || maybe_trim(orig_trimmed, kind);
    None.or_else(test(LineKind::Directive))
        .or_else(test(LineKind::Comment))
        .or_else(test(LineKind::Documentation))
        .unwrap_or((orig, LineKind::Source))
}

/// Determines if a character is ignorable start-of-line space.
fn is_space(c: char) -> bool {
    c == ' ' || c == '\t'
}

#[cfg(test)]
mod tests {
    use super::*;
    use pretty_assertions::assert_eq;

    #[test]
    fn is_space_test() {
        assert_eq!("% xyz...  ", "  \t % xyz...  ".trim_start_matches(is_space));
    }
}
