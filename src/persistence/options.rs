#![warn(missing_docs)]

use clap::ArgMatches;
use serde::Deserialize;
use std::{env, fs, path::Path, path::PathBuf, result::Result};

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
            return Err(PersistenceError::ReadFileError(Some(config_path)));
        };

        let mut imgsim_options: ImgsimOptions = match toml::from_str(&config_toml_str) {
            Ok(toml) => toml,
            Err(toml_error) => {
                return Err(PersistenceError::DeserializeError(String::from(
                    toml_error.message(),
                )));
            }
        };

        if imgsim_options.debug() {
            println!("imgsim_options parsed from config.toml:");
            dbg!(&imgsim_options);
        }

        // Get working directory to use if no input_dir arg
        let working_directory: PathBuf = match env::current_dir() {
            Ok(dir) => dir,
            Err(_) => {
                return Err(PersistenceError::ReadFileError(None));
            }
        };

        // Default to working dir
        if imgsim_options.args.input_dir.to_str().unwrap().len() == 0 {
            imgsim_options.args.input_dir = working_directory
        }

        // Get input_dir arg from cli
        let input_dir_cli_arg: Option<&PathBuf> = arg_matches.get_one::<PathBuf>("input_dir");

        // If input_dir cli arg given and exists, replace input_dir from config.toml
        // Return ReadFileError if input directory doesn't exist
        if let Some(input_dir_cli_arg_unwrapped) = input_dir_cli_arg {
            if !input_dir_cli_arg_unwrapped.exists() {
                return Err(PersistenceError::ReadFileError(Some(PathBuf::from(
                    input_dir_cli_arg_unwrapped,
                ))));
            }
            imgsim_options.args.input_dir = PathBuf::from(input_dir_cli_arg_unwrapped);
        };

        // Debug imgsim_options
        if imgsim_options.debug() {
            println!("imgsim_options updated by cli args:");
            dbg!(&imgsim_options);
        }
        return Ok(imgsim_options);
    }

    /// Return the directory of images imgsim compares.
    pub fn input_dir(&self) -> &Path {
        &self.args.input_dir
    }

    /// Return the algorithm used to determine image pixel similarity.
    pub fn pixelsim_alg(&self) -> &PixelsimAlg {
        &self.args.pixelsim_alg
    }

    /// Return the algorithm used to determine pixel clustering.
    pub fn clustering_alg(&self) -> &ClusteringAlg {
        &self.args.clustering_alg
    }

    /// Return the algorithm used to determine image similarity.
    pub fn similarity_alg(&self) -> &SimilarityAlg {
        &self.args.similarity_alg
    }

    /// If `true`, print debug messages.
    pub fn debug(&self) -> bool {
        self.settings.debug
    }
}
