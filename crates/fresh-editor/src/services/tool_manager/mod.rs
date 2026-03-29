//! Tool Manager: discovers, installs, updates, and manages external development tools.
//!
//! This module implements the Rust core of the hybrid tool management system.
//! TypeScript plugins provide "recipes" (metadata + URL construction), while
//! this Rust code handles the platform-specific heavy lifting:
//!
//! - Secure HTTPS downloading with progress reporting
//! - Checksum verification (SHA-256)
//! - Cross-platform archive extraction (zip, tar.gz, tar.xz, gz)
//! - Binary shimming (symlinks on Unix, .cmd wrappers on Windows)
//! - Tool inventory persistence

mod downloader;
mod extractor;
mod inventory;
mod shimmer;

pub use downloader::Downloader;
pub use extractor::Extractor;
pub use inventory::{ToolEntry, ToolInventory};
pub use shimmer::Shimmer;

use std::path::PathBuf;

/// Returns the platform-specific root directory for tool installations.
///
/// | Platform    | Path                              |
/// |-------------|-----------------------------------|
/// | Linux/macOS | `~/.local/share/fresh/tools/`     |
/// | Windows     | `%LOCALAPPDATA%\fresh\tools\`     |
pub fn tools_root() -> PathBuf {
    #[cfg(target_os = "windows")]
    {
        let base = std::env::var("LOCALAPPDATA")
            .map(PathBuf::from)
            .unwrap_or_else(|_| dirs::data_local_dir().unwrap_or_else(|| PathBuf::from(".")));
        base.join("fresh").join("tools")
    }
    #[cfg(not(target_os = "windows"))]
    {
        dirs::data_local_dir()
            .unwrap_or_else(|| {
                dirs::home_dir()
                    .unwrap_or_else(|| PathBuf::from("."))
                    .join(".local")
                    .join("share")
            })
            .join("fresh")
            .join("tools")
    }
}

/// Returns the path to the `bin/` directory containing tool shims.
pub fn tools_bin_dir() -> PathBuf {
    tools_root().join("bin")
}
