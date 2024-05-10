use std::{fmt, path::PathBuf};

#[derive(Debug)]

/// All the possible persistence-related errors.
pub enum PersistenceError {
    /// Sent when unable to read a file, including the offending file path.
    ReadFileError(PathBuf),
    /// Sent when unable to write a file, including the offending file path.
    WriteFileError(PathBuf),
    /// Sent when unable to deserialise a file. Includes toml::de::Error.message().
    DeserializeError(String),
}
impl fmt::Display for PersistenceError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::ReadFileError(path_buf) => {
                let path_buf_str = path_buf.to_str().unwrap_or("");
                write!(
                    f,
                    "ReadFileError: Failed to read file path {}",
                    path_buf_str
                )
            }
            Self::WriteFileError(path_buf) => {
                let path_buf_str = path_buf.to_str().unwrap_or("");
                write!(
                    f,
                    "WriteFileError: Failed to write file path {}",
                    path_buf_str
                )
            }
            Self::DeserializeError(string) => {
                write!(f, "DeserializeError: {}", string)
            }
        }
    }
}
