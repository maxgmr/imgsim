#![warn(missing_docs)]

use serde::Deserialize;
use std::time::Instant;
use strum_macros::EnumIter;

use crate::{ImgsimImage, ImgsimOptions, MatchEnumAsStr};

#[derive(Debug, Deserialize, EnumIter)]
/// Denotes the type of pixel clustering algorithm being utilised.
pub enum ClusteringAlg {
    #[serde(alias = "kmeans")]
    /// K-means clustering: https://en.wikipedia.org/wiki/K-means_clustering
    KMeans,
    #[serde(alias = "agglo", alias = "agglomerative", alias = "agg")]
    /// Agglomerative clustering
    Agglomerative,
}
impl MatchEnumAsStr for ClusteringAlg {}

/// Take [ImgsimImage], build pixel clusters, then return
pub fn get_clusters<'a>(
    imgsim_image: ImgsimImage<'a>,
    imgsim_options: &ImgsimOptions,
) -> ImgsimImage<'a> {
    match imgsim_options.clustering_alg() {
        ClusteringAlg::Agglomerative => agglomerative(imgsim_image, imgsim_options),
        ClusteringAlg::KMeans => k_means(imgsim_image, imgsim_options),
    }
}

pub fn agglomerative<'a>(
    imgsim_image: ImgsimImage<'a>,
    imgsim_options: &ImgsimOptions,
) -> ImgsimImage<'a> {
    let start_time = Instant::now();
    // Get nth percentile dist
    let mut sorted_dists: Vec<f32> = imgsim_image
        .pixeldist_factors()
        .iter()
        .map(|factor| factor.distance())
        .collect::<Vec<f32>>();
    sorted_dists.sort_unstable_by(|a, b| a.partial_cmp(b).unwrap());
    let nth_percentile_dist = if let Some(val) = sorted_dists
        .get((sorted_dists.len() as f32 * imgsim_options.agglo_tolerance()).ceil() as usize)
    {
        val
    } else {
        sorted_dists
            .get((sorted_dists.len() as f32 * imgsim_options.agglo_tolerance()).floor() as usize)
            .unwrap()
    };
    for factor in imgsim_image.pixeldist_factors() {}
    let elapsed_time = start_time.elapsed();
    if imgsim_options.debug() {
        println!(
            "Built {} clusters of {} in {:.2?}.",
            imgsim_image.clusters().len(),
            imgsim_image.name(),
            elapsed_time
        );
    } else if imgsim_options.verbose() {
        println!(
            "\"{}\" clusters done in {:.2?}.",
            imgsim_image.name(),
            elapsed_time
        );
    }
    imgsim_image
}

pub fn k_means<'a>(
    imgsim_image: ImgsimImage<'a>,
    imgsim_options: &ImgsimOptions,
) -> ImgsimImage<'a> {
    let start_time = Instant::now();
    // TODO
    imgsim_image
}
