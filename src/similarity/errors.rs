#![warn(missing_docs)]

use std::fmt;

#[derive(Debug)]
/// Error produced when the given pair cannot be found in an ImageSimilarityMatrix.
pub struct ImageSimilarityMatrixNoMatchError(pub &'static str);
impl fmt::Display for ImageSimilarityMatrixNoMatchError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "ImageSimilarityMatrixNoMatchError: {}", &self.0)
    }
}
