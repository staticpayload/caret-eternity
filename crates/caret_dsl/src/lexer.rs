// Caret DSL - Lexer
//
// Copyright (c) 2025 Caret Contributors
//
// Licensed under the MIT License:
// https://opensource.org/licenses/MIT

use crate::error::{Error, Result, Span};
use std::iter::Peekable;
use std::str::Chars;

/// Token produced by the lexer
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Token {
    /// Kind of token
    pub kind: TokenKind,
    /// Source location
    pub span: Span,
}

impl Token {
    /// Create a new token
    pub fn new(kind: TokenKind, span: Span) -> Self {
        Self { kind, span }
    }
}

/// Token kind
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TokenKind {
    // Keywords
    Source,
    Sink,
    Process,
    Pipeline,
    Link,
    Connect,
    Import,
    Export,
    Const,

    // Identifiers and literals
    Identifier,
    StringLiteral,
    NumberLiteral,
    BooleanLiteral,

    // Operators and punctuation
    Assign,        // =
    Arrow,         // ->
    Dot,           // .
    Comma,         // ,
    Colon,         // :
    Semicolon,     // ;
    Pipe,          // |

    // Brackets
    LeftParen,     // (
    RightParen,    // )
    LeftBrace,     // {
    RightBrace,    // }
    LeftBracket,   // [
    RightBracket,  // ]

    // Misc
    Comment,
    Whitespace,
    Newline,

    // End of input
    Eof,
}

impl TokenKind {
    /// Check if this is a keyword
    fn from_keyword(s: &str) -> Option<Self> {
        match s {
            "source" => Some(TokenKind::Source),
            "sink" => Some(TokenKind::Sink),
            "process" => Some(TokenKind::Process),
            "pipeline" => Some(TokenKind::Pipeline),
            "link" => Some(TokenKind::Link),
            "connect" => Some(TokenKind::Connect),
            "import" => Some(TokenKind::Import),
            "export" => Some(TokenKind::Export),
            "const" => Some(TokenKind::Const),
            "true" | "false" => Some(TokenKind::BooleanLiteral),
            _ => None,
        }
    }

    /// Get the keyword name for this token kind
    pub fn as_keyword(self) -> Option<&'static str> {
        match self {
            TokenKind::Source => Some("source"),
            TokenKind::Sink => Some("sink"),
            TokenKind::Process => Some("process"),
            TokenKind::Pipeline => Some("pipeline"),
            TokenKind::Link => Some("link"),
            TokenKind::Connect => Some("connect"),
            TokenKind::Import => Some("import"),
            TokenKind::Export => Some("export"),
            TokenKind::Const => Some("const"),
            _ => None,
        }
    }
}

/// Lexer for the Caret DSL
pub struct Lexer<'a> {
    input: &'a str,
    chars: Peekable<Chars<'a>>,
    pos: usize,
    line: usize,
    column: usize,
}

impl<'a> Lexer<'a> {
    /// Create a new lexer
    pub fn new(input: &'a str) -> Self {
        Self {
            input,
            chars: input.chars().peekable(),
            pos: 0,
            line: 1,
            column: 1,
        }
    }

    /// Get the current position
    fn current_span(&self) -> Span {
        Span::new(self.pos, self.pos, self.line, self.column)
    }

    /// Peek at the next character
    fn peek(&mut self) -> Option<&char> {
        self.chars.peek()
    }

    /// Consume the next character
    fn next(&mut self) -> Option<char> {
        let c = self.chars.next()?;
        self.pos += c.len_utf8();
        if c == '\n' {
            self.line += 1;
            self.column = 1;
        } else {
            self.column += 1;
        }
        Some(c)
    }

    /// Check if we're at EOF
    fn is_eof(&mut self) -> bool {
        self.peek().is_none()
    }

    /// Tokenize the entire input
    pub fn tokenize(mut self) -> Result<Vec<Token>> {
        let mut tokens = Vec::new();

        loop {
            let token = self.lex_token()?;

            // Filter out whitespace, comments, and newlines
            if token.kind != TokenKind::Whitespace && token.kind != TokenKind::Comment && token.kind != TokenKind::Newline {
                tokens.push(token.clone());
                // Stop at EOF
                if token.kind == TokenKind::Eof {
                    break;
                }
            }
        }

        Ok(tokens)
    }

    /// Lex a single token
    fn lex_token(&mut self) -> Result<Token> {
        let start = self.current_span();

        match self.peek() {
            None => Ok(Token::new(TokenKind::Eof, start)),
            Some(&c) => match c {
                '#' => self.lex_comment(),
                '"' => self.lex_string(),
                '0'..='9' => self.lex_number(),
                '_' | 'a'..='z' | 'A'..='Z' => self.lex_identifier_or_keyword(),
                '\n' => self.lex_newline(),
                ' ' | '\t' | '\r' => self.lex_whitespace(),
                '=' => {
                    self.next();
                    if let Some(&'>') = self.peek() {
                        self.next();
                        Ok(Token::new(TokenKind::Arrow, Span::new(start.start, self.pos, start.line, start.column)))
                    } else {
                        Ok(Token::new(TokenKind::Assign, start))
                    }
                }
                '-' => {
                    self.next();
                    if let Some(&'>') = self.peek() {
                        self.next();
                        Ok(Token::new(TokenKind::Arrow, Span::new(start.start, self.pos, start.line, start.column)))
                    } else {
                        Err(Error::lexical("unexpected '-'", start))
                    }
                }
                '.' => { self.next(); Ok(Token::new(TokenKind::Dot, start)) }
                ',' => { self.next(); Ok(Token::new(TokenKind::Comma, start)) }
                ':' => { self.next(); Ok(Token::new(TokenKind::Colon, start)) }
                ';' => { self.next(); Ok(Token::new(TokenKind::Semicolon, start)) }
                '|' => { self.next(); Ok(Token::new(TokenKind::Pipe, start)) }
                '(' => { self.next(); Ok(Token::new(TokenKind::LeftParen, start)) }
                ')' => { self.next(); Ok(Token::new(TokenKind::RightParen, start)) }
                '{' => { self.next(); Ok(Token::new(TokenKind::LeftBrace, start)) }
                '}' => { self.next(); Ok(Token::new(TokenKind::RightBrace, start)) }
                '[' => { self.next(); Ok(Token::new(TokenKind::LeftBracket, start)) }
                ']' => { self.next(); Ok(Token::new(TokenKind::RightBracket, start)) }
                _ => Err(Error::lexical(format!("unexpected character '{}'", c), start)),
            }
        }
    }

    fn lex_comment(&mut self) -> Result<Token> {
        let start = self.current_span();
        self.next(); // '#'
        while let Some(&c) = self.peek() {
            if c == '\n' {
                break;
            }
            self.next();
        }
        Ok(Token::new(TokenKind::Comment, Span::new(start.start, self.pos, start.line, start.column)))
    }

    fn lex_string(&mut self) -> Result<Token> {
        let start = self.current_span();
        self.next(); // opening quote
        let mut content = String::new();

        while let Some(&c) = self.peek() {
            if c == '"' {
                self.next();
                return Ok(Token::new(TokenKind::StringLiteral, Span::new(start.start, self.pos, start.line, start.column)));
            }
            if c == '\\' {
                self.next();
                if let Some(&escaped) = self.peek() {
                    match escaped {
                        'n' => content.push('\n'),
                        't' => content.push('\t'),
                        'r' => content.push('\r'),
                        '\\' => content.push('\\'),
                        '"' => content.push('"'),
                        _ => return Err(Error::lexical(format!("invalid escape sequence '\\{}'", escaped), self.current_span())),
                    }
                    self.next();
                }
            } else {
                content.push(self.next().unwrap());
            }
        }

        Err(Error::lexical("unterminated string literal", Span::new(start.start, self.pos, start.line, start.column)))
    }

    fn lex_number(&mut self) -> Result<Token> {
        let start = self.current_span();

        // Lex integer or float
        while let Some(&c) = self.peek() {
            if c.is_ascii_digit() || c == '.' || c == '_' {
                self.next();
            } else {
                break;
            }
        }

        Ok(Token::new(TokenKind::NumberLiteral, Span::new(start.start, self.pos, start.line, start.column)))
    }

    fn lex_identifier_or_keyword(&mut self) -> Result<Token> {
        let start = self.current_span();
        let start_pos = self.pos;

        while let Some(&c) = self.peek() {
            if c == '_' || c.is_ascii_alphanumeric() {
                self.next();
            } else {
                break;
            }
        }

        let text = &self.input[start_pos..self.pos];
        let kind = TokenKind::from_keyword(text).unwrap_or(TokenKind::Identifier);

        Ok(Token::new(kind, Span::new(start.start, self.pos, start.line, start.column)))
    }

    fn lex_newline(&mut self) -> Result<Token> {
        let start = self.current_span();
        self.next();
        Ok(Token::new(TokenKind::Newline, Span::new(start.start, self.pos, start.line, start.column)))
    }

    fn lex_whitespace(&mut self) -> Result<Token> {
        let start = self.current_span();
        while let Some(&c) = self.peek() {
            if c == ' ' || c == '\t' || c == '\r' {
                self.next();
            } else {
                break;
            }
        }
        Ok(Token::new(TokenKind::Whitespace, Span::new(start.start, self.pos, start.line, start.column)))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_empty_input() {
        let lexer = Lexer::new("");
        let tokens = lexer.tokenize().unwrap();
        assert_eq!(tokens.len(), 1);
        assert_eq!(tokens[0].kind, TokenKind::Eof);
    }

    #[test]
    fn test_keywords() {
        let lexer = Lexer::new("source sink process pipeline link connect");
        let tokens = lexer.tokenize().unwrap();
        assert_eq!(tokens[0].kind, TokenKind::Source);
        assert_eq!(tokens[1].kind, TokenKind::Sink);
        assert_eq!(tokens[2].kind, TokenKind::Process);
        assert_eq!(tokens[3].kind, TokenKind::Pipeline);
        assert_eq!(tokens[4].kind, TokenKind::Link);
        assert_eq!(tokens[5].kind, TokenKind::Connect);
    }

    #[test]
    fn test_identifiers() {
        let lexer = Lexer::new("foo bar_baz baz123");
        let tokens = lexer.tokenize().unwrap();
        assert_eq!(tokens[0].kind, TokenKind::Identifier);
        assert_eq!(tokens[1].kind, TokenKind::Identifier);
        assert_eq!(tokens[2].kind, TokenKind::Identifier);
    }

    #[test]
    fn test_string_literals() {
        let lexer = Lexer::new("\"hello\" \"world\\n\"");
        let tokens = lexer.tokenize().unwrap();
        assert_eq!(tokens[0].kind, TokenKind::StringLiteral);
        assert_eq!(tokens[1].kind, TokenKind::StringLiteral);
    }

    #[test]
    fn test_arrow() {
        let lexer = Lexer::new("->");
        let tokens = lexer.tokenize().unwrap();
        assert_eq!(tokens[0].kind, TokenKind::Arrow);
    }

    #[test]
    fn test_comments() {
        let lexer = Lexer::new("# comment\nsource");
        let tokens = lexer.tokenize().unwrap();
        assert_eq!(tokens[0].kind, TokenKind::Source);
    }
}
