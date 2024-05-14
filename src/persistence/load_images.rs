use std::fs;
use std::path::PathBuf;

use crate::PersistenceError;
use crate::{ImgsimImage, ImgsimOptions};

/// Loads vector of images from given directory
pub fn load_images(imgsim_options: &ImgsimOptions) -> Result<Vec<ImgsimImage>, PersistenceError> {
    let images = fs::read_dir(imgsim_options.input_dir())
        .unwrap()
        .filter_map(|entry| {
            entry.ok().and_then(|ok_entry| {
                let path = ok_entry.path();
                if path.is_file() {
                    ImgsimImage::new(path, imgsim_options)
                } else {
                    None
                }
            })
        })
        .collect::<Vec<ImgsimImage>>();
    if images.len() == 0 {
        Err(PersistenceError::EmptyInputDirError(Some(PathBuf::from(
            imgsim_options.input_dir(),
        ))))
    } else {
        return Ok(images);
    }
}
