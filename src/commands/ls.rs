use crate::config::Config;

pub fn run(config: &Config, json: bool) {
    if json {
        print_json(config);
    } else {
        print_human(config);
    }
}

fn print_human(config: &Config) {
    println!("Configured hosts:\n");

    let mut hosts: Vec<_> = config.hosts.iter().collect();
    hosts.sort_by_key(|(name, _)| *name);

    for (name, host) in hosts {
        let role = host.role.as_deref().unwrap_or("-");
        println!("  {} ({})", name, host.host);
        println!("    role: {}", role);

        if !host.paths.is_empty() {
            println!("    paths:");
            let mut paths: Vec<_> = host.paths.iter().collect();
            paths.sort_by_key(|(alias, _)| *alias);
            for (alias, path) in paths {
                let marker = if alias == &config.defaults.default_path_alias {
                    " (default)"
                } else {
                    ""
                };
                println!("      {}:{} -> {}{}", name, alias, path, marker);
            }
        }
        println!();
    }

    println!("Default dump target: {}", config.shares.default);
    println!("Dump layout: {}", config.shares.layout);
}

fn print_json(config: &Config) {
    #[derive(serde::Serialize)]
    struct LsOutput<'a> {
        hosts: Vec<HostInfo<'a>>,
        default_dump: &'a str,
        dump_layout: &'a str,
    }

    #[derive(serde::Serialize)]
    struct HostInfo<'a> {
        name: &'a str,
        host: &'a str,
        role: Option<&'a str>,
        paths: Vec<PathInfo<'a>>,
    }

    #[derive(serde::Serialize)]
    struct PathInfo<'a> {
        alias: &'a str,
        path: &'a str,
        is_default: bool,
    }

    let mut hosts: Vec<_> = config
        .hosts
        .iter()
        .map(|(name, host)| {
            let mut paths: Vec<_> = host
                .paths
                .iter()
                .map(|(alias, path)| PathInfo {
                    alias,
                    path,
                    is_default: alias == &config.defaults.default_path_alias,
                })
                .collect();
            paths.sort_by_key(|p| p.alias);

            HostInfo {
                name,
                host: &host.host,
                role: host.role.as_deref(),
                paths,
            }
        })
        .collect();
    hosts.sort_by_key(|h| h.name);

    let output = LsOutput {
        hosts,
        default_dump: &config.shares.default,
        dump_layout: &config.shares.layout,
    };

    if let Ok(json) = serde_json::to_string_pretty(&output) {
        println!("{}", json);
    }
}
