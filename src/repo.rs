use anyhow::{Context, Result};
use std::path::{Path, PathBuf};
use std::process::Command;

use crate::config::{expand_tilde, Repository};

pub fn expand_path(path: &str) -> Result<PathBuf> {
    let expanded = expand_tilde(path)?;
    Ok(PathBuf::from(expanded))
}

/// Check repository status
pub fn check_repository(repo: &Repository) -> Result<bool> {
    let path = Path::new(&repo.path);

    // Check if path exists
    if !path.exists() {
        anyhow::bail!("Repository path does not exist: {}", repo.path);
    }

    // Check if .git directory exists
    let git_dir = path.join(".git");
    if !git_dir.exists() {
        anyhow::bail!("Not a git repository: {}", repo.path);
    }

    // Run git status to check for changes
    let output = Command::new("git")
        .current_dir(&repo.path)
        .args(["status", "--porcelain"])
        .output()
        .context("Failed to execute git status")?;

    if !output.status.success() {
        anyhow::bail!("Failed to get git status for repository: {}", repo.path);
    }

    // Check if there are changes (non-empty output means changes)
    let has_changes = !output.stdout.is_empty();

    Ok(has_changes)
}

/// Get current branch name
pub fn get_current_branch(repo_path: &str) -> Result<String> {
    let output = Command::new("git")
        .current_dir(repo_path)
        .args(["branch", "--show-current"])
        .output()
        .context("Failed to get current branch")?;

    if !output.status.success() {
        anyhow::bail!("Failed to get current branch for repository: {}", repo_path);
    }

    let branch = String::from_utf8(output.stdout)
        .context("Invalid UTF-8 in branch name")?
        .trim()
        .to_string();

    Ok(branch)
}

/// Create branch
pub fn create_branch(repo_path: &str, branch_name: &str, dry_run: bool) -> Result<()> {
    if dry_run {
        println!("Would create branch '{}' in {}", branch_name, repo_path);
        return Ok(());
    }

    println!("Creating branch '{}' in {}", branch_name, repo_path);

    // Save original branch
    let original_branch = get_current_branch(repo_path)?;

    // Check if branch already exists
    let output = Command::new("git")
        .current_dir(repo_path)
        .args(["branch"])
        .output()
        .context("Failed to list branches")?;

    let branches = String::from_utf8(output.stdout).context("Invalid UTF-8 in branch list")?;

    let branch_exists = branches
        .lines()
        .any(|line| line.trim().ends_with(branch_name));

    if branch_exists {
        // If branch exists, check out
        let status = Command::new("git")
            .current_dir(repo_path)
            .args(["checkout", branch_name])
            .status()
            .context("Failed to checkout existing branch")?;

        if !status.success() {
            anyhow::bail!("Failed to checkout existing branch: {}", branch_name);
        }
    } else {
        // Create new branch
        let status = Command::new("git")
            .current_dir(repo_path)
            .args(["checkout", "-b", branch_name])
            .status()
            .context("Failed to create new branch")?;

        if !status.success() {
            // Return to original branch on failure
            let _ = Command::new("git")
                .current_dir(repo_path)
                .args(["checkout", &original_branch])
                .status();

            anyhow::bail!("Failed to create branch: {}", branch_name);
        }
    }

    Ok(())
}

/// Return to original branch
pub fn checkout_original_branch(
    repo_path: &str,
    original_branch: &str,
    dry_run: bool,
) -> Result<()> {
    if dry_run {
        println!(
            "Would checkout original branch '{}' in {}",
            original_branch, repo_path
        );
        return Ok(());
    }

    println!(
        "Checking out original branch '{}' in {}",
        original_branch, repo_path
    );

    let status = Command::new("git")
        .current_dir(repo_path)
        .args(["checkout", original_branch])
        .status()
        .context("Failed to checkout original branch")?;

    if !status.success() {
        anyhow::bail!("Failed to checkout original branch: {}", original_branch);
    }

    Ok(())
}

/// Pull repository
pub fn pull_repository(repo_path: &str, dry_run: bool) -> Result<()> {
    if dry_run {
        println!("Would pull latest changes in {}", repo_path);
        return Ok(());
    }

    println!("Pulling latest changes in {}", repo_path);

    let status = Command::new("git")
        .current_dir(repo_path)
        .args(["pull"])
        .status()
        .context("Failed to pull repository")?;

    if !status.success() {
        anyhow::bail!("Failed to pull repository: {}", repo_path);
    }

    Ok(())
}
