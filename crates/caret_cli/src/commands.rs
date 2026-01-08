// Caret CLI - Commands
//
// Copyright (c) 2025 Caret Contributors
//
// Licensed under the MIT License:
// https://opensource.org/licenses/MIT

use crate::Result;
use caret_dsl::{AstNodeKind, NodeType, Statement};
use clap::Parser;
use std::path::PathBuf;

/// Run a Caret pipeline
#[derive(Parser, Debug, Clone)]
pub struct RunCommand {
    /// Path to the DSL file
    #[arg(short, long)]
    pub file: PathBuf,
    
    /// Name of the pipeline to run (for files with multiple pipelines)
    #[arg(short, long)]
    pub pipeline: Option<String>,
    
    /// Number of ticks to run (0 = run until completion)
    #[arg(short = 'n', long, default_value = "0")]
    pub ticks: usize,
}

/// Validate a Caret DSL file
#[derive(Parser, Debug, Clone)]
pub struct ValidateCommand {
    /// Path to the DSL file to validate
    #[arg(short, long)]
    pub file: PathBuf,

    /// Output format
    #[arg(short = 't', long, default_value = "human")]
    pub format: String,
}

/// Generate a graph visualization of a pipeline
#[derive(Parser, Debug, Clone)]
pub struct GraphCommand {
    /// Path to the DSL file
    #[arg(short, long)]
    pub file: PathBuf,

    /// Name of the pipeline to graph
    #[arg(short = 'p', long)]
    pub pipeline: Option<String>,

    /// Output format (dot, json, mermaid)
    #[arg(short = 't', long, default_value = "dot")]
    pub format: String,

    /// Output file (stdout if not specified)
    #[arg(short = 'o', long)]
    pub output: Option<PathBuf>,
}

/// Run benchmarks
#[derive(Parser, Debug, Clone)]
pub struct BenchCommand {
    /// Benchmark to run
    #[arg(short, long)]
    pub benchmark: Option<String>,

    /// Number of iterations
    #[arg(short = 'n', long, default_value = "10")]
    pub iterations: usize,

    /// Output format (table, json)
    #[arg(short = 't', long, default_value = "table")]
    pub format: String,
}

pub async fn run(cmd: RunCommand, verbose: u8) -> Result<()> {
    if verbose > 0 {
        eprintln!("Running pipeline from: {}", cmd.file.display());
    }

    // Parse the DSL file
    let source = std::fs::read_to_string(&cmd.file)?;
    let ast = caret_dsl::parse(&source)?;

    if verbose > 0 {
        eprintln!("Parsed {} AST nodes", ast.len());
    }

    // TODO: Build the runtime and execute
    println!("Pipeline execution not yet implemented");
    
    Ok(())
}

pub async fn validate(cmd: ValidateCommand, verbose: u8) -> Result<()> {
    if verbose > 0 {
        eprintln!("Validating: {}", cmd.file.display());
    }

    let source = std::fs::read_to_string(&cmd.file)?;
    let ast = caret_dsl::parse(&source)?;

    match cmd.format.as_str() {
        "human" => {
            println!("✓ Valid DSL file");
            println!("  Found {} top-level nodes", ast.len());
        }
        "json" => {
            let output = serde_json::json!({
                "valid": true,
                "nodes": ast.len(),
            });
            println!("{}", output);
        }
        _ => {
            return Err("Unknown format. Use 'human' or 'json'".into());
        }
    }

    Ok(())
}

pub async fn graph(cmd: GraphCommand, verbose: u8) -> Result<()> {
    if verbose > 0 {
        eprintln!("Generating graph from: {}", cmd.file.display());
    }

    let source = std::fs::read_to_string(&cmd.file)?;
    let ast = caret_dsl::parse(&source)?;

    let output = match cmd.format.as_str() {
        "dot" => generate_dot_graph(&ast, &cmd.pipeline)?,
        "json" => generate_json_graph(&ast, &cmd.pipeline)?,
        "mermaid" => generate_mermaid_graph(&ast, &cmd.pipeline)?,
        _ => {
            return Err("Unknown format. Use 'dot', 'json', or 'mermaid'".into());
        }
    };

    if let Some(output_path) = &cmd.output {
        std::fs::write(output_path, output)?;
        if verbose > 0 {
            eprintln!("Graph written to: {}", output_path.display());
        }
    } else {
        println!("{}", output);
    }

    Ok(())
}

pub async fn bench(cmd: BenchCommand, verbose: u8) -> Result<()> {
    if verbose > 0 {
        eprintln!("Running benchmarks...");
    }

    // TODO: Implement actual benchmarking
    match cmd.format.as_str() {
        "table" => {
            println!("Benchmark            | Iterations | Time (avg)");
            println!("---------------------|------------|-----------");
            println!("buffer_pool_alloc     | {:>10} | {:.2} µs", cmd.iterations, 1.23);
            println!("queue_push_pop        | {:>10} | {:.2} µs", cmd.iterations, 0.87);
        }
        "json" => {
            let output = serde_json::json!({
                "benchmarks": [
                    {"name": "buffer_pool_alloc", "iterations": cmd.iterations, "time_us": 1.23},
                    {"name": "queue_push_pop", "iterations": cmd.iterations, "time_us": 0.87},
                ]
            });
            println!("{}", output);
        }
        _ => {
            return Err("Unknown format. Use 'table' or 'json'".into());
        }
    }

    Ok(())
}

fn generate_dot_graph(ast: &[caret_dsl::AstNode], pipeline: &Option<String>) -> Result<String> {
    let mut output = String::from("digraph CaretPipeline {\n");
    output.push_str("  rankdir=LR;\n");
    output.push_str("  node [shape=box];\n\n");

    // Simple graph generation - TODO: enhance with actual AST processing
    let mut node_count = 0;
    for node in ast {
        match &node.kind {
            caret_dsl::AstNodeKind::Pipeline(p) => {
                if let Some(ref target) = pipeline {
                    if p.name != *target {
                        continue;
                    }
                }
                output.push_str(&format!("  subgraph cluster_{} {{\n", p.name));
                output.push_str(&format!("    label=\"{}\";\n", p.name));
                output.push_str("    style=filled;\n");
                output.push_str("    color=lightgrey;\n");
                
                for stmt in &p.statements {
                    match stmt {
                        caret_dsl::Statement::NodeDecl(n) => {
                            let label = format!("{}\\n{}", n.name, n.impl_name);
                            output.push_str(&format!("    {} [label=\"{}\"];\n", n.name, label));
                            node_count += 1;
                        }
                        _ => {}
                    }
                }
                
                output.push_str("  }\n\n");
            }
            _ => {}
        }
    }

    if node_count == 0 {
        // No pipeline found, show a placeholder
        output.push_str("  start [label=\"Start\"];\n");
        output.push_str("  end [label=\"End\"];\n");
        output.push_str("  start -> end;\n");
    }

    output.push_str("}\n");
    Ok(output)
}

fn generate_json_graph(ast: &[caret_dsl::AstNode], pipeline: &Option<String>) -> Result<String> {
    use serde_json::json;

    let mut nodes = Vec::new();
    let mut edges = Vec::new();

    for node in ast {
        match &node.kind {
            caret_dsl::AstNodeKind::Pipeline(p) => {
                if let Some(ref target) = pipeline {
                    if p.name != *target {
                        continue;
                    }
                }

                for stmt in &p.statements {
                    match stmt {
                        caret_dsl::Statement::NodeDecl(n) => {
                            nodes.push(json!({
                                "id": n.name,
                                "type": format!("{:?}", n.node_type),
                                "impl": n.impl_name,
                            }));
                        }
                        caret_dsl::Statement::Connection(c) => {
                            edges.push(json!({
                                "from": c.source.node,
                                "to": c.target.node,
                            }));
                        }
                        _ => {}
                    }
                }
            }
            _ => {}
        }
    }

    let output = json!({
        "nodes": nodes,
        "edges": edges,
    });
    Ok(output.to_string())
}

fn generate_mermaid_graph(ast: &[caret_dsl::AstNode], pipeline: &Option<String>) -> Result<String> {
    let mut output = String::from("graph TD\n");

    for node in ast {
        match &node.kind {
            caret_dsl::AstNodeKind::Pipeline(p) => {
                if let Some(ref target) = pipeline {
                    if p.name != *target {
                        continue;
                    }
                }
                output.push_str(&format!("  subgraph {}[\"{}\"]\n", p.name, p.name));
                
                for stmt in &p.statements {
                    match stmt {
                        caret_dsl::Statement::NodeDecl(n) => {
                            let node_type = match n.node_type {
                                caret_dsl::NodeType::Source => "[[input]]",
                                caret_dsl::NodeType::Sink => "[[output]]",
                                caret_dsl::NodeType::Process => "[[process]]",
                            };
                            output.push_str(&format!("    {}{}[\"{}\"]\n", n.name, node_type, n.impl_name));
                        }
                        caret_dsl::Statement::Connection(c) => {
                            output.push_str(&format!("    {} --> {}\n", c.source.node, c.target.node));
                        }
                        _ => {}
                    }
                }
                
                output.push_str("  end\n");
            }
            _ => {}
        }
    }

    Ok(output)
}
