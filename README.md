# Multi-Repository Updater (MRU)

**Effortlessly update packages across multiple repositories with a single command! 🚀**

MRU is a command-line tool built in Rust that helps you manage package dependencies across multiple repositories. It's perfect for teams working with microservices or multiple frontend applications that share common dependencies.

## Features

- ✨ **Update packages across repositories** - Update a package to the same version across all your JavaScript/TypeScript repositories
- 🔄 **Automatic Git workflow** - Creates branches, commits changes, and pushes to GitHub
- 🤖 **Pull Request automation** - Automatically creates PRs for your updates
- 📊 **Compare package versions** - See which repositories are using which versions
- 📦 **Multiple package managers** - Supports npm, yarn, and pnpm
- 🏠 **Tilde path support** - Use `~` in your repository paths for convenience

## Installation

### From Source

```bash
# Clone the repository
git clone https://github.com/tolluset/mru.git
cd mru

# Build the project
cargo build --release

# Optional: Add to your PATH
cp target/release/mru /usr/local/bin/
```

### Using Cargo

```bash
cargo install mru
```

## Quick Start

1. **Add repositories to your config**

```bash
mru add-repo ~/projects/my-app https://github.com/example/my-app
mru add-repo ~/projects/my-api https://github.com/example/my-api
```

2. **Update a package across all repositories**

```bash
mru update react "^18.2.0"
```

3. **Create Pull Requests automatically**

```bash
mru update lodash "^4.17.21" --pull-request
```

## Usage

### Managing Repositories

- **Add a repository**

```bash
mru add-repo <LOCAL_PATH> <GITHUB_URL>
```

- **Remove a repository**

```bash
mru remove-repo <LOCAL_PATH>
```

- **List all repositories**

```bash
mru list-repos
```

- **Clone a repository and add it to config**

```bash
mru clone https://github.com/example/my-repo --add
```

### Package Management

- **Update a package**

```bash
mru update <PACKAGE_NAME> <VERSION> [OPTIONS]

Options:

--message, -m: Custom commit message
--pull-request, -p: Create a pull request
--dry-run, -d: Show what would happen without making changes
```

- **Set default package manager**

```bash
mru set-package-manager <PACKAGE_MANAGER>

# Example
mru set-package-manager pnpm
```

MRU determines the package manager in the following order:
1. Check repository's lock files (pnpm-lock.yaml, yarn.lock, package-lock.json)
2. Check configured default package manager
3. Use npm if neither is found

- **Compare package versions**

```bash
mru compare <PACKAGE_NAME>
```

- **List all packages in repositories**

```bash
mru list-packages
```

- **List packages in a specific repository**

```bash
mru list-packages --repo ~/projects/my-app
```

## Configuration

MRU stores its configuration in ~/.config/mru/config.toml. You can edit this file directly if needed, but it's recommended to use the CLI commands.

Example configuration:

```toml
default_commit_message = "chore: update dependencies"

[[repositories]]
path = "~/projects/my-app"
github_url = "https://github.com/example/my-app"

[[repositories]]
path = "/absolute/path/to/my-api"
github_url = "https://github.com/example/my-api"
```

## Requirements

- Rust 1.56 or later
- Git
- GitHub CLI (for PR creation)
- npm, yarn, or pnpm (depending on your projects)

## Examples

- **Update React across all repositories**

```bash
mru update react "^18.2.0" --message "chore: update React to v18.2.0" --pull-request
```

- **Dry run to see what would change**

```bash
mru update typescript "~5.0.4" --dry-run
```

- **Compare lodash versions**

```bash
mru compare lodash

Output:

Comparing package 'lodash' across repositories:
/home/user/projects/my-app: ^4.17.20
/home/user/projects/my-api: ^4.17.21
/home/user/projects/my-ui-lib: Not found
```

## License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.
