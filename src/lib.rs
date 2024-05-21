//! Image similarity-related tools for:
//! * Pixel distance
//! * Pixel clustering
//! * Image comparison

#![warn(missing_docs)]

use std::fmt::Debug;

mod clustering;
mod data;
mod persistence;
mod pixeldist;
mod similarity;

pub use persistence::errors::PersistenceError;
pub use persistence::load_images::load_images;
pub use persistence::options::ImgsimOptions;

pub use data::helpers;
pub use data::imgsim_image::{ImgsimImage, PixeldistFactor};
pub use data::kd_tree;

pub use pixeldist::algs::{get_pixeldist, PixeldistAlg};

pub use clustering::algs::{get_clusters, ClusteringAlg};

pub use similarity::algs::{get_similarities, ImageSimilarityMatrix, SimilarityAlg};
pub use similarity::errors::ImageSimilarityMatrixNoMatchError;

/// Values that implement [MatchEnumAsStr] can compare their enum values to a given `&str` (case-insensitive)
///
/// See function [MatchEnumAsStr::match_enum_as_str]
pub trait MatchEnumAsStr: Debug {
    /// Return `true` if the given enum value matches the given string (case-insensitive)
    ///
    /// # Examples
    ///
    /// Here, `Colour::Red` matches the given string:
    ///
    /// ```
    /// use imgsim::MatchEnumAsStr;
    /// #[derive(Debug)]
    /// enum Colour {
    ///     Red,
    ///     Blue,
    ///     Green,
    /// }
    /// impl MatchEnumAsStr for Colour {}
    /// assert_eq![Colour::Red.match_enum_as_str("red"), true]
    /// ```
    ///
    /// Here, `Colour::Blue` does not match the given string:
    ///
    /// ```
    /// use imgsim::MatchEnumAsStr;
    /// #[derive(Debug)]
    /// enum Colour {
    ///     Red,
    ///     Blue,
    ///     Green,
    /// }
    /// impl MatchEnumAsStr for Colour {}
    /// assert_eq![Colour::Blue.match_enum_as_str("yellow"), false]
    /// ```
    fn match_enum_as_str(&self, string: &str) -> bool {
        format!("{:?}", &self).to_lowercase() == string
    }
}
