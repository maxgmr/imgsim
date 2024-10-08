use clap::{command, Arg};
use std::{path::PathBuf, process};

use imgsim::{get_similarities, load_images, ImgsimOptions};

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
        .arg(
            Arg::new("force")
                .short('f')
                .long("force")
                .action(clap::ArgAction::SetTrue)
                .help("Allow imgsim to run with discouraged settings")
        )
        .arg(
            Arg::new("output_dir")
                .short('o')
                .long("output")
                .value_parser(clap::value_parser!(PathBuf))
                .help("The directory to which debug images are saved. Leave this blank to not save any debug images.")
        )
        .get_matches();

    let imgsim_options = match ImgsimOptions::build(match_result) {
        Ok(imgsim_options) => imgsim_options,
        Err(persistence_error) => {
            eprintln!("{}", persistence_error);
            process::exit(1);
        }
    };

    let mut images = match load_images(&imgsim_options) {
        Ok(images) => images,
        Err(persistence_error) => {
            eprintln!("{}", persistence_error);
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

    let image_similarity_matrix = get_similarities(&images, &imgsim_options);
    image_similarity_matrix.print();
    process::exit(0);
}
