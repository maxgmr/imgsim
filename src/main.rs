use clap::{command, Arg};
use std::{path::PathBuf, process};

use imgsim::{load_images, ImgsimOptions};

const CONFIG_PATH_STR: &str = "./config/config.toml";

fn main() {
    let match_result = command!()
        .about("A tool that finds similar images through various methods.")
        .arg(
            Arg::new("input_dir")
                .value_parser(clap::value_parser!(PathBuf))
                .help("The path to the directory of images you wish to compare (default: working directory)"),
        )
        .arg(
            Arg::new("pixelsim_alg")
                .short('p')
                .long("pixelsim")
                .help("Choose the algorithm for pixel similarity"),
        )
        .arg(
            Arg::new("clustering_alg")
                .short('c')
                .long("clustering")
                .help("Choose the algorithm for pixel clustering"),
        )
        .arg(
            Arg::new("similarity_alg")
                .short('s')
                .long("similarity")
                .help("Choose the algorithm for image similarity"),
        )
        .get_matches();

    let imgsim_options = match ImgsimOptions::build(CONFIG_PATH_STR, match_result) {
        Ok(imgsim_options) => imgsim_options,
        Err(persistence_error) => {
            eprintln!("{}", persistence_error.to_string());
            process::exit(1);
        }
    };

    let images = match load_images(imgsim_options.input_dir()) {
        Ok(images) => images,
        Err(persistence_error) => {
            eprintln!("{}", persistence_error.to_string());
            process::exit(1);
        }
    };
    process::exit(0);
}
