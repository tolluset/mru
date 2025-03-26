use anyhow::{Context, Result};
use std::path::PathBuf;
use std::process::Command;

use crate::repo::expand_path;

/// GitHub CLI is installed and authenticated
pub fn check_gh_cli() -> Result<bool> {
    let output = Command::new("gh")
        .args(["auth", "status"])
        .output()
        .context("Failed to check GitHub CLI authentication. Is GitHub CLI installed?")?;

    Ok(output.status.success())
}

/// Create Pull Request
pub fn create_pr(
    repo_path: &str,
    github_url: &str,
    branch_name: &str,
    title: &str,
    dry_run: bool,
    draft: bool,
    body: Option<&str>,
) -> Result<String> {
    let path = expand_path(repo_path)?;

    if dry_run {
        println!(
            "Would create PR for branch '{}' with title: '{}'",
            branch_name, title
        );
        return Ok(String::from("dry-run-pr-url"));
    }

    // Check if GitHub CLI is installed
    if !check_gh_cli()? {
        anyhow::bail!(
            "GitHub CLI is not installed or not authenticated. Please run 'gh auth login'"
        );
    }

    println!(
        "Creating PR for branch '{}' with title: '{}'",
        branch_name, title
    );

    // Create PR
    let mut args = vec![
        "pr",
        "create",
        "--title",
        title,
        "--head",
        branch_name,
    ];

    if draft {
        args.push("--draft");
    }

    if let Some(body_text) = body {
        args.extend_from_slice(&["--body", body_text]);
    }

    let output = Command::new("gh")
        .current_dir(&path)
        .args(&args)
        .output()
        .context("Failed to create PR")?;

    if !output.status.success() {
        let error = String::from_utf8_lossy(&output.stderr);

        // PR already exists
        if error.contains("already exists") || error.contains("already a pull request") {
            println!("PR already exists for branch '{}'", branch_name);

            // Get existing PR URL
            let url_output = Command::new("gh")
                .current_dir(&path)
                .args([
                    "pr",
                    "view",
                    "--json",
                    "url",
                    "--jq",
                    ".url",
                    "--head",
                    branch_name,
                ])
                .output()
                .context("Failed to get existing PR URL")?;

            if url_output.status.success() {
                let url = String::from_utf8_lossy(&url_output.stdout)
                    .trim()
                    .to_string();
                println!("Existing PR URL: {}", url);
                return Ok(url);
            }

            return Ok(String::from("existing-pr-url-not-found"));
        }

        anyhow::bail!("Failed to create PR: {}", error);
    }

    // Get PR URL
    let url_output = String::from_utf8_lossy(&output.stdout).trim().to_string();
    println!("PR created: {}", url_output);

    Ok(url_output)
}

/// Check PR status
pub fn check_pr_status(repo_path: &str, branch_name: &str) -> Result<String> {
    let path = expand_path(repo_path)?;

    // Check if GitHub CLI is installed
    if !check_gh_cli()? {
        anyhow::bail!("GitHub CLI is not installed or not authenticated");
    }

    // Check PR status
    let output = Command::new("gh")
        .current_dir(&path)
        .args([
            "pr",
            "view",
            "--json",
            "state",
            "--jq",
            ".state",
            "--head",
            branch_name,
        ])
        .output()
        .context("Failed to check PR status")?;

    if !output.status.success() {
        // PR does not exist
        return Ok(String::from("NO_PR"));
    }

    let status = String::from_utf8_lossy(&output.stdout).trim().to_string();
    Ok(status)
}

/// Get PR list
pub fn list_prs(repo_path: &str, state: &str) -> Result<Vec<(String, String, String)>> {
    let path = expand_path(repo_path)?;

    // Check if GitHub CLI is installed
    if !check_gh_cli()? {
        anyhow::bail!("GitHub CLI is not installed or not authenticated");
    }

    // Get PR list
    let output = Command::new("gh")
        .current_dir(&path)
        .args([
            "pr",
            "list",
            "--json",
            "title,headRefName,url",
            "--state",
            state,
        ])
        .output()
        .context("Failed to list PRs")?;

    if !output.status.success() {
        let error = String::from_utf8_lossy(&output.stderr);
        anyhow::bail!("Failed to list PRs: {}", error);
    }

    let json_output = String::from_utf8_lossy(&output.stdout);
    let prs: Vec<serde_json::Value> =
        serde_json::from_str(&json_output).context("Failed to parse PR list JSON")?;

    let mut result = Vec::new();
    for pr in prs {
        let title = pr["title"].as_str().unwrap_or("").to_string();
        let branch = pr["headRefName"].as_str().unwrap_or("").to_string();
        let url = pr["url"].as_str().unwrap_or("").to_string();

        result.push((title, branch, url));
    }

    Ok(result)
}

/// Merge PR
pub fn merge_pr(repo_path: &str, branch_name: &str, merge_method: &str) -> Result<bool> {
    let path = expand_path(repo_path)?;

    // Check if GitHub CLI is installed
    if !check_gh_cli()? {
        anyhow::bail!("GitHub CLI is not installed or not authenticated");
    }

    println!("Merging PR for branch '{}'", branch_name);

    // Merge PR
    let output = Command::new("gh")
        .current_dir(&path)
        .args(["pr", "merge", "--head", branch_name, "--", merge_method])
        .output()
        .context("Failed to merge PR")?;

    if !output.status.success() {
        let error = String::from_utf8_lossy(&output.stderr);

        // PR already merged
        if error.contains("already merged") {
            println!("PR for branch '{}' is already merged", branch_name);
            return Ok(true);
        }

        anyhow::bail!("Failed to merge PR: {}", error);
    }

    println!("PR merged successfully");
    Ok(true)
}

/// Fork repository
pub fn fork_repository(github_url: &str, output_dir: &str) -> Result<String> {
    // Check if GitHub CLI is installed
    if !check_gh_cli()? {
        anyhow::bail!("GitHub CLI is not installed or not authenticated");
    }

    println!("Forking repository: {}", github_url);

    // Fork repository and clone
    let output = Command::new("gh")
        .args(["repo", "fork", github_url, "--clone", "--dir", output_dir])
        .output()
        .context("Failed to fork repository")?;

    if !output.status.success() {
        let error = String::from_utf8_lossy(&output.stderr);
        anyhow::bail!("Failed to fork repository: {}", error);
    }

    // Get forked repository URL
    let path = expand_path(output_dir)?;
    let url_output = Command::new("git")
        .current_dir(&path)
        .args(["remote", "get-url", "origin"])
        .output()
        .context("Failed to get forked repository URL")?;

    if !url_output.status.success() {
        anyhow::bail!("Failed to get forked repository URL");
    }

    let forked_url = String::from_utf8_lossy(&url_output.stdout)
        .trim()
        .to_string();
    println!("Repository forked: {}", forked_url);

    Ok(forked_url)
}

/// Clone repository
pub fn clone_repository(github_url: &str, output_dir: &str) -> Result<()> {
    println!("Cloning repository: {}", github_url);

    let output = Command::new("git")
        .args(["clone", github_url, output_dir])
        .output()
        .context("Failed to clone repository")?;

    if !output.status.success() {
        let error = String::from_utf8_lossy(&output.stderr);
        anyhow::bail!("Failed to clone repository: {}", error);
    }

    println!("Repository cloned to: {}", output_dir);
    Ok(())
}
