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

// Define our uniform color constants.
const BLUE: &str = "\x1b[94m";    // Light blue
const GREEN: &str = "\x1b[32m";   // Green
const RED: &str = "\x1b[31m";     // Red
const YELLOW: &str = "\x1b[93m";  // Light yellow
const RESET: &str = "\x1b[0m";

#[derive(Parser)]
#[command(
    name = "mdcode",
    version = "1.0.0",
    about = "Martin's simple code management tool using Git.",
    arg_required_else_help = true,
    after_help = "\
Diff Modes:
  mdcode diff <directory>
    => Compare current working directory vs most recent commit.
  mdcode diff <directory> <n>
    => Compare current working directory vs commit selected by n (0 is most recent, 1 for next, etc.).
  mdcode diff <directory> <n> <m>
    => Compare commit selected by n (before) vs commit selected by m (after).",
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
    /// Subcommand to run: new, update, info, or diff (short aliases shown)
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
    => Compare commit selected by n (before) vs commit selected by m (after)."
    )]
    Diff {
        /// Directory of the repository to diff
        directory: String,
        /// Optional version numbers (0 is most recent; 1, 2, ... select older commits)
        #[arg(num_args = 0..=2)]
        versions: Vec<i32>,
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

    if let Err(_) = run() {
        std::process::exit(1);
    }
}

/// Returns true if any component of the entry's path is named "target".
fn is_in_target(entry: &walkdir::DirEntry) -> bool {
    entry.path().components().any(|comp| comp.as_os_str() == "target")
}

/// Create a new repository and make an initial commit.
/// If a valid git repository (with a HEAD) already exists in the directory, return an error.
/// Otherwise, proceed even if the directory was created by Cargo.
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
    let repo = Repository::init(dir)?;

    log::info!("Initializing Git repository...");
    if !dry_run {
        create_gitignore(dir, dry_run)?;
    }
    let added_count = add_files_to_git(dir, &source_files, dry_run)?;

    if !dry_run {
        let mut index = repo.index()?;
        index.write()?;
        let tree_id = index.write_tree()?;
        let tree = repo.find_tree(tree_id)?;
        let signature = Signature::now("mdcode", "mdcode@example.com")?;
        repo.commit(Some("HEAD"), &signature, &signature, "Initial commit", &tree, &[])?;
    }

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

/// Scan the entire directory tree and count total files, skipping any entries under "target".
fn scan_total_files(dir: &str) -> Result<usize, Box<dyn Error>> {
    log::debug!("Scanning source tree in '{}'...", dir);
    let mut total = 0;
    for entry in WalkDir::new(dir).into_iter().filter_map(Result::ok) {
        if is_in_target(&entry) {
            continue;
        }
        if entry.file_type().is_file() {
            total += 1;
        }
    }
    log::debug!("Scan complete - found {} files", total);
    Ok(total)
}

/// Scan for source files (ignoring files under target directories).
fn scan_source_files(dir: &str) -> Result<(Vec<PathBuf>, usize), Box<dyn Error>> {
    log::debug!("Scanning for source files in '{}'...", dir);
    let mut source_files = Vec::new();
    let mut count = 0;
    for entry in WalkDir::new(dir).into_iter().filter_map(Result::ok) {
        if is_in_target(&entry) {
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

/// Diff commits based on provided version numbers.
/// If an invalid index is specified, logs an error (with "Error:" in light blue) and returns an error.
fn diff_command(dir: &str, versions: &Vec<i32>, dry_run: bool) -> Result<(), Box<dyn Error>> {
    let repo = Repository::open(dir)?;
    let before_commit = match if versions.is_empty() { get_commit_by_index(&repo, 0) } else { get_commit_by_index(&repo, versions[0]) } {
        Ok(c) => c,
        Err(_) => {
            log::error!("{}Error:{} invalid repo indexes specified", BLUE, RESET);
            return Err("invalid repo indexes specified".into());
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
        let after_commit = match get_commit_by_index(&repo, versions[1]) {
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
        (PathBuf::from(dir), "current".to_string())
    };

    log::info!("{}Comparing {} with {}{}", YELLOW, before_timestamp, after_timestamp_str, RESET);

    if let Err(e) = Command::new("WinMergeU.exe")
        .arg(&before_temp_dir)
        .arg(&after_dir)
        .spawn() {
        log::error!("Failed to spawn WinMergeU: {}", e);
    }
    Ok(())
}

/// Detect file type based on file extension.
/// Now includes ".toml" files (returned as "TOML").
fn detect_file_type(file_path: &Path) -> Option<&'static str> {
    let extension = file_path.extension()?.to_str()?.to_lowercase();
    match extension.as_str() {
        "c" | "h" | "cpp" | "hpp" | "cc" => Some("C/C++"),
        "pas" | "pp" => Some("Pascal"),
        "rb" => Some("Ruby"),
        "sh" | "csh" => Some("Shell Script"),
        "cs" => Some("C#"),
        "rs" => Some("Rust"),
        "py" => Some("Python"),
        "toml" => Some("TOML"),
        "md" | "txt" | "rst" => Some("Documentation"),
        _ => None,
    }
}

/// Display repository info. Commits are displayed in ascending order (oldest first)
/// but the index is calculated so that the newest commit is 0 and older ones have higher numbers.
/// For example, if there are N commits then the oldest is displayed with index [N-1] and the newest with [000].
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
    let ignore_patterns = vec!["target/", "*.tmp", "*.log"];
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

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use std::path::Path;
    use tempfile::tempdir;

    #[test]
    fn test_detect_file_type() {
        let path = Path::new("test.c");
        assert_eq!(detect_file_type(path), Some("C/C++"));
        let path = Path::new("test.cpp");
        assert_eq!(detect_file_type(path), Some("C/C++"));
        let path = Path::new("test.pas");
        assert_eq!(detect_file_type(path), Some("Pascal"));
        let path = Path::new("test.rb");
        assert_eq!(detect_file_type(path), Some("Ruby"));
        let path = Path::new("script.sh");
        assert_eq!(detect_file_type(path), Some("Shell Script"));
        let path = Path::new("program.cs");
        assert_eq!(detect_file_type(path), Some("C#"));
        let path = Path::new("lib.rs");
        assert_eq!(detect_file_type(path), Some("Rust"));
        let path = Path::new("app.py");
        assert_eq!(detect_file_type(path), Some("Python"));
        let path = Path::new("Cargo.toml");
        assert_eq!(detect_file_type(path), Some("TOML"));
        let path = Path::new("README.md");
        assert_eq!(detect_file_type(path), Some("Documentation"));
        let path = Path::new("unknown.xyz");
        assert_eq!(detect_file_type(path), None);
    }

    #[test]
    fn test_generate_gitignore_content() {
        let content = generate_gitignore_content(".").unwrap();
        let expected = "target/\n*.tmp\n*.log";
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
