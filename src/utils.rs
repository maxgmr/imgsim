//! General utilities used by `imgsim`.
use std::env;

const VERSION_MESSAGE: &str = concat!(
    env!("CARGO_PKG_VERSION"),
    " -",
    env!("VERGEN_GIT_DESCRIBE"),
    " (",
    env!("VERGEN_BUILD_DATE"),
    ")"
);
