#[warn(missing_docs)]
use image::{ImageError, Rgba, RgbaImage};
use rayon::prelude::*;
use std::path::PathBuf;
use std::time::Instant;

use crate::{get_pixeldist, ImgsimOptions};

/// An [image::RgbaImage] with metadata, similarity factors, and clusters.
pub struct ImgsimImage<'a> {
    name: String,
    path: PathBuf,
    rgba_image: RgbaImage,
    pixeldist_factors: Vec<PixeldistFactor<'a>>,
}
impl<'a> ImgsimImage<'a> {
    /// Creates a new [ImgsimImage] by loading the image at the given file path.
    ///
    /// Returns [Option::Some]\([ImgsimImage]\) if an image at that path exists and that image loads successfully.
    ///
    /// Returns [Option::None](std::option::Option) and prints a warning message if an image exists, but there was a problem reading the image.
    ///
    /// Returns [Option::None](std::option::Option) without printing a message if the file path does not link to an image.
    pub fn new(image_path: PathBuf) -> Option<ImgsimImage<'a>> {
        match (image_path.file_name(), image::open(&image_path)) {
            (Some(file_name), Ok(image)) => {
                if let Some(name) = file_name.to_str() {
                    Some(ImgsimImage {
                        name: String::from(name),
                        path: image_path,
                        rgba_image: image.to_rgba8(),
                        pixeldist_factors: Vec::new(),
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
    pub fn build_pixeldist_factors(&'a mut self, imgsim_options: &ImgsimOptions) {
        let start_time = Instant::now();
        self.pixeldist_factors
            .par_extend(
                self.rgba_image
                    .par_enumerate_pixels()
                    .flat_map(|(x, y, pixel)| {
                        let mut temp_vec: Vec<PixeldistFactor<'a>> = Vec::new();
                        // Right neighbour
                        if let Some(right_neighbour) = self.rgba_image.get_pixel_checked(x + 1, y) {
                            temp_vec.push(PixeldistFactor::new(
                                &pixel,
                                &right_neighbour,
                                get_pixeldist(
                                    &pixel,
                                    &right_neighbour,
                                    imgsim_options.pixeldist_alg(),
                                ),
                            ))
                        }
                        // Bottom-right neighbour
                        if let Some(b_right_neighbour) =
                            self.rgba_image.get_pixel_checked(x + 1, y + 1)
                        {
                            temp_vec.push(PixeldistFactor::new(
                                &pixel,
                                &b_right_neighbour,
                                get_pixeldist(
                                    &pixel,
                                    &b_right_neighbour,
                                    imgsim_options.pixeldist_alg(),
                                ),
                            ))
                        }
                        // Bottom neighbour
                        if let Some(bottom_neighbour) = self.rgba_image.get_pixel_checked(x, y + 1)
                        {
                            temp_vec.push(PixeldistFactor::new(
                                &pixel,
                                &bottom_neighbour,
                                get_pixeldist(
                                    &pixel,
                                    &bottom_neighbour,
                                    imgsim_options.pixeldist_alg(),
                                ),
                            ))
                        }
                        // Bottom-left neighbour
                        if x > 0 {
                            if let Some(b_left_neighbour) =
                                self.rgba_image.get_pixel_checked(x - 1, y + 1)
                            {
                                temp_vec.push(PixeldistFactor::new(
                                    &pixel,
                                    &b_left_neighbour,
                                    get_pixeldist(
                                        &pixel,
                                        &b_left_neighbour,
                                        imgsim_options.pixeldist_alg(),
                                    ),
                                ))
                            }
                        }
                        temp_vec
                    }),
            );
        let elapsed_time = start_time.elapsed();
        if imgsim_options.debug() {
            println!(
                "Built {} factors of {} in {:.2?}.",
                self.pixeldist_factors.len(),
                self.name,
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
}

struct PixeldistFactor<'a> {
    pixel_a: &'a Rgba<u8>,
    pixel_b: &'a Rgba<u8>,
    distance: f32,
}
impl<'a> PixeldistFactor<'a> {
    fn new(pixel_a: &'a Rgba<u8>, pixel_b: &'a Rgba<u8>, distance: f32) -> PixeldistFactor<'a> {
        PixeldistFactor {
            pixel_a,
            pixel_b,
            distance,
        }
    }
}
