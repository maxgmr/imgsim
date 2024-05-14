#![warn(missing_docs)]

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
        SimilarityAlg::ColourSim => output_matrix.colour_sim(&imgsim_options),
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

    fn get(
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

    fn colour_sim(&mut self, imgsim_options: &ImgsimOptions) {}
}
