use clap::{command, Arg};
use std::{env, path::PathBuf, process};

const DEBUG: bool = true;

fn main() {
    let match_result = command!()
        .about("A tool that finds similar images through various methods.")
        .arg(
            Arg::new("input_dir")
                .value_parser(clap::value_parser!(PathBuf))
                .help("The path to the directory of images you wish to compare"),
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

    let working_directory: PathBuf = match env::current_dir() {
        Ok(dir) => dir,
        Err(_) => {
            eprintln!("Error: Current directory does not exist, or there are insufficient permissions to access the current directory.");
            process::exit(1);
        }
    };

    let input_dir: &PathBuf = match_result
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
    }

    if DEBUG {
        dbg!(input_dir);
    }
}
