//! Image similarity-related tools for:
//! * Pixel similarity
//! * Pixel clustering
//! * Image comparison

#![warn(missing_docs)]

use std::fmt::Debug;

mod clustering;
mod data;
mod persistence;
mod pixelsim;
mod similarity;

pub use data::imgsim_image::ImgsimImage;
pub use persistence::errors::PersistenceError;
pub use persistence::load_images::load_images;
pub use persistence::options::ImgsimOptions;

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
