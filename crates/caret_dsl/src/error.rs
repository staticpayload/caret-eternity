// Caret DSL - Error types
//
// Copyright (c) 2025 Caret Contributors
//
// Licensed under the MIT License:
// https://opensource.org/licenses/MIT

use std::fmt;

/// DSL error type
#[derive(Debug, Clone)]
pub struct Error {
    /// Kind of error
    pub kind: ErrorKind,
    /// Error message
    pub message: String,
    /// Source location
    pub span: Option<Span>,
}

/// Error kind
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ErrorKind {
    /// Lexical error
    Lexical,
    /// Syntax error
    Syntax,
    /// Semantic error
    Semantic,
    /// IO error
    Io,
}

/// Source location
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Span {
    /// Start byte offset
    pub start: usize,
    /// End byte offset
    pub end: usize,
    /// Line number (1-indexed)
    pub line: usize,
    /// Column number (1-indexed)
    pub column: usize,
}

impl Span {
    /// Create a new span
    pub fn new(start: usize, end: usize, line: usize, column: usize) -> Self {
        Self { start, end, line, column }
    }

    /// Create a span at the beginning of input
    pub fn start() -> Self {
        Self { start: 0, end: 0, line: 1, column: 1 }
    }

    /// Extend a span to include another
    pub fn extend(&self, other: Span) -> Span {
        Span {
            start: self.start.min(other.start),
            end: self.end.max(other.end),
            line: self.line,
            column: self.column,
        }
    }
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if let Some(span) = self.span {
            write!(f, "{}:{}: {}: ", span.line, span.column, self.kind.as_str())?;
        } else {
            write!(f, "{}: ", self.kind.as_str())?;
        }
        write!(f, "{}", self.message)
    }
}

impl ErrorKind {
    fn as_str(&self) -> &str {
        match self {
            ErrorKind::Lexical => "lexical error",
            ErrorKind::Syntax => "syntax error",
            ErrorKind::Semantic => "semantic error",
            ErrorKind::Io => "io error",
        }
    }
}

impl std::error::Error for Error {}

/// Result type for DSL operations
pub type Result<T> = std::result::Result<T, Error>;

impl Error {
    /// Create a new lexical error
    pub fn lexical(message: impl Into<String>, span: Span) -> Self {
        Self {
            kind: ErrorKind::Lexical,
            message: message.into(),
            span: Some(span),
        }
    }

    /// Create a new syntax error
    pub fn syntax(message: impl Into<String>, span: Span) -> Self {
        Self {
            kind: ErrorKind::Syntax,
            message: message.into(),
            span: Some(span),
        }
    }

    /// Create a new semantic error
    pub fn semantic(message: impl Into<String>, span: Span) -> Self {
        Self {
            kind: ErrorKind::Semantic,
            message: message.into(),
            span: Some(span),
        }
    }

    /// Create a new IO error
    pub fn io(message: impl Into<String>) -> Self {
        Self {
            kind: ErrorKind::Io,
            message: message.into(),
            span: None,
        }
    }

    /// Add a span to an error
    pub fn with_span(mut self, span: Span) -> Self {
        self.span = Some(span);
        self
    }

    /// Format the error with source context
    pub fn format_with_source(&self, source: &str) -> String {
        if let Some(span) = self.span {
            let lines: Vec<&str> = source.lines().collect();
            if let Some(line) = lines.get(span.line - 1) {
                let indent = "    ";
                let pointer = " ".repeat(span.column - 1) + &"^".repeat(span.end - span.start);
                return format!("{}\n{}{}\n{}{}", self, indent, line, indent, pointer);
            }
        }
        format!("{}", self)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_display() {
        let span = Span::new(0, 5, 1, 1);
        let err = Error::syntax("unexpected token", span);
        assert_eq!(format!("{}", err), "1:1: syntax error: unexpected token");
    }

    #[test]
    fn test_span_extend() {
        let a = Span::new(0, 5, 1, 1);
        let b = Span::new(10, 15, 1, 10);
        let c = a.extend(b);
        assert_eq!(c.start, 0);
        assert_eq!(c.end, 15);
    }
}
