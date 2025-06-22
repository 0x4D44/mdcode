# mdcode

**mdcode** is a simple code management tool built on top of Git, written in Rust. It provides a clean command-line interface for creating new repositories, updating commits, viewing repository history, performing diffs, and integrating seamlessly with GitHub. With built-in support for excluding build artifacts and virtual environments, mdcode helps keep your repository clean and focused on your source code.

## Table of Contents
- [Features](#features)
- [Installation](#installation)
- [Usage](#usage)
  - [Local Workflow](#local-workflow)
  - [GitHub Integration](#github-integration)
- [Commands](#commands)
- [Configuration](#configuration)
- [Contributing](#contributing)
- [License](#license)

## Features
- **Repository Initialization:** Quickly create new local Git repositories with an initial commit.
- **Automated Updates:** Stage and commit changes with a single command.
- **Repository Info:** Display the latest commits and file change summaries.
- **Diffing:** Compare different versions of your code using external diff tools.
- **GitHub Integration:** Automatically create a GitHub repository, add it as a remote, and push your local branch.
- **Exclusion of Unwanted Files:** Automatically ignores directories like `target`, `bin`, `obj`, `venv`, `.venv`, and `env` so that build artifacts and virtual environments do not clutter your repository.
- **Multi-Language Support:** Recognizes various file types (Rust, Python, C/C++, Java, etc.) to accurately identify source files.

## Installation

### Prerequisites
- **Rust:** Install Rust from [rust-lang.org](https://www.rust-lang.org/tools/install).
- **Git:** Ensure Git is installed. Download from [git-scm.com](https://git-scm.com/downloads).
- **GitHub Personal Access Token:** For GitHub integration, create a personal access token with the necessary permissions (typically the `repo` scope).  
  - On Windows:  
    ```batch
    set GITHUB_TOKEN=your_token_here
    ```  
  - On Unix/Linux:  
    ```bash
    export GITHUB_TOKEN=your_token_here
    ```

### Build from Source
Clone the repository and build it using Cargo:

```bash
git clone https://github.com/yourusername/mdcode.git
cd mdcode
cargo build --release
