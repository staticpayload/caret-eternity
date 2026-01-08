// Caret DSL - Pipeline definition language
//
// Copyright (c) 2025 Caret Contributors
//
// Licensed under the MIT License:
// https://opensource.org/licenses/MIT

#![warn(missing_docs)]
#![warn(clippy::all)]

mod error;
mod lexer;
mod parser;
mod ast;

pub use error::{Error, Result, Span};
pub use lexer::{Lexer, Token, TokenKind};
pub use parser::Parser;
pub use ast::{
    AstNode, Statement, NodeDecl, Property, PropertyKey, PropertyValue,
    PipelineDecl, Connection, Expr,
};

/// Parse a DSL string into an AST
pub fn parse(source: &str) -> Result<Vec<AstNode>> {
    let lexer = Lexer::new(source);
    let tokens = lexer.tokenize()?;
    let mut parser = Parser::new(tokens);
    parser.parse()
}

/// Parse a DSL file
pub fn parse_file(path: impl AsRef<std::path::Path>) -> Result<Vec<AstNode>> {
    let source = std::fs::read_to_string(path.as_ref())
        .map_err(|e| Error::io(format!("failed to read file: {}", e)))?;
    parse(&source)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_empty_source() {
        let ast = parse("").unwrap();
        assert!(ast.is_empty());
    }

    #[test]
    fn test_comment_only() {
        let ast = parse("# just a comment").unwrap();
        assert!(ast.is_empty());
    }

    #[test]
    fn test_simple_node_decl() {
        let ast = parse("source input = file()").unwrap();
        assert_eq!(ast.len(), 1);
    }

    #[test]
    fn test_pipeline_decl() {
        let ast = parse(
            "pipeline my_pipeline {
                source input = file()
                sink output = file()
                link input -> output
            }"
        ).unwrap();
        assert_eq!(ast.len(), 1);
    }
}
