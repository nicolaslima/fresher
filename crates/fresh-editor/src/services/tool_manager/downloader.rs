//! File downloader with progress reporting and SHA-256 verification.

use anyhow::{bail, Context, Result};
use sha2::{Digest, Sha256};
use std::io::Write;
use std::path::Path;

/// Callback for download progress updates.
pub type ProgressCallback = Box<dyn Fn(u64, Option<u64>) + Send>;

/// Downloads files with progress reporting and optional checksum verification.
pub struct Downloader;

impl Downloader {
    /// Download a file from `url` to `dest`, optionally verifying SHA-256.
    ///
    /// The `on_progress` callback receives `(bytes_downloaded, total_bytes)`.
    /// `total_bytes` is `None` if the server doesn't send `Content-Length`.
    pub fn download(
        url: &str,
        dest: &Path,
        expected_sha256: Option<&str>,
        on_progress: Option<ProgressCallback>,
    ) -> Result<()> {
        // Ensure parent directory exists
        if let Some(parent) = dest.parent() {
            std::fs::create_dir_all(parent)
                .with_context(|| format!("Failed to create directory {}", parent.display()))?;
        }

        let response = ureq::get(url)
            .call()
            .with_context(|| format!("Failed to download {url}"))?;

        let total_bytes = response
            .headers()
            .get("content-length")
            .and_then(|v| v.to_str().ok())
            .and_then(|v| v.parse::<u64>().ok());

        let mut reader = response.into_body().into_reader();
        let mut file = std::fs::File::create(dest)
            .with_context(|| format!("Failed to create file {}", dest.display()))?;

        let mut hasher = Sha256::new();
        let mut bytes_downloaded: u64 = 0;
        let mut buf = [0u8; 8192];

        loop {
            let n = std::io::Read::read(&mut reader, &mut buf)
                .with_context(|| format!("Failed to read from {url}"))?;
            if n == 0 {
                break;
            }
            file.write_all(&buf[..n])
                .with_context(|| format!("Failed to write to {}", dest.display()))?;
            hasher.update(&buf[..n]);
            bytes_downloaded += n as u64;

            if let Some(ref cb) = on_progress {
                cb(bytes_downloaded, total_bytes);
            }
        }

        file.flush()?;
        drop(file);

        // Verify checksum
        if let Some(expected) = expected_sha256 {
            let actual = format!("{:x}", hasher.finalize());
            if actual != expected {
                // Clean up the corrupt file
                let _ = std::fs::remove_file(dest);
                bail!(
                    "SHA-256 mismatch for {}: expected {}, got {}",
                    dest.display(),
                    expected,
                    actual
                );
            }
        }

        Ok(())
    }
}
