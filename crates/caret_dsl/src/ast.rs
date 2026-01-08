// Caret DSL - Abstract Syntax Tree
//
// Copyright (c) 2025 Caret Contributors
//
// Licensed under the MIT License:
// https://opensource.org/licenses/MIT

use crate::error::Span;

/// Top-level AST node
#[derive(Debug, Clone, PartialEq)]
pub struct AstNode {
    /// Type of node
    pub kind: AstNodeKind,
    /// Source location
    pub span: Span,
}

/// AST node kind
#[derive(Debug, Clone, PartialEq)]
pub enum AstNodeKind {
    /// Statement
    Statement(Statement),
    /// Pipeline declaration
    Pipeline(PipelineDecl),
}

impl AstNode {
    /// Create a new AST node
    pub fn new(kind: AstNodeKind, span: Span) -> Self {
        Self { kind, span }
    }
}

/// Statement
#[derive(Debug, Clone, PartialEq)]
pub enum Statement {
    /// Node declaration (source/sink/process)
    NodeDecl(NodeDecl),
    /// Connection between nodes
    Connection(Connection),
    /// Import statement
    Import { path: String, span: Span },
    /// Export statement
    Export { name: String, span: Span },
    /// Constant declaration
    Const { name: String, value: PropertyValue, span: Span },
}

/// Node type declaration
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum NodeType {
    Source,
    Sink,
    Process,
}

/// Node declaration
#[derive(Debug, Clone, PartialEq)]
pub struct NodeDecl {
    /// Node type (source/sink/process)
    pub node_type: NodeType,
    /// Node name
    pub name: String,
    /// Implementation (e.g., "file", "memory", "transform")
    pub impl_name: String,
    /// Properties/arguments
    pub properties: Vec<Property>,
    /// Source location
    pub span: Span,
}

/// Property in a node declaration
#[derive(Debug, Clone, PartialEq)]
pub struct Property {
    /// Property key (name)
    pub key: PropertyKey,
    /// Property value
    pub value: PropertyValue,
}

/// Property key
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PropertyKey {
    /// Named property (e.g., "path")
    Named(String),
    /// Positional argument index
    Positional(usize),
}

/// Property value
#[derive(Debug, Clone, PartialEq)]
pub enum PropertyValue {
    /// String literal
    String(String),
    /// Number literal
    Number(f64),
    /// Boolean literal
    Boolean(bool),
    /// Identifier/reference
    Identifier(String),
    /// Array of values
    Array(Vec<PropertyValue>),
}

/// Connection between nodes
#[derive(Debug, Clone, PartialEq)]
pub struct Connection {
    /// Source node
    pub source: ConnectionEndpoint,
    /// Target node
    pub target: ConnectionEndpoint,
    /// Source location
    pub span: Span,
}

/// Connection endpoint (node.port or just node)
#[derive(Debug, Clone, PartialEq)]
pub struct ConnectionEndpoint {
    /// Node name
    pub node: String,
    /// Optional port name
    pub port: Option<String>,
    /// Source location
    pub span: Span,
}

/// Pipeline declaration
#[derive(Debug, Clone, PartialEq)]
pub struct PipelineDecl {
    /// Pipeline name
    pub name: String,
    /// Statements within the pipeline
    pub statements: Vec<Statement>,
    /// Source location
    pub span: Span,
}

/// Expression
#[derive(Debug, Clone, PartialEq)]
pub enum Expr {
    /// String literal
    StringLiteral(String),
    /// Number literal
    NumberLiteral(f64),
    /// Boolean literal
    BooleanLiteral(bool),
    /// Identifier
    Identifier(String),
    /// Binary operation
    BinaryOp {
        op: BinaryOp,
        left: Box<Self>,
        right: Box<Self>,
    },
    /// Array literal
    Array(Vec<Self>),
}

/// Binary operation
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BinaryOp {
    Add,
    Sub,
    Mul,
    Div,
    Mod,
    Eq,
    Ne,
    Lt,
    Le,
    Gt,
    Ge,
    And,
    Or,
}

impl NodeType {
    /// Get the corresponding keyword
    pub fn keyword(&self) -> &'static str {
        match self {
            NodeType::Source => "source",
            NodeType::Sink => "sink",
            NodeType::Process => "process",
        }
    }
}

impl PropertyValue {
    /// Check if this is a string value
    pub fn is_string(&self) -> bool {
        matches!(self, PropertyValue::String(_))
    }

    /// Get as string if applicable
    pub fn as_string(&self) -> Option<&str> {
        match self {
            PropertyValue::String(s) => Some(s),
            _ => None,
        }
    }

    /// Check if this is a number value
    pub fn is_number(&self) -> bool {
        matches!(self, PropertyValue::Number(_))
    }

    /// Get as number if applicable
    pub fn as_number(&self) -> Option<f64> {
        match self {
            PropertyValue::Number(n) => Some(*n),
            _ => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_node_decl() {
        let decl = NodeDecl {
            node_type: NodeType::Source,
            name: "input".to_string(),
            impl_name: "file".to_string(),
            properties: vec![
                Property {
                    key: PropertyKey::Named("path".to_string()),
                    value: PropertyValue::String("input.txt".to_string()),
                }
            ],
            span: Span::start(),
        };
        assert_eq!(decl.node_type.keyword(), "source");
    }

    #[test]
    fn test_connection_endpoint() {
        let endpoint = ConnectionEndpoint {
            node: "input".to_string(),
            port: Some("output".to_string()),
            span: Span::start(),
        };
        assert_eq!(endpoint.node, "input");
        assert_eq!(endpoint.port, Some("output".to_string()));
    }
}
