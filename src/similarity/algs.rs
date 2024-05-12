#![warn(missing_docs)]

use serde::Deserialize;
use strum_macros::EnumIter;

use crate::MatchEnumAsStr;

#[derive(Debug, Deserialize, EnumIter)]
/// Denotes the type of image similarity algorithm being utilised.
pub enum SimilarityAlg {
    #[serde(alias = "coloursim", alias = "colorsim")]
    /// Matches similar images based on the average colour of their most distinct clusters.
    ColourSim,
}
impl MatchEnumAsStr for SimilarityAlg {}
