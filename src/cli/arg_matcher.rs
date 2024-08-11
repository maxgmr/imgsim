//! Route CLI arguments to different bits of functionality.
use color_eyre::eyre::{self, eyre};

use super::arg_parser::Cli;

/// Match the CLI arguments with their proper functionality.
pub fn match_args(args: Cli) -> eyre::Result<()> {
    // TODO
    Ok(())
}
