#![warn(missing_docs)]

use std::{fmt, path::PathBuf};

#[derive(Debug)]

/// All the possible persistence-related errors.
pub enum PersistenceError {
    /// Sent when unable to read a file, including the offending file path.
    ReadFileError(Option<PathBuf>),
    /// Sent when unable to write a file, including the offending file path.
    WriteFileError(Option<PathBuf>),
    /// Sent when unable to deserialise a file. Includes toml::de::Error.message().
    DeserializeError(String),
    /// Sent when input directory is empty.
    EmptyInputDirError(Option<PathBuf>),
    /// Sent when a file is given instead of a directory.
    NotDirectoryError(Option<PathBuf>),
    /// Sent when an unknown option is given.
    UnknownOptionError(String),
}

fn path_buf_as_str<'a>(path_buf: &'a Option<PathBuf>) -> &'a str {
    return match path_buf {
        Some(buf) => buf.to_str().unwrap_or(""),
        _ => "",
    };
}

impl fmt::Display for PersistenceError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::ReadFileError(path_buf) => {
                write!(
                    f,
                    "ReadFileError: Failed to read file path {}",
                    path_buf_as_str(path_buf)
                )
            }
            Self::WriteFileError(path_buf) => {
                write!(
                    f,
                    "WriteFileError: Failed to write file path {}",
                    path_buf_as_str(path_buf)
                )
            }
            Self::DeserializeError(string) => {
                write!(f, "DeserializeError: {}", string)
            }
            Self::EmptyInputDirError(path_buf) => {
                write!(
                    f,
                    "EmptyInputDirError: No images in {} to compare.",
                    path_buf_as_str(path_buf)
                )
            }
            Self::NotDirectoryError(path_buf) => {
                write!(
                    f,
                    "NotDirectoryError: Path {} is not a directory.",
                    path_buf_as_str(path_buf)
                )
            }
            Self::UnknownOptionError(string) => {
                write!(
                    f,
                    "UnknownOptionError: Given option \"{}\" does not match any known values.",
                    string
                )
            }
        }
    }
}
