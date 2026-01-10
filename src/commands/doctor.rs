use crate::config::Config;
use crate::resolve;
use crate::transfer;
use std::process::Command;

pub fn run(test_hosts: &[String], config: &Config, verbose: bool) -> bool {
    let mut all_ok = true;

    println!("quick-copy doctor\n");

    // Check required tools
    println!("Checking tools:");
    all_ok &= check_tool("ssh", true);
    all_ok &= check_tool("scp", true);
    all_ok &= check_tool("zip", false); // Optional for receiving
    all_ok &= check_tool("unzip", false); // Optional

    println!();

    // Check config
    println!("Configuration:");
    println!("  hosts configured: {}", config.hosts.len());
    println!("  default path alias: {}", config.defaults.default_path_alias);
    println!("  default dump target: {}", config.shares.default);
    println!("  staging directory: {}", config.defaults.staging_dir);

    // Check staging dir exists
    let staging = std::path::Path::new(&config.defaults.staging_dir);
    if staging.exists() && staging.is_dir() {
        println!("  staging dir exists: yes");
    } else {
        println!("  staging dir exists: NO (will be created)");
    }

    println!();

    // Test connectivity if requested
    if !test_hosts.is_empty() {
        println!("Testing connectivity:");
        for host_name in test_hosts {
            match resolve::resolve(host_name, config) {
                Ok(resolved) => {
                    print!("  {} ({})... ", host_name, resolved.host);
                    match transfer::test_connectivity(&resolved, verbose) {
                        Ok(_) => println!("ok"),
                        Err(e) => {
                            println!("FAILED");
                            if verbose {
                                eprintln!("    {}", e);
                            }
                            all_ok = false;
                        }
                    }
                }
                Err(e) => {
                    println!("  {}: {}", host_name, e);
                    all_ok = false;
                }
            }
        }
        println!();
    }

    // Summary
    if all_ok {
        println!("All checks passed.");
    } else {
        println!("Some checks failed. See above for details.");
    }

    all_ok
}

fn check_tool(name: &str, required: bool) -> bool {
    let status = Command::new("which").arg(name).output();

    match status {
        Ok(output) if output.status.success() => {
            let path = String::from_utf8_lossy(&output.stdout);
            println!("  {}: {}", name, path.trim());
            true
        }
        _ => {
            if required {
                println!("  {}: NOT FOUND (required)", name);
                false
            } else {
                println!("  {}: not found (optional)", name);
                true
            }
        }
    }
}
