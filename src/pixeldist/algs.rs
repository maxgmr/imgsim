#![warn(missing_docs)]

use image::Rgba;
use serde::Deserialize;
use strum_macros::EnumIter;

use crate::MatchEnumAsStr;

#[derive(Debug, Deserialize, EnumIter)]
/// Denotes the type of pixel distance algorithm being utilised.
pub enum PixeldistAlg {
    #[serde(alias = "euclidean")]
    /// Standard Euclidean distance between two pixels' sRGB values.
    Euclidean,
    #[serde(alias = "redmean")]
    /// Euclidean distance scaled to better approximate human colour perception.
    Redmean,
}
impl MatchEnumAsStr for PixeldistAlg {}

/// Get the colour distance between two pixels. The method by which this distance is calculated is determined by [PixeldistAlg] `pixeldist_alg`.
pub fn get_pixeldist(pixel_a: &Rgba<u8>, pixel_b: &Rgba<u8>, pixeldist_alg: &PixeldistAlg) -> f32 {
    match pixeldist_alg {
        PixeldistAlg::Euclidean => euclidean(pixel_a, pixel_b),
        PixeldistAlg::Redmean => redmean(pixel_a, pixel_b),
    }
}

pub fn alpha_only_dist(a_a: u8, a_b: u8) -> f32 {
    (a_a as i16 - a_b as i16).abs() as f32 / 255.0
}

pub fn euclidean(pixel_a: &Rgba<u8>, pixel_b: &Rgba<u8>) -> f32 {
    let max_diff_for_normalisation: f32 = 260100.0;
    let ar = pixel_a[0] as f32;
    let ag = pixel_a[1] as f32;
    let ab = pixel_a[2] as f32;
    let br = pixel_b[0] as f32;
    let bg = pixel_b[1] as f32;
    let bb = pixel_b[2] as f32;
    if pixel_a[3] != 0 && pixel_b[3] != 0 {
        (((ar - br).powf(2.0) + (ag - bg).powf(2.0) + (ab - bb).powf(2.0)) as f32).sqrt()
            / max_diff_for_normalisation.sqrt()
    } else {
        alpha_only_dist(pixel_a[3], pixel_b[3])
    }
}

pub fn redmean(pixel_a: &Rgba<u8>, pixel_b: &Rgba<u8>) -> f32 {
    let max_diff_for_normalisation: f32 = 650250.0;
    let ar = pixel_a[0] as f32;
    let ag = pixel_a[1] as f32;
    let ab = pixel_a[2] as f32;
    let br = pixel_b[0] as f32;
    let bg = pixel_b[1] as f32;
    let bb = pixel_b[2] as f32;

    if pixel_a[3] != 0 && pixel_b[3] != 0 {
        let rmean = (ar + br) as f32 / 2 as f32;

        let r_multiple = 2.0 + (rmean / 256.0);
        let g_multiple = 4.0;
        let b_multiple = 2.0 + ((255.0 - rmean) / 256.0);

        let delta_r_sq = (ar - br).powf(2.0);
        let delta_g_sq = (ag - bg).powf(2.0);
        let delta_b_sq = (ab - bb).powf(2.0);

        ((r_multiple * delta_r_sq as f32)
            + (g_multiple * delta_g_sq as f32)
            + (b_multiple * delta_b_sq as f32))
            .sqrt()
            / max_diff_for_normalisation.sqrt()
    } else {
        alpha_only_dist(pixel_a[3], pixel_b[3])
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use pretty_assertions::assert_eq;

    #[test]
    fn aod_max() {
        assert_eq!(alpha_only_dist(255, 0), 1.0);
    }

    #[test]
    fn aod_min() {
        assert_eq!(alpha_only_dist(255, 255), 0.0)
    }

    #[test]
    fn aod_misc() {
        assert_eq!(81.0 / 255.0, alpha_only_dist(42, 123))
    }
}
