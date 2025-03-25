use anyhow::{Context, Result};
use std::path::PathBuf;
use std::process::Command;

use crate::repo::expand_path;

/// GitHub CLI가 설치되어 있고 인증되어 있는지 확인
pub fn check_gh_cli() -> Result<bool> {
    let output = Command::new("gh")
        .args(["auth", "status"])
        .output()
        .context("Failed to check GitHub CLI authentication. Is GitHub CLI installed?")?;

    Ok(output.status.success())
}

/// Pull Request 생성
pub fn create_pr(
    repo_path: &str,
    github_url: &str,
    branch_name: &str,
    title: &str,
    dry_run: bool,
) -> Result<String> {
    let path = expand_path(repo_path)?;

    if dry_run {
        println!(
            "Would create PR for branch '{}' with title: '{}'",
            branch_name, title
        );
        return Ok(String::from("dry-run-pr-url"));
    }

    // GitHub CLI가 설치되어 있는지 확인
    if !check_gh_cli()? {
        anyhow::bail!(
            "GitHub CLI is not installed or not authenticated. Please run 'gh auth login'"
        );
    }

    println!(
        "Creating PR for branch '{}' with title: '{}'",
        branch_name, title
    );

    // PR 생성
    let output = Command::new("gh")
        .current_dir(&path)
        .args([
            "pr",
            "create",
            "--title",
            title,
            "--body",
            &format!("Automated dependency update for {}", branch_name),
            "--head",
            branch_name,
        ])
        .output()
        .context("Failed to create PR")?;

    if !output.status.success() {
        let error = String::from_utf8_lossy(&output.stderr);

        // PR이 이미 존재하는 경우 (에러 메시지에서 확인)
        if error.contains("already exists") || error.contains("already a pull request") {
            println!("PR already exists for branch '{}'", branch_name);

            // 기존 PR URL 가져오기
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

    // PR URL 가져오기
    let url_output = String::from_utf8_lossy(&output.stdout).trim().to_string();
    println!("PR created: {}", url_output);

    Ok(url_output)
}

/// PR 상태 확인
pub fn check_pr_status(repo_path: &str, branch_name: &str) -> Result<String> {
    let path = expand_path(repo_path)?;

    // GitHub CLI가 설치되어 있는지 확인
    if !check_gh_cli()? {
        anyhow::bail!("GitHub CLI is not installed or not authenticated");
    }

    // PR 상태 확인
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
        // PR이 없는 경우
        return Ok(String::from("NO_PR"));
    }

    let status = String::from_utf8_lossy(&output.stdout).trim().to_string();
    Ok(status)
}

/// PR 목록 가져오기
pub fn list_prs(repo_path: &str, state: &str) -> Result<Vec<(String, String, String)>> {
    let path = expand_path(repo_path)?;

    // GitHub CLI가 설치되어 있는지 확인
    if !check_gh_cli()? {
        anyhow::bail!("GitHub CLI is not installed or not authenticated");
    }

    // PR 목록 가져오기
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

/// PR 병합
pub fn merge_pr(repo_path: &str, branch_name: &str, merge_method: &str) -> Result<bool> {
    let path = expand_path(repo_path)?;

    // GitHub CLI가 설치되어 있는지 확인
    if !check_gh_cli()? {
        anyhow::bail!("GitHub CLI is not installed or not authenticated");
    }

    println!("Merging PR for branch '{}'", branch_name);

    // PR 병합
    let output = Command::new("gh")
        .current_dir(&path)
        .args(["pr", "merge", "--head", branch_name, "--", merge_method])
        .output()
        .context("Failed to merge PR")?;

    if !output.status.success() {
        let error = String::from_utf8_lossy(&output.stderr);

        // PR이 이미 병합된 경우
        if error.contains("already merged") {
            println!("PR for branch '{}' is already merged", branch_name);
            return Ok(true);
        }

        anyhow::bail!("Failed to merge PR: {}", error);
    }

    println!("PR merged successfully");
    Ok(true)
}

/// 레포지토리 포크
pub fn fork_repository(github_url: &str, output_dir: &str) -> Result<String> {
    // GitHub CLI가 설치되어 있는지 확인
    if !check_gh_cli()? {
        anyhow::bail!("GitHub CLI is not installed or not authenticated");
    }

    println!("Forking repository: {}", github_url);

    // 레포지토리 포크 및 클론
    let output = Command::new("gh")
        .args(["repo", "fork", github_url, "--clone", "--dir", output_dir])
        .output()
        .context("Failed to fork repository")?;

    if !output.status.success() {
        let error = String::from_utf8_lossy(&output.stderr);
        anyhow::bail!("Failed to fork repository: {}", error);
    }

    // 포크된 레포지토리 URL 가져오기
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

/// 레포지토리 클론
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
