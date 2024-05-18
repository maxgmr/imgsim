#![warn(missing_docs)]

use image::GenericImageView;
use rand::Rng;
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

    let mut lookup_time_cuml = Duration::new(0, 0);
    let mut comparison_time_cuml = Duration::new(0, 0);
    let mut take_time_cuml = Duration::new(0, 0);
    let mut update_lookup_time_cuml = Duration::new(0, 0);
    let mut append_time_cuml = Duration::new(0, 0);

    imgsim_image.pixeldist_factors().iter().for_each(|factor| {
        let lookup_start_time = Instant::now();
        // This actually takes the longest now! Wow!
        let a_cluster: usize = new_cluster_lookup[factor.a_coords()];
        let b_cluster: usize = new_cluster_lookup[factor.b_coords()];
        // let a_cluster: usize = *new_cluster_lookup.get(factor.a_coords()).unwrap();
        // let b_cluster: usize = *new_cluster_lookup.get(factor.b_coords()).unwrap();
        lookup_time_cuml += lookup_start_time.elapsed();

        if (a_cluster != b_cluster) && (factor.distance() < *nth_percentile_dist) {
            fn consume_cluster(
                predator_cluster_id: usize,
                prey_cluster_id: usize,
                new_pixel_clusters: &mut BTreeMap<usize, Vec<(u32, u32)>>,
                new_cluster_lookup: &mut BTreeMap<(u32, u32), usize>,
                take_time_cuml: &mut Duration,
                update_lookup_time_cuml: &mut Duration,
                append_time_cuml: &mut Duration,
            ) {
                // 2-move, 1-read a consumes b
                // remove all items from prey's cluster
                let take_start_time = Instant::now();
                let mut prey_cluster_items =
                    take(new_pixel_clusters.get_mut(&prey_cluster_id).unwrap());
                *take_time_cuml += take_start_time.elapsed();

                // change the cluster of each of b's items on the lookup table
                let update_lookup_start_time = Instant::now();
                prey_cluster_items.iter().for_each(|coords| {
                    *new_cluster_lookup.get_mut(coords).unwrap() = predator_cluster_id;
                });
                *update_lookup_time_cuml += update_lookup_start_time.elapsed();

                // move all of b's cluster's items to a's cluster
                let append_start_time = Instant::now();
                new_pixel_clusters
                    .get_mut(&predator_cluster_id)
                    .unwrap()
                    .append(&mut prey_cluster_items);
                *append_time_cuml += append_start_time.elapsed();
            }
            let comp_start_time = Instant::now();
            if new_pixel_clusters[&a_cluster].len() > new_pixel_clusters[&b_cluster].len() {
                comparison_time_cuml += comp_start_time.elapsed();
                consume_cluster(
                    a_cluster,
                    b_cluster,
                    &mut new_pixel_clusters,
                    &mut new_cluster_lookup,
                    &mut take_time_cuml,
                    &mut update_lookup_time_cuml,
                    &mut append_time_cuml,
                );
            } else {
                comparison_time_cuml += comp_start_time.elapsed();
                consume_cluster(
                    b_cluster,
                    a_cluster,
                    &mut new_pixel_clusters,
                    &mut new_cluster_lookup,
                    &mut take_time_cuml,
                    &mut update_lookup_time_cuml,
                    &mut append_time_cuml,
                )
            }
        }
    });

    if imgsim_options.debug() {
        println!("\"{}\": Lookups in {:.2?}; comparisons in {:.2?}; takes in {:.2?}; updates in {:.2?}; appends in {:.2?}",
                imgsim_image.name(),
                lookup_time_cuml,
                comparison_time_cuml,
                take_time_cuml,
                update_lookup_time_cuml,
                append_time_cuml
            );
    }

    *imgsim_image.cluster_lookup_mut() = new_cluster_lookup;
    *imgsim_image.pixel_clusters_mut() = new_pixel_clusters;
}

pub fn k_means(imgsim_image: &mut ImgsimImage, imgsim_options: &ImgsimOptions) {
    // Optimal k: Silhouette (https://en.wikipedia.org/wiki/Silhouette_(clustering))
    // Seeding: k-means++ (https://en.wikipedia.org/wiki/K-means%2B%2B)
    // 3d tree: https://dl.acm.org/doi/pdf/10.1145/361002.361007

    // Timers for debugging
    let mut seeding_time_cuml = Duration::new(0, 0);

    // Lookup table to easily find a pixel's cluster
    let mut new_cluster_lookup: BTreeMap<(u32, u32), usize> = BTreeMap::new();
    // This replaces the placeholder pixel_clusters
    let mut new_pixel_clusters: BTreeMap<usize, Vec<(u32, u32)>> = BTreeMap::new();

    let mut silhouettes: Vec<f32> = Vec::with_capacity(imgsim_options.max_k());

    // Iterate through possible k numbers until a reasonable silhouette is achieved
    for k in 2..(imgsim_options.max_k() + 1) {
        // STEP I: k-means++ seeding
        let seeding_start = Instant::now();
        let mut centroids: Vec<((u32, u32), [u8; 3])> = Vec::with_capacity(k);
        // Randomly select first centroid
        let rand_x = rand::thread_rng().gen_range(0..imgsim_image.rgba_image().width());
        let rand_y = rand::thread_rng().gen_range(0..imgsim_image.rgba_image().height());
        let image::Rgba([r_i, g_i, b_i, _]) = *imgsim_image.rgba_image().get_pixel(rand_x, rand_y);
        centroids.push(((rand_x, rand_y), [r_i, g_i, b_i]));

        // Compute remaining k-1 centroids
        for _ in 0..(k - 1) {
            // Keep track of pixel with maximum distance to all existing centroids
            let mut max_dist = ((0_u32, 0_u32), [0_u8, 0_u8, 0_u8], 0.0_f32);

            imgsim_image
                .rgba_image()
                .enumerate_pixels()
                .for_each(|(x, y, pixel)| {
                    let mut closest_centroid_dist = f32::MAX;
                    let image::Rgba([r_p, g_p, b_p, _]) = *pixel;

                    // Get the distance of the centroid closest to this pixel
                    centroids.iter().for_each(|(_, [r_c, g_c, b_c])| {
                        let dist = euclidean_dist(r_p, g_p, b_p, *r_c, *g_c, *b_c);
                        if dist < closest_centroid_dist {
                            closest_centroid_dist = dist
                        }
                    });

                    // If distance to closest centroid is larger than any other pixel's distance to closest centroid so far, replace
                    if closest_centroid_dist > max_dist.2 {
                        max_dist = ((x, y), [r_p, g_p, b_p], closest_centroid_dist)
                    }
                });

            // The pixel with the greatest distance from all other centroids is the new centroid
            centroids.push((max_dist.0, max_dist.1));
        }
        seeding_time_cuml += seeding_start.elapsed();

        // STEP II:
        if imgsim_options.debug() {
            println!(
                "\"{}\": Seeding in {:.2?};",
                imgsim_image.name(),
                seeding_time_cuml,
            );
        }
    }

    *imgsim_image.cluster_lookup_mut() = new_cluster_lookup;
    *imgsim_image.pixel_clusters_mut() = new_pixel_clusters;
}

// TODO: Consolidate all Euclidean distance functions as single helper function
fn euclidean_dist(r_a: u8, b_a: u8, g_a: u8, r_b: u8, b_b: u8, g_b: u8) -> f32 {
    (((r_a as i64 - r_b as i64).pow(2)
        + (g_a as i64 - g_b as i64).pow(2)
        + (b_a as i64 - b_b as i64).pow(2)) as f32)
        .sqrt()
}

#[cfg(test)]
mod tests {
    use super::*;
    use pretty_assertions::assert_eq;

    #[test]
    fn euclidean_dist_max() {
        let ans = (195075.0_f32).sqrt();
        assert_eq!(ans, euclidean_dist(255, 0, 255, 0, 255, 0));
    }
}
