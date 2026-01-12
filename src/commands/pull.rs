use crate::config::Config;
use crate::output::TransferResult;
use crate::resolve;
use crate::transfer;
use std::path::PathBuf;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum PullError {
    #[error("{0}")]
    ResolveError(#[from] resolve::ResolveError),
    #[error("{0}")]
    TransferError(#[from] transfer::TransferError),
    #[error("failed to extract archive: {0}")]
    ExtractError(String),
    #[error("failed to get current directory")]
    NoCwd,
}

pub fn run(
    source: &str,
    extract: bool,
    config: &Config,
    verbose: bool,
    dry_run: bool,
) -> Result<TransferResult, PullError> {
    // Resolve source
    let resolved = resolve::resolve(source, config)?;

    // Get current directory
    let cwd = std::env::current_dir().map_err(|_| PullError::NoCwd)?;

    if dry_run {
        println!("[DRY RUN] Would pull remote directory:");
        println!("  Source: {}@{}:{}", resolved.user, resolved.host, resolved.path);
        println!("  Destination: {}", cwd.display());
        if extract {
            println!("  Extract: Yes (would extract after download)");
        } else {
            println!("  Extract: No (would keep as zip)");
        }

        return Ok(TransferResult {
            source: format!("{}:{}", resolved.host, resolved.path),
            dest_host: hostname::get()
                .ok()
                .and_then(|h| h.into_string().ok())
                .unwrap_or_else(|| "localhost".to_string()),
            dest_path: cwd.display().to_string(),
            bytes: 0,
            duration_ms: 0,
            mode: "pull (dry-run)".to_string(),
            archive_path: None,
        });
    }

    // Combine excludes from config
    let excludes = config.defaults.zip.exclude.clone();

    // Create archive on remote
    if verbose {
        eprintln!("Creating archive on remote {}...", resolved.host);
    }
    let remote_archive_path = transfer::remote_zip(
        &resolved,
        &resolved.path,
        &excludes,
        &config.defaults.staging_dir,
        verbose,
    )?;

    // Get archive filename
    let archive_name = remote_archive_path
        .split('/')
        .last()
        .unwrap_or("archive.zip");
    let local_archive_path = cwd.join(archive_name);

    // Transfer from remote to local
    if verbose {
        eprintln!("Downloading archive...");
    }
    let start = std::time::Instant::now();
    transfer::scp_from_remote(&resolved, &remote_archive_path, &local_archive_path, verbose)?;
    let duration = start.elapsed();

    let size = local_archive_path.metadata().map(|m| m.len()).unwrap_or(0);

    // Extract locally if requested
    let final_path = if extract {
        if verbose {
            eprintln!("Extracting archive...");
        }
        extract_local(&local_archive_path)?
    } else {
        local_archive_path.display().to_string()
    };

    // Clean up remote archive
    if verbose {
        eprintln!("Cleaning up remote archive...");
    }
    let _ = transfer::remote_rm(&resolved, &remote_archive_path, verbose);

    Ok(TransferResult {
        source: format!("{}:{}", resolved.host, resolved.path),
        dest_host: hostname::get()
            .ok()
            .and_then(|h| h.into_string().ok())
            .unwrap_or_else(|| "localhost".to_string()),
        dest_path: final_path,
        bytes: size,
        duration_ms: duration.as_millis() as u64,
        mode: "pull".to_string(),
        archive_path: Some(local_archive_path.display().to_string()),
    })
}

/// Extract a zip file locally
fn extract_local(archive_path: &PathBuf) -> Result<String, PullError> {
    use std::fs::File;
    use zip::ZipArchive;

    let file = File::open(archive_path)
        .map_err(|e| PullError::ExtractError(format!("failed to open archive: {}", e)))?;

    let mut archive = ZipArchive::new(file)
        .map_err(|e| PullError::ExtractError(format!("invalid zip archive: {}", e)))?;

    let extract_dir = archive_path
        .parent()
        .ok_or_else(|| PullError::ExtractError("invalid archive path".to_string()))?;

    for i in 0..archive.len() {
        let mut file = archive.by_index(i)
            .map_err(|e| PullError::ExtractError(format!("failed to read entry: {}", e)))?;

        let outpath = match file.enclosed_name() {
            Some(path) => extract_dir.join(path),
            None => continue,
        };

        if file.name().ends_with('/') {
            std::fs::create_dir_all(&outpath)
                .map_err(|e| PullError::ExtractError(format!("failed to create directory: {}", e)))?;
        } else {
            if let Some(p) = outpath.parent() {
                if !p.exists() {
                    std::fs::create_dir_all(p)
                        .map_err(|e| PullError::ExtractError(format!("failed to create directory: {}", e)))?;
                }
            }
            let mut outfile = File::create(&outpath)
                .map_err(|e| PullError::ExtractError(format!("failed to create file: {}", e)))?;
            std::io::copy(&mut file, &mut outfile)
                .map_err(|e| PullError::ExtractError(format!("failed to extract file: {}", e)))?;
        }
    }

    // Remove the archive after extraction
    let _ = std::fs::remove_file(archive_path);

    Ok(extract_dir.display().to_string())
}
