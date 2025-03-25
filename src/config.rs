use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

#[derive(Debug, Serialize, Deserialize)]
pub struct Config {
    pub default_commit_message: String,
    pub repositories: Vec<Repository>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Repository {
    pub path: String,
    pub github_url: String,
}

impl Config {
    pub fn load() -> Result<Self> {
        let config_path = get_config_path()?;
        let config_dir = config_path.parent().unwrap();

        if !config_path.exists() {
            fs::create_dir_all(config_dir)?;
            let default_config = Config {
                default_commit_message: "chore: update dependencies".to_string(),
                repositories: Vec::new(),
            };
            let toml = toml::to_string(&default_config)?;
            fs::write(&config_path, toml)?;
            return Ok(default_config);
        }

        let content = fs::read_to_string(&config_path).context("Failed to read config file")?;
        let config: Config = toml::from_str(&content).context("Failed to parse config file")?;

        let mut expanded_repos = Vec::new();
        for repo in &config.repositories {
            let expanded_path = expand_tilde(&repo.path)?;
            expanded_repos.push(Repository {
                path: expanded_path,
                github_url: repo.github_url.clone(),
            });
        }

        Ok(Config {
            default_commit_message: config.default_commit_message,
            repositories: expanded_repos,
        })
    }

    pub fn save(&self) -> Result<()> {
        let config_path = get_config_path()?;
        let config_dir = config_path.parent().unwrap();

        fs::create_dir_all(config_dir)?;

        let toml = toml::to_string(self)?;
        fs::write(&config_path, toml)?;

        Ok(())
    }

    pub fn add_repository(&mut self, path: String, github_url: String) -> Result<()> {
        // 중복 체크 (물결표 확장 후)
        let expanded_path = expand_tilde(&path)?;

        for repo in &self.repositories {
            let repo_expanded_path = expand_tilde(&repo.path)?;
            if repo_expanded_path == expanded_path || repo.github_url == github_url {
                anyhow::bail!("Repository already exists in config");
            }
        }

        // 원래 경로 저장 (물결표 유지)
        self.repositories.push(Repository { path, github_url });
        self.save()?;

        Ok(())
    }

    pub fn remove_repository(&mut self, path: &str) -> Result<()> {
        let expanded_path = expand_tilde(path)?;
        let initial_len = self.repositories.len();

        // 확장된 경로로 비교하여 제거
        let mut i = 0;
        while i < self.repositories.len() {
            let repo_expanded_path = expand_tilde(&self.repositories[i].path)?;
            if repo_expanded_path == expanded_path {
                self.repositories.remove(i);
            } else {
                i += 1;
            }
        }

        if self.repositories.len() == initial_len {
            anyhow::bail!("Repository not found: {}", path);
        }

        self.save()?;
        Ok(())
    }
}

pub fn get_config_path() -> Result<PathBuf> {
    // 홈 디렉토리 가져오기
    let home = dirs::home_dir().ok_or_else(|| anyhow::anyhow!("Could not find home directory"))?;

    // ~/.config/mru/config.toml 경로 생성
    let config_path = home.join(".config").join("mru").join("config.toml");

    Ok(config_path)
}

pub fn expand_tilde(path: &str) -> Result<String> {
    if path.starts_with("~/") {
        let home =
            dirs::home_dir().ok_or_else(|| anyhow::anyhow!("Could not find home directory"))?;

        // ~/ 부분을 제거하고 홈 디렉토리와 결합
        let path_without_tilde = &path[2..];
        Ok(home.join(path_without_tilde).to_string_lossy().to_string())
    } else {
        Ok(path.to_string())
    }
}
