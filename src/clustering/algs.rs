#![warn(missing_docs)]

use serde::Deserialize;
use strum_macros::EnumIter;

use crate::MatchEnumAsStr;

#[derive(Debug, Deserialize, EnumIter)]
/// Denotes the type of pixel clustering algorithm being utilised.
pub enum ClusteringAlg {
    #[serde(alias = "kmeans")]
    /// K-means clustering: https://en.wikipedia.org/wiki/K-means_clustering
    KMeans,
}
impl MatchEnumAsStr for ClusteringAlg {}
