#![warn(missing_docs)]

use clap::ArgMatches;
use std::{env, fs, path::Path, path::PathBuf, process, result::Result};
use toml::Table;

const CONFIG_PATH_STR: &str = "./config/config.toml";
const DEFAULT_PATH_STR: &str = "./config/default.toml";

pub enum PixelsimAlg {
    Euclidean,
}

pub enum ClusteringAlg {
    Kmeans,
}

pub enum SimilarityAlg {
    Coloursim,
}

pub struct ImgsimOptions {
    input_dir: PathBuf,
    pixelsim_alg: PixelsimAlg,
    clustering_alg: ClusteringAlg,
    similarity_alg: SimilarityAlg,
}
impl ImgsimOptions {
    fn load_config() -> Result<Table, &'static str> {
        let config_path = Path::new(CONFIG_PATH_STR);
        let default_path = Path::new(DEFAULT_PATH_STR);
        // TODO: Overwrite config.toml with contents of default.toml
        //     - For now, just use default.toml
        if default_path.is_file() {
            let file_text = match fs::read_to_string(default_path) {
                Ok(text) => text,
                _ => return Err("Error: Could not read config file"),
            };
            match file_text.parse::<Table>() {
                Ok(table) => Ok(table),
                _ => return Err("Error: Could not read config file"),
            }
        } else {
            Err("Error: Could not find config files")
        }
    }

    pub fn build(arg_matches: ArgMatches) -> ImgsimOptions {
        Self::load_config();
        // Get input_dir (default = current working directory)
        let working_directory: PathBuf = match env::current_dir() {
            Ok(dir) => dir,
            Err(_) => {
                eprintln!("Error: Current directory does not exist, or there are insufficient permissions to access the current directory.");
                process::exit(1);
            }
        };

        let input_dir: &PathBuf = arg_matches
            .get_one::<PathBuf>("input_dir")
            .unwrap_or(&working_directory);

        if !input_dir.exists() {
            let input_dir_str = if let Some(dir_str) = input_dir.to_str() {
                dir_str
            } else {
                eprintln!("Error: Problem with input directory (likely does not exist).");
                process::exit(1);
            };
            eprintln!("Error: Input directory \"{input_dir_str}\" does not exist.");
            process::exit(1);
        };

        // TEMP
        ImgsimOptions {
            input_dir: PathBuf::from(input_dir),
            pixelsim_alg: PixelsimAlg::Euclidean,
            clustering_alg: ClusteringAlg::Kmeans,
            similarity_alg: SimilarityAlg::Coloursim,
        }
    }
}
