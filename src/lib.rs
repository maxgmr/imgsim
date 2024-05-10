//! Image similarity-related tools for:
//! * Pixel similarity
//! * Pixel clustering
//! * Image comparison

#![warn(missing_docs)]

mod clustering;
mod persistence;
mod pixelsim;
mod similarity;

pub use persistence::errors::PersistenceError;
pub use persistence::options::ImgsimOptions;
