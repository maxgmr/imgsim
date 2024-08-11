//! Parse command-line arguments.
use clap::{ArgGroup, Parser, Subcommand};

/// The CLI argument parser.
#[derive(Parser, Debug)]
#[command(name = "imgsim")]
#[command(author)]
// #[command(version = )]
pub struct Cli {}
