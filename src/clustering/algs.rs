#![warn(missing_docs)]

use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub enum ClusteringAlg {
    #[serde(alias = "kmeans")]
    KMeans,
}
