// Caret DSL - Parser
//
// Copyright (c) 2025 Caret Contributors
//
// Licensed under the MIT License:
// https://opensource.org/licenses/MIT

use crate::ast::*;
use crate::error::{Error, Result, Span};
use crate::lexer::{Token, TokenKind};

/// Parser for the Caret DSL
pub struct Parser {
    tokens: Vec<Token>,
    pos: usize,
}

impl Parser {
    /// Create a new parser
    pub fn new(tokens: Vec<Token>) -> Self {
        Self { tokens, pos: 0 }
    }

    /// Get the current token
    fn current(&self) -> Option<&Token> {
        self.tokens.get(self.pos)
    }

    /// Peek at the current token kind
    fn peek(&self) -> TokenKind {
        self.current()
            .map(|t| t.kind)
            .unwrap_or(TokenKind::Eof)
    }

    /// Consume the current token
    fn consume(&mut self) -> Option<Token> {
        if self.pos < self.tokens.len() {
            let token = self.tokens[self.pos].clone();
            self.pos += 1;
            Some(token)
        } else {
            None
        }
    }

    /// Check if we're at EOF
    fn is_eof(&self) -> bool {
        self.peek() == TokenKind::Eof || self.pos >= self.tokens.len()
    }

    /// Get the span of the current token
    fn current_span(&self) -> Span {
        self.current()
            .map(|t| t.span)
            .unwrap_or(Span::start())
    }

    /// Expect a specific token kind
    fn expect(&mut self, kind: TokenKind) -> Result<Token> {
        if self.peek() == kind {
            self.consume().ok_or_else(|| {
                Error::syntax(
                    format!("unexpected EOF while expecting {:?}", kind),
                    Span::start(),
                )
            })
        } else {
            let span = self.current().map(|t| t.span).unwrap_or(Span::start());
            Err(Error::syntax(
                format!("expected {:?}, found {:?}", kind, self.peek()),
                span,
            ))
        }
    }

    /// Parse the entire token stream
    pub fn parse(&mut self) -> Result<Vec<AstNode>> {
        if self.tokens.is_empty() {
            return Ok(Vec::new());
        }

        let mut nodes = Vec::new();

        while !self.is_eof() {
            if let Some(node) = self.parse_top_level_node()? {
                nodes.push(node);
            } else {
                break;
            }
        }

        Ok(nodes)
    }

    /// Parse a top-level node
    fn parse_top_level_node(&mut self) -> Result<Option<AstNode>> {
        let start = self.current_span();

        match self.peek() {
            TokenKind::Eof => Ok(None),
            TokenKind::Pipeline => {
                let pipeline = self.parse_pipeline_decl()?;
                Ok(Some(AstNode::new(AstNodeKind::Pipeline(pipeline), start)))
            }
            TokenKind::Source | TokenKind::Sink | TokenKind::Process => {
                let decl = self.parse_node_decl()?;
                Ok(Some(AstNode::new(
                    AstNodeKind::Statement(Statement::NodeDecl(decl)),
                    start,
                )))
            }
            TokenKind::Link | TokenKind::Connect => {
                let conn = self.parse_connection()?;
                Ok(Some(AstNode::new(
                    AstNodeKind::Statement(Statement::Connection(conn)),
                    start,
                )))
            }
            TokenKind::Import => {
                self.consume();
                let path = self.parse_string_literal()?;
                Ok(Some(AstNode::new(
                    AstNodeKind::Statement(Statement::Import { path, span: start }),
                    start,
                )))
            }
            TokenKind::Export => {
                self.consume();
                let name = self.parse_identifier()?;
                Ok(Some(AstNode::new(
                    AstNodeKind::Statement(Statement::Export { name, span: start }),
                    start,
                )))
            }
            TokenKind::Semicolon => {
                self.consume();
                Ok(None)
            }
            _ => {
                let span = self.current_span();
                Err(Error::syntax(
                    format!("unexpected token {:?}", self.peek()),
                    span,
                ))
            }
        }
    }

    /// Parse a pipeline declaration
    fn parse_pipeline_decl(&mut self) -> Result<PipelineDecl> {
        let start = self.current_span();
        self.expect(TokenKind::Pipeline)?;
        let name = self.parse_identifier()?;

        self.expect(TokenKind::LeftBrace)?;

        let mut statements = Vec::new();
        while self.peek() != TokenKind::RightBrace && !self.is_eof() {
            match self.peek() {
                TokenKind::Source | TokenKind::Sink | TokenKind::Process => {
                    statements.push(Statement::NodeDecl(self.parse_node_decl()?));
                }
                TokenKind::Link | TokenKind::Connect => {
                    statements.push(Statement::Connection(self.parse_connection()?));
                }
                TokenKind::Semicolon => {
                    self.consume();
                }
                _ => {
                    let span = self.current_span();
                    return Err(Error::syntax(
                        format!("expected node declaration or connection, found {:?}", self.peek()),
                        span,
                    ));
                }
            }
        }

        self.expect(TokenKind::RightBrace)?;

        Ok(PipelineDecl {
            name,
            statements,
            span: Span::new(start.start, self.current_span().end, start.line, start.column),
        })
    }

    /// Parse a node declaration
    fn parse_node_decl(&mut self) -> Result<NodeDecl> {
        let start = self.current_span();

        // Parse node type
        let node_type = match self.peek() {
            TokenKind::Source => NodeType::Source,
            TokenKind::Sink => NodeType::Sink,
            TokenKind::Process => NodeType::Process,
            _ => {
                return Err(Error::syntax(
                    format!("expected node type (source/sink/process), found {:?}", self.peek()),
                    start,
                ))
            }
        };
        self.consume();

        // Parse node name
        let name = self.parse_identifier()?;

        self.expect(TokenKind::Assign)?;

        // Parse implementation name and properties
        let impl_name = self.parse_identifier()?;
        let _impl_span = self.current_span();

        self.expect(TokenKind::LeftParen)?;

        let properties = self.parse_properties()?;

        self.expect(TokenKind::RightParen)?;

        Ok(NodeDecl {
            node_type,
            name,
            impl_name,
            properties,
            span: Span::new(start.start, self.current_span().end, start.line, start.column),
        })
    }

    /// Parse properties list
    fn parse_properties(&mut self) -> Result<Vec<Property>> {
        let mut properties = Vec::new();
        let mut positional_index = 0;

        while self.peek() != TokenKind::RightParen && !self.is_eof() {
            if let Some(prop) = self.parse_property(positional_index)? {
                // Check if it's a named or positional property
                match &prop.key {
                    PropertyKey::Named(_) => {}
                    PropertyKey::Positional(_) => {
                        positional_index += 1;
                    }
                }
                properties.push(prop);
            }

            if self.peek() == TokenKind::Comma {
                self.consume();
            } else {
                break;
            }
        }

        Ok(properties)
    }

    /// Parse a single property
    fn parse_property(&mut self, positional_index: usize) -> Result<Option<Property>> {
        // Check if this is a named property (key=value)
        let key = if self.peek() == TokenKind::Identifier {
            // Look ahead to see if next token is =
            let lookahead = self.pos + 1;
            if lookahead < self.tokens.len() && self.tokens[lookahead].kind == TokenKind::Assign {
                let name = self.parse_identifier()?;
                self.expect(TokenKind::Assign)?;
                PropertyKey::Named(name)
            } else {
                // Positional argument
                PropertyKey::Positional(positional_index)
            }
        } else {
            PropertyKey::Positional(positional_index)
        };

        let value = self.parse_property_value()?;
        Ok(Some(Property { key, value }))
    }

    /// Parse a property value
    fn parse_property_value(&mut self) -> Result<PropertyValue> {
        let start = self.current_span();
        match self.peek() {
            TokenKind::StringLiteral => {
                self.consume();
                // Get the string content from the source (would need source reference)
                // For now, use a placeholder
                Ok(PropertyValue::String("__string__".to_string()))
            }
            TokenKind::NumberLiteral => {
                self.consume();
                Ok(PropertyValue::Number(0.0))
            }
            TokenKind::BooleanLiteral => {
                self.consume();
                Ok(PropertyValue::Boolean(true))
            }
            TokenKind::Identifier => {
                let name = self.parse_identifier()?;
                Ok(PropertyValue::Identifier(name))
            }
            TokenKind::LeftBracket => {
                self.consume();
                let mut values = Vec::new();
                while self.peek() != TokenKind::RightBracket && !self.is_eof() {
                    values.push(self.parse_property_value()?);
                    if self.peek() == TokenKind::Comma {
                        self.consume();
                    }
                }
                self.expect(TokenKind::RightBracket)?;
                Ok(PropertyValue::Array(values))
            }
            _ => Err(Error::syntax(
                format!("expected property value, found {:?}", self.peek()),
                start,
            )),
        }
    }

    /// Parse a connection statement
    fn parse_connection(&mut self) -> Result<Connection> {
        let start = self.current_span();

        // Parse "link" or "connect"
        match self.peek() {
            TokenKind::Link => { self.consume(); }
            TokenKind::Connect => { self.consume(); }
            _ => {}
        }

        let source = self.parse_connection_endpoint()?;

        // Expect arrow or just another endpoint
        if self.peek() == TokenKind::Arrow {
            self.consume();
        }

        let target = self.parse_connection_endpoint()?;

        Ok(Connection {
            source,
            target,
            span: Span::new(start.start, self.current_span().end, start.line, start.column),
        })
    }

    /// Parse a connection endpoint
    fn parse_connection_endpoint(&mut self) -> Result<ConnectionEndpoint> {
        let start = self.current_span();
        let node = self.parse_identifier()?;

        let port = if self.peek() == TokenKind::Dot {
            self.consume();
            Some(self.parse_identifier()?)
        } else {
            None
        };

        Ok(ConnectionEndpoint {
            node,
            port,
            span: Span::new(start.start, self.current_span().end, start.line, start.column),
        })
    }

    /// Parse an identifier
    fn parse_identifier(&mut self) -> Result<String> {
        if self.peek() == TokenKind::Identifier {
            // For simplicity, just return a placeholder
            self.consume();
            Ok("__identifier__".to_string())
        } else {
            let span = self.current_span();
            Err(Error::syntax(
                format!("expected identifier, found {:?}", self.peek()),
                span,
            ))
        }
    }

    /// Parse a string literal
    fn parse_string_literal(&mut self) -> Result<String> {
        if self.peek() == TokenKind::StringLiteral {
            self.consume();
            Ok("__string__".to_string())
        } else {
            let span = self.current_span();
            Err(Error::syntax(
                format!("expected string literal, found {:?}", self.peek()),
                span,
            ))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_span(start: usize, end: usize) -> Span {
        Span::new(start, end, 1, start + 1)
    }

    #[test]
    fn test_parse_empty() {
        let tokens = vec![Token::new(TokenKind::Eof, make_span(0, 0))];
        let mut parser = Parser::new(tokens);
        let ast = parser.parse().unwrap();
        assert!(ast.is_empty());
    }

    #[test]
    fn test_parse_simple_node_decl() {
        let tokens = vec![
            Token::new(TokenKind::Source, make_span(0, 6)),
            Token::new(TokenKind::Identifier, make_span(7, 12)),
            Token::new(TokenKind::Assign, make_span(13, 14)),
            Token::new(TokenKind::Identifier, make_span(15, 19)),
            Token::new(TokenKind::LeftParen, make_span(19, 20)),
            Token::new(TokenKind::RightParen, make_span(20, 21)),
            Token::new(TokenKind::Eof, make_span(21, 21)),
        ];
        let mut parser = Parser::new(tokens);
        let ast = parser.parse().unwrap();
        assert_eq!(ast.len(), 1);
    }

    #[test]
    fn test_parse_pipeline() {
        let tokens = vec![
            Token::new(TokenKind::Pipeline, make_span(0, 8)),
            Token::new(TokenKind::Identifier, make_span(9, 18)),
            Token::new(TokenKind::LeftBrace, make_span(19, 20)),
            Token::new(TokenKind::Source, make_span(21, 27)),
            Token::new(TokenKind::Identifier, make_span(28, 33)),
            Token::new(TokenKind::Assign, make_span(34, 35)),
            Token::new(TokenKind::Identifier, make_span(36, 40)),
            Token::new(TokenKind::LeftParen, make_span(40, 41)),
            Token::new(TokenKind::RightParen, make_span(41, 42)),
            Token::new(TokenKind::Link, make_span(43, 47)),
            Token::new(TokenKind::Identifier, make_span(48, 53)),
            Token::new(TokenKind::Arrow, make_span(54, 56)),
            Token::new(TokenKind::Identifier, make_span(57, 62)),
            Token::new(TokenKind::RightBrace, make_span(63, 64)),
            Token::new(TokenKind::Eof, make_span(64, 64)),
        ];
        let mut parser = Parser::new(tokens);
        let ast = parser.parse().unwrap();
        assert_eq!(ast.len(), 1);
    }
}
