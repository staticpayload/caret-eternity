// Caret CLI - Command line interface for Caret
//
// Copyright (c) 2025 Caret Contributors
//
// Licensed under the MIT License:
// https://opensource.org/licenses/MIT

#![warn(missing_docs)]
#![warn(clippy::all)]

mod commands;
mod error;

pub use error::{Error, Result};

use clap::{Parser, Subcommand};
use commands::{RunCommand, ValidateCommand, GraphCommand, BenchCommand};

/// Caret - A high performance stream and graph runtime
#[derive(Parser, Debug, Clone)]
#[command(name = "caret")]
#[command(author, version, about, long_about = None)]
pub struct Cli {
    /// Increase verbosity (can be used multiple times)
    #[arg(short, long, action = clap::ArgAction::Count)]
    pub verbose: u8,

    /// Subcommand to run
    #[command(subcommand)]
    pub command: Command,
}

/// Caret subcommands
#[derive(Subcommand, Debug, Clone)]
pub enum Command {
    /// Run a Caret pipeline
    Run(RunCommand),

    /// Validate a Caret DSL file
    Validate(ValidateCommand),

    /// Generate a graph visualization of a pipeline
    Graph(GraphCommand),

    /// Run benchmarks
    Bench(BenchCommand),
}

/// Run a Caret command
pub async fn run_command(cli: Cli) -> Result<()> {
    match cli.command {
        Command::Run(cmd) => commands::run(cmd, cli.verbose).await,
        Command::Validate(cmd) => commands::validate(cmd, cli.verbose).await,
        Command::Graph(cmd) => commands::graph(cmd, cli.verbose).await,
        Command::Bench(cmd) => commands::bench(cmd, cli.verbose).await,
    }
}
