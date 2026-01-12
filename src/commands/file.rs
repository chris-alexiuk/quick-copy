use crate::config::Config;
use crate::output::TransferResult;
use crate::resolve;
use crate::transfer;
use std::path::Path;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum FileError {
    #[error("file not found: {0}")]
    NotFound(String),
    #[error("path is a directory, not a file: {0}")]
    IsDirectory(String),
    #[error("{0}")]
    ResolveError(#[from] resolve::ResolveError),
    #[error("{0}")]
    TransferError(#[from] transfer::TransferError),
    #[error("remote file exists: {0} (use --overwrite to replace)")]
    RemoteExists(String),
}

pub fn run(
    path: &Path,
    dest: &str,
    overwrite: bool,
    config: &Config,
    verbose: bool,
    dry_run: bool,
) -> Result<TransferResult, FileError> {
    // Validate local file
    if !path.exists() {
        return Err(FileError::NotFound(path.display().to_string()));
    }
    if path.is_dir() {
        return Err(FileError::IsDirectory(path.display().to_string()));
    }

    // Resolve destination
    let resolved = resolve::resolve(dest, config)?;

    // Build remote path
    let filename = path
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("file");
    let remote_path = format!("{}/{}", resolved.path, filename);

    let size = path.metadata().map(|m| m.len()).unwrap_or(0);

    if dry_run {
        println!("[DRY RUN] Would copy file:");
        println!("  Source: {}", path.display());
        println!("  Destination: {}@{}:{}", resolved.user, resolved.host, remote_path);
        println!("  Size: {} bytes", size);
        if !overwrite {
            println!("  Check: Would verify remote file doesn't exist");
        }

        return Ok(TransferResult {
            source: path.display().to_string(),
            dest_host: resolved.host,
            dest_path: remote_path,
            bytes: size,
            duration_ms: 0,
            mode: "file (dry-run)".to_string(),
            archive_path: None,
        });
    }

    // Check if remote exists (unless overwrite)
    if !overwrite && transfer::remote_file_exists(&resolved, &remote_path, verbose)? {
        return Err(FileError::RemoteExists(remote_path));
    }

    // Ensure remote directory exists
    transfer::ensure_remote_dir(&resolved, &remote_path, verbose)?;

    // Transfer
    let start = std::time::Instant::now();
    transfer::scp_file(path, &resolved, &remote_path, verbose)?;
    let duration = start.elapsed();

    Ok(TransferResult {
        source: path.display().to_string(),
        dest_host: resolved.host,
        dest_path: remote_path,
        bytes: size,
        duration_ms: duration.as_millis() as u64,
        mode: "file".to_string(),
        archive_path: None,
    })
}
