use image::error::ImageError;

use std::fs;
use std::path::{Path, PathBuf};

use super::super::data::imgsim_image::ImgsimImage;
use super::errors::PersistenceError;

/// Loads vector of images from given directory
pub fn load_images(input_dir_path: &Path) -> Result<Vec<ImgsimImage>, PersistenceError> {
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
        .collect::<Vec<ImgsimImage>>();
    if images.len() == 0 {
        Err(PersistenceError::EmptyInputDirError(Some(PathBuf::from(
            input_dir_path,
        ))))
    } else {
        return Ok(images);
    }
}

fn load_image(image_path: &Path) -> Option<ImgsimImage> {
    match (image_path.file_name(), image::open(image_path)) {
        (Some(file_name), Ok(image)) => {
            if let Some(name) = file_name.to_str() {
                Some(ImgsimImage::new(String::from(name), image))
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
