use crate::archive;
use crate::config::Config;
use crate::output::TransferResult;
use crate::resolve;
use crate::transfer;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum DirError {
    #[error("{0}")]
    ArchiveError(#[from] archive::ArchiveError),
    #[error("{0}")]
    ResolveError(#[from] resolve::ResolveError),
    #[error("{0}")]
    TransferError(#[from] transfer::TransferError),
}

pub fn run(
    dest: &str,
    name: Option<&str>,
    extra_excludes: &[String],
    extract: bool,
    config: &Config,
    verbose: bool,
    dry_run: bool,
) -> Result<TransferResult, DirError> {
    // Get current directory
    let cwd = std::env::current_dir().map_err(|_| archive::ArchiveError::NoCwd)?;

    // Combine excludes from config and CLI
    let mut excludes = config.defaults.zip.exclude.clone();
    excludes.extend(extra_excludes.iter().cloned());

    // Resolve destination
    let resolved = resolve::resolve(dest, config)?;

    if dry_run {
        println!("[DRY RUN] Would zip and copy directory:");
        println!("  Source: {}", cwd.display());
        println!("  Destination: {}@{}:{}", resolved.user, resolved.host, resolved.path);
        println!("  Excludes: {:?}", excludes);
        if extract {
            println!("  Extract: Yes (would extract after upload)");
        }

        return Ok(TransferResult {
            source: cwd.display().to_string(),
            dest_host: resolved.host,
            dest_path: resolved.path,
            bytes: 0,
            duration_ms: 0,
            mode: "dir (dry-run)".to_string(),
            archive_path: None,
        });
    }

    // Create archive
    if verbose {
        eprintln!("Creating archive of {}...", cwd.display());
    }
    let archive_path = archive::create_archive(&cwd, &excludes, &config.defaults.staging_dir, name)?;

    // Build remote path
    let archive_name = archive_path
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("archive.zip");
    let remote_path = format!("{}/{}", resolved.path, archive_name);

    // Ensure remote directory exists
    transfer::ensure_remote_dir(&resolved, &remote_path, verbose)?;

    // Transfer
    let start = std::time::Instant::now();
    transfer::scp_file(&archive_path, &resolved, &remote_path, verbose)?;
    let duration = start.elapsed();

    let size = archive_path.metadata().map(|m| m.len()).unwrap_or(0);

    // Optionally extract on remote
    let final_path = if extract {
        if verbose {
            eprintln!("Extracting on remote...");
        }
        transfer::remote_unzip(&resolved, &remote_path, verbose)?
    } else {
        remote_path.clone()
    };

    // Clean up local archive
    let _ = std::fs::remove_file(&archive_path);

    Ok(TransferResult {
        source: cwd.display().to_string(),
        dest_host: resolved.host,
        dest_path: final_path,
        bytes: size,
        duration_ms: duration.as_millis() as u64,
        mode: "dir".to_string(),
        archive_path: Some(remote_path),
    })
}
