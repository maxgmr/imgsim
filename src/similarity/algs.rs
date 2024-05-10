#![warn(missing_docs)]

use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub enum SimilarityAlg {
    #[serde(alias = "coloursim", alias = "colorsim")]
    ColourSim,
}
