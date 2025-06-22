/*
This file implements "mdcode", a simple code management tool built on top of Git.
It provides a command-line interface with commands for creating new repositories,
updating repositories (staging changes and committing), displaying repository information,
diffing repository versions, and integrating with GitHub (creating a repo and pushing changes).

Key features and structure:
- Uses Clap for parsing command-line arguments.
- Leverages git2 for Git repository operations (initial commit, diffing, etc.).
- Scans directories for source files using WalkDir while filtering out build artifact directories 
  ("target", "bin", "obj", "venv", ".venv", "env") that are commonly generated in various build and virtual environment setups.
- Provides utility functions for generating a .gitignore file, detecting file types, and checking out Git trees.
- Uses colored logging to provide clear output to the user.
- Integrates with GitHub using octocrab for API calls.
*/

use clap::{Parser, Subcommand};
use git2::{Delta, Repository, Signature, ErrorCode, ObjectType, Sort};
use chrono::{Utc, TimeZone, LocalResult};
use std::env;
use std::error::Error;
use std::fs;
use std::fs::File;
use std::io::{self, Write};
use std::path::{Path, PathBuf};
use std::process::Command;
use walkdir::WalkDir;
use octocrab;
use tokio::runtime::Runtime;

// Define our uniform color constants.
const BLUE: &str = "\x1b[94m";    // Light blue
const GREEN: &str = "\x1b[32m";     // Green
const RED: &str = "\x1b[31m";       // Red
const YELLOW: &str = "\x1b[93m";    // Light yellow
const RESET: &str = "\x1b[0m";

#[derive(Parser)]
#[command(
    name = "mdcode",
    version = "1.3.0",  // Updated version number
    about = "Martin's simple code management tool using Git.",
    arg_required_else_help = true,
    after_help = "\
Diff Modes:
  mdcode diff <directory>
    => Compare current working directory vs most recent commit.
  mdcode diff <directory> <n>
    => Compare current working directory vs commit selected by n (0 is most recent, 1 for next, etc.).
  mdcode diff <directory> <n> <m>
    => Compare commit selected by n (before) vs commit selected by m (after).
  mdcode diff <directory> H <n>
    => Compare GitHub HEAD (before) vs local commit selected by n (after).",
    help_template = "\
{bin} {version}
{about}

USAGE:
    {usage}

COMMANDS:
{subcommands}

OPTIONS:
{options}
"
)]
struct Cli {
    /// Command to run: new, update, info, diff, gh_create, or gh_push (short aliases shown)
    #[command(subcommand)]
    command: Commands,

    /// Perform a dry run (no changes will be made)
    #[arg(long)]
    dry_run: bool,
}

#[derive(Subcommand)]
enum Commands {
    #[command(
        visible_alias = "n",
        about = "Create a new repository with initial commit"
    )]
    New {
        /// Directory in which to create the repository
        directory: String,
    },
    #[command(
        visible_alias = "u",
        about = "Update an existing repository (stage changes and commit)"
    )]
    Update {
        /// Directory of the repository to update
        directory: String,
    },
    #[command(
        visible_alias = "i",
        about = "Display repository info (latest 20 commits)"
    )]
    Info {
        /// Directory of the repository to inspect
        directory: String,
    },
    #[command(
        visible_alias = "d",
        about = "Diff versions",
        long_about = "Diff versions.
Modes:
  mdcode diff <directory>
    => Compare current working directory vs most recent commit.
  mdcode diff <directory> <n>
    => Compare current working directory vs commit selected by n (0 is most recent, 1 for next, etc.).
  mdcode diff <directory> <n> <m>
    => Compare commit selected by n (before) vs commit selected by m (after).
  mdcode diff <directory> H <n>
    => Compare GitHub HEAD (before) vs local commit selected by n (after)."
    )]
    Diff {
        /// Directory of the repository to diff
        directory: String,
        /// Optional version numbers (0 is most recent; 1, 2, ... select older commits)
        #[arg(num_args = 0..=2)]
        versions: Vec<String>,
    },
    #[command(
        name = "gh_create",
        visible_alias = "g",
        about = "Create a GitHub repository from the local repository, add it as remote, and push current state"
    )]
    GhCreate {
        /// Directory of the local repository (e.g. '.' for current directory)
        directory: String,
        /// Optional description for the GitHub repository
        #[arg(short, long)]
        description: Option<String>,
    },
    #[command(
        name = "gh_push",
        visible_alias = "p",
        about = "Push changes to the GitHub remote"
    )]
    GhPush {
        /// Directory of the local repository
        directory: String,
        /// Name of the remote to push to (default: origin)
        #[arg(short, long, default_value = "origin")]
        remote: String,
    },
}

fn run() -> Result<(), Box<dyn Error>> {
    let cli = Cli::parse();

    match &cli.command {
        Commands::New { directory } => {
            log::info!("Creating new repository in '{}'", directory);
            new_repository(directory, cli.dry_run)?;
        }
        Commands::Update { directory } => {
            log::info!("Updating repository in '{}'", directory);
            // In interactive use, pass None to prompt the user.
            update_repository(directory, cli.dry_run, None)?;
        }
        Commands::Info { directory } => {
            log::info!("Displaying repository info for '{}'", directory);
            info_repository(directory)?;
        }
        Commands::Diff { directory, versions } => {
            log::info!("Diffing repository '{}' with versions {:?}", directory, versions);
            diff_command(directory, versions, cli.dry_run)?;
        }
        Commands::GhCreate { directory, description } => {
            log::info!("Creating GitHub repository from local directory '{}'", directory);
            // Deduce repository name from the provided directory.
            let repo_name = {
                let path = Path::new(directory);
                // If directory is ".", use current dir.
                let actual = if path == Path::new(".") {
                    env::current_dir()?
                } else {
                    path.to_path_buf()
                };
                actual.file_name()
                    .ok_or("Could not determine repository name from directory")?
                    .to_string_lossy()
                    .to_string()
            };

            let rt = Runtime::new()?;
            let created_repo = rt.block_on(gh_create(&repo_name, description.clone()))?;
            // Use the clone URL from the created repository.
            let remote_url = created_repo.clone_url
                .ok_or("GitHub repository did not return a clone URL")?;
            // Add the remote "origin" to the local repository.
            add_remote(directory, "origin", remote_url.as_str())?;
            // Automatically push the current branch.
            gh_push(directory, "origin")?;
        },
        Commands::GhPush { directory, remote } => {
            log::info!("Pushing local repository '{}' to remote '{}'", directory, remote);
            gh_push(directory, remote)?;
        },
    }
    Ok(())
}

fn main() {
    // Initialize env_logger with a custom format:
    // - For error-level logs, print "Error:" in light blue.
    env_logger::Builder::new()
        .format(|buf, record| {
            if record.level() == log::Level::Error {
                writeln!(buf, "{}Error:{} {}", BLUE, RESET, record.args())
            } else {
                writeln!(buf, "{}", record.args())
            }
        })
        .filter(None, log::LevelFilter::Info)
        .init();

    if let Err(e) = run() {
        eprintln!("{}Error:{} {}", BLUE, RESET, e);
        std::process::exit(1);
    }
}

/// Returns true if any component of the entry's path is an excluded directory.
///
/// The tool ignores common build and virtual environment folders: `target`,
/// `bin`, `obj`, `venv`, `.venv`, and `env`.
fn is_in_excluded_dir(entry: &walkdir::DirEntry) -> bool {
    entry.path().components().any(|comp| {
        comp.as_os_str() == "target" ||
        comp.as_os_str() == "bin" ||
        comp.as_os_str() == "obj" ||
        comp.as_os_str() == "venv" ||
        comp.as_os_str() == ".venv" ||
        comp.as_os_str() == "env"
    })
}

/// Create a new repository and make an initial commit.
fn new_repository(dir: &str, dry_run: bool) -> Result<(), Box<dyn Error>> {
    if !check_git_installed() {
        log::error!("Git is not installed. Please install Git from https://git-scm.com/downloads");
        return Err("Git not installed".into());
    }

    if Path::new(dir).exists() {
        if let Ok(repo) = Repository::open(dir) {
            if repo.head().is_ok() {
                log::error!("git repository already exists in directory '{}'", dir);
                return Err("git repository already exists".into());
            }
        }
    }

    let total_files = scan_total_files(dir)?;
    let (source_files, _source_count) = scan_source_files(dir)?;
    
    if !Path::new(dir).exists() {
        log::info!("Directory '{}' does not exist. Creating...", dir);
        if !dry_run {
            fs::create_dir_all(dir)?;
        }
    }
    if dry_run {
        log::info!("Dry run enabled - repository will not be created.");
    }

    let added_count = if dry_run {
        source_files.len()
    } else {
        let repo = Repository::init(dir)?;

        log::info!("Initializing Git repository...");
        create_gitignore(dir, false)?;
        let count = add_files_to_git(dir, &source_files, false)?;

        let mut index = repo.index()?;
        index.write()?;
        let tree_id = index.write_tree()?;
        let tree = repo.find_tree(tree_id)?;
        let signature = Signature::now("mdcode", "mdcode@example.com")?;
        repo.commit(Some("HEAD"), &signature, &signature, "Initial commit", &tree, &[])?;
        count
    };

    log::info!("{}New files added:{} {}",
        BLUE,
        RESET,
        source_files.iter()
            .map(|p| format!("{}{}{}", GREEN, p.to_string_lossy(), RESET))
            .collect::<Vec<String>>()
            .join(", ")
    );
    log::info!("{}Final result:{} {}{} source files added out of {} total files{}",
        BLUE,
        RESET,
        YELLOW,
        added_count,
        total_files,
        RESET
    );

    Ok(())
}

/// Update an existing repository by staging changes and creating a commit.
/// After staging, if commit_msg is None the user is prompted for a commit message (defaulting to "Updated files").
fn update_repository(dir: &str, dry_run: bool, commit_msg: Option<&str>) -> Result<(), Box<dyn Error>> {
    let repo = match Repository::open(dir) {
        Ok(r) => r,
        Err(_) => {
            log::error!("{}Error:{} No git repository in directory '{}'", BLUE, RESET, dir);
            return Err("No git repository".into());
        }
    };
    log::info!("Staging changes...");
    let (source_files, _) = scan_source_files(dir)?;
    let _ = add_files_to_git(dir, &source_files, dry_run)?;

    let mut index = repo.index()?;
    index.write()?;
    let new_tree_id = index.write_tree()?;
    let new_tree = repo.find_tree(new_tree_id)?;
    let parent_commit = get_last_commit(&repo)?;
    if new_tree_id == parent_commit.tree()?.id() {
        log::info!("No changes to commit.");
        return Ok(());
    }
    let parent_tree = parent_commit.tree()?;
    let diff = repo.diff_tree_to_tree(Some(&parent_tree), Some(&new_tree), None)?;
    let mut changed_files = Vec::new();
    diff.foreach(
        &mut |delta, _| {
            match delta.status() {
                Delta::Added => {
                    if let Some(path) = delta.new_file().path() {
                        changed_files.push(format!("{}{}{}", GREEN, path.to_string_lossy(), RESET));
                    }
                },
                Delta::Deleted => {
                    if let Some(path) = delta.old_file().path() {
                        changed_files.push(format!("{}{}{}", RED, path.to_string_lossy(), RESET));
                    }
                },
                _ => {
                    if let Some(path) = delta.new_file().path().or(delta.old_file().path()) {
                        changed_files.push(path.to_string_lossy().to_string());
                    }
                },
            }
            true
        },
        None,
        None,
        None,
    )?;
    log::info!("{}Changed:{} {}", BLUE, RESET, changed_files.join(", "));

    // Determine commit message.
    let final_message = if let Some(msg) = commit_msg {
        msg.to_string()
    } else {
        print!("Enter commit message [default: Updated files]: ");
        io::stdout().flush()?;
        let mut msg = String::new();
        io::stdin().read_line(&mut msg)?;
        if msg.trim().is_empty() {
            "Updated files".to_string()
        } else {
            msg.trim().to_string()
        }
    };
    log::info!("{}Creating commit:{} '{}'", BLUE, RESET, final_message);
    if !dry_run {
        let signature = Signature::now("mdcode", "mdcode@example.com")?;
        repo.commit(Some("HEAD"), &signature, &signature, &final_message, &new_tree, &[&parent_commit])?;
    }
    log::info!("{}{} changes staged and committed.{}", YELLOW, changed_files.len(), RESET);
    Ok(())
}

/// Scan the entire directory tree and count total files, skipping any entries under excluded directories.
fn scan_total_files(dir: &str) -> Result<usize, Box<dyn Error>> {
    log::debug!("Scanning source tree in '{}'...", dir);
    let mut total = 0;
    for entry in WalkDir::new(dir).into_iter().filter_map(Result::ok) {
        if is_in_excluded_dir(&entry) {
            continue;
        }
        if entry.file_type().is_file() {
            total += 1;
        }
    }
    log::debug!("Scan complete - found {} files", total);
    Ok(total)
}

/// Scan for source files (ignoring files under excluded directories).
fn scan_source_files(dir: &str) -> Result<(Vec<PathBuf>, usize), Box<dyn Error>> {
    log::debug!("Scanning for source files in '{}'...", dir);
    let mut source_files = Vec::new();
    let mut count = 0;
    for entry in WalkDir::new(dir).into_iter().filter_map(Result::ok) {
        if is_in_excluded_dir(&entry) {
            continue;
        }
        if entry.file_type().is_file() {
            if detect_file_type(entry.path()).is_some() {
                source_files.push(entry.path().to_path_buf());
                count += 1;
            }
        }
    }
    log::debug!("{} source files found", count);
    Ok((source_files, count))
}

/// Add the provided source files to the Git index.
fn add_files_to_git(dir: &str, files: &Vec<PathBuf>, dry_run: bool) -> Result<usize, Box<dyn Error>> {
    let repo = Repository::open(dir)?;
    let mut index = repo.index()?;
    for file in files {
        if !dry_run {
            let relative_path = file.strip_prefix(dir).unwrap_or(file);
            index.add_path(relative_path)?;
        }
    }
    index.write()?;
    log::debug!("Added {} files to Git", files.len());
    Ok(files.len())
}

/// Check if Git is installed.
fn check_git_installed() -> bool {
    if let Ok(output) = Command::new("git").arg("--version").output() {
        output.status.success()
    } else {
        false
    }
}

/// Retrieve the last commit from the repository.
fn get_last_commit(repo: &Repository) -> Result<git2::Commit, Box<dyn Error>> {
    let obj = repo.head()?.resolve()?.peel(ObjectType::Commit)?;
    let commit = obj.into_commit().map_err(|_| "Couldn't find commit")?;
    Ok(commit)
}

/// Retrieve a commit by index (0 is most recent, 1 is next, etc.).
fn get_commit_by_index(repo: &Repository, idx: i32) -> Result<git2::Commit, Box<dyn Error>> {
    let mut revwalk = repo.revwalk()?;
    revwalk.push_head()?;
    revwalk.set_sorting(Sort::TIME)?;
    let commits: Vec<_> = revwalk.collect::<Result<Vec<_>, _>>()?;
    if (idx as usize) < commits.len() {
        repo.find_commit(commits[idx as usize]).map_err(|e| e.into())
    } else {
        Err("Index out of bounds".into())
    }
}

/// Retrieve the commit pointed to by the remote HEAD on GitHub.
fn get_remote_head_commit(dir: &str) -> Result<git2::Commit, Box<dyn Error>> {
    // Fetch the latest changes from the remote named "origin".
    let fetch_status = Command::new("git")
        .arg("-C")
        .arg(dir)
        .arg("fetch")
        .arg("origin")
        .status()?;
    if !fetch_status.success() {
        return Err("git fetch failed".into());
    }

    let repo = Repository::open(dir)?;
    // origin/HEAD is a symbolic reference to the default branch.
    let head_ref = repo.find_reference("refs/remotes/origin/HEAD")?;
    let target = head_ref
        .symbolic_target()
        .ok_or("origin/HEAD has no target")?;
    let branch_ref = repo.find_reference(target)?;
    let oid = branch_ref.target().ok_or("Remote HEAD has no target")?;
    repo.find_commit(oid).map_err(|e| e.into())
}

/// Diff commits based on provided version numbers.
fn diff_command(dir: &str, versions: &Vec<String>, dry_run: bool) -> Result<(), Box<dyn Error>> {
    let repo = Repository::open(dir)?;
    let before_commit = if versions.len() == 2 && versions[0].to_uppercase() == "H" {
        get_remote_head_commit(dir)?
    } else {
        let idx = if versions.is_empty() { 0 } else { versions[0].parse::<i32>().map_err(|_| "invalid repo indexes specified")? };
        match get_commit_by_index(&repo, idx) {
            Ok(c) => c,
            Err(_) => {
                log::error!("{}Error:{} invalid repo indexes specified", BLUE, RESET);
                return Err("invalid repo indexes specified".into());
            }
        }
    };
    let before_tree = before_commit.tree()?;
    let before_timestamp = match Utc.timestamp_opt(before_commit.time().seconds(), 0) {
        LocalResult::Single(dt) => dt.naive_utc().format("%Y-%m-%d_%H%M%S").to_string(),
        _ => return Err("Invalid timestamp".into()),
    };
    let before_prefix = format!("before.{}.{}", dir, before_timestamp);
    let before_temp_dir = create_temp_dir(&before_prefix)?;
    if !dry_run {
        checkout_tree_to_dir(&repo, &before_tree, &before_temp_dir)?;
    }
    log::info!("Checked out 'before' snapshot to {:?}", before_temp_dir);

    let (after_dir, after_timestamp_str) = if versions.len() == 2 {
        if versions[0].to_uppercase() == "H" {
            let idx = versions[1].parse::<i32>().map_err(|_| "invalid repo indexes specified")?;
            let after_commit = match get_commit_by_index(&repo, idx) {
                Ok(c) => c,
                Err(_) => {
                    log::error!("{}Error:{} invalid repo indexes specified", BLUE, RESET);
                    return Err("invalid repo indexes specified".into());
                }
            };
            let after_tree = after_commit.tree()?;
            let after_timestamp = match Utc.timestamp_opt(after_commit.time().seconds(), 0) {
                LocalResult::Single(dt) => dt.naive_utc().format("%Y-%m-%d_%H%M%S").to_string(),
                _ => return Err("Invalid timestamp".into()),
            };
            let after_prefix = format!("after.{}.{}", dir, after_timestamp);
            let temp = create_temp_dir(&after_prefix)?;
            if !dry_run {
                checkout_tree_to_dir(&repo, &after_tree, &temp)?;
            }
            log::info!("Checked out 'after' snapshot to {:?}", temp);
            (temp, after_timestamp)
        } else {
            let idx = versions[1].parse::<i32>().map_err(|_| "invalid repo indexes specified")?;
            let after_commit = match get_commit_by_index(&repo, idx) {
                Ok(c) => c,
                Err(_) => {
                    log::error!("{}Error:{} invalid repo indexes specified", BLUE, RESET);
                    return Err("invalid repo indexes specified".into());
                }
            };
            let after_tree = after_commit.tree()?;
            let after_timestamp = match Utc.timestamp_opt(after_commit.time().seconds(), 0) {
                LocalResult::Single(dt) => dt.naive_utc().format("%Y-%m-%d_%H%M%S").to_string(),
                _ => return Err("Invalid timestamp".into()),
            };
            let after_prefix = format!("after.{}.{}", dir, after_timestamp);
            let temp = create_temp_dir(&after_prefix)?;
            if !dry_run {
                checkout_tree_to_dir(&repo, &after_tree, &temp)?;
            }
            log::info!("Checked out 'after' snapshot to {:?}", temp);
            (temp, after_timestamp)
        }
    } else {
        (PathBuf::from(dir), "current".to_string())
    };

    log::info!("{}Comparing {} with {}{}", YELLOW, before_timestamp, after_timestamp_str, RESET);

    // Launch the diff tool only if not a dry run.
    if !dry_run {
        if let Err(e) = launch_diff_tool(&before_temp_dir, &after_dir) {
            log::error!("Failed to launch diff tool: {}", e);
        }
    }
    Ok(())
}

/// Launch a diff tool: try WinMergeU.exe first, then fall back to windiff.exe.
fn launch_diff_tool(before: &Path, after: &Path) -> Result<(), Box<dyn Error>> {
    match Command::new("WinMergeU.exe").arg(before).arg(after).spawn() {
        Ok(_) => {
            log::info!("Launched WinMergeU.exe.");
            Ok(())
        },
        Err(e) => {
            log::warn!("WinMergeU.exe failed to launch: {}. Trying windiff.exe...", e);
            match Command::new("windiff.exe").arg(before).arg(after).spawn() {
                Ok(_) => {
                    log::info!("Launched windiff.exe.");
                    Ok(())
                },
                Err(e2) => {
                    Err(format!("Failed to launch both diff tools. Windiff error: {}", e2).into())
                }
            }
        }
    }
}

/// Detect file type based on file extension.
/// Returns a string representing the file’s category if recognized.
fn detect_file_type(file_path: &Path) -> Option<&'static str> {
    // Allow a file named "LICENSE" (case-insensitive) as a recognized type.
    if let Some(file_name) = file_path.file_name()?.to_str() {
        if file_name.eq_ignore_ascii_case("LICENSE") {
            return Some("License");
        }
    }
    
    let extension = file_path.extension()?.to_str()?.to_lowercase();
    match extension.as_str() {
        // Source Code
        "c"  => Some("C"),
        "cpp" | "cc" | "cxx" => Some("C++"),
        "h"  => Some("C/C++ Header"),
        "hpp" | "hh" | "hxx" => Some("C++ Header"),
        "java" => Some("Java"),
        "py"   => Some("Python"),
        "rb"   => Some("Ruby"),
        "cs"   => Some("C#"),
        "go"   => Some("Go"),
        "php"  => Some("PHP"),
        "rs"   => Some("Rust"),
        "swift" => Some("Swift"),
        "kt" | "kts" => Some("Kotlin"),
        "scala" => Some("Scala"),
        "js"  | "jsx" => Some("JavaScript"),
        "ts"  | "tsx" => Some("TypeScript"),
        "sh"  | "bash" | "zsh" => Some("Shell Script"),
        "bat"  => Some("Batch Script"),
        "ps1"  => Some("PowerShell"),
        // Markup / Documentation
        "html" | "htm" => Some("HTML"),
        "css" | "scss" | "sass" | "less" => Some("CSS"),
        "xml"  => Some("XML"),
        "json" => Some("JSON"),
        "yml"  | "yaml" => Some("YAML"),
        "toml" => Some("TOML"),
        "md"   | "txt" | "rst" | "adoc" => Some("Documentation"),
        // Configuration / Build
        "ini" | "cfg" | "conf" => Some("Configuration"),
        "sln" => Some("Solution File"),
        "csproj" => Some("C# Project File"),
        "pom" => Some("Maven Project File"),
        "gradle" => Some("Gradle Build File"),
        // Database
        "sql" => Some("SQL"),
        // Images & Assets
        "jpg" | "jpeg" => Some("Image"),
        "png" => Some("Image"),
        "bmp" => Some("Image"),
        "gif" => Some("Image"),
        "tiff" => Some("Image"),
        "webp" => Some("Image"),
        "svg" => Some("Vector Image"),
        "ico" => Some("Icon"),
        "cur" => Some("Cursor"),
        "dlg" => Some("Dialog File"),
        _ => None,
    }
}

/// Display repository info. Commits are displayed in ascending order (oldest first)
/// but the index is calculated so that the newest commit is 0 and older ones have higher numbers.
fn info_repository(dir: &str) -> Result<(), Box<dyn Error>> {
    let repo = match Repository::open(dir) {
        Ok(r) => r,
        Err(e) => {
            if e.code() == ErrorCode::NotFound {
                log::error!("No git repository in directory '{}'", dir);
                return Err("No git repository".into());
            } else {
                log::error!("{}", e);
                return Err(e.into());
            }
        }
    };

    if let Err(e) = repo.head() {
        if e.message().contains("reference 'refs/heads/master' not found")
            || e.message().contains("reference 'refs/heads/main' not found")
        {
            log::error!("Git repository exists in '{}' but no commits - probably initialized via 'cargo new'", dir);
            return Err("Empty repository: no commits exist".into());
        } else {
            log::error!("{}", e);
            return Err(e.into());
        }
    }

    let mut revwalk = repo.revwalk()?;
    revwalk.push_head()?;
    revwalk.set_sorting(Sort::TIME)?;
    let commit_ids: Vec<_> = revwalk.collect::<Result<Vec<_>, _>>()?;
    // Reverse to get oldest first.
    let commit_ids: Vec<_> = commit_ids.into_iter().rev().collect();
    let total = commit_ids.len();
    for (i, commit_id) in commit_ids.iter().enumerate() {
        let commit = repo.find_commit(*commit_id)?;
        let summary = commit.summary().unwrap_or("(no message)");
        let seconds = commit.time().seconds();
        let naive = match Utc.timestamp_opt(seconds, 0) {
            LocalResult::Single(dt) => dt.naive_utc(),
            _ => {
                log::error!("Invalid timestamp in commit");
                return Err("Invalid timestamp".into());
            }
        };
        let formatted_time = format!("{}", naive.format("%Y-%m-%d %H:%M:%S (%a)"));
        let tree = commit.tree()?;
        let diff = if commit.parent_count() > 0 {
            let parent_tree = commit.parent(0)?.tree()?;
            repo.diff_tree_to_tree(Some(&parent_tree), Some(&tree), None)?
        } else {
            repo.diff_tree_to_tree(None, Some(&tree), None)?
        };
        let mut file_list = Vec::new();
        diff.foreach(
            &mut |delta, _| {
                match delta.status() {
                    Delta::Added => {
                        if let Some(path) = delta.new_file().path() {
                            file_list.push(format!("{}{}{}", GREEN, path.to_string_lossy(), RESET));
                        }
                    },
                    Delta::Deleted => {
                        if let Some(path) = delta.old_file().path() {
                            file_list.push(format!("{}{}{}", RED, path.to_string_lossy(), RESET));
                        }
                    },
                    _ => {
                        if let Some(path) = delta.new_file().path().or(delta.old_file().path()) {
                            file_list.push(path.to_string_lossy().to_string());
                        }
                    },
                }
                true
            },
            None,
            None,
            None,
        )?;
        // Calculate displayed index: newest commit is 0.
        let display_index = total - 1 - i;
        let idx_str = format!("[{:03}]", display_index);
        log::info!("{}{} {} | {}M:{} {} | {}F:{} {}{}",
            YELLOW, idx_str, formatted_time,
            BLUE, RESET, summary,
            BLUE, RESET, file_list.join(", "),
            RESET
        );
    }
    Ok(())
}

/// Create a .gitignore file at the repository root.
fn create_gitignore(dir: &str, dry_run: bool) -> Result<(), Box<dyn Error>> {
    let gitignore_path = Path::new(dir).join(".gitignore");
    log::info!("Creating .gitignore at '{}'", gitignore_path.display());
    let content = generate_gitignore_content(dir)?;
    if !dry_run {
        fs::write(gitignore_path, content)?;
    }
    Ok(())
}

/// Generate the content for the .gitignore file.
fn generate_gitignore_content(_dir: &str) -> Result<String, Box<dyn Error>> {
    log::debug!("Generating .gitignore content...");
    // Ignore common build and virtual environment directories
    let ignore_patterns = vec![
        "target/",
        "bin/",
        "obj/",
        "venv/",
        ".venv/",
        "env/",
        "*.tmp",
        "*.log",
    ];
    Ok(ignore_patterns.join("\n"))
}

/// Recursively check out a Git tree into the target directory.
fn checkout_tree_to_dir(repo: &Repository, tree: &git2::Tree, target: &Path) -> Result<(), Box<dyn Error>> {
    fs::create_dir_all(target)?;
    for entry in tree.iter() {
        let name = entry.name().ok_or("Invalid UTF-8 in filename")?;
        let entry_path = target.join(name);
        match entry.kind() {
            Some(git2::ObjectType::Tree) => {
                let subtree = repo.find_tree(entry.id())?;
                checkout_tree_to_dir(repo, &subtree, &entry_path)?;
            },
            Some(git2::ObjectType::Blob) => {
                let blob = repo.find_blob(entry.id())?;
                let mut file = File::create(&entry_path)?;
                file.write_all(blob.content())?;
            },
            _ => {}
        }
    }
    Ok(())
}

/// Create a temporary directory with the given prefix.
fn create_temp_dir(prefix: &str) -> Result<PathBuf, Box<dyn Error>> {
    let mut base = env::temp_dir();
    let unique = format!("{}", std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH)?.as_nanos());
    base.push(format!("{}.{}", prefix, unique));
    fs::create_dir_all(&base)?;
    Ok(base)
}

/// Create a GitHub repository using the GitHub API.
/// 
/// Requires the environment variable GITHUB_TOKEN to be set.
/// Returns the created repository.
async fn gh_create(name: &str, description: Option<String>) -> Result<octocrab::models::Repository, Box<dyn std::error::Error>> {
    let token = std::env::var("GITHUB_TOKEN").expect("GITHUB_TOKEN not set");
    let octocrab = octocrab::Octocrab::builder()
        .personal_token(token)
        .build()?;

    // POST to /user/repos with a JSON payload containing "name" and "description"
    let repo: octocrab::models::Repository = octocrab
        .post("/user/repos", Some(&serde_json::json!( {
            "name": name,
            "description": description.unwrap_or_default()
        })))
        .await?;
    println!("Created GitHub repository: {}", repo.html_url);
    Ok(repo)
}

/// Add a remote to the local repository.
fn add_remote(directory: &str, remote_name: &str, remote_url: &str) -> Result<(), Box<dyn Error>> {
    let repo = Repository::open(directory)?;
    // If the remote already exists, skip adding.
    if repo.find_remote(remote_name).is_err() {
        repo.remote(remote_name, remote_url)?;
        log::info!("Added remote '{}' with URL '{}'", remote_name, remote_url);
    } else {
        log::info!("Remote '{}' already exists", remote_name);
    }
    Ok(())
}

/// Check if the remote branch exists.
fn remote_branch_exists(directory: &str, remote: &str, branch: &str) -> Result<bool, Box<dyn Error>> {
    let output = Command::new("git")
        .arg("-C")
        .arg(directory)
        .arg("ls-remote")
        .arg("--heads")
        .arg(remote)
        .arg(branch)
        .output()?;
    if output.status.success() {
        Ok(!output.stdout.is_empty())
    } else {
        Ok(false)
    }
}

/// Push local changes to the GitHub remote using the system's Git CLI.
/// This function determines the current branch name from the repository HEAD.
fn gh_push(directory: &str, remote: &str) -> Result<(), Box<dyn std::error::Error>> {
    let repo = Repository::open(directory)?;
    let head = repo.head()?;
    let branch = head.shorthand().unwrap_or("master");

    // Check if the remote branch exists.
    let branch_exists = remote_branch_exists(directory, remote, branch)?;

    if branch_exists {
        println!("Auto-pulling changes from remote '{}' for branch '{}'", remote, branch);
        let pull_status = Command::new("git")
            .arg("-C")
            .arg(directory)
            .arg("pull")
            .arg(remote)
            .arg(branch)
            .arg("--no-edit")
            .status()?;
        if !pull_status.success() {
            eprintln!("Auto-pull failed. This may be due to merge conflicts.");
            println!("Please follow these steps to resolve merge conflicts:");
            println!("1. Run 'git status' in the repository to see the files with conflicts.");
            println!("2. Open the conflicted files and resolve the conflicts manually.");
            println!("3. After resolving, add the files using 'git add <file>' for each conflicted file.");
            println!("4. Commit the merge with 'git commit' (if needed).");
            println!("5. Finally, re-run 'mdcode p .' to push your changes.");
            return Err("Merge failed. Please resolve conflicts and try again.".into());
        }
    } else {
        println!("Remote branch '{}' does not exist. Skipping pull.", branch);
    }

    println!("Pushing local repository '{}' to remote '{}'", directory, remote);
    let push_status = if branch_exists {
        Command::new("git")
            .arg("-C")
            .arg(directory)
            .arg("push")
            .arg(remote)
            .arg(branch)
            .status()?
    } else {
        // If branch doesn't exist, push and set upstream.
        Command::new("git")
            .arg("-C")
            .arg(directory)
            .arg("push")
            .arg("-u")
            .arg(remote)
            .arg(branch)
            .status()?
    };

    if push_status.success() {
        println!("Successfully pushed changes to GitHub.");
        Ok(())
    } else {
        Err("Failed to push changes.".into())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use std::path::Path;
    use tempfile::tempdir;

    #[test]
    fn test_detect_file_type_source_code() {
        // C / C++
        assert_eq!(detect_file_type(Path::new("test.c")), Some("C"));
        assert_eq!(detect_file_type(Path::new("test.cpp")), Some("C++"));
        assert_eq!(detect_file_type(Path::new("test.cc")), Some("C++"));
        assert_eq!(detect_file_type(Path::new("test.cxx")), Some("C++"));
        assert_eq!(detect_file_type(Path::new("test.h")), Some("C/C++ Header"));
        assert_eq!(detect_file_type(Path::new("test.hpp")), Some("C++ Header"));

        // Other languages
        assert_eq!(detect_file_type(Path::new("test.java")), Some("Java"));
        assert_eq!(detect_file_type(Path::new("test.py")), Some("Python"));
        assert_eq!(detect_file_type(Path::new("test.rb")), Some("Ruby"));
        assert_eq!(detect_file_type(Path::new("test.cs")), Some("C#"));
        assert_eq!(detect_file_type(Path::new("test.go")), Some("Go"));
        assert_eq!(detect_file_type(Path::new("test.php")), Some("PHP"));
        assert_eq!(detect_file_type(Path::new("test.rs")), Some("Rust"));
        assert_eq!(detect_file_type(Path::new("test.swift")), Some("Swift"));
        assert_eq!(detect_file_type(Path::new("test.kt")), Some("Kotlin"));
        assert_eq!(detect_file_type(Path::new("test.kts")), Some("Kotlin"));
        assert_eq!(detect_file_type(Path::new("test.scala")), Some("Scala"));
        assert_eq!(detect_file_type(Path::new("test.js")), Some("JavaScript"));
        assert_eq!(detect_file_type(Path::new("test.jsx")), Some("JavaScript"));
        assert_eq!(detect_file_type(Path::new("test.ts")), Some("TypeScript"));
        assert_eq!(detect_file_type(Path::new("test.tsx")), Some("TypeScript"));
        assert_eq!(detect_file_type(Path::new("test.sh")), Some("Shell Script"));
        assert_eq!(detect_file_type(Path::new("test.bash")), Some("Shell Script"));
        assert_eq!(detect_file_type(Path::new("test.zsh")), Some("Shell Script"));
        assert_eq!(detect_file_type(Path::new("test.bat")), Some("Batch Script"));
        assert_eq!(detect_file_type(Path::new("test.ps1")), Some("PowerShell"));
    }

    #[test]
    fn test_detect_file_type_markup_and_config() {
        // Markup and documentation
        assert_eq!(detect_file_type(Path::new("index.html")), Some("HTML"));
        assert_eq!(detect_file_type(Path::new("style.css")), Some("CSS"));
        assert_eq!(detect_file_type(Path::new("script.scss")), Some("CSS"));
        assert_eq!(detect_file_type(Path::new("doc.xml")), Some("XML"));
        assert_eq!(detect_file_type(Path::new("data.json")), Some("JSON"));
        assert_eq!(detect_file_type(Path::new("config.yml")), Some("YAML"));
        assert_eq!(detect_file_type(Path::new("config.yaml")), Some("YAML"));
        assert_eq!(detect_file_type(Path::new("Cargo.toml")), Some("TOML"));
        assert_eq!(detect_file_type(Path::new("README.md")), Some("Documentation"));
        assert_eq!(detect_file_type(Path::new("notes.txt")), Some("Documentation"));
        assert_eq!(detect_file_type(Path::new("manual.rst")), Some("Documentation"));
        assert_eq!(detect_file_type(Path::new("guide.adoc")), Some("Documentation"));

        // Configuration / Build
        assert_eq!(detect_file_type(Path::new("settings.ini")), Some("Configuration"));
        assert_eq!(detect_file_type(Path::new("config.cfg")), Some("Configuration"));
        assert_eq!(detect_file_type(Path::new("app.conf")), Some("Configuration"));
        assert_eq!(detect_file_type(Path::new("project.sln")), Some("Solution File"));
        assert_eq!(detect_file_type(Path::new("app.csproj")), Some("C# Project File"));
        assert_eq!(detect_file_type(Path::new("pom.xml")), Some("XML")); // Note: Maven's pom.xml is XML
        assert_eq!(detect_file_type(Path::new("build.gradle")), Some("Gradle Build File"));

        // Database
        assert_eq!(detect_file_type(Path::new("schema.sql")), Some("SQL"));
    }

    #[test]
    fn test_detect_file_type_images_and_assets() {
        // Raster images
        assert_eq!(detect_file_type(Path::new("image.jpg")), Some("Image"));
        assert_eq!(detect_file_type(Path::new("image.jpeg")), Some("Image"));
        assert_eq!(detect_file_type(Path::new("image.png")), Some("Image"));
        assert_eq!(detect_file_type(Path::new("image.bmp")), Some("Image"));
        assert_eq!(detect_file_type(Path::new("image.gif")), Some("Image"));
        assert_eq!(detect_file_type(Path::new("image.tiff")), Some("Image"));
        assert_eq!(detect_file_type(Path::new("image.webp")), Some("Image"));
        // Vector and icons
        assert_eq!(detect_file_type(Path::new("vector.svg")), Some("Vector Image"));
        assert_eq!(detect_file_type(Path::new("icon.ico")), Some("Icon"));
        assert_eq!(detect_file_type(Path::new("cursor.cur")), Some("Cursor"));
        // Other asset
        assert_eq!(detect_file_type(Path::new("dialog.dlg")), Some("Dialog File"));
    }

    #[test]
    fn test_generate_gitignore_content() {
        let content = generate_gitignore_content(".").unwrap();
        let expected = "target/\nbin/\nobj/\nvenv/\n.venv/\nenv/\n*.tmp\n*.log";
        assert_eq!(content, expected);
    }

    #[test]
    fn test_new_repository_and_gitignore() {
        if !check_git_installed() {
            eprintln!("Skipping test: Git not installed");
            return;
        }
        let temp_dir = tempdir().unwrap();
        let repo_path = temp_dir.path().join("repo");
        let repo_str = repo_path.to_str().unwrap();
        new_repository(repo_str, false).unwrap();
        assert!(Path::new(repo_str).join(".git").exists(), ".git directory should exist");
        assert!(Path::new(repo_str).join(".gitignore").exists(), ".gitignore file should exist");
    }

    #[test]
    fn test_update_repository() {
        if !check_git_installed() {
            eprintln!("Skipping test: Git not installed");
            return;
        }
        let temp_dir = tempdir().unwrap();
        let repo_path = temp_dir.path().join("repo");
        let repo_str = repo_path.to_str().unwrap();
        new_repository(repo_str, false).unwrap();
        let file_path = repo_path.join("new_file.txt");
        fs::write(&file_path, "Hello, mdcode!").unwrap();
        // Provide a commit message to avoid hanging.
        update_repository(repo_str, false, Some("Test commit message")).unwrap();
        let repo = Repository::open(repo_str).unwrap();
        let mut revwalk = repo.revwalk().unwrap();
        revwalk.push_head().unwrap();
        let commits: Vec<_> = revwalk.collect();
        assert!(commits.len() >= 2, "Repository should have at least two commits");
    }

    #[test]
    fn test_info_repository() {
        if !check_git_installed() {
            eprintln!("Skipping test: Git not installed");
            return;
        }
        let temp_dir = tempdir().unwrap();
        let repo_path = temp_dir.path().join("repo");
        let repo_str = repo_path.to_str().unwrap();
        new_repository(repo_str, false).unwrap();
        let file_path = repo_path.join("info_test.txt");
        fs::write(&file_path, "Test info output").unwrap();
        update_repository(repo_str, false, Some("Test commit message")).unwrap();
        info_repository(repo_str).unwrap();
    }
}
