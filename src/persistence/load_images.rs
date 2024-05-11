use image::error::ImageError;
use image::DynamicImage;

use std::fs;
use std::path::{Path, PathBuf};

use super::errors::PersistenceError;

/// Loads vector of images from given directory
pub fn load_images(input_dir_path: &Path) -> Result<Vec<DynamicImage>, PersistenceError> {
    let images = fs::read_dir(input_dir_path)
        .unwrap()
        .filter_map(|entry| {
            entry.ok().and_then(|ok_entry| {
                let path = ok_entry.path();
                if path.is_file() {
                    load_image(&path)
                } else {
                    None
                }
            })
        })
        .collect::<Vec<DynamicImage>>();
    if images.len() == 0 {
        Err(PersistenceError::EmptyInputDirError(Some(PathBuf::from(
            input_dir_path,
        ))))
    } else {
        return Ok(images);
    }
}

fn load_image(image_path: &Path) -> Option<DynamicImage> {
    match image::open(image_path) {
        Ok(image) => Some(image),
        Err(ImageError::Unsupported(_)) => None,
        Err(image_error) => {
            eprintln!("{}", image_error.to_string());
            None
        }
    }
}
