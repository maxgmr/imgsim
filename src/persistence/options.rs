#![warn(missing_docs)]

use clap::ArgMatches;
use serde::Deserialize;
use std::{env, fs, path::Path, path::PathBuf, result::Result};
use strum::IntoEnumIterator;

use crate::{ClusteringAlg, MatchEnumAsStr, PersistenceError, PixeldistAlg, SimilarityAlg};

const CONFIG_PATH_STR: &str = ".config/imgsim/config.toml";

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
}

#[derive(Debug, Deserialize)]
struct Args {
    input_dir: PathBuf,
    #[serde(skip_serializing_if = "Option::is_none")]
    output_dir: Option<PathBuf>,
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
    pub fn build(arg_matches: ArgMatches) -> Result<ImgsimOptions, PersistenceError> {
        // Load config
        let mut config_path = home::home_dir().unwrap();
        config_path.push(CONFIG_PATH_STR);
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

        // If cli arg given and exists, replace input_dir from config.toml
        // Return ReadFileError if input directory doesn't exist or isn't a directory
        fn verify_cli_arg_path(
            cli_arg: Option<&PathBuf>,
        ) -> Result<Option<PathBuf>, PersistenceError> {
            if let Some(cli_arg_unwrapped) = cli_arg {
                if !cli_arg_unwrapped.exists() {
                    return Err(PersistenceError::ReadFileError(Some(PathBuf::from(
                        cli_arg_unwrapped,
                    ))));
                }

                if !cli_arg_unwrapped.is_dir() {
                    return Err(PersistenceError::NotDirectoryError(Some(PathBuf::from(
                        cli_arg_unwrapped,
                    ))));
                }
                return Ok(Some(PathBuf::from(cli_arg_unwrapped)));
            };
            return Ok(None);
        }

        // Get input_dir arg from cli and update imgsim_options if provided
        match verify_cli_arg_path(arg_matches.get_one::<PathBuf>("input_dir")) {
            Ok(Some(path)) => imgsim_options.args.input_dir = path,
            Err(error) => return Err(error),
            _ => (),
        }

        // Get output_dir arg from cli and update imgsim_options if provided
        match verify_cli_arg_path(arg_matches.get_one::<PathBuf>("output_dir")) {
            Ok(Some(path)) => imgsim_options.args.output_dir = Some(path),
            Err(error) => return Err(error),
            _ => (),
        }

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
    pub fn output_dir(&self) -> &Option<PathBuf> {
        &self.args.output_dir
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
