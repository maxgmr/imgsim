//! Image similarity-related tools for:
//! * Pixel similarity
//! * Pixel clustering
//! * Image comparison

#![warn(missing_docs)]

use clap::ArgMatches;

mod clustering;
mod persistence;
mod pixelsim;
mod similarity;

pub fn get_imgsim_options(match_result: ArgMatches) -> persistence::options::ImgsimOptions {
    persistence::options::ImgsimOptions::build(match_result)
}
