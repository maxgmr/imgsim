//! General utilities used by `imgsim`.
use std::{env, fs};

use camino::Utf8PathBuf;
use clap::crate_authors;
use color_eyre::eyre::{self, eyre};
use directories::ProjectDirs;

const VERSION_MESSAGE: &str = concat!(
    env!("CARGO_PKG_VERSION"),
    " -",
    env!("VERGEN_GIT_DESCRIBE"),
    " (",
    env!("VERGEN_BUILD_DATE"),
    ")"
);

/// Get the version, author info, and directories of `imgsim`.
pub fn info() -> String {
    let author = crate_authors!();
    format!(
        "\
{VERSION_MESSAGE}

Author: {author}

Config Directory: {}",
        config_dir().unwrap(),
    )
}

/// Set up `imgsim` on first-time startup.
pub fn setup() -> eyre::Result<()> {
    // Create the directory where `imgsim` configuration data is stored.
    if fs::metadata(config_dir()?).is_err() {
        fs::create_dir_all(config_dir()?)?;
    }

    Ok(())
}

/// Get the directory where `imgsim` configuration data is stored.
pub fn config_dir() -> eyre::Result<Utf8PathBuf> {
    if let Some(utf8_path_buf) = get_env_var_path("DATA") {
        // Prioritise user-set path.
        Ok(utf8_path_buf)
    } else if let Some(proj_dirs) = project_directory() {
        // Second priority: XDG-standardised local dir.
        match Utf8PathBuf::from_path_buf(proj_dirs.config_local_dir().to_path_buf()) {
            Ok(utf8_path_buf) => Ok(utf8_path_buf),
            Err(path_buf) => Err(eyre!(
                "Path to data directory {:?} contains non-UTF-8 content.",
                path_buf
            )),
        }
    } else {
        // Last priority: .config folder relative to current working directory
        Ok(Utf8PathBuf::from(".").join(".config"))
    }
}

// Helper function to get an environment variable path with the given suffix.
fn get_env_var_path(suffix: &str) -> Option<Utf8PathBuf> {
    env::var(format!("{}_{}", crate_name_constant_case(), suffix))
        .ok()
        .map(Utf8PathBuf::from)
}

/// Get the crate name in CONSTANT_CASE.
pub fn crate_name_constant_case() -> String {
    env!("CARGO_CRATE_NAME").to_uppercase().to_string()
}

/// Get the directory of this project.
pub fn project_directory() -> Option<ProjectDirs> {
    ProjectDirs::from("ca", "maxgmr", env!("CARGO_PKG_NAME"))
}
