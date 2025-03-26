mod cli;
mod config;
mod git;
mod github;
mod package;
mod repo;

use anyhow::Result;
use clap::Parser;

fn main() -> Result<()> {
    let cli = cli::Cli::parse();
    let mut config = config::Config::load()?;

    match &cli.command {
        cli::Commands::Update {
            package,
            version,
            message,
            pull_request,
            dry_run,
        } => {
            cli::handle_update(
                &config,
                package,
                version,
                message.as_deref(),
                *pull_request,
                *dry_run,
            )?;
        }

        cli::Commands::AddRepo { path } => {
            cli::handle_add_repo(&mut config, path)?;
        }

        cli::Commands::RemoveRepo { path } => {
            cli::handle_remove_repo(&mut config, path)?;
        }

        cli::Commands::ListRepos => {
            cli::handle_list_repos(&config)?;
        }

        cli::Commands::Compare { package } => {
            cli::handle_compare(&config, package)?;
        }

        cli::Commands::ListPackages { repo } => {
            cli::handle_list_packages(&config, repo.as_deref())?;
        }

        cli::Commands::Clone {
            github_url,
            output,
            add,
        } => {
            cli::handle_clone(&mut config, github_url, output.as_deref(), *add)?;
        }

        cli::Commands::SetPackageManager { name } => {
            cli::handle_set_package_manager(&mut config, name)?;
        }
    }

    Ok(())
}
