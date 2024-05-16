#![warn(missing_docs)]

use rayon::prelude::*;
use serde::Deserialize;
use std::collections::HashMap;
use strum_macros::EnumIter;

use crate::{ImageSimilarityMatrixNoMatchError, ImgsimImage, ImgsimOptions, MatchEnumAsStr};

#[derive(Debug, Deserialize, EnumIter)]
/// Denotes the type of image similarity algorithm being utilised.
pub enum SimilarityAlg {
    #[serde(alias = "coloursim", alias = "colorsim")]
    /// Matches similar images based on the average colour of their most distinct clusters.
    ColourSim,
    #[serde(alias = "clustersize")]
    /// Matches similar images based on the relative shape and size of their most distinct clusters.
    ClusterSize,
}
impl MatchEnumAsStr for SimilarityAlg {}

/// Get each image's similarity to every other image
pub fn get_similarities(
    images: &Vec<ImgsimImage>,
    imgsim_options: &ImgsimOptions,
) -> ImageSimilarityMatrix {
    let mut output_matrix = ImageSimilarityMatrix::from(images);
    match imgsim_options.similarity_alg() {
        SimilarityAlg::ColourSim => output_matrix.colour_sim(&images, &imgsim_options),
        SimilarityAlg::ClusterSize => output_matrix.cluster_size(&images, &imgsim_options),
    };
    return output_matrix;
}

/// A matrix of [ImgsimImage] pairings and their similarities.
#[derive(Debug)]
pub struct ImageSimilarityMatrix {
    matrix: HashMap<(String, String), Option<f32>>,
}
impl ImageSimilarityMatrix {
    /// Builds an empty [ImageSimilarityMatrix] out of a provided list of images.
    fn from(images: &Vec<ImgsimImage>) -> ImageSimilarityMatrix {
        let combinations = (images.len().pow(2) - images.len()) / 2;
        let mut matrix: HashMap<(String, String), Option<f32>> =
            HashMap::with_capacity(combinations);

        for i in 0..images.len() {
            for j in i + 1..images.len() {
                matrix.insert(
                    (
                        String::from(images[i].name()),
                        String::from(images[j].name()),
                    ),
                    None,
                );
            }
        }
        ImageSimilarityMatrix { matrix }
    }

    /// Returns the matrix.
    pub fn matrix(&self) -> &HashMap<(String, String), Option<f32>> {
        &self.matrix
    }

    /// Return the similarity of the two given [ImgsimImage] names.
    pub fn get(
        &self,
        (name1, name2): (&str, &str),
    ) -> Result<&Option<f32>, ImageSimilarityMatrixNoMatchError> {
        if let Some(option_similarity) =
            self.matrix.get(&(String::from(name1), String::from(name2)))
        {
            Ok(option_similarity)
        } else if let Some(option_similarity) =
            self.matrix.get(&(String::from(name2), String::from(name1)))
        {
            Ok(option_similarity)
        } else {
            Err(ImageSimilarityMatrixNoMatchError("Cannot find entry"))
        }
    }

    fn colour_sim(&mut self, images: &Vec<ImgsimImage>, imgsim_options: &ImgsimOptions) {
        #[derive(Debug)]
        struct ClusterInfo {
            _cluster_id: usize,
            size: usize,
            average_rgba: (u8, u8, u8, u8),
        }
        // I. SETUP
        // For each image, get a list of its clusters and their average RGBA values, sorted by size.
        let clusters_info: Vec<(&str, Vec<ClusterInfo>)> = images
            .par_iter()
            .map(|image| {
                let img_size =
                    image.rgba_image().width() as usize * image.rgba_image().height() as usize;
                let mut clusters_info: Vec<ClusterInfo> = Vec::with_capacity(img_size);
                for cluster in image.pixel_clusters() {
                    let size = cluster.1.len();
                    if size
                        > (img_size as f32 * imgsim_options.coloursim_cluster_cutoff()).round()
                            as usize
                    {
                        let colour_sum =
                            cluster.1.iter().fold((0, 0, 0, 0), |accumulator, coords| {
                                let image::Rgba(data) =
                                    *image.rgba_image().get_pixel(coords.0, coords.1);
                                (
                                    accumulator.0 + data[0] as u128,
                                    accumulator.1 + data[1] as u128,
                                    accumulator.2 + data[2] as u128,
                                    accumulator.3 + data[3] as u128,
                                )
                            });
                        clusters_info.push(ClusterInfo {
                            _cluster_id: *cluster.0,
                            size: cluster.1.len(),
                            average_rgba: (
                                (colour_sum.0 / size as u128) as u8,
                                (colour_sum.1 / size as u128) as u8,
                                (colour_sum.2 / size as u128) as u8,
                                (colour_sum.3 / size as u128) as u8,
                            ),
                        });
                    }
                }
                if clusters_info.len() == 0 {
                    eprintln!(
                        "Warning: \"{}\" has no clusters above {}% of the image. Cannot compare.",
                        image.name(),
                        imgsim_options.coloursim_cluster_cutoff() * 100.0
                    );
                }
                clusters_info.sort_by(|a, b| b.size.cmp(&a.size));
                (image.name(), clusters_info)
            })
            .collect();

        // II. COMPARE
        // Generate the similarity of each image pairing based on the average colours of their most dominant clusters
        let mut new_matrix: HashMap<(String, String), Option<f32>> =
            HashMap::with_capacity(self.matrix.len());
        self.matrix
            .iter_mut()
            .for_each(|((image_a_name, image_b_name), _)| {
                // Get refs to image_a's and image_b's clusters' info
                let clusters_info_a = &clusters_info
                    .iter()
                    .find(|(name, _)| *name == image_a_name)
                    .unwrap()
                    .1;
                let clusters_info_b = &clusters_info
                    .iter()
                    .find(|(name, _)| *name == image_b_name)
                    .unwrap()
                    .1;
                if clusters_info_a.len() > 0 && clusters_info_b.len() > 0 {
                    // Calculate Similarity
                    // TODO: Try accuracy of taking average rather than summing them up
                    let mut new_similarity = 0.0;
                    let mut i = 0;
                    while i < clusters_info_a.len() && i < clusters_info_b.len() {
                        new_similarity += avg_colour_sim(
                            clusters_info_a[i].average_rgba.0,
                            clusters_info_a[i].average_rgba.1,
                            clusters_info_a[i].average_rgba.2,
                            clusters_info_a[i].average_rgba.3,
                            clusters_info_b[i].average_rgba.0,
                            clusters_info_b[i].average_rgba.1,
                            clusters_info_b[i].average_rgba.2,
                            clusters_info_b[i].average_rgba.3,
                        );
                        i += 1;
                    }
                    new_matrix.insert(
                        (String::from(image_a_name), String::from(image_b_name)),
                        Some(new_similarity),
                    );
                }
            });
        self.matrix = new_matrix;
    }

    fn cluster_size(&mut self, images: &Vec<ImgsimImage>, imgsim_options: &ImgsimOptions) {
        #[derive(Debug)]
        struct ClusterInfo {
            _cluster_id: usize,
            size: usize,
            proportional_start: (f32, f32),
            proportional_width: f32,
            proportional_height: f32,
        }
        // I. SETUP
        // Build a lookup table of each image's cluster's proportional dimensions and locations.
        let clusters_info: Vec<(&str, Vec<ClusterInfo>)> = images
            .par_iter()
            .map(|image| {
                let img_size =
                    image.rgba_image().width() as usize * image.rgba_image().height() as usize;
                let mut clusters_info: Vec<ClusterInfo> = Vec::with_capacity(img_size);
                for cluster in image.pixel_clusters() {
                    let size = cluster.1.len();
                    if size
                        > (img_size as f32 * imgsim_options.clustersize_cluster_cutoff()).round()
                            as usize
                    {
                        // Plot out a quadrilateral that contains the entire cluster
                        let left_x = cluster
                            .1
                            .iter()
                            .min_by(|(a, _), (b, _)| a.cmp(b))
                            .unwrap()
                            .0;
                        let top_y = cluster
                            .1
                            .iter()
                            .min_by(|(_, a), (_, b)| a.cmp(b))
                            .unwrap()
                            .1;
                        let right_x = cluster
                            .1
                            .iter()
                            .max_by(|(a, _), (b, _)| a.cmp(b))
                            .unwrap()
                            .0;
                        let bottom_y = cluster
                            .1
                            .iter()
                            .max_by(|(_, a), (_, b)| a.cmp(b))
                            .unwrap()
                            .1;

                        let proportional_start = (
                            left_x as f32 / image.rgba_image().width() as f32,
                            top_y as f32 / image.rgba_image().height() as f32,
                        );

                        let proportional_width =
                            (right_x - left_x) as f32 / image.rgba_image().width() as f32;

                        let proportional_height =
                            (bottom_y - top_y) as f32 / image.rgba_image().height() as f32;

                        clusters_info.push(ClusterInfo {
                            _cluster_id: *cluster.0,
                            size: cluster.1.len(),
                            proportional_start,
                            proportional_width,
                            proportional_height,
                        });
                    }
                }
                if clusters_info.len() == 0 {
                    eprintln!(
                        "Warning: \"{}\" has no clusters above {}% of the image. Cannot compare.",
                        image.name(),
                        imgsim_options.clustersize_cluster_cutoff() * 100.0
                    );
                }
                clusters_info.sort_by(|a, b| b.size.cmp(&a.size));
                (image.name(), clusters_info)
            })
            .collect();

        // II. COMPARE
        // Generate the similarity of each image pairing based on the size and location of their most dominant clusters
        let mut new_matrix: HashMap<(String, String), Option<f32>> =
            HashMap::with_capacity(self.matrix.len());
        self.matrix
            .iter_mut()
            .for_each(|((image_a_name, image_b_name), _)| {
                // Get refs to image_a's and image_b's clusters' info
                let clusters_info_a = &clusters_info
                    .iter()
                    .find(|(name, _)| *name == image_a_name)
                    .unwrap()
                    .1;
                let clusters_info_b = &clusters_info
                    .iter()
                    .find(|(name, _)| *name == image_b_name)
                    .unwrap()
                    .1;
                if clusters_info_a.len() > 0 && clusters_info_b.len() > 0 {
                    let mut new_similarity = 0.0;
                    let mut count = 0;
                    let mut i = 0;

                    while i < clusters_info_a.len() && i < clusters_info_b.len() {
                        new_similarity += proportional_similarity_coords(
                            &clusters_info_a[i].proportional_start,
                            &clusters_info_b[i].proportional_start,
                        );
                        new_similarity += proportional_similarity(
                            clusters_info_a[i].proportional_width,
                            clusters_info_b[i].proportional_width,
                        );
                        new_similarity += proportional_similarity(
                            clusters_info_a[i].proportional_height,
                            clusters_info_b[i].proportional_height,
                        );
                        i += 1;
                        count += 3;
                    }

                    new_matrix.insert(
                        (String::from(image_a_name), String::from(image_b_name)),
                        Some(new_similarity / count as f32),
                    );
                }
            });
        self.matrix = new_matrix;
    }
}

fn avg_colour_sim(r_a: u8, g_a: u8, b_a: u8, a_a: u8, r_b: u8, g_b: u8, b_b: u8, a_b: u8) -> f32 {
    let max_dist: f32 = 510.0;
    fn delta_sq(a: u8, b: u8) -> f32 {
        (a as f32 - b as f32).powf(2.0)
    }
    let delta_r_sq = delta_sq(r_a, r_b);
    let delta_g_sq = delta_sq(g_a, g_b);
    let delta_b_sq = delta_sq(b_a, b_b);
    let delta_a_sq = delta_sq(a_a, a_b);
    let ans =
        (((delta_r_sq + delta_g_sq + delta_b_sq + delta_a_sq).sqrt() / max_dist) - 0.5) * -2.0;
    return ans;
}

fn proportional_similarity_coords((x_a, y_a): &(f32, f32), (x_b, y_b): &(f32, f32)) -> f32 {
    ((2.0_f32).sqrt() / ((x_a - x_b).powf(2.0) + (y_a - y_b).powf(2.0)).sqrt()) - 0.5
}

fn proportional_similarity(val_a: f32, val_b: f32) -> f32 {
    // val_a & val_b all already range from 0 to 1; no need to normalise.
    ((val_a - val_b).powf(2.0) - 0.5) * -2.0
}

#[cfg(test)]
mod tests {
    use super::*;
    use pretty_assertions::assert_eq;

    #[test]
    fn acs_same() {
        assert_eq!(avg_colour_sim(123, 64, 42, 255, 123, 64, 42, 255), 1.0);
    }

    #[test]
    fn acs_opposite() {
        assert_eq!(avg_colour_sim(255, 255, 255, 255, 0, 0, 0, 0), -1.0)
    }

    #[test]
    fn acs_misc() {
        // ~0.792
        let ans: f32 = (((2826.0_f32).sqrt() / 510.0) - 0.5) * (-2.0);
        assert_eq!(avg_colour_sim(45, 129, 226, 255, 69, 174, 241, 255), ans);
    }
}
