use anyhow::Result;
use clap::{Parser, Subcommand};

use crate::config::Config;
use crate::git;
use crate::github;
use crate::package;

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand)]
pub enum Commands {
    /// Update a package in all repositories
    Update {
        /// Package name to update
        package: String,

        /// New version to set
        version: String,

        /// Commit message (optional)
        #[arg(short, long)]
        message: Option<String>,

        /// Create pull request
        #[arg(short, long)]
        pull_request: bool,

        /// Dry run (don't make any changes)
        #[arg(short, long)]
        dry_run: bool,
    },

    /// Add a new repository to the config
    AddRepo {
        /// Local path to the repository
        path: String,
    },

    /// Remove a repository from the config
    RemoveRepo {
        /// Local path to the repository
        path: String,
    },

    /// List all configured repositories
    ListRepos,

    /// Compare package versions across repositories
    Compare {
        /// Package name to compare
        package: String,
    },

    /// List all packages in a repository
    ListPackages {
        /// Repository path (optional, uses all repositories if not specified)
        #[arg(short, long)]
        repo: Option<String>,
    },

    /// Clone a repository
    Clone {
        /// GitHub URL of the repository
        github_url: String,

        /// Local path to clone to
        #[arg(short, long)]
        output: Option<String>,

        /// Add to config after cloning
        #[arg(short, long)]
        add: bool,
    },

    /// Set default package manager
    SetPackageManager {
        /// Package manager name (npm, yarn, pnpm)
        name: String,
    },
}

/// Handle update command
pub fn handle_update(
    config: &Config,
    package: &str,
    version: &str,
    message: Option<&str>,
    pull_request: bool,
    dry_run: bool,
) -> Result<()> {
    if config.repositories.is_empty() {
        println!("No repositories configured. Use 'add-repo' command to add repositories.");
        return Ok(());
    }

    let commit_message = message
        .unwrap_or(&format!("chore: update {} to {}", package, version))
        .to_string();

    if dry_run {
        println!("DRY RUN MODE - No changes will be made");
    }

    println!(
        "Updating package '{}' to version '{}' in {} repositories",
        package,
        version,
        config.repositories.len()
    );

    for repo in &config.repositories {
        if let Err(e) = git::update_package_workflow(
            repo,
            package,
            version,
            &commit_message,
            pull_request,
            dry_run,
            config,
        ) {
            eprintln!("Error processing repository {}: {}", repo.path, e);

            // 사용자에게 계속할지 물어보기
            if !prompt_continue() {
                println!("Aborting update process");
                break;
            }
        }
    }

    Ok(())
}

/// Handle add repository command
pub fn handle_add_repo(config: &mut Config, path: &str) -> Result<()> {
    match config.add_repository(path.to_string()) {
        Ok(_) => {
            println!("Repository added successfully: {}", path);
            Ok(())
        }
        Err(e) => {
            eprintln!("Failed to add repository: {}", e);
            Err(e)
        }
    }
}

/// Handle remove repository command
pub fn handle_remove_repo(config: &mut Config, path: &str) -> Result<()> {
    match config.remove_repository(path) {
        Ok(_) => {
            println!("Repository removed successfully: {}", path);
            Ok(())
        }
        Err(e) => {
            eprintln!("Failed to remove repository: {}", e);
            Err(e)
        }
    }
}

/// Handle list repositories command
pub fn handle_list_repos(config: &Config) -> Result<()> {
    if config.repositories.is_empty() {
        println!("No repositories configured");
    } else {
        println!("Configured repositories:");
        for (i, repo) in config.repositories.iter().enumerate() {
            println!("{}. Path: {}", i + 1, repo.path);

            // Git 상태 확인
            match git::check_status(&repo.path) {
                Ok(has_changes) => {
                    if has_changes {
                        println!("   Status: Changes present");
                    } else {
                        println!("   Status: Clean");
                    }

                    // 현재 브랜치 표시
                    if let Ok(branch) = git::get_current_branch(&repo.path) {
                        println!("   Branch: {}", branch);
                    }

                    // 패키지 매니저 감지
                    if let Ok(pkg_manager) = package::detect_package_manager(&repo.path) {
                        println!("   Package Manager: {}", pkg_manager);
                    }
                }
                Err(e) => println!("   Status check failed: {}", e),
            }
        }
    }

    Ok(())
}

/// Handle package version comparison command
pub fn handle_compare(config: &Config, package: &str) -> Result<()> {
    if config.repositories.is_empty() {
        println!("No repositories configured");
        return Ok(());
    }

    println!("Comparing package '{}' across repositories:", package);

    let mut repo_paths = Vec::new();
    for repo in &config.repositories {
        repo_paths.push(repo.path.as_str());
    }

    let versions = package::compare_package_versions(&repo_paths, package)?;

    for (repo_path, version) in versions {
        match version {
            Some(v) => println!("{}: {}", repo_path, v),
            None => println!("{}: Not found", repo_path),
        }
    }

    Ok(())
}

/// Handle list packages command
pub fn handle_list_packages(config: &Config, repo_path: Option<&str>) -> Result<()> {
    if config.repositories.is_empty() && repo_path.is_none() {
        println!("No repositories configured");
        return Ok(());
    }

    let repositories = if let Some(path) = repo_path {
        // Process specific repository only
        let repo = config
            .repositories
            .iter()
            .find(|r| r.path == path)
            .ok_or_else(|| anyhow::anyhow!("Repository not found: {}", path))?;

        vec![repo]
    } else {
        // Process all repositories
        config.repositories.iter().collect()
    };

    for repo in repositories {
        println!("Packages in {}:", repo.path);

        match package::list_all_packages(&repo.path) {
            Ok(packages) => {
                if packages.is_empty() {
                    println!("  No packages found");
                } else {
                    // Group packages by type
                    let mut deps = Vec::new();
                    let mut dev_deps = Vec::new();
                    let mut peer_deps = Vec::new();

                    for (name, version, dep_type) in packages {
                        match dep_type.as_str() {
                            "dependencies" => deps.push((name, version)),
                            "devDependencies" => dev_deps.push((name, version)),
                            "peerDependencies" => peer_deps.push((name, version)),
                            _ => {}
                        }
                    }

                    if !deps.is_empty() {
                        println!("  Dependencies:");
                        for (name, version) in deps {
                            println!("    {}: {}", name, version);
                        }
                    }

                    if !dev_deps.is_empty() {
                        println!("  Dev Dependencies:");
                        for (name, version) in dev_deps {
                            println!("    {}: {}", name, version);
                        }
                    }

                    if !peer_deps.is_empty() {
                        println!("  Peer Dependencies:");
                        for (name, version) in peer_deps {
                            println!("    {}: {}", name, version);
                        }
                    }
                }
            }
            Err(e) => println!("  Error listing packages: {}", e),
        }
    }

    Ok(())
}

/// 레포지토리 클론 명령 처리
pub fn handle_clone(
    config: &mut Config,
    github_url: &str,
    output: Option<&str>,
    add: bool,
) -> Result<()> {
    // Determine output directory
    let output_dir = if let Some(dir) = output {
        dir.to_string()
    } else {
        // Extract repository name from URL
        let repo_name = github_url
            .split('/')
            .last()
            .map(|s| s.trim_end_matches(".git"))
            .unwrap_or("repo")
            .to_string();

        repo_name
    };

    // Clone repository
    github::clone_repository(github_url, &output_dir)?;

    // Add to config
    if add {
        let path = std::fs::canonicalize(&output_dir)
            .map_err(|e| anyhow::anyhow!("Failed to resolve path: {}", e))?
            .to_string_lossy()
            .to_string();

        handle_add_repo(config, &path)?;
    }

    Ok(())
}

/// Handle set package manager command
pub fn handle_set_package_manager(config: &mut Config, name: &str) -> Result<()> {
    let valid_managers = vec!["npm", "yarn", "pnpm"];
    if !valid_managers.contains(&name) {
        anyhow::bail!(
            "Invalid package manager. Must be one of: {:?}",
            valid_managers
        );
    }

    config.default_package_manager = Some(name.to_string());
    config.save()?;
    println!("Default package manager set to: {}", name);
    Ok(())
}

/// Ask user if they want to continue
fn prompt_continue() -> bool {
    use std::io::{self, Write};

    print!("Continue with remaining repositories? [y/N]: ");
    io::stdout().flush().unwrap();

    let mut input = String::new();
    if io::stdin().read_line(&mut input).is_err() {
        return false;
    }

    input.trim().eq_ignore_ascii_case("y")
}
