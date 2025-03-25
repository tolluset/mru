use anyhow::{Context, Result};
use std::path::PathBuf;
use std::process::Command;

use crate::config::Repository;
use crate::repo::expand_path;

/// 현재 브랜치 이름 가져오기
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

/// 브랜치 생성
pub fn create_branch(repo_path: &str, branch_name: &str, dry_run: bool) -> Result<String> {
    let path = expand_path(repo_path)?;

    // 현재 브랜치 저장 (나중에 실패 시 복원용)
    let original_branch = get_current_branch(repo_path)?;

    if dry_run {
        println!("Would create branch '{}' in {}", branch_name, repo_path);
        return Ok(original_branch);
    }

    println!("Creating branch '{}' in {}", branch_name, repo_path);

    // 브랜치가 이미 존재하는지 확인
    let output = Command::new("git")
        .current_dir(&path)
        .args(["branch", "--list", branch_name])
        .output()
        .context("Failed to list branches")?;

    let branch_exists = !output.stdout.is_empty();

    if branch_exists {
        // 브랜치가 이미 존재하면 체크아웃
        let status = Command::new("git")
            .current_dir(&path)
            .args(["checkout", branch_name])
            .status()
            .context("Failed to checkout existing branch")?;

        if !status.success() {
            anyhow::bail!("Failed to checkout existing branch: {}", branch_name);
        }
    } else {
        // 새 브랜치 생성
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

/// 변경사항 스테이징
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

/// 변경사항 커밋
pub fn commit_changes(repo_path: &str, message: &str, dry_run: bool) -> Result<()> {
    let path = expand_path(repo_path)?;

    if dry_run {
        println!("Would commit changes with message: '{}'", message);
        return Ok(());
    }

    println!("Committing changes with message: '{}'", message);

    // 스테이징된 변경사항이 있는지 확인
    let output = Command::new("git")
        .current_dir(&path)
        .args(["diff", "--staged", "--name-only"])
        .output()
        .context("Failed to check staged changes")?;

    if output.stdout.is_empty() {
        println!("No staged changes to commit");
        return Ok(());
    }

    // 커밋 실행
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

/// 브랜치 푸시
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

/// 원래 브랜치로 돌아가기
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

/// 레포지토리 상태 확인
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

    // 변경사항이 있는지 확인 (출력이 비어있지 않으면 변경사항 있음)
    let has_changes = !output.stdout.is_empty();

    Ok(has_changes)
}

/// 레포지토리 풀
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

/// 패키지 업데이트 작업 수행
pub fn update_package_workflow(
    repo: &Repository,
    package_name: &str,
    version: &str,
    commit_message: &str,
    create_pr: bool,
    dry_run: bool,
) -> Result<()> {
    println!("\n=== Processing repository: {} ===", repo.path);

    // 1. 현재 브랜치 저장
    let original_branch = get_current_branch(&repo.path)?;

    // 2. 브랜치 생성
    let branch_name = format!(
        "update-{}-{}",
        package_name,
        version.replace("^", "").replace("~", "")
    );
    create_branch(&repo.path, &branch_name, dry_run)?;

    // 3. package.json 업데이트 (이 함수는 package.rs에 있음)
    let updated = crate::package::update_package(&repo.path, package_name, version, dry_run)?;

    if !updated {
        println!(
            "Package '{}' is already at version '{}', skipping",
            package_name, version
        );
        // 원래 브랜치로 돌아가기
        checkout_branch(&repo.path, &original_branch, dry_run)?;
        return Ok(());
    }

    // 4. pnpm install 실행 (이 함수는 package.rs에 있음)
    crate::package::run_install(&repo.path, dry_run)?;

    // 5. 변경사항 스테이징
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

    // 6. 변경사항 커밋
    commit_changes(&repo.path, commit_message, dry_run)?;

    // 7. GitHub에 푸시
    push_branch(&repo.path, &branch_name, dry_run)?;

    // 8. PR 생성 (선택적) - 이 함수는 github.rs에 구현 예정
    // if create_pr {
    //     if let Err(e) = crate::github::create_pr(
    //         &repo.path,
    //         &repo.github_url,
    //         &branch_name,
    //         commit_message,
    //         dry_run,
    //     ) {
    //         eprintln!("Warning: Failed to create PR: {}", e);
    //     }
    // }

    println!(
        "✅ Successfully updated {} to {} in {}",
        package_name, version, repo.path
    );

    // 9. 원래 브랜치로 돌아가기
    checkout_branch(&repo.path, &original_branch, dry_run)?;

    Ok(())
}
