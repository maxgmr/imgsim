#![warn(missing_docs)]

use serde::Deserialize;
use std::{
    collections::BTreeMap,
    mem::take,
    time::{Duration, Instant},
};
use strum_macros::EnumIter;

use crate::{ImgsimImage, ImgsimOptions, MatchEnumAsStr};

#[derive(Debug, Deserialize, EnumIter)]
/// Denotes the type of pixel clustering algorithm being utilised.
pub enum ClusteringAlg {
    #[serde(alias = "kmeans")]
    /// K-means clustering: <https://en.wikipedia.org/wiki/K-means_clustering>
    KMeans,
    #[serde(alias = "agglo", alias = "agglomerative", alias = "agg")]
    /// Agglomerative clustering: More info at <https://github.com/maxgmr/imgsim>
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

/// Builds pixel clusters through agglomeration.
///
/// If two neighbouring pixels have a distance smaller than the nth-percentile distance (set in
/// config.toml), then their clusters are merged together.
pub fn agglomerative(imgsim_image: &mut ImgsimImage, imgsim_options: &ImgsimOptions) {
    // Get nth percentile dist
    let percentile_dist_start = Instant::now();

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

    let percentile_dist_elapsed = percentile_dist_start.elapsed();
    if imgsim_options.debug() {
        println!(
            "\t\"{}\": {:.0}th-centile dist = {:.5}, built in {:.2?}",
            imgsim_image.name(),
            imgsim_options.agglo_tolerance() * 100.0,
            nth_percentile_dist,
            percentile_dist_elapsed,
        );
    }

    // Lookup table to easily find a pixel's cluster
    let mut new_cluster_lookup: BTreeMap<(u32, u32), usize> = BTreeMap::new();
    // This replaces the placeholder pixel_clusters
    let mut new_pixel_clusters: BTreeMap<usize, Vec<(u32, u32)>> = BTreeMap::new();

    let build_maps_start = Instant::now();
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

    let build_maps_elapsed = build_maps_start.elapsed();
    if imgsim_options.debug() {
        println!(
            "\t\"{}\": tables built in {:.2?}.",
            imgsim_image.name(),
            build_maps_elapsed,
        );
    }

    let mut lookup_time_cum = Duration::new(0, 0);
    let mut comparison_time_cum = Duration::new(0, 0);
    let mut take_time_cum = Duration::new(0, 0);
    let mut update_lookup_time_cum = Duration::new(0, 0);
    let mut append_time_cum = Duration::new(0, 0);

    imgsim_image.pixeldist_factors().iter().for_each(|factor| {
        let lookup_start_time = Instant::now();
        // This actually takes the longest now! Wow!
        let a_cluster: usize = new_cluster_lookup[factor.a_coords()];
        let b_cluster: usize = new_cluster_lookup[factor.b_coords()];
        // let a_cluster: usize = *new_cluster_lookup.get(factor.a_coords()).unwrap();
        // let b_cluster: usize = *new_cluster_lookup.get(factor.b_coords()).unwrap();
        lookup_time_cum += lookup_start_time.elapsed();

        if (a_cluster != b_cluster) && (factor.distance() < *nth_percentile_dist) {
            fn consume_cluster(
                predator_cluster_id: usize,
                prey_cluster_id: usize,
                new_pixel_clusters: &mut BTreeMap<usize, Vec<(u32, u32)>>,
                new_cluster_lookup: &mut BTreeMap<(u32, u32), usize>,
                take_time_cum: &mut Duration,
                update_lookup_time_cum: &mut Duration,
                append_time_cum: &mut Duration,
            ) {
                // 2-move, 1-read a consumes b
                // remove all items from prey's cluster
                let take_start_time = Instant::now();
                let mut prey_cluster_items =
                    take(new_pixel_clusters.get_mut(&prey_cluster_id).unwrap());
                *take_time_cum += take_start_time.elapsed();

                // change the cluster of each of b's items on the lookup table
                let update_lookup_start_time = Instant::now();
                prey_cluster_items.iter().for_each(|coords| {
                    *new_cluster_lookup.get_mut(coords).unwrap() = predator_cluster_id;
                });
                *update_lookup_time_cum += update_lookup_start_time.elapsed();

                // move all of b's cluster's items to a's cluster
                let append_start_time = Instant::now();
                new_pixel_clusters
                    .get_mut(&predator_cluster_id)
                    .unwrap()
                    .append(&mut prey_cluster_items);
                *append_time_cum += append_start_time.elapsed();
            }
            let comp_start_time = Instant::now();
            if new_pixel_clusters[&a_cluster].len() > new_pixel_clusters[&b_cluster].len() {
                comparison_time_cum += comp_start_time.elapsed();
                consume_cluster(
                    a_cluster,
                    b_cluster,
                    &mut new_pixel_clusters,
                    &mut new_cluster_lookup,
                    &mut take_time_cum,
                    &mut update_lookup_time_cum,
                    &mut append_time_cum,
                );
            } else {
                comparison_time_cum += comp_start_time.elapsed();
                consume_cluster(
                    b_cluster,
                    a_cluster,
                    &mut new_pixel_clusters,
                    &mut new_cluster_lookup,
                    &mut take_time_cum,
                    &mut update_lookup_time_cum,
                    &mut append_time_cum,
                )
            }
        }
    });

    if imgsim_options.debug() {
        println!("\"{}\": Lookups in {:.2?}; comparisons in {:.2?}; takes in {:.2?}; updates in {:.2?}; appends in {:.2?}",
                imgsim_image.name(), 
                lookup_time_cum, 
                comparison_time_cum, 
                take_time_cum, 
                update_lookup_time_cum, 
                append_time_cum
            );
    }

    *imgsim_image.cluster_lookup_mut() = new_cluster_lookup;
    *imgsim_image.pixel_clusters_mut() = new_pixel_clusters;
}

pub fn k_means(imgsim_image: &mut ImgsimImage, imgsim_options: &ImgsimOptions) {
}
