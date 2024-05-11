#![warn(missing_docs)]

use serde::Deserialize;
use strum_macros::EnumIter;

use crate::MatchEnumAsStr;

#[derive(Debug, Deserialize, EnumIter)]
pub enum PixelsimAlg {
    #[serde(alias = "euclidean")]
    Euclidean,
    #[serde(alias = "redmean")]
    Redmean,
}
impl MatchEnumAsStr for PixelsimAlg {}
