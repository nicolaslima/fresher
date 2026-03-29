//! Binary shimming: creates lightweight launchers for installed tools.
//!
//! - **Unix**: relative symlinks in `bin/`
//! - **Windows**: `.cmd` wrapper scripts

use anyhow::{Context, Result};
use std::path::Path;

/// Creates and removes tool shims.
pub struct Shimmer;

impl Shimmer {
    /// Create a shim named `shim_name` that points to `target_path`.
    ///
    /// On Unix, this creates a symlink: `bin/<shim_name> → <target_path>`
    /// On Windows, this creates a `.cmd` wrapper: `bin\<shim_name>.cmd`
    pub fn create(bin_dir: &Path, shim_name: &str, target_path: &Path) -> Result<()> {
        std::fs::create_dir_all(bin_dir)
            .with_context(|| format!("Failed to create bin directory {}", bin_dir.display()))?;

        #[cfg(unix)]
        {
            Self::create_unix_shim(bin_dir, shim_name, target_path)
        }

        #[cfg(windows)]
        {
            Self::create_windows_shim(bin_dir, shim_name, target_path)
        }
    }

    /// Remove a shim by name.
    pub fn remove(bin_dir: &Path, shim_name: &str) -> Result<()> {
        #[cfg(unix)]
        {
            let shim_path = bin_dir.join(shim_name);
            if shim_path.exists() || shim_path.symlink_metadata().is_ok() {
                std::fs::remove_file(&shim_path)
                    .with_context(|| format!("Failed to remove shim {}", shim_path.display()))?;
            }
        }

        #[cfg(windows)]
        {
            let shim_path = bin_dir.join(format!("{shim_name}.cmd"));
            if shim_path.exists() {
                std::fs::remove_file(&shim_path)
                    .with_context(|| format!("Failed to remove shim {}", shim_path.display()))?;
            }
        }

        Ok(())
    }

    #[cfg(unix)]
    fn create_unix_shim(bin_dir: &Path, shim_name: &str, target_path: &Path) -> Result<()> {
        let shim_path = bin_dir.join(shim_name);

        // Remove existing shim if present
        if shim_path.exists() || shim_path.symlink_metadata().is_ok() {
            std::fs::remove_file(&shim_path).with_context(|| {
                format!("Failed to remove existing shim {}", shim_path.display())
            })?;
        }

        // Create symlink (use absolute path for reliability)
        std::os::unix::fs::symlink(target_path, &shim_path).with_context(|| {
            format!(
                "Failed to create symlink {} → {}",
                shim_path.display(),
                target_path.display()
            )
        })?;

        Ok(())
    }

    #[cfg(windows)]
    fn create_windows_shim(bin_dir: &Path, shim_name: &str, target_path: &Path) -> Result<()> {
        let shim_path = bin_dir.join(format!("{shim_name}.cmd"));
        let target_str = target_path.to_string_lossy();

        let content = format!("@ECHO off\r\nSETLOCAL\r\n\"{}\" %*\r\n", target_str);

        std::fs::write(&shim_path, content)
            .with_context(|| format!("Failed to write shim {}", shim_path.display()))?;

        Ok(())
    }

    /// Set a file as executable (chmod +x). No-op on Windows.
    pub fn set_executable(path: &Path) -> Result<()> {
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let metadata = std::fs::metadata(path)
                .with_context(|| format!("Failed to read metadata for {}", path.display()))?;
            let mut perms = metadata.permissions();
            let mode = perms.mode() | 0o111; // Add execute for user/group/other
            perms.set_mode(mode);
            std::fs::set_permissions(path, perms).with_context(|| {
                format!("Failed to set executable permission on {}", path.display())
            })?;
        }
        #[cfg(windows)]
        {
            let _ = path; // no-op
        }
        Ok(())
    }
}
