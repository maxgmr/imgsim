#![warn(missing_docs)]

use clap::ArgMatches;
use serde::Deserialize;
use std::{env, fs, path::Path, path::PathBuf, process, result::Result};

use super::super::{
    clustering::algs::ClusteringAlg, pixelsim::algs::PixelsimAlg, similarity::algs::SimilarityAlg,
};
use super::errors::PersistenceError;

#[derive(Debug, Deserialize)]
struct Settings {
    debug: bool,
}

#[derive(Debug, Deserialize)]
struct Args {
    input_dir: PathBuf,
    pixelsim_alg: PixelsimAlg,
    clustering_alg: ClusteringAlg,
    similarity_alg: SimilarityAlg,
}

/// A variety of options and parameters that determine how imgsim acts. Values are accessed through the methods.
#[derive(Debug, Deserialize)]
pub struct ImgsimOptions {
    args: Args,
    settings: Settings,
}
impl ImgsimOptions {
    /// Create a new ImgsimOptions. Return [PersistenceError] on failure to read file or deserialise.
    ///
    /// Argument `config_path_str` must point to a valid `.toml` file.
    pub fn build(
        config_path_str: &str,
        arg_matches: ArgMatches,
    ) -> Result<ImgsimOptions, PersistenceError> {
        // Load config
        let config_path = PathBuf::from(config_path_str);
        let config_toml_str = if let Ok(string) = fs::read_to_string(&config_path) {
            string
        } else {
            return Err(PersistenceError::ReadFileError(config_path));
        };

        let mut imgsim_options: ImgsimOptions = match toml::from_str(&config_toml_str) {
            Ok(toml) => toml,
            Err(toml_error) => {
                return Err(PersistenceError::DeserializeError(String::from(
                    toml_error.message(),
                )));
            }
        };

        // Get input_dir (default = current working directory)
        let working_directory: PathBuf = match env::current_dir() {
            Ok(dir) => dir,
            Err(_) => {
                // TODO: Propagate error
                eprintln!("Error: Working directory does not exist, or there are insufficient permissions to access the current directory.");
                process::exit(1);
            }
        };

        let input_dir_arg: &PathBuf = arg_matches
            .get_one::<PathBuf>("input_dir")
            .unwrap_or(&working_directory);

        if !input_dir_arg.exists() {
            let input_dir_str = if let Some(dir_str) = input_dir_arg.to_str() {
                dir_str
            } else {
                eprintln!("Error: Problem with input directory (likely does not exist).");
                process::exit(1);
            };
            eprintln!("Error: Input directory \"{input_dir_str}\" does not exist.");
            process::exit(1);
        };

        dbg!(&imgsim_options);

        return Ok(imgsim_options);
    }

    /// Return the directory of images imgsim compares.
    pub fn input_dir(&self) -> &Path {
        &self.args.input_dir
    }

    pub fn pixelsim_alg(&self) -> &PixelsimAlg {
        &self.args.pixelsim_alg
    }

    pub fn clustering_alg(&self) -> &ClusteringAlg {
        &self.args.clustering_alg
    }

    pub fn similarity_alg(&self) -> &SimilarityAlg {
        &self.args.similarity_alg
    }

    pub fn debug(&self) -> bool {
        self.settings.debug
    }
}
