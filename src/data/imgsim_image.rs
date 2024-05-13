#[warn(missing_docs)]
use image::{ImageError, RgbaImage};
use rayon::prelude::*;
use std::{collections::BTreeMap, path::PathBuf, time::Instant};

use crate::{get_clusters, get_pixeldist, ImgsimOptions};

/// An [image::RgbaImage] with metadata, similarity factors, and clusters.
pub struct ImgsimImage {
    name: String,
    path: PathBuf,
    rgba_image: RgbaImage,
    pixeldist_factors: Vec<PixeldistFactor>,
    cluster_lookup: BTreeMap<(u32, u32), usize>,
    pixel_clusters: BTreeMap<usize, Vec<(u32, u32)>>,
}
impl ImgsimImage {
    /// Creates a new [ImgsimImage] by loading the image at the given file path.
    ///
    /// Returns [Option::Some]\([ImgsimImage]\) if an image at that path exists and that image loads successfully.
    ///
    /// Returns [Option::None](std::option::Option) and prints a warning message if an image exists, but there was a problem reading the image.
    ///
    /// Returns [Option::None](std::option::Option) without printing a message if the file path does not link to an image.
    pub fn new(image_path: PathBuf) -> Option<ImgsimImage> {
        match (image_path.file_name(), image::open(&image_path)) {
            (Some(file_name), Ok(image)) => {
                if let Some(name) = file_name.to_str() {
                    let rgba_image = image.to_rgba8();
                    let image_width = rgba_image.width();
                    let image_height = rgba_image.height();
                    Some(ImgsimImage {
                        name: String::from(name),
                        path: image_path,
                        rgba_image,
                        pixeldist_factors: Vec::with_capacity(
                            4 * image_height as usize * image_width as usize,
                        ),
                        cluster_lookup: BTreeMap::new(),
                        pixel_clusters: BTreeMap::new(),
                    })
                } else {
                    eprintln!(
                        "Warning: Could not parse file name at {}",
                        image_path.to_str().unwrap_or("unknown directory")
                    );
                    None
                }
            }
            (_, Err(ImageError::Unsupported(_))) => None,
            (Some(file_name), Err(image_error)) => {
                eprintln!(
                    "Warning @ {}: {}",
                    file_name.to_str().unwrap_or("unknown file"),
                    image_error.to_string()
                );
                None
            }
            (_, Err(image_error)) => {
                eprintln!("Warning: {}", image_error.to_string());
                None
            }
            (_, Ok(_)) => {
                eprintln!(
                    "Warning: Could not parse file name at {}, so cannot use valid image",
                    image_path.to_str().unwrap_or("unknown directory")
                );
                None
            }
        }
    }

    /// Builds pixel distance factors between all the pixels in the image using the chosen algorithm.
    pub fn build_factors(&mut self, imgsim_options: &ImgsimOptions) {
        let start_time = Instant::now();
        self.pixeldist_factors
            .par_extend(
                self.rgba_image
                    .par_enumerate_pixels()
                    .flat_map(|(x, y, pixel)| {
                        let mut temp_vec: Vec<PixeldistFactor> = Vec::new();
                        // Right neighbour
                        if let Some(right_neighbour) = self.rgba_image.get_pixel_checked(x + 1, y) {
                            temp_vec.push(PixeldistFactor::new(
                                (x, y),
                                (x + 1, y),
                                get_pixeldist(&pixel, &right_neighbour, &imgsim_options),
                            ))
                        }
                        // Bottom-right neighbour
                        if let Some(b_right_neighbour) =
                            self.rgba_image.get_pixel_checked(x + 1, y + 1)
                        {
                            temp_vec.push(PixeldistFactor::new(
                                (x, y),
                                (x + 1, y + 1),
                                get_pixeldist(&pixel, &b_right_neighbour, &imgsim_options),
                            ))
                        }
                        // Bottom neighbour
                        if let Some(bottom_neighbour) = self.rgba_image.get_pixel_checked(x, y + 1)
                        {
                            temp_vec.push(PixeldistFactor::new(
                                (x, y),
                                (x, y + 1),
                                get_pixeldist(&pixel, &bottom_neighbour, &imgsim_options),
                            ))
                        }
                        // Bottom-left neighbour
                        if x > 0 {
                            if let Some(b_left_neighbour) =
                                self.rgba_image.get_pixel_checked(x - 1, y + 1)
                            {
                                temp_vec.push(PixeldistFactor::new(
                                    (x, y),
                                    (x - 1, y + 1),
                                    get_pixeldist(&pixel, &b_left_neighbour, &imgsim_options),
                                ))
                            }
                        }
                        temp_vec
                    }),
            );
        let elapsed_time = start_time.elapsed();
        if imgsim_options.debug() {
            println!(
                "\"{}\": Built {} factors in {:.2?}.",
                self.name,
                self.pixeldist_factors.len(),
                elapsed_time
            );
        } else if imgsim_options.verbose() {
            println!("\"{}\" factors done in {:.2?}.", self.name, elapsed_time);
        }
    }

    pub fn build_clusters(&mut self, imgsim_options: &ImgsimOptions) {
        let start_time = Instant::now();
        get_clusters(self, imgsim_options);
        let elapsed_time = start_time.elapsed();
        if imgsim_options.debug() || imgsim_options.verbose() {
            println!(
                "\"{}\": Built {} clusters in {:.2?}.",
                self.name,
                self.pixel_clusters.len(),
                elapsed_time
            );
        }
    }

    /// Returns the name of the image.
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Returns a reference to the path of the image as [std::path::PathBuf].
    pub fn path(&self) -> &PathBuf {
        &self.path
    }

    /// Returns a reference to the image as an [image::RgbaImage].
    pub fn rgba_image(&self) -> &RgbaImage {
        &self.rgba_image
    }

    /// Returns a reference to the image's pixel distance factors.
    pub fn pixeldist_factors(&self) -> &Vec<PixeldistFactor> {
        &self.pixeldist_factors
    }

    /// Returns a reference to the image's cluster lookup table.
    pub fn cluster_lookup(&self) -> &BTreeMap<(u32, u32), usize> {
        &self.cluster_lookup
    }

    /// Returns a mutable reference to the image's cluster lookup table.
    pub fn cluster_lookup_mut(&mut self) -> &mut BTreeMap<(u32, u32), usize> {
        &mut self.cluster_lookup
    }

    /// Returns a reference to the image's pixel clusters.
    pub fn pixel_clusters(&self) -> &BTreeMap<usize, Vec<(u32, u32)>> {
        &self.pixel_clusters
    }

    /// Returns a mutable reference to the image's pixel clusters.
    pub fn pixel_clusters_mut(&mut self) -> &mut BTreeMap<usize, Vec<(u32, u32)>> {
        &mut self.pixel_clusters
    }
}

pub struct PixeldistFactor {
    a_coords: (u32, u32),
    b_coords: (u32, u32),
    distance: f32,
}
impl PixeldistFactor {
    pub fn new(a_coords: (u32, u32), b_coords: (u32, u32), distance: f32) -> PixeldistFactor {
        PixeldistFactor {
            a_coords,
            b_coords,
            distance,
        }
    }

    pub fn a_coords(&self) -> &(u32, u32) {
        &self.a_coords
    }

    pub fn b_coords(&self) -> &(u32, u32) {
        &self.b_coords
    }

    pub fn distance(&self) -> f32 {
        self.distance
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use image::Rgba;
    use pretty_assertions::assert_eq;

    fn gen_test_image() -> RgbaImage {
        let mut test_image = RgbaImage::new(2, 2);
        *test_image.get_pixel_mut(0, 0) = Rgba([255, 0, 0, 255]); // Red
        *test_image.get_pixel_mut(1, 0) = Rgba([0, 0, 255, 255]); // Blue
        *test_image.get_pixel_mut(0, 1) = Rgba([255, 255, 0, 255]); // Yellow
        *test_image.get_pixel_mut(1, 1) = Rgba([0, 0, 255, 0]); // Green
        test_image
    }
}
