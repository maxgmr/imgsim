#![warn(missing_docs)]

use rand::Rng;
use rayon::prelude::*;
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
            if new_cluster_lookup.insert((x, y), cluster_id).is_some() {
                panic!("Tried to add {:?} to lookup table twice", (x, y))
            }
            if new_pixel_clusters
                .insert(cluster_id, vec![(x, y)])
                .is_some()
            {
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
    // TODO: Turn into structs instead of tuples (centroid, cluster_lookup, pixel_clusters)

    // Optimal k: Silhouette (https://en.wikipedia.org/wiki/Silhouette_(clustering))
    // Seeding: k-means++ (https://en.wikipedia.org/wiki/K-means%2B%2B)
    // Naïve k-means

    // If k-means cycles for this many iterations, return
    const MAX_CYCLES: usize = 30;

    // Timers for debugging
    let mut seeding_time_cuml = Duration::new(0, 0);
    let mut k_means_cuml = Duration::new(0, 0);
    let mut copy_old_cuml = Duration::new(0, 0);
    let mut find_closest_cuml = Duration::new(0, 0);
    let mut move_pixels_cuml = Duration::new(0, 0);
    let mut new_centroids_cuml = Duration::new(0, 0);

    // Lookup table to easily find a pixel's cluster
    let mut new_cluster_lookup: BTreeMap<(u32, u32), usize> = BTreeMap::new();
    // This replaces the placeholder pixel_clusters
    let mut new_pixel_clusters: BTreeMap<usize, Vec<(u32, u32)>> = BTreeMap::new();

    // Current best lookup table
    let mut best_cluster_lookup: BTreeMap<(u32, u32), usize> = BTreeMap::new();
    // Current best pixel clusters
    let mut best_pixel_clusters: BTreeMap<usize, Vec<(u32, u32)>> = BTreeMap::new();

    let mut silhouette: Vec<f32> = Vec::with_capacity(imgsim_options.max_k());

    // Iterate through possible k numbers until a reasonable silhouette is achieved
    for k in 2..(imgsim_options.max_k() + 1) {
        // Reset new_cluster_lookup and new_pixel_clusters from last iteration
        new_cluster_lookup.clear();
        new_pixel_clusters.clear();

        // STEP I: k-means++ seeding
        let seeding_start = Instant::now();
        let mut centroids: Vec<((u32, u32), [u8; 3])> = Vec::with_capacity(k);
        // Randomly select first centroid
        let rand_x = rand::thread_rng().gen_range(0..imgsim_image.rgba_image().width());
        let rand_y = rand::thread_rng().gen_range(0..imgsim_image.rgba_image().height());
        let image::Rgba([r_i, g_i, b_i, _]) = *imgsim_image.rgba_image().get_pixel(rand_x, rand_y);
        centroids.push(((rand_x, rand_y), [r_i, g_i, b_i]));
        new_cluster_lookup.insert((rand_x, rand_y), 0);
        new_pixel_clusters.insert(0, vec![(rand_x, rand_y)]);

        // Compute remaining k-1 centroids
        for cluster_id in 0..(k - 1) {
            // Keep track of pixel with maximum distance to all existing centroids
            let mut max_dist_pixel = ((0_u32, 0_u32), [0_u8, 0_u8, 0_u8], 0.0_f32);

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
                    if closest_centroid_dist > max_dist_pixel.2 {
                        max_dist_pixel = ((x, y), [r_p, g_p, b_p], closest_centroid_dist)
                    }
                });

            // The pixel with the greatest distance from all other centroids is the new centroid
            centroids.push((max_dist_pixel.0, max_dist_pixel.1));
            new_cluster_lookup.insert(max_dist_pixel.0, cluster_id + 1);
            new_pixel_clusters.insert(cluster_id + 1, vec![max_dist_pixel.0]);
        }
        seeding_time_cuml += seeding_start.elapsed();

        // STEP II: Naïve k-means
        let k_means_start = Instant::now();

        let mut converged: bool = false;
        let mut iteration_count: usize = 0;
        let mut wcss_history: Vec<f32> = Vec::new();
        while !converged {
            // To know if converged, make a copy of the old cluster map
            let copy_old_start = Instant::now();
            let mut old_cluster_lookup: BTreeMap<(u32, u32), usize> = BTreeMap::new();
            old_cluster_lookup.extend(new_cluster_lookup.iter().map(|(k, v)| (*k, *v)));
            copy_old_cuml += copy_old_start.elapsed();

            // Reset clusters
            new_cluster_lookup.clear();
            new_pixel_clusters.clear();

            let find_closest_start = Instant::now();

            #[derive(Debug)]
            struct Movement {
                coords: (u32, u32),
                cluster_to: usize,
            }

            // Assign each point to closest centroid
            let movement_list: Vec<Movement> = imgsim_image
                .rgba_image()
                .par_enumerate_pixels()
                .map(|(x, y, pixel)| {
                    let image::Rgba([r, g, b, _]) = *pixel;
                    // Sort the pixel into the cluster with the closest centroid
                    let mut min_dist = f32::MAX;
                    let mut min_centroid_coords = (0, 0);
                    for ((c_x, c_y), [c_r, c_g, c_b]) in &centroids {
                        let dist = euclidean_dist(r, g, b, *c_r, *c_g, *c_b);
                        if dist < min_dist {
                            min_dist = dist;
                            min_centroid_coords = (*c_x, *c_y);
                        }
                    }
                    Movement {
                        coords: (x, y),
                        cluster_to: centroids
                            .iter()
                            .position(|(coords, _)| *coords == min_centroid_coords)
                            .unwrap(),
                    }
                })
                .collect();
            find_closest_cuml += find_closest_start.elapsed();

            let move_pixels_start = Instant::now();
            for movement in movement_list {
                // Update cluster lookup table
                new_cluster_lookup.insert(movement.coords, movement.cluster_to);
                // Add value to new cluster group
                if let Some(val) = new_pixel_clusters.get_mut(&movement.cluster_to) {
                    val.push(movement.coords);
                } else {
                    new_pixel_clusters.insert(movement.cluster_to, vec![movement.coords]);
                }
            }
            move_pixels_cuml += move_pixels_start.elapsed();

            let mut new_wcss: f32 = 0.0;

            // Calculate new centroids
            let new_centroids_start = Instant::now();
            centroids = centroids
                .into_iter()
                .map(|(coords, _)| {
                    let centroid_cluster_id = new_cluster_lookup[&coords];
                    let (r_sum, g_sum, b_sum) = new_pixel_clusters[&centroid_cluster_id]
                        .iter()
                        .fold((0_u32, 0_u32, 0_u32), |(a_r, a_g, a_b), (x, y)| {
                            let image::Rgba([r, g, b, _]) =
                                *imgsim_image.rgba_image().get_pixel(*x, *y);
                            (a_r + r as u32, a_g + g as u32, a_b + b as u32)
                        });
                    let cl_len = new_pixel_clusters[&centroid_cluster_id].len();
                    let (r_mean, g_mean, b_mean): (f32, f32, f32) = (
                        r_sum as f32 / cl_len as f32,
                        g_sum as f32 / cl_len as f32,
                        b_sum as f32 / cl_len as f32,
                    );

                    // Get pixel in cluster closest to mean & make it the new centroid
                    let mut closest_dist = f32::MAX;
                    let mut closest_coords: (u32, u32) = (0, 0);
                    let mut closest_rgb: [u8; 3] = [0, 0, 0];
                    new_pixel_clusters[&centroid_cluster_id]
                        .iter()
                        .for_each(|(x, y)| {
                            let image::Rgba([r, g, b, _]) =
                                *imgsim_image.rgba_image().get_pixel(*x, *y);
                            let dist =
                                euclidean_dist(r, g, b, r_mean as u8, g_mean as u8, b_mean as u8);
                            new_wcss += dist;
                            if dist < closest_dist {
                                closest_coords = (*x, *y);
                                closest_dist = dist;
                                closest_rgb = [r, g, b];
                            }
                        });
                    (closest_coords, closest_rgb)
                })
                .collect();
            new_centroids_cuml += new_centroids_start.elapsed();

            // Check if converged
            converged = (|| {
                let mut is_same = true;
                for i in 0..imgsim_image.rgba_image().width() {
                    for j in 0..imgsim_image.rgba_image().height() {
                        if let Some(old_val) = old_cluster_lookup.get(&(i, j)) {
                            if let Some(new_val) = new_cluster_lookup.get(&(i, j)) {
                                if old_val != new_val {
                                    // println!("[{}, {}]: {} != {}", i, j, old_val, new_val);
                                    is_same = false;
                                }
                            } else {
                                // println!("really weird and bad @ [{}, {}]", i, j);
                                return false;
                            }
                        } else if new_cluster_lookup.contains_key(&(i, j)) {
                            return false;
                        }
                    }
                }
                is_same
            })();

            iteration_count += 1;
            wcss_history.push(new_wcss);

            let wcslen = wcss_history.len();
            // Check to see if stuck in cycle.
            // No point in checking if the number of iterations is less than the max allowed cycles.
            // Only return when on the lower-WCSS step of the cycle.
            if wcslen >= MAX_CYCLES && (wcss_history[wcslen - 1] < wcss_history[wcslen - 2]) {
                let mut list_a: Vec<f32> =
                    Vec::with_capacity((wcslen as f32 / 2.0_f32).ceil() as usize);
                let mut list_b: Vec<f32> =
                    Vec::with_capacity((wcslen as f32 / 2.0_f32).ceil() as usize);
                for i in 0..MAX_CYCLES {
                    if i % 2 == 0 {
                        list_a.push(wcss_history[(wcslen - 1) - i]);
                    } else {
                        list_b.push(wcss_history[(wcslen - 1) - i]);
                    }
                }

                let first_a = list_a[0];
                let first_b = list_b[0];
                // If the last MAX_CYCLES cycles were simply going back and forth between two values, then converged
                if list_a.iter().all(|wcss| *wcss == first_a)
                    && list_b.iter().all(|wcss| *wcss == first_b)
                {
                    if imgsim_options.debug() {
                        println!("Converged after {}-length cycle", MAX_CYCLES);
                    }
                    converged = true;
                }
            }
            if imgsim_options.debug() {
                if !converged {
                    // println!(
                    //     "\tnot converged : {} ({} -> {})",
                    //     iteration_count,
                    //     if wcslen > 1 {
                    //         wcss_history[wcslen - 2]
                    //     } else {
                    //         0.0
                    //     },
                    //     wcss_history[wcslen - 1]
                    // );
                } else {
                    println!(
                        "\"{}\": Converged @ k={} in {} iterations.",
                        imgsim_image.name(),
                        k,
                        iteration_count
                    );
                }
            }
        }

        k_means_cuml += k_means_start.elapsed();
    }

    if imgsim_options.debug() {
        println!(
            "\"{}\":\nSeeding in {:.2?};\nk-means in {:.2?} {{\n\tcopy old in {:.2?},\n\tfind closest in {:.2?},\n\tmove pixels in {:.2?},\n\tnew centroids in {:.2?},\n}}",
            imgsim_image.name(),
            seeding_time_cuml,
            k_means_cuml,
            copy_old_cuml,
            find_closest_cuml,
            move_pixels_cuml,
            new_centroids_cuml,
        );
    }

    // STEP III: Silhouette
    imgsim_image.rgba_image().par_pixels();

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
