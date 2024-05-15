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
    };
    return output_matrix;
}

/// A matrix of [ImgsimImage] pairings and their similarities.
pub struct ImageSimilarityMatrix {
    matrix: HashMap<(String, String), Option<f32>>,
}
impl ImageSimilarityMatrix {
    /// Builds an empty [ImageSimilarityMatrix] out of a provided list of images.
    fn from(images: &Vec<ImgsimImage>) -> ImageSimilarityMatrix {
        let combinations = images.len().pow(2) - images.len();
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
        struct ClusterInfo {
            cluster_id: usize,
            size: usize,
            img_size: usize,
            average_rgba: (u8, u8, u8, u8),
        }
        // TODO general refactor/improve
        // I. SETUP
        // For each image, get a list of its clusters and their average RGBA values, sorted by size.
        let mut clusters_info: Vec<(&str, Vec<ClusterInfo>)> = images
            .par_iter()
            .map(|image| {
                let img_size =
                    image.rgba_image().width() as usize * image.rgba_image().height() as usize;
                let mut clusters_info: Vec<ClusterInfo> = Vec::with_capacity(img_size);
                for cluster in image.pixel_clusters() {
                    let size = cluster.1.len();
                    let colour_sum = cluster.1.iter().fold((0, 0, 0, 0), |accumulator, coords| {
                        let image::Rgba(data) = *image.rgba_image().get_pixel(coords.0, coords.1);
                        (
                            accumulator.0 + data[0] as u128,
                            accumulator.1 + data[1] as u128,
                            accumulator.2 + data[2] as u128,
                            accumulator.3 + data[3] as u128,
                        )
                    });
                    if size > (img_size as f32 * imgsim_options.cluster_cutoff()).round() as usize {
                        clusters_info.push(ClusterInfo {
                            cluster_id: *cluster.0,
                            size: cluster.1.len(),
                            img_size,
                            average_rgba: (
                                (colour_sum.0 / size as u128) as u8,
                                (colour_sum.1 / size as u128) as u8,
                                (colour_sum.2 / size as u128) as u8,
                                (colour_sum.3 / size as u128) as u8,
                            ),
                        });
                    }
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
                    .find(|(name, _)| *name == image_a_name)
                    .unwrap()
                    .1;
                if clusters_info_a.len() == 0 {
                    eprintln!(
                        "Warning: \"{}\" has no clusters above {}% of the image. Cannot compare.",
                        image_a_name,
                        imgsim_options.cluster_cutoff() * 100.0
                    );
                    new_matrix.insert(
                        (String::from(image_a_name), String::from(image_b_name)),
                        None,
                    );
                } else if clusters_info_b.len() == 0 {
                    eprintln!(
                        "Warning: \"{}\" has no clusters above {}% of the image. Cannot compare.",
                        image_b_name,
                        imgsim_options.cluster_cutoff() * 100.0
                    );
                    new_matrix.insert(
                        (String::from(image_a_name), String::from(image_b_name)),
                        None,
                    );
                } else {
                    // Calculate Similarity
                    let mut new_similarity = 0.0;
                    let mut i = 0;
                    while i < clusters_info_a.len() && i < clusters_info_b.len() {}
                    new_matrix.insert(
                        (String::from(image_a_name), String::from(image_b_name)),
                        Some(new_similarity),
                    );
                }
            });
        self.matrix = new_matrix;
    }
}
