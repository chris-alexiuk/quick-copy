use clap::{Parser, Subcommand};
use std::path::PathBuf;

#[derive(Parser)]
#[command(name = "quick-copy")]
#[command(author, version, about = "Fast SSH/SCP copy tool with named targets")]
#[command(propagate_version = true)]
pub struct Cli {
    /// Path to config file
    #[arg(short, long, global = true)]
    pub config: Option<PathBuf>,

    /// Show verbose output including ssh/scp commands
    #[arg(short, long, global = true)]
    pub verbose: bool,

    /// Output in JSON format for scripting
    #[arg(long, global = true)]
    pub json: bool,

    /// Preview operations without executing them
    #[arg(long, global = true)]
    pub dry_run: bool,

    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand)]
pub enum Commands {
    /// Copy a single file to a remote destination
    #[command(alias = "f")]
    File {
        /// Local file to copy
        path: PathBuf,

        /// Destination (host, host:alias, or host:/path)
        dest: String,

        /// Overwrite existing files without prompting
        #[arg(long)]
        overwrite: bool,
    },

    /// Zip and copy current directory to a remote destination
    #[command(alias = "d")]
    Dir {
        /// Destination (host, host:alias, or host:/path)
        dest: String,

        /// Custom archive name (default: <dirname>-<timestamp>.zip)
        #[arg(short, long)]
        name: Option<String>,

        /// Additional exclude patterns (glob syntax)
        #[arg(short, long)]
        exclude: Vec<String>,

        /// Extract the archive on the remote after upload
        #[arg(long)]
        extract: bool,
    },

    /// Dump file or directory to shared storage (Ganymede by default)
    Dump {
        /// Path to dump (default: current directory)
        path: Option<PathBuf>,

        /// Target share (default: from config)
        #[arg(long)]
        to: Option<String>,
    },

    /// List configured hosts and path aliases
    Ls,

    /// Check prerequisites and configuration
    Doctor {
        /// Test SSH connectivity to specific hosts
        #[arg(long)]
        test: Vec<String>,
    },

    /// Pull a remote directory to current working directory
    #[command(alias = "p")]
    Pull {
        /// Source (host:alias or host:/path)
        source: String,

        /// Keep the zip archive without extracting
        #[arg(long)]
        no_extract: bool,
    },

    /// Show version information
    Version,
}
