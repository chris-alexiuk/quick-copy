mod archive;
mod cli;
mod commands;
mod config;
mod output;
mod resolve;
mod transfer;

use clap::Parser;
use cli::{Cli, Commands};
use config::Config;
use output::Output;
use std::process::ExitCode;

fn main() -> ExitCode {
    let cli = Cli::parse();

    // Handle version command early (no config needed)
    if matches!(cli.command, Commands::Version) {
        println!("quick-copy {}", env!("CARGO_PKG_VERSION"));
        return ExitCode::SUCCESS;
    }

    // Load config
    let config = match Config::load(cli.config.clone()) {
        Ok(c) => c,
        Err(e) => {
            eprintln!("error: {}", e);
            eprintln!("\nCreate a config file at ~/.config/quick-copy/config.yaml");
            eprintln!("See: https://github.com/your-repo/quick-copy#configuration");
            return ExitCode::FAILURE;
        }
    };

    let output = if cli.json {
        Output::Json
    } else {
        Output::Human
    };

    match cli.command {
        Commands::File { path, dest, overwrite } => {
            match commands::file::run(&path, &dest, overwrite, &config, cli.verbose) {
                Ok(result) => {
                    output.print(&result);
                    ExitCode::SUCCESS
                }
                Err(e) => {
                    eprintln!("error: {}", e);
                    ExitCode::FAILURE
                }
            }
        }

        Commands::Dir { dest, name, exclude, extract } => {
            match commands::dir::run(&dest, name.as_deref(), &exclude, extract, &config, cli.verbose) {
                Ok(result) => {
                    output.print(&result);
                    ExitCode::SUCCESS
                }
                Err(e) => {
                    eprintln!("error: {}", e);
                    ExitCode::FAILURE
                }
            }
        }

        Commands::Dump { path, to } => {
            match commands::dump::run(path.as_deref(), to.as_deref(), &config, cli.verbose) {
                Ok(result) => {
                    output.print(&result);
                    ExitCode::SUCCESS
                }
                Err(e) => {
                    eprintln!("error: {}", e);
                    ExitCode::FAILURE
                }
            }
        }

        Commands::Pull { source, no_extract } => {
            match commands::pull::run(&source, !no_extract, &config, cli.verbose) {
                Ok(result) => {
                    output.print(&result);
                    ExitCode::SUCCESS
                }
                Err(e) => {
                    eprintln!("error: {}", e);
                    ExitCode::FAILURE
                }
            }
        }

        Commands::Ls => {
            commands::ls::run(&config, cli.json);
            ExitCode::SUCCESS
        }

        Commands::Doctor { test } => {
            let ok = commands::doctor::run(&test, &config, cli.verbose);
            if ok {
                ExitCode::SUCCESS
            } else {
                ExitCode::FAILURE
            }
        }

        Commands::Version => {
            // Already handled above
            ExitCode::SUCCESS
        }
    }
}
