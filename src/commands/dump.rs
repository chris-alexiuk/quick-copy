use crate::archive;
use crate::config::Config;
use crate::output::TransferResult;
use crate::resolve;
use crate::transfer;
use std::path::Path;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum DumpError {
    #[error("path not found: {0}")]
    NotFound(String),
    #[error("{0}")]
    ArchiveError(#[from] archive::ArchiveError),
    #[error("{0}")]
    ResolveError(#[from] resolve::ResolveError),
    #[error("{0}")]
    TransferError(#[from] transfer::TransferError),
}

pub fn run(
    path: Option<&Path>,
    to: Option<&str>,
    config: &Config,
    verbose: bool,
) -> Result<TransferResult, DumpError> {
    // Default to current directory
    let source_path = match path {
        Some(p) => p.to_path_buf(),
        None => std::env::current_dir().map_err(|_| archive::ArchiveError::NoCwd)?,
    };

    if !source_path.exists() {
        return Err(DumpError::NotFound(source_path.display().to_string()));
    }

    // Resolve target share
    let share_dest = to.unwrap_or(&config.shares.default);
    let resolved = resolve::resolve(share_dest, config)?;

    // Build dump layout path
    let layout_path = build_layout_path(&config.shares.layout);
    let base_remote_path = format!("{}/{}", resolved.path, layout_path);

    let (local_file, remote_path, is_archive) = if source_path.is_dir() {
        // Zip directory
        let excludes = config.defaults.zip.exclude.clone();
        if verbose {
            eprintln!("Creating archive of {}...", source_path.display());
        }
        let archive_path =
            archive::create_archive(&source_path, &excludes, &config.defaults.staging_dir, None)?;

        let archive_name = archive_path
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("archive.zip");

        let remote = format!("{}/{}", base_remote_path, archive_name);
        (archive_path, remote, true)
    } else {
        // Single file
        let filename = source_path
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("file");
        let remote = format!("{}/{}", base_remote_path, filename);
        (source_path.clone(), remote, false)
    };

    // Ensure remote directory exists
    transfer::ensure_remote_dir(&resolved, &remote_path, verbose)?;

    // Transfer
    let start = std::time::Instant::now();
    transfer::scp_file(&local_file, &resolved, &remote_path, verbose)?;
    let duration = start.elapsed();

    let size = local_file.metadata().map(|m| m.len()).unwrap_or(0);

    // Clean up archive if we created one
    if is_archive {
        let _ = std::fs::remove_file(&local_file);
    }

    Ok(TransferResult {
        source: source_path.display().to_string(),
        dest_host: resolved.host,
        dest_path: remote_path.clone(),
        bytes: size,
        duration_ms: duration.as_millis() as u64,
        mode: "dump".to_string(),
        archive_path: if is_archive { Some(remote_path) } else { None },
    })
}

fn build_layout_path(layout: &str) -> String {
    let source = hostname::get()
        .map(|h| h.to_string_lossy().to_string())
        .unwrap_or_else(|_| "local".to_string());

    let date = archive::date_string();

    layout.replace("{source}", &source).replace("{date}", &date)
}
