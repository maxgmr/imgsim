#[warn(missing_docs)]
use image::DynamicImage;

pub struct ImgsimImage {
    name: String,
    dynamic_image: DynamicImage,
}
impl ImgsimImage {
    pub fn new(name: String, dynamic_image: DynamicImage) -> ImgsimImage {
        ImgsimImage {
            name,
            dynamic_image,
        }
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn dynamic_image(&self) -> &DynamicImage {
        &self.dynamic_image
    }
}
