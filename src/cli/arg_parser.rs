//! Parse command-line arguments.
use camino::Utf8PathBuf;
use clap::{ArgGroup, Parser, Subcommand};

use crate::utils;

/// The CLI argument parser.
#[derive(Parser, Debug)]
#[command(name = "imgsim")]
#[command(author)]
#[command(version = utils::info())]
#[command(about = "Utilise various methods to determine the similarity of a group of images.")]
pub struct Cli {
    /// Input directory. Defaults to current working directory.
    pub input_dir: Option<Utf8PathBuf>,
    /// The selected method by which `imgsim` will measure image similarity.
    #[command(subcommand)]
    pub command: Command,
}

/// All the possible ways a user can measure image similarity.
#[derive(Subcommand, Debug)]
pub enum Command {
    /// `clustersize`: Measure image similarity by the size and shape of their most significant
    /// clusters.
    #[command(alias = "cs")]
    #[clap(
        group(
            ArgGroup::new("clustersize")
                .required(false)
                .args(&["tolerance"])
        )
    )]
    Clustersize {
        /// Clustering tolerance.
        #[clap(short, long, requires = "tolerance_amount")]
        tolerance: bool,
        /// Amount of tolerance.
        tolerance_amount: Option<u32>,
    },
}
