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
            Arg::new("pixeldist_alg")
                .short('p')
                .long("pixeldist")
                .help("Choose the algorithm for pixel distance"),
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
        .arg(
            Arg::new("verbose")
                .short('v')
                .long("verbose")
                .action(clap::ArgAction::SetTrue)
                .help("Print more messages to terminal")
        )
        .get_matches();

    let imgsim_options = match ImgsimOptions::build(CONFIG_PATH_STR, match_result) {
        Ok(imgsim_options) => imgsim_options,
        Err(persistence_error) => {
            eprintln!("{}", persistence_error.to_string());
            process::exit(1);
        }
    };

    let mut images = match load_images(&imgsim_options) {
        Ok(images) => images,
        Err(persistence_error) => {
            eprintln!("{}", persistence_error.to_string());
            process::exit(1);
        }
    };
    if imgsim_options.debug() || imgsim_options.verbose() {
        println!(
            "{} images loaded from {:?}:\n{}",
            images.len(),
            imgsim_options.input_dir(),
            images
                .iter()
                .map(|image| format!("\t{}", image.name()))
                .collect::<Vec<String>>()
                .join("\n")
        )
    }

    images.iter_mut().for_each(|image| {
        image.build_factors(&imgsim_options);
        image.build_clusters(&imgsim_options);
        image.save_cluster_image(&imgsim_options);
    });

    process::exit(0);
}
