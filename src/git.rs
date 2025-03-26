use anyhow::{Context, Result};
use std::path::PathBuf;
use std::process::Command;

use crate::config::Repository;
use crate::repo::expand_path;
use crate::config::Config;

/// Get current branch name
pub fn get_current_branch(repo_path: &str) -> Result<String> {
    let path = expand_path(repo_path)?;

    let output = Command::new("git")
        .current_dir(path)
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
pub fn create_branch(repo_path: &str, branch_name: &str, dry_run: bool) -> Result<String> {
    let path = expand_path(repo_path)?;

    // Save current branch (for restoration in case of failure)
    let original_branch = get_current_branch(repo_path)?;

    if dry_run {
        println!("Would create branch '{}' in {}", branch_name, repo_path);
        return Ok(original_branch);
    }

    println!("Creating branch '{}' in {}", branch_name, repo_path);

    // Check if branch already exists
    let output = Command::new("git")
        .current_dir(&path)
        .args(["branch", "--list", branch_name])
        .output()
        .context("Failed to list branches")?;

    let branch_exists = !output.stdout.is_empty();

    if branch_exists {
        // If branch exists, check out
        let status = Command::new("git")
            .current_dir(&path)
            .args(["checkout", branch_name])
            .status()
            .context("Failed to checkout existing branch")?;

        if !status.success() {
            anyhow::bail!("Failed to checkout existing branch: {}", branch_name);
        }
    } else {
        // If branch does not exist, create new branch
        let status = Command::new("git")
            .current_dir(&path)
            .args(["checkout", "-b", branch_name])
            .status()
            .context("Failed to create new branch")?;

        if !status.success() {
            anyhow::bail!("Failed to create branch: {}", branch_name);
        }
    }

    Ok(original_branch)
}

/// Stage changes
pub fn stage_changes(repo_path: &str, files: &[&str], dry_run: bool) -> Result<()> {
    let path = expand_path(repo_path)?;

    if dry_run {
        println!("Would stage files in {}: {:?}", repo_path, files);
        return Ok(());
    }

    println!("Staging files in {}: {:?}", repo_path, files);

    let mut cmd = Command::new("git");
    cmd.current_dir(&path).arg("add");

    for file in files {
        cmd.arg(file);
    }

    let status = cmd.status().context("Failed to stage changes")?;

    if !status.success() {
        anyhow::bail!("Failed to stage changes");
    }

    Ok(())
}

/// Commit changes
pub fn commit_changes(repo_path: &str, message: &str, dry_run: bool) -> Result<()> {
    let path = expand_path(repo_path)?;

    if dry_run {
        println!("Would commit changes with message: '{}'", message);
        return Ok(());
    }

    println!("Committing changes with message: '{}'", message);

    // Check if there are staged changes
    let output = Command::new("git")
        .current_dir(&path)
        .args(["diff", "--staged", "--name-only"])
        .output()
        .context("Failed to check staged changes")?;

    if output.stdout.is_empty() {
        println!("No staged changes to commit");
        return Ok(());
    }

    // Commit changes
    let status = Command::new("git")
        .current_dir(&path)
        .args(["commit", "-m", message])
        .status()
        .context("Failed to commit changes")?;

    if !status.success() {
        anyhow::bail!("Failed to commit changes");
    }

    Ok(())
}

/// Push branch
pub fn push_branch(repo_path: &str, branch_name: &str, dry_run: bool) -> Result<()> {
    let path = expand_path(repo_path)?;

    if dry_run {
        println!("Would push branch '{}' to origin", branch_name);
        return Ok(());
    }

    println!("Pushing branch '{}' to origin", branch_name);

    let status = Command::new("git")
        .current_dir(&path)
        .args(["push", "--set-upstream", "origin", branch_name])
        .status()
        .context("Failed to push branch")?;

    if !status.success() {
        anyhow::bail!("Failed to push branch: {}", branch_name);
    }

    Ok(())
}

/// Return to original branch
pub fn checkout_branch(repo_path: &str, branch_name: &str, dry_run: bool) -> Result<()> {
    let path = expand_path(repo_path)?;

    if dry_run {
        println!("Would checkout branch '{}' in {}", branch_name, repo_path);
        return Ok(());
    }

    println!("Checking out branch '{}' in {}", branch_name, repo_path);

    let status = Command::new("git")
        .current_dir(&path)
        .args(["checkout", branch_name])
        .status()
        .context("Failed to checkout branch")?;

    if !status.success() {
        anyhow::bail!("Failed to checkout branch: {}", branch_name);
    }

    Ok(())
}

/// Check repository status
pub fn check_status(repo_path: &str) -> Result<bool> {
    let path = expand_path(repo_path)?;

    let output = Command::new("git")
        .current_dir(&path)
        .args(["status", "--porcelain"])
        .output()
        .context("Failed to check git status")?;

    if !output.status.success() {
        anyhow::bail!("Failed to check git status");
    }

    // Check if there are changes (non-empty output means changes)
    let has_changes = !output.stdout.is_empty();

    Ok(has_changes)
}

/// Pull repository
pub fn pull_repository(repo_path: &str, dry_run: bool) -> Result<()> {
    let path = expand_path(repo_path)?;

    if dry_run {
        println!("Would pull latest changes in {}", repo_path);
        return Ok(());
    }

    println!("Pulling latest changes in {}", repo_path);

    let status = Command::new("git")
        .current_dir(&path)
        .args(["pull"])
        .status()
        .context("Failed to pull repository")?;

    if !status.success() {
        anyhow::bail!("Failed to pull repository: {}", repo_path);
    }

    Ok(())
}

/// Execute package update workflow
pub fn update_package_workflow(
    repo: &Repository,
    package_name: &str,
    version: &str,
    commit_message: &str,
    create_pr: bool,
    dry_run: bool,
    config: &Config,
) -> Result<()> {
    println!("\n=== Processing repository: {} ===", repo.path);

    // 1. Save current branch
    let original_branch = get_current_branch(&repo.path)?;

    // 2. Create branch
    let branch_name = format!(
        "update-{}-{}",
        package_name,
        version.replace("^", "").replace("~", "")
    );
    create_branch(&repo.path, &branch_name, dry_run)?;

    // 3. Update package.json (this function is in package.rs)
    let updated = crate::package::update_package(&repo.path, package_name, version, dry_run)?;

    if !updated {
        println!(
            "Package '{}' is already at version '{}', skipping",
            package_name, version
        );
        // Return to original branch
        checkout_branch(&repo.path, &original_branch, dry_run)?;
        return Ok(());
    }

    // 4. Run package install with default package manager
    let pkg_manager = match crate::package::detect_package_manager(&repo.path) {
        Ok(manager) => manager,
        Err(_) => config.default_package_manager.clone().unwrap(),
    };
    crate::package::run_install_with_manager(&repo.path, &pkg_manager, dry_run)?;

    // 5. Stage changes
    stage_changes(
        &repo.path,
        &[
            "package.json",
            "pnpm-lock.yaml",
            "yarn.lock",
            "package-lock.json",
        ],
        dry_run,
    )?;

    // 6. Commit changes
    commit_changes(&repo.path, commit_message, dry_run)?;

    // 7. Push to GitHub
    push_branch(&repo.path, &branch_name, dry_run)?;

    // 8. Create PR (optional) - this function will be implemented in github.rs
    if create_pr {
        if let Err(e) = crate::github::create_pr(
            &repo.path,
            &repo.github_url,
            &branch_name,
            commit_message,
            dry_run,
            true, // draft by default
            None, // use default body
        ) {
            eprintln!("Warning: Failed to create PR: {}", e);
        }
    }

    println!(
        "✅ Successfully updated {} to {} in {}",
        package_name, version, repo.path
    );

    // 9. Return to original branch
    checkout_branch(&repo.path, &original_branch, dry_run)?;

    Ok(())
}
