use core::fmt;

/// A line-column pair representing the begin or end of a `Span`.
#[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd)]
pub struct LineColumn {
    /// The 1-indexed line in the source code on which the span starts or ends
    /// (inclusive).
    pub line: usize,
    /// The 0-indexed column (in UTF-8 characters) in the source code on which
    /// the span starts or ends (inclusive).
    pub column: usize,
}

impl fmt::Debug for LineColumn {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}:{}", self.line, self.column)
    }
}

impl fmt::Display for LineColumn {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Line: {}, Column: {}", self.line, self.column)
    }
}

impl Default for LineColumn {
    fn default() -> Self {
        Self { line: 1, column: 0 }
    }
}

impl LineColumn {
    /// Creates a new `LineColumn` with the given line and column.
    pub fn new(line: usize, column: usize) -> LineColumn {
        LineColumn { line, column }
    }

    pub(crate) fn advance(&mut self, ch: char) {
        if ch == '\n' {
            self.column = 0;
            self.line += 1;
        } else {
            self.column += 1;
        }
    }
}

/// A trait that can provide the `Span` of the complete contents of a syntax
/// tree node.
pub trait Spanned {
    /// Returns a `Span` covering the complete contents of this syntax tree node.
    fn span(&self) -> Span;
}

/// A region of source code.
#[derive(Copy, Clone, PartialEq, Eq, Default)]
pub struct Span {
    pub(crate) start: LineColumn,
    pub(crate) end: LineColumn,
}

impl fmt::Debug for Span {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "bytes({}..{})", self.start, self.end)
    }
}

impl Span {
    /// Creates a new `Span`.
    pub fn new() -> Self {
        Self::default()
    }

    /// Creates a new `Span` with the given start and end byte offsets.
    pub fn with(start: LineColumn, end: LineColumn) -> Self {
        Self { start, end }
    }

    /*
    /// Gets the starting location for this span.
    pub fn start(&self, info: &SourceInfo) -> LineColumn {
        info.offset_line_column(self.start)
    }

    /// Gets the ending location for this span.
    pub fn end(&self, info: &SourceInfo) -> LineColumn {
        info.offset_line_column(self.end)
    }
    */
}

/*
#[derive(Debug)]
pub struct SourceInfo {
    /// The span of the source code.
    span: Span,
    /// The offset corresponding to the beginning of all lines.
    lines: Vec<usize>,
}

impl SourceInfo {
    /// Creates a new `SourceInfo` with the given input string.
    pub(crate) fn new(src: &str) -> Self {
        let (len, lines) = lines_offsets(src);
        let span = Span::with(0, len);
        Self { span, lines }
    }

    fn offset_line_column(&self, offset: usize) -> LineColumn {
        assert!((self.span.start..=self.span.end).contains(&offset));
        let offset = offset - self.span.start;
        match self.lines.binary_search(&offset) {
            Ok(found) => LineColumn {
                line: found + 1,
                column: 0,
            },
            Err(idx) => LineColumn {
                line: idx,
                column: offset - self.lines[idx - 1],
            },
        }
    }
}

/// Computes the offsets of each line in the given source string and the total number of characters
fn lines_offsets(s: &str) -> (usize, Vec<usize>) {
    let mut lines = vec![0];
    let mut total = 0;

    for ch in s.chars() {
        total += 1;
        if ch == '\n' {
            lines.push(total);
        }
    }

    (total, lines)
}
*/

/*
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn span_location() {
        let src = "
            SELECT * FROM users
            WHERE id = 1";
        let info = SourceInfo::new(src);
        // println!("{:?}", info);
        assert_eq!(info.span, Span::with(0, 57));
        assert_eq!(info.lines, vec![0, 1, 33]);

        let select_span = Span::with(13, 19);
        assert_eq!(select_span.start(&info), LineColumn::new(2, 12));
        assert_eq!(select_span.end(&info), LineColumn::new(2, 18));
        let _1_span = Span::with(56, 57);
        assert_eq!(_1_span.start(&info), LineColumn::new(3, 23));
        assert_eq!(_1_span.end(&info), LineColumn::new(3, 24));
    }
}
*/
