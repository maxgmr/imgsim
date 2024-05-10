#![warn(missing_docs)]

use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub enum PixelsimAlg {
    #[serde(alias = "euclidean")]
    Euclidean,
}
