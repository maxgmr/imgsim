#![warn(missing_docs)]

use serde::Deserialize;
use strum_macros::EnumIter;

use crate::MatchEnumAsStr;

#[derive(Debug, Deserialize, EnumIter)]
pub enum ClusteringAlg {
    #[serde(alias = "kmeans")]
    KMeans,
}
impl MatchEnumAsStr for ClusteringAlg {}
