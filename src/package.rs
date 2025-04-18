use anyhow::{Context, Result};
use serde_json::{json, Value, Map};
use std::fs;
use std::path::Path;
use std::process::Command;

use crate::repo::expand_path;

/// Update specific package version in package.json
pub fn update_package(
    repo_path: &str,
    package_name: &str,
    version: &str,
    dry_run: bool,
) -> Result<bool> {
    let path = expand_path(repo_path)?;
    let package_json_path = path.join("package.json");

    if !package_json_path.exists() {
        anyhow::bail!("package.json not found in repository: {}", repo_path);
    }

    let content = fs::read_to_string(&package_json_path).context("Failed to read package.json")?;
    let mut package_json: Value = serde_json::from_str(&content).context("Failed to parse package.json")?;
    let mut updated = false;

    // Update dependencies
    if let Some(deps) = package_json.get_mut("dependencies") {
        if let Some(pkg) = deps.get_mut(package_name) {
            if pkg.as_str().unwrap_or("") != version {
                if !dry_run {
                    *pkg = json!(version);
                }
                updated = true;
                println!(
                    "Updated {} in dependencies from {} to {}",
                    package_name,
                    pkg.as_str().unwrap_or("unknown"),
                    version
                );
            }
        }
    }

    // Update devDependencies
    if let Some(dev_deps) = package_json.get_mut("devDependencies") {
        if let Some(pkg) = dev_deps.get_mut(package_name) {
            if pkg.as_str().unwrap_or("") != version {
                if !dry_run {
                    *pkg = json!(version);
                }
                updated = true;
                println!(
                    "Updated {} in devDependencies from {} to {}",
                    package_name,
                    pkg.as_str().unwrap_or("unknown"),
                    version
                );
            }
        }
    }

    // Update peerDependencies
    if let Some(peer_deps) = package_json.get_mut("peerDependencies") {
        if let Some(pkg) = peer_deps.get_mut(package_name) {
            if pkg.as_str().unwrap_or("") != version {
                if !dry_run {
                    *pkg = json!(version);
                }
                updated = true;
                println!(
                    "Updated {} in peerDependencies from {} to {}",
                    package_name,
                    pkg.as_str().unwrap_or("unknown"),
                    version
                );
            }
        }
    }

    if updated && !dry_run {
        let formatted = serde_json::to_string_pretty(&package_json)?;
        fs::write(package_json_path, formatted)?;
        println!("Saved changes to package.json in {}", repo_path);
    } else if !updated {
        println!(
            "Package '{}' is already at version '{}' or not found",
            package_name, version
        );
    }

    Ok(updated)
}

/// Detect package manager (pnpm, yarn, npm)
pub fn detect_package_manager(repo_path: &str) -> Result<String> {
    let path = expand_path(repo_path)?;

    // Check for pnpm-lock.yaml
    if path.join("pnpm-lock.yaml").exists() {
        return Ok("pnpm".to_string());
    }

    // Check for yarn.lock
    if path.join("yarn.lock").exists() {
        return Ok("yarn".to_string());
    }

    // Check for package-lock.json
    if path.join("package-lock.json").exists() {
        return Ok("npm".to_string());
    }

    // No lock file found
    anyhow::bail!("No package manager lock file found")
}

/// Run package install with specified package manager
pub fn run_install_with_manager(repo_path: &str, pkg_manager: &str, dry_run: bool) -> Result<()> {
    let path = expand_path(repo_path)?;

    if dry_run {
        println!("Would run {} install in {}", pkg_manager, repo_path);
        return Ok(());
    }

    println!("Running {} install in {}", pkg_manager, repo_path);

    let status = Command::new(pkg_manager)
        .current_dir(&path)
        .arg("install")
        .status()
        .context(format!("Failed to run {} install", pkg_manager))?;

    if !status.success() {
        anyhow::bail!("{} install failed", pkg_manager);
    }

    Ok(())
}

/// Check package version
pub fn get_package_version(repo_path: &str, package_name: &str) -> Result<Option<String>> {
    let path = expand_path(repo_path)?;
    let package_json_path = path.join("package.json");

    if !package_json_path.exists() {
        anyhow::bail!("package.json not found in repository: {}", repo_path);
    }

    let content = fs::read_to_string(&package_json_path).context("Failed to read package.json")?;

    let package_json: Value =
        serde_json::from_str(&content).context("Failed to parse package.json")?;

    // dependencies 확인
    if let Some(deps) = package_json.get("dependencies") {
        if let Some(version) = deps.get(package_name) {
            if let Some(version_str) = version.as_str() {
                return Ok(Some(version_str.to_string()));
            }
        }
    }

    // devDependencies 확인
    if let Some(dev_deps) = package_json.get("devDependencies") {
        if let Some(version) = dev_deps.get(package_name) {
            if let Some(version_str) = version.as_str() {
                return Ok(Some(version_str.to_string()));
            }
        }
    }

    // peerDependencies 확인
    if let Some(peer_deps) = package_json.get("peerDependencies") {
        if let Some(version) = peer_deps.get(package_name) {
            if let Some(version_str) = peer_deps.get(package_name).and_then(|v| v.as_str()) {
                return Ok(Some(version_str.to_string()));
            }
        }
    }

    // 패키지를 찾지 못함
    Ok(None)
}

/// Get all package list
pub fn list_all_packages(repo_path: &str) -> Result<Vec<(String, String, String)>> {
    let path = expand_path(repo_path)?;
    let package_json_path = path.join("package.json");

    if !package_json_path.exists() {
        anyhow::bail!("package.json not found in repository: {}", repo_path);
    }

    let content = fs::read_to_string(&package_json_path).context("Failed to read package.json")?;

    let package_json: Value =
        serde_json::from_str(&content).context("Failed to parse package.json")?;

    let mut packages = Vec::new();

    // dependencies 수집
    if let Some(deps) = package_json.get("dependencies").and_then(|d| d.as_object()) {
        for (name, version) in deps {
            if let Some(version_str) = version.as_str() {
                packages.push((
                    name.clone(),
                    version_str.to_string(),
                    "dependencies".to_string(),
                ));
            }
        }
    }

    // devDependencies 수집
    if let Some(dev_deps) = package_json
        .get("devDependencies")
        .and_then(|d| d.as_object())
    {
        for (name, version) in dev_deps {
            if let Some(version_str) = version.as_str() {
                packages.push((
                    name.clone(),
                    version_str.to_string(),
                    "devDependencies".to_string(),
                ));
            }
        }
    }

    // peerDependencies 수집
    if let Some(peer_deps) = package_json
        .get("peerDependencies")
        .and_then(|d| d.as_object())
    {
        for (name, version) in peer_deps {
            if let Some(version_str) = version.as_str() {
                packages.push((
                    name.clone(),
                    version_str.to_string(),
                    "peerDependencies".to_string(),
                ));
            }
        }
    }

    Ok(packages)
}

/// Compare package versions across multiple repositories
pub fn compare_package_versions(
    repos: &[&str],
    package_name: &str,
) -> Result<Vec<(String, Option<String>)>> {
    let mut results = Vec::new();

    for &repo_path in repos {
        let version = get_package_version(repo_path, package_name)?;
        results.push((repo_path.to_string(), version));
    }

    Ok(results)
}
