#![warn(missing_docs)]

use serde::Deserialize;
use std::{collections::BTreeMap, mem::take};
use strum_macros::EnumIter;

use crate::{ImgsimImage, ImgsimOptions, MatchEnumAsStr, PixeldistFactor};

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
pub fn get_clusters(imgsim_image: &mut ImgsimImage, imgsim_options: &ImgsimOptions) {
    match imgsim_options.clustering_alg() {
        ClusteringAlg::Agglomerative => {
            agglomerative(imgsim_image, imgsim_options);
        }
        ClusteringAlg::KMeans => {
            k_means(imgsim_image, imgsim_options);
        }
    };
}

pub fn agglomerative(imgsim_image: &mut ImgsimImage, imgsim_options: &ImgsimOptions) {
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

    if imgsim_options.debug() {
        println!(
            "\"{}\": {}th-percentile distance = {}",
            imgsim_image.name(),
            imgsim_options.agglo_tolerance() * 100.0,
            nth_percentile_dist
        );
    }

    // Lookup table to easily find a pixel's cluster
    let mut new_cluster_lookup: BTreeMap<(u32, u32), usize> = BTreeMap::new();
    // This replaces the placeholder pixel_clusters
    let mut new_pixel_clusters: BTreeMap<usize, Vec<(u32, u32)>> = BTreeMap::new();

    // Assign each pixel to its own cluster
    let mut cluster_id: usize = 0;
    imgsim_image
        .rgba_image()
        .enumerate_pixels()
        .for_each(|(x, y, _)| {
            if let Some(_) = new_cluster_lookup.insert((x, y), cluster_id) {
                panic!("Tried to add {:?} to lookup table twice", (x, y))
            }
            if let Some(_) = new_pixel_clusters.insert(cluster_id, vec![(x, y)]) {
                panic!("Tried to add {} to new_pixel_clusters twice", cluster_id)
            }
            cluster_id += 1;
        });

    imgsim_image.pixeldist_factors().iter().for_each(|factor| {
        let a_cluster: usize = *new_cluster_lookup.get(factor.a_coords()).unwrap();
        let b_cluster: usize = *new_cluster_lookup.get(factor.b_coords()).unwrap();

        if (a_cluster != b_cluster) && (factor.distance() < *nth_percentile_dist) {
            // TODO try other way around for performance
            // a's cluster consumes b's cluster
            let mut b_cluster_items = take(new_pixel_clusters.get_mut(&b_cluster).unwrap());
            b_cluster_items.iter().for_each(|coords| {
                *new_cluster_lookup.get_mut(coords).unwrap() = a_cluster;
            });
            new_pixel_clusters
                .get_mut(&a_cluster)
                .unwrap()
                .append(&mut b_cluster_items);
        }
    });

    *imgsim_image.cluster_lookup_mut() = new_cluster_lookup;
    *imgsim_image.pixel_clusters_mut() = new_pixel_clusters;
}

pub fn k_means(imgsim_image: &mut ImgsimImage, imgsim_options: &ImgsimOptions) {
    // TODO
}
