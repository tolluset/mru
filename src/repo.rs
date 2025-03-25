use anyhow::{Context, Result};
use std::path::{Path, PathBuf};
use std::process::Command;

use crate::config::Repository;

/// 물결표(~)를 포함한 경로를 절대 경로로 확장
pub fn expand_path(path: &str) -> Result<PathBuf> {
    if path.starts_with("~/") {
        let home =
            dirs::home_dir().ok_or_else(|| anyhow::anyhow!("Could not find home directory"))?;

        // ~/ 부분을 제거하고 홈 디렉토리와 결합
        let path_without_tilde = &path[2..];
        Ok(home.join(path_without_tilde))
    } else {
        Ok(PathBuf::from(path))
    }
}

/// 레포지토리 상태 확인
pub fn check_repository(repo: &Repository) -> Result<bool> {
    let path = Path::new(&repo.path);

    // 경로가 존재하는지 확인
    if !path.exists() {
        anyhow::bail!("Repository path does not exist: {}", repo.path);
    }

    // .git 디렉토리가 있는지 확인
    let git_dir = path.join(".git");
    if !git_dir.exists() {
        anyhow::bail!("Not a git repository: {}", repo.path);
    }

    // git status 실행하여 변경사항 확인
    let output = Command::new("git")
        .current_dir(&repo.path)
        .args(["status", "--porcelain"])
        .output()
        .context("Failed to execute git status")?;

    if !output.status.success() {
        anyhow::bail!("Failed to get git status for repository: {}", repo.path);
    }

    // 변경사항이 있는지 확인 (출력이 비어있지 않으면 변경사항 있음)
    let has_changes = !output.stdout.is_empty();

    Ok(has_changes)
}

/// 현재 브랜치 이름 가져오기
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

/// 브랜치 생성
pub fn create_branch(repo_path: &str, branch_name: &str, dry_run: bool) -> Result<()> {
    if dry_run {
        println!("Would create branch '{}' in {}", branch_name, repo_path);
        return Ok(());
    }

    println!("Creating branch '{}' in {}", branch_name, repo_path);

    // 기존 브랜치 저장
    let original_branch = get_current_branch(repo_path)?;

    // 브랜치가 이미 존재하는지 확인
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
        // 브랜치가 이미 존재하면 체크아웃
        let status = Command::new("git")
            .current_dir(repo_path)
            .args(["checkout", branch_name])
            .status()
            .context("Failed to checkout existing branch")?;

        if !status.success() {
            anyhow::bail!("Failed to checkout existing branch: {}", branch_name);
        }
    } else {
        // 새 브랜치 생성
        let status = Command::new("git")
            .current_dir(repo_path)
            .args(["checkout", "-b", branch_name])
            .status()
            .context("Failed to create new branch")?;

        if !status.success() {
            // 실패 시 원래 브랜치로 돌아가기
            let _ = Command::new("git")
                .current_dir(repo_path)
                .args(["checkout", &original_branch])
                .status();

            anyhow::bail!("Failed to create branch: {}", branch_name);
        }
    }

    Ok(())
}

/// 원래 브랜치로 돌아가기
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

/// 레포지토리 풀
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
