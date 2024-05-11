#![warn(missing_docs)]

use serde::Deserialize;
use strum_macros::EnumIter;

use crate::MatchEnumAsStr;

#[derive(Debug, Deserialize, EnumIter)]
pub enum SimilarityAlg {
    #[serde(alias = "coloursim", alias = "colorsim")]
    ColourSim,
}
impl MatchEnumAsStr for SimilarityAlg {}
