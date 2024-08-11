//! Utilise various methods to determine the similarity of a group of images.
#![warn(missing_docs)]

pub mod cli;
pub mod utils;

// Re-exports
pub use cli::arg_parser::Cli;
