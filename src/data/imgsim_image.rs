#[warn(missing_docs)]
use image::{ImageError, RgbaImage};
use rayon::prelude::*;
use std::path::{Path, PathBuf};

/// An [image::RgbaImage] with metadata, similarity factors, and clusters.
pub struct ImgsimImage {
    name: String,
    path: PathBuf,
    rgba_image: RgbaImage,
}
impl ImgsimImage {
    /// Creates a new [ImgsimImage] by loading the image at the given file path.
    ///
    /// Returns [Option::Some]\([ImgsimImage]\) if an image at that path exists and that image loads successfully.
    ///
    /// Returns [Option::None](std::option::Option) and prints a warning message if an image exists, but there was a problem reading the image.
    ///
    /// Returns [Option::None](std::option::Option) without printing a message if the file path does not link to an image.
    pub fn new(image_path: &Path) -> Option<ImgsimImage> {
        match (image_path.file_name(), image::open(image_path)) {
            (Some(file_name), Ok(image)) => {
                if let Some(name) = file_name.to_str() {
                    Some(ImgsimImage {
                        name: String::from(name),
                        path: PathBuf::from(image_path),
                        rgba_image: image.to_rgba8(),
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

    /// TODO Remove: For benchmarking only
    pub fn darken_par(&mut self) {
        let start_time = std::time::Instant::now();
        self.rgba_image
            .par_enumerate_pixels_mut()
            .for_each(|(x, y, pixel)| {
                let r = (0.3 * pixel[0] as f32) as u8;
                let g = (0.3 * pixel[1] as f32) as u8;
                let b = (0.3 * pixel[2] as f32) as u8;
                *pixel = image::Rgba([r, b, g, pixel[3]]);
            });
        let elapsed = start_time.elapsed();
        println!(
            "darken_par time taken ({}x{}): {:.2?}",
            self.rgba_image.height(),
            self.rgba_image.width(),
            elapsed
        );
    }

    /// TODO Remove: For benchmarking only
    pub fn darken(&mut self) {
        let start_time = std::time::Instant::now();
        self.rgba_image
            .enumerate_pixels_mut()
            .for_each(|(x, y, pixel)| {
                let r = (0.3 * pixel[0] as f32) as u8;
                let g = (0.3 * pixel[1] as f32) as u8;
                let b = (0.3 * pixel[2] as f32) as u8;
                *pixel = image::Rgba([r, b, g, pixel[3]]);
            });
        let elapsed = start_time.elapsed();
        println!(
            "darken time taken ({}x{}): {:.2?}",
            self.rgba_image.height(),
            self.rgba_image.width(),
            elapsed
        );
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
