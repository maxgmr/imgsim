//! Launcher for `imgsim` CLI.
use clap::Parser;
use color_eyre::eyre;

use imgsim::{cli::arg_matcher, utils, Cli};

fn main() -> eyre::Result<()> {
    color_eyre::install()?;

    // Setup: If certain directories don't exist, create them.
    utils::setup()?;

    let args = Cli::parse();
    arg_matcher::match_args(args)?;
    Ok(())
}
