#![warn(missing_docs)]

use clap::ArgMatches;
use serde::Deserialize;
use std::{env, fs, path::Path, path::PathBuf, result::Result};
use strum::IntoEnumIterator;

use crate::{ClusteringAlg, MatchEnumAsStr, PersistenceError, PixeldistAlg, SimilarityAlg};

#[derive(Debug, Deserialize)]
struct ColoursimOptions {
    cluster_cutoff: f32,
}

#[derive(Debug, Deserialize)]
struct AgglomerativeOptions {
    tolerance: f32,
}

#[derive(Debug, Deserialize)]
struct Settings {
    debug: bool,
    verbose: bool,
    max_width: u32,
    max_height: u32,
    output_dir: PathBuf,
}

#[derive(Debug, Deserialize)]
struct Args {
    input_dir: PathBuf,
    pixeldist_alg: PixeldistAlg,
    clustering_alg: ClusteringAlg,
    similarity_alg: SimilarityAlg,
}

/// A variety of options and parameters that determine how imgsim acts. Values are accessed through the methods.
#[derive(Debug, Deserialize)]
pub struct ImgsimOptions {
    args: Args,
    settings: Settings,
    agglomerative_options: AgglomerativeOptions,
    coloursim_options: ColoursimOptions,
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
        // Return ReadFileError if input directory doesn't exist or isn't a directory
        if let Some(input_dir_cli_arg_unwrapped) = input_dir_cli_arg {
            if !input_dir_cli_arg_unwrapped.exists() {
                return Err(PersistenceError::ReadFileError(Some(PathBuf::from(
                    input_dir_cli_arg_unwrapped,
                ))));
            }

            if !input_dir_cli_arg_unwrapped.is_dir() {
                return Err(PersistenceError::NotDirectoryError(Some(PathBuf::from(
                    input_dir_cli_arg_unwrapped,
                ))));
            }

            imgsim_options.args.input_dir = PathBuf::from(input_dir_cli_arg_unwrapped);
        };

        // Update any config values with corresponding cli args
        fn get_cli_arg<T: IntoEnumIterator + MatchEnumAsStr>(
            arg_matches: &ArgMatches,
            id: &str,
        ) -> Option<T> {
            let cli_arg_val = arg_matches.get_one::<String>(id);
            if let Some(val) = cli_arg_val {
                // match value to enum
                for option in T::iter() {
                    if option.match_enum_as_str(val) {
                        return Some(option);
                    }
                }
                None
            } else {
                None
            }
        }

        // update pixeldist_alg if given in cli
        if let Some(arg) = get_cli_arg::<PixeldistAlg>(&arg_matches, "pixeldist_alg") {
            imgsim_options.args.pixeldist_alg = arg;
        }

        // update clustering_alg if given in cli
        if let Some(arg) = get_cli_arg::<ClusteringAlg>(&arg_matches, "clustering_alg") {
            imgsim_options.args.clustering_alg = arg;
        }

        // update similarity_alg if given in cli
        if let Some(arg) = get_cli_arg::<SimilarityAlg>(&arg_matches, "similarity_alg") {
            imgsim_options.args.similarity_alg = arg;
        }

        // update verbose
        imgsim_options.settings.verbose = arg_matches.get_flag("verbose");

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
    pub fn pixeldist_alg(&self) -> &PixeldistAlg {
        &self.args.pixeldist_alg
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

    /// If `true`, print messages to terminal.
    pub fn verbose(&self) -> bool {
        self.settings.verbose
    }

    /// Return the max width of an input image;
    pub fn max_width(&self) -> u32 {
        self.settings.max_width
    }

    /// Return the max height of an input image.
    pub fn max_height(&self) -> u32 {
        self.settings.max_height
    }

    /// Return the output directory path.
    pub fn output_dir(&self) -> &PathBuf {
        &self.settings.output_dir
    }

    /// Return the tolerance of the agglomerative clustering algorithm.
    pub fn agglo_tolerance(&self) -> f32 {
        self.agglomerative_options.tolerance
    }

    /// Return the cluster cutoff point for the coloursim similarity algorithm.
    pub fn cluster_cutoff(&self) -> f32 {
        self.coloursim_options.cluster_cutoff
    }
}
