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
                    ImgsimImage::new(&path)
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
