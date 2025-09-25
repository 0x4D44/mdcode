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

use chrono::{LocalResult, TimeZone, Utc};
use clap::{ArgAction, Parser, Subcommand};
use git2::{Delta, ErrorCode, ObjectType, Repository, Signature, Sort};
use semver::Version as SemverVersion;
use std::env;
use std::error::Error;
use std::fs;
use std::fs::File;
use std::io::{self, Write};
use std::path::{Path, PathBuf};
use std::process::Command;
use tokio::runtime::Runtime;
// walkdir remains for other areas; ignore's walker handles file scanning honoring .gitignore
// use walkdir::WalkDir;
use ignore::{gitignore::GitignoreBuilder, WalkBuilder as IgnoreWalkBuilder};

// Define our uniform color constants.
const BLUE: &str = "\x1b[94m"; // Light blue
const GREEN: &str = "\x1b[32m"; // Green
const RED: &str = "\x1b[31m"; // Red
const YELLOW: &str = "\x1b[93m"; // Light yellow
const RESET: &str = "\x1b[0m";

#[derive(Clone, Copy)]
enum RepoVisibility {
    Public,
    Private,
    Internal,
}

#[derive(Parser)]
#[command(
    name = "mdcode",
    version = "1.9.1",
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
    => Compare GitHub HEAD (before) vs local commit selected by n (after).
  mdcode diff <directory> L
    => Compare GitHub HEAD (before) vs current working directory (after).",
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
    /// Command to run: new, update, info, diff, gh_create, gh_push, gh_fetch, or gh_sync (short aliases shown)
    #[command(subcommand)]
    command: Commands,

    /// Perform a dry run (no changes will be made)
    #[arg(long)]
    dry_run: bool,

    /// Maximum file size to auto-stage (in MB). Use to include large assets per-invocation.
    /// Default: 50 MB.
    #[arg(long = "max-file-mb", default_value_t = 50)]
    max_file_mb: u64,
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
    => Compare GitHub HEAD (before) vs local commit selected by n (after).
  mdcode diff <directory> L
    => Compare GitHub HEAD (before) vs current working directory (after)."
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
        /// Create the repository as public visibility
        #[arg(long, action = ArgAction::SetTrue)]
        public: bool,
        /// Create the repository as private visibility (default)
        #[arg(long, action = ArgAction::SetTrue)]
        private: bool,
        /// Create the repository as internal visibility (orgs only)
        #[arg(long, action = ArgAction::SetTrue)]
        internal: bool,
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
    #[command(
        name = "gh_fetch",
        visible_alias = "gf",
        about = "Fetch changes from the GitHub remote and list them"
    )]
    GhFetch {
        /// Directory of the local repository
        directory: String,
        /// Name of the remote to fetch from (default: origin)
        #[arg(short, long, default_value = "origin")]
        remote: String,
    },
    #[command(
        name = "gh_sync",
        visible_alias = "gs",
        about = "Synchronize the local repository with the GitHub remote"
    )]
    GhSync {
        /// Directory of the local repository
        directory: String,
        /// Name of the remote to sync with (default: origin)
        #[arg(short, long, default_value = "origin")]
        remote: String,
    },
    #[command(
        name = "tag",
        visible_alias = "t",
        about = "Create an annotated git tag for the current HEAD"
    )]
    Tag {
        /// Directory of the local repository (e.g. '.' for current directory)
        directory: String,
        /// Optional explicit version (semver). If not provided, read Cargo.toml or prompt.
        #[arg(short, long)]
        version: Option<String>,
        /// Optional tag message. Defaults to 'Release v<version>'.
        #[arg(short, long)]
        message: Option<String>,
        /// Do not push the created tag to the remote (pushes by default).
        #[arg(long = "no-push", action = ArgAction::SetTrue)]
        no_push: bool,
        /// Remote name to push to (used with --push). Defaults to 'origin'.
        #[arg(long, default_value = "origin")]
        remote: String,
        /// Overwrite an existing tag of the same name.
        #[arg(long, action = ArgAction::SetTrue)]
        force: bool,
        /// Allow tagging when the working tree has uncommitted changes.
        #[arg(long, action = ArgAction::SetTrue)]
        allow_dirty: bool,
    },
}

fn run() -> Result<(), Box<dyn Error>> {
    let cli = Cli::parse();

    match &cli.command {
        Commands::New { directory } => {
            log::info!("Creating new repository in '{}'", directory);
            new_repository(directory, cli.dry_run, cli.max_file_mb)?;
        }
        Commands::Update { directory } => {
            log::info!("Updating repository in '{}'", directory);
            // In interactive use, pass None to prompt the user.
            update_repository(directory, cli.dry_run, None, cli.max_file_mb)?;
        }
        Commands::Info { directory } => {
            log::info!("Displaying repository info for '{}'", directory);
            info_repository(directory)?;
        }
        Commands::Diff {
            directory,
            versions,
        } => {
            log::info!(
                "Diffing repository '{}' with versions {:?}",
                directory,
                versions
            );
            diff_command(directory, versions, cli.dry_run)?;
        }
        Commands::GhCreate {
            directory,
            description,
            public,
            private,
            internal,
        } => {
            log::info!(
                "Creating GitHub repository from local directory '{}'",
                directory
            );
            // Deduce repository name from the provided directory.
            let repo_name = {
                let path = Path::new(directory);
                // If directory is ".", use current dir.
                let actual = if path == Path::new(".") {
                    env::current_dir()?
                } else {
                    path.to_path_buf()
                };
                actual
                    .file_name()
                    .ok_or("Could not determine repository name from directory")?
                    .to_string_lossy()
                    .to_string()
            };
            // Determine visibility; default to private if unspecified.
            let mut selected = None;
            if *public {
                selected = Some(RepoVisibility::Public);
            }
            if *private {
                selected = Some(RepoVisibility::Private);
            }
            if *internal {
                selected = Some(RepoVisibility::Internal);
            }
            // If multiple flags were provided, return an error.
            let count = (*public as u8) + (*private as u8) + (*internal as u8);
            if count > 1 {
                return Err("Provide only one of --public/--private/--internal".into());
            }
            let visibility = selected.unwrap_or(RepoVisibility::Private);

            if let Some(gh_cmd) = gh_cli_path() {
                log::info!("Detected GitHub CLI. Using 'gh repo create' flow.");
                gh_create_via_cli(
                    &gh_cmd,
                    directory,
                    &repo_name,
                    description.clone(),
                    visibility,
                )?;
            } else {
                log::info!("GitHub CLI not found. Falling back to API token auth.");
                log::debug!("PATH: {}", env::var("PATH").unwrap_or_default());
                let rt = Runtime::new()?;
                let created_repo = rt.block_on(gh_create_api(&repo_name, description.clone()))?;
                // Use the HTTPS clone URL from the created repository.
                let remote_url = created_repo
                    .clone_url
                    .ok_or("GitHub repository did not return a clone URL")?;
                // Add the remote "origin" to the local repository.
                add_remote(directory, "origin", remote_url.as_str())?;
                // Automatically push the current branch.
                gh_push(directory, "origin")?;
            }
        }
        Commands::GhPush { directory, remote } => {
            log::info!(
                "Pushing local repository '{}' to remote '{}'",
                directory,
                remote
            );
            gh_push(directory, remote)?;
        }
        Commands::GhFetch { directory, remote } => {
            log::info!(
                "Fetching remote changes for repository '{}' from '{}'",
                directory,
                remote
            );
            gh_fetch(directory, remote)?;
        }
        Commands::GhSync { directory, remote } => {
            log::info!(
                "Synchronizing local repository '{}' with remote '{}'",
                directory,
                remote
            );
            gh_sync(directory, remote)?;
        }
        Commands::Tag {
            directory,
            version,
            message,
            no_push,
            remote,
            force,
            allow_dirty,
        } => {
            log::info!("Tagging release in '{}'", directory);
            tag_release(
                directory,
                version.clone(),
                message.clone(),
                !*no_push,
                remote,
                *force,
                *allow_dirty,
                cli.dry_run,
            )?;
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

    if let Err(e) = run() {
        eprintln!("{}Error:{} {}", BLUE, RESET, e);
        std::process::exit(1);
    }
}

/// Read `[package].version` from `Cargo.toml` in `dir`.
fn read_version_from_cargo_toml(dir: &str) -> Result<Option<String>, Box<dyn Error>> {
    let cargo_toml_path = Path::new(dir).join("Cargo.toml");
    if !cargo_toml_path.exists() {
        return Ok(None);
    }
    let contents = fs::read_to_string(&cargo_toml_path)?;
    let value: toml::Value = contents.parse::<toml::Value>()?;
    if let Some(pkg) = value.get("package") {
        if let Some(ver) = pkg.get("version").and_then(|v| v.as_str()) {
            return Ok(Some(ver.to_string()));
        }
    }
    Ok(None)
}

/// Check if working tree has uncommitted changes in tracked files.
/// Ignores untracked files and whitespace/EOL-only changes.
#[allow(dead_code)]
fn is_dirty(dir: &str) -> Result<bool, Box<dyn Error>> {
    let repo = Repository::open(dir)?;
    // No commits yet => not dirty for our purposes.
    if repo.head().is_err() {
        return Ok(false);
    }

    // First, use libgit2 statuses to see if any tracked files are modified or staged.
    let mut opts = git2::StatusOptions::new();
    opts.include_untracked(false)
        .include_ignored(false)
        .recurse_untracked_dirs(false)
        .exclude_submodules(true)
        .renames_head_to_index(true)
        .show(git2::StatusShow::IndexAndWorkdir);
    let statuses = repo.statuses(Some(&mut opts))?;
    let mut has_candidate_changes = false;
    for s in statuses.iter() {
        let st = s.status();
        if st.intersects(
            git2::Status::INDEX_NEW
                | git2::Status::INDEX_MODIFIED
                | git2::Status::INDEX_DELETED
                | git2::Status::INDEX_RENAMED
                | git2::Status::INDEX_TYPECHANGE
                | git2::Status::WT_MODIFIED
                | git2::Status::WT_DELETED
                | git2::Status::WT_RENAMED
                | git2::Status::WT_TYPECHANGE,
        ) {
            has_candidate_changes = true;
            break;
        }
    }
    if !has_candidate_changes {
        return Ok(false);
    }

    // If there are candidate changes, confirm by byte-compare after normalizing EOL.
    let workdir = repo.workdir().ok_or("No workdir")?;
    let head_tree = repo.head()?.peel_to_tree()?;

    fn normalize_eol(data: Vec<u8>) -> Vec<u8> {
        // Replace CRLF with LF
        let mut out = Vec::with_capacity(data.len());
        let mut i = 0;
        while i < data.len() {
            if i + 1 < data.len() && data[i] == b'\r' && data[i + 1] == b'\n' {
                out.push(b'\n');
                i += 2;
            } else {
                out.push(data[i]);
                i += 1;
            }
        }
        out
    }

    for s in statuses.iter() {
        let st = s.status();
        if !(st.intersects(
            git2::Status::INDEX_NEW
                | git2::Status::INDEX_MODIFIED
                | git2::Status::INDEX_DELETED
                | git2::Status::INDEX_RENAMED
                | git2::Status::INDEX_TYPECHANGE
                | git2::Status::WT_MODIFIED
                | git2::Status::WT_DELETED
                | git2::Status::WT_RENAMED
                | git2::Status::WT_TYPECHANGE,
        )) {
            continue;
        }
        // If staged new/deleted/typechange exists, it is dirty.
        if st.intersects(
            git2::Status::INDEX_NEW | git2::Status::INDEX_DELETED | git2::Status::INDEX_TYPECHANGE,
        ) {
            #[cfg(test)]
            eprintln!(
                "is_dirty: staged-change status={:?} path={:?}",
                st,
                s.path()
            );
            return Ok(true);
        }
        // Compare HEAD blob vs workdir after normalizing EOL; if equal, ignore.
        if let Some(rel) = s.path() {
            let head_entry = head_tree.get_path(Path::new(rel));
            if let Ok(head_entry) = head_entry {
                if let Ok(blob) = repo.find_blob(head_entry.id()) {
                    let head_bytes = normalize_eol(blob.content().to_vec());
                    let wt_path = workdir.join(rel);
                    if let Ok(wt_bytes_raw) = std::fs::read(&wt_path) {
                        let wt_bytes = normalize_eol(wt_bytes_raw);
                        if head_bytes == wt_bytes {
                            continue; // spurious EOL-only change; ignore
                        } else {
                            #[cfg(test)]
                            eprintln!(
                                "is_dirty: content-diff path={} head_len={} wt_len={}",
                                rel,
                                head_bytes.len(),
                                wt_bytes.len()
                            );
                            return Ok(true);
                        }
                    } else {
                        #[cfg(test)]
                        eprintln!("is_dirty: worktree read failed path={}", rel);
                        return Ok(true);
                    }
                } else {
                    #[cfg(test)]
                    eprintln!("is_dirty: blob lookup failed path={}", rel);
                    return Ok(true);
                }
            } else {
                // Not found in HEAD (renamed?), consider dirty.
                #[cfg(test)]
                eprintln!("is_dirty: path not in HEAD: {}", rel);
                return Ok(true);
            }
        }
    }
    Ok(false)
}

/// Normalize and validate a semver string, enforcing a leading 'v' in the tag.
fn normalize_semver_tag(input: &str) -> Result<(SemverVersion, String), Box<dyn Error>> {
    let trimmed = input.trim().trim_start_matches('v');
    let parsed = SemverVersion::parse(trimmed)?;
    let tag = format!("v{}", parsed);
    Ok((parsed, tag))
}

/// Create an annotated tag for the current HEAD.
#[allow(clippy::too_many_arguments)]
fn tag_release(
    directory: &str,
    version_flag: Option<String>,
    message_flag: Option<String>,
    push: bool,
    remote: &str,
    force: bool,
    _allow_dirty: bool,
    dry_run: bool,
) -> Result<(), Box<dyn Error>> {
    let repo = Repository::open(directory)?;

    // Do not block on a dirty working tree; tagging uses HEAD.
    // Keep --allow-dirty for backward compatibility but no longer enforce cleanliness by default.

    // Determine version: CLI flag > Cargo.toml > prompt
    let version_str = if let Some(v) = version_flag {
        v
    } else if let Some(v) = read_version_from_cargo_toml(directory)? {
        log::info!("Using version from Cargo.toml: {}", v);
        v
    } else {
        print!("Enter version (e.g., 0.1.0): ");
        io::stdout().flush()?;
        let mut buf = String::new();
        io::stdin().read_line(&mut buf)?;
        buf.trim().to_string()
    };

    // Validate and normalize to tag name with leading 'v'
    let (_semver, tag_name) = normalize_semver_tag(&version_str)?;
    // Ensure message; default to tag name itself (e.g., "v0.1.0").
    let message = message_flag.unwrap_or_else(|| tag_name.clone());

    // Check existing tag
    let tag_ref_name = format!("refs/tags/{}", tag_name);
    let exists = repo.find_reference(&tag_ref_name).is_ok();
    if exists && !force {
        return Err(format!(
            "tag '{}' already exists; use --force to overwrite",
            tag_name
        )
        .into());
    }

    if dry_run {
        log::info!(
            "[dry-run] Would run: git -C {} tag -a {}{} -m \"{}\"",
            directory,
            tag_name,
            if force { " -f" } else { "" },
            message
        );
        if push {
            log::info!(
                "[dry-run] Would run: git -C {} push {} {}",
                directory,
                remote,
                tag_name
            );
        }
        return Ok(());
    }

    // Create or update annotated tag via git CLI (matches user's expectation).
    let mut tag_args = vec!["-C", directory, "tag", "-a", &tag_name, "-m", &message];
    if exists && !force {
        return Err(format!(
            "tag '{}' already exists; use --force to overwrite",
            tag_name
        )
        .into());
    }
    if force {
        // If --force was requested, add -f to update the tag.
        tag_args.push("-f");
    }
    let status = Command::new("git").args(&tag_args).status()?;
    if !status.success() {
        return Err("failed to create tag via git".into());
    }
    println!("Created tag '{}'", tag_name);

    if push {
        // Validate remote exists
        repo.find_remote(remote)
            .map_err(|_| format!("remote '{}' not found", remote))?;
        let status = Command::new("git")
            .arg("-C")
            .arg(directory)
            .arg("push")
            .arg(remote)
            .arg(&tag_name)
            .status()?;
        if !status.success() {
            return Err("failed to push tag".into());
        }
        println!("Pushed tag '{}' to '{}'", tag_name, remote);
    }

    Ok(())
}

#[cfg(test)]
mod tests_tag {
    use super::*;
    use std::fs::File;
    use std::io::Write as IoWrite;
    use tempfile::tempdir;

    #[test]
    fn test_normalize_semver_tag_variants() {
        let (_, t1) = normalize_semver_tag("1.2.3").unwrap();
        assert_eq!(t1, "v1.2.3");
        let (_, t2) = normalize_semver_tag("v1.2.3").unwrap();
        assert_eq!(t2, "v1.2.3");
        let (_, t3) = normalize_semver_tag("  v2.0.0  ").unwrap();
        assert_eq!(t3, "v2.0.0");
    }

    #[test]
    fn test_read_version_from_cargo_toml() {
        let dir = tempdir().unwrap();
        let path = dir.path().join("Cargo.toml");
        let mut f = File::create(&path).unwrap();
        writeln!(
            f,
            "[package]\nname=\"x\"\nversion=\"0.9.1\"\nedition=\"2021\"\n"
        )
        .unwrap();
        let v = read_version_from_cargo_toml(dir.path().to_str().unwrap()).unwrap();
        assert_eq!(v, Some("0.9.1".to_string()));
    }

    #[test]
    fn test_is_dirty_ignores_untracked() {
        if !check_git_installed() {
            eprintln!("git not installed; skipping test");
            return;
        }
        let dir = tempdir().unwrap();
        let d = dir.path();
        // init repo
        std::process::Command::new("git")
            .arg("-C")
            .arg(d)
            .arg("init")
            .status()
            .unwrap();
        // configure local git identity
        std::process::Command::new("git")
            .arg("-C")
            .arg(d)
            .arg("config")
            .arg("user.name")
            .arg("mdcode-test")
            .status()
            .unwrap();
        std::process::Command::new("git")
            .arg("-C")
            .arg(d)
            .arg("config")
            .arg("user.email")
            .arg("mdcode@test.local")
            .status()
            .unwrap();

        // Ensure consistent line ending behavior on Windows to avoid false positives
        // when checking for dirty state in tests.
        std::process::Command::new("git")
            .arg("-C")
            .arg(d)
            .arg("config")
            .arg("core.autocrlf")
            .arg("false")
            .status()
            .unwrap();
        std::process::Command::new("git")
            .arg("-C")
            .arg(d)
            .arg("config")
            .arg("core.filemode")
            .arg("false")
            .status()
            .unwrap();

        // create tracked file and commit
        let mut tf = File::create(d.join("tracked.txt")).unwrap();
        writeln!(tf, "hello").unwrap();
        drop(tf); // ensure contents are flushed before adding
        std::process::Command::new("git")
            .arg("-C")
            .arg(d)
            .arg("add")
            .arg("tracked.txt")
            .status()
            .unwrap();
        std::process::Command::new("git")
            .arg("-C")
            .arg(d)
            .arg("commit")
            .arg("-m")
            .arg("init")
            .status()
            .unwrap();
        // create an untracked file
        let mut uf = File::create(d.join("untracked.txt")).unwrap();
        writeln!(uf, "temp").unwrap();
        // is_dirty should be false (ignoring untracked)
        assert_eq!(is_dirty(d.to_str().unwrap()).unwrap(), false);
        // modify tracked file to make it dirty
        let mut tf2 = File::create(d.join("tracked.txt")).unwrap();
        writeln!(tf2, "more").unwrap();
        drop(tf2);
        assert_eq!(is_dirty(d.to_str().unwrap()).unwrap(), true);
    }
}

#[cfg(test)]
mod tests_detect_and_cap {
    use super::*;
    use std::io::Write as IoWrite;
    use tempfile::tempdir;

    #[test]
    fn test_detect_file_type_audio_fonts_and_textlike() {
        assert_eq!(detect_file_type(Path::new("x.wav")), Some("Audio"));
        assert_eq!(detect_file_type(Path::new("x.MP3")), Some("Audio"));
        assert_eq!(detect_file_type(Path::new("x.flac")), Some("Audio"));
        assert_eq!(detect_file_type(Path::new("x.ipynb")), Some("Notebook"));
        assert_eq!(detect_file_type(Path::new("x.proto")), Some("Protobuf"));
        assert_eq!(detect_file_type(Path::new("x.gql")), Some("GraphQL"));
        assert_eq!(detect_file_type(Path::new("x.thrift")), Some("Thrift"));
        assert_eq!(detect_file_type(Path::new("x.r")), Some("R"));
        assert_eq!(detect_file_type(Path::new("x.jl")), Some("Julia"));
        assert_eq!(detect_file_type(Path::new("x.mm")), Some("Objective-C++"));
        assert_eq!(detect_file_type(Path::new("x.ttf")), Some("Font"));
        assert_eq!(detect_file_type(Path::new("x.woff2")), Some("Font"));
    }

    #[test]
    fn test_detect_file_type_special_filenames() {
        assert_eq!(detect_file_type(Path::new("LICENSE")), Some("License"));
        assert_eq!(
            detect_file_type(Path::new("Dockerfile")),
            Some("Build Script")
        );
        assert_eq!(
            detect_file_type(Path::new("Makefile")),
            Some("Build Script")
        );
        assert_eq!(detect_file_type(Path::new("CMakeLists.txt")), Some("CMake"));
    }

    #[test]
    fn test_detect_file_type_installer_scripts() {
        assert_eq!(
            detect_file_type(Path::new("setup.iss")),
            Some("Installer Script")
        );
        assert_eq!(
            detect_file_type(Path::new("SETUP.ISS")),
            Some("Installer Script")
        );
    }

    #[test]
    fn test_detect_file_type_lockfiles() {
        assert_eq!(detect_file_type(Path::new("Cargo.lock")), Some("Lockfile"));
        assert_eq!(
            detect_file_type(Path::new("Gemfile.lock")),
            Some("Lockfile")
        );
        assert_eq!(detect_file_type(Path::new("yarn.lock")), Some("Lockfile"));
    }

    #[test]
    fn test_scan_source_files_respects_size_cap() {
        let dir = tempdir().unwrap();
        let d = dir.path();
        // small recognized file
        let mut f_small = File::create(d.join("small.wav")).unwrap();
        f_small.write_all(&vec![0u8; 1024]).unwrap(); // 1 KB

        // large recognized file (~2 MB)
        let mut f_large = File::create(d.join("large.mp3")).unwrap();
        f_large.write_all(&vec![1u8; 2 * 1024 * 1024]).unwrap();

        // cap = 1 MB
        let (files, count) = scan_source_files(d.to_str().unwrap(), 1).unwrap();
        let names: Vec<String> = files
            .iter()
            .map(|p| p.file_name().unwrap().to_string_lossy().to_string())
            .collect();
        assert_eq!(count, 1);
        assert!(names.contains(&"small.wav".to_string()));
        assert!(!names.contains(&"large.mp3".to_string()));
    }

    #[test]
    fn test_scan_respects_gitignore() {
        let dir = tempdir().unwrap();
        let d = dir.path();
        // Recognized file we will ignore via .gitignore
        std::fs::write(d.join("README.md"), b"Ignored doc").unwrap();
        std::fs::write(d.join(".gitignore"), b"# ignore readme\nREADME.md\n").unwrap();
        // Another recognized file that should remain
        std::fs::write(
            d.join("Cargo.toml"),
            b"[package]\nname='x'\nversion='0.1.0'\n",
        )
        .unwrap();

        let (files, _count) = scan_source_files(d.to_str().unwrap(), 50).unwrap();
        let names: Vec<String> = files
            .iter()
            .map(|p| p.file_name().unwrap().to_string_lossy().to_string())
            .collect();
        assert!(names.contains(&"Cargo.toml".to_string()));
        assert!(!names.contains(&"README.md".to_string()));
    }

    #[test]
    fn test_scan_includes_lockfiles() {
        let dir = tempdir().unwrap();
        let d = dir.path();
        std::fs::write(d.join("Cargo.lock"), b"[[package]]").unwrap();
        std::fs::write(d.join("Gemfile.lock"), b"GEM\n").unwrap();

        let (files, _count) = scan_source_files(d.to_str().unwrap(), 50).unwrap();
        let names: Vec<String> = files
            .iter()
            .map(|p| p.file_name().unwrap().to_string_lossy().to_string())
            .collect();
        assert!(names.contains(&"Cargo.lock".to_string()));
        assert!(names.contains(&"Gemfile.lock".to_string()));
    }

    #[test]
    fn test_scan_ignores_target_ci() {
        let dir = tempdir().unwrap();
        let d = dir.path();
        // Simulate Rust CI build artifact under target_ci
        let fp = d.join("target_ci").join("debug").join(".fingerprint");
        std::fs::create_dir_all(&fp).unwrap();
        std::fs::write(fp.join("lib-anyhow.json"), b"{}").unwrap();
        // A legitimate source/config file in the root
        std::fs::write(
            d.join("Cargo.toml"),
            b"[package]\nname='x'\nversion='0.1.0'\n",
        )
        .unwrap();

        let (files, _count) = scan_source_files(d.to_str().unwrap(), 50).unwrap();
        let names: Vec<String> = files
            .iter()
            .map(|p| p.strip_prefix(d).unwrap().to_string_lossy().to_string())
            .collect();
        assert!(names.iter().any(|n| n == "Cargo.toml"));
        assert!(
            !names
                .iter()
                .any(|n| n.contains("target_ci") || n.contains(".fingerprint")),
            "should ignore files under target_ci"
        );
    }
}

/// Returns true if any component of the entry's path is an excluded directory.
///
/// The tool ignores common build and virtual environment folders: `target`,
/// `target_ci` (Rust CI artifacts), `bin`, `obj`, `venv`, `.venv`, and `env`.
fn is_in_excluded_path(path: &Path) -> bool {
    path.components()
        .any(|comp| match comp.as_os_str().to_str() {
            Some("target") | Some("target_ci") => true,
            Some("bin") | Some("obj") => true,
            Some("venv") | Some(".venv") | Some("env") => true,
            // Always skip VCS metadata directories if encountered during a walk.
            Some(".git") | Some(".hg") | Some(".svn") => true,
            _ => false,
        })
}

/// Create a new repository and make an initial commit.
fn new_repository(dir: &str, dry_run: bool, max_file_mb: u64) -> Result<(), Box<dyn Error>> {
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
    let (source_files, _source_count) = scan_source_files(dir, max_file_mb)?;

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
        let (signature, sig_src) = resolve_signature_with_source(&repo)?;
        log::info!(
            "Using Git author: {} <{}> (source: {})",
            signature.name().unwrap_or("(unknown)"),
            signature.email().unwrap_or("(unknown)"),
            sig_src
        );
        repo.commit(
            Some("HEAD"),
            &signature,
            &signature,
            "Initial commit",
            &tree,
            &[],
        )?;
        count
    };

    log::info!(
        "{}New files added:{} {}",
        BLUE,
        RESET,
        source_files
            .iter()
            .map(|p| format!("{}{}{}", GREEN, p.to_string_lossy(), RESET))
            .collect::<Vec<String>>()
            .join(", ")
    );
    log::info!(
        "{}Final result:{} {}{} source files added out of {} total files{}",
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
fn update_repository(
    dir: &str,
    dry_run: bool,
    commit_msg: Option<&str>,
    max_file_mb: u64,
) -> Result<(), Box<dyn Error>> {
    let repo = match Repository::open(dir) {
        Ok(r) => r,
        Err(_) => {
            log::error!(
                "{}Error:{} No git repository in directory '{}'",
                BLUE,
                RESET,
                dir
            );
            return Err("No git repository".into());
        }
    };
    log::info!("Staging changes...");
    let (source_files, _) = scan_source_files(dir, max_file_mb)?;
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
                }
                Delta::Deleted => {
                    if let Some(path) = delta.old_file().path() {
                        changed_files.push(format!("{}{}{}", RED, path.to_string_lossy(), RESET));
                    }
                }
                _ => {
                    if let Some(path) = delta.new_file().path().or(delta.old_file().path()) {
                        changed_files.push(path.to_string_lossy().to_string());
                    }
                }
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
        let (signature, sig_src) = resolve_signature_with_source(&repo)?;
        log::info!(
            "Using Git author: {} <{}> (source: {})",
            signature.name().unwrap_or("(unknown)"),
            signature.email().unwrap_or("(unknown)"),
            sig_src
        );
        repo.commit(
            Some("HEAD"),
            &signature,
            &signature,
            &final_message,
            &new_tree,
            &[&parent_commit],
        )?;
    }
    log::info!(
        "{}{} changes staged and committed.{}",
        YELLOW,
        changed_files.len(),
        RESET
    );
    Ok(())
}

/// Scan the entire directory tree and count total files, skipping any entries under excluded directories.
fn scan_total_files(dir: &str) -> Result<usize, Box<dyn Error>> {
    log::debug!("Scanning source tree in '{}'...", dir);
    let mut total = 0;
    // Build a local .gitignore matcher (best-effort); ignore walker should
    // already respect .gitignore, but we also guard in-code to be explicit.
    let gi = {
        let mut b = GitignoreBuilder::new(dir);
        let _ = b.add(Path::new(dir).join(".gitignore"));
        b.build().ok()
    };
    for result in IgnoreWalkBuilder::new(dir)
        .hidden(false)
        .git_ignore(true)
        .git_global(true)
        .git_exclude(true)
        .ignore(true)
        .build()
    {
        let entry = match result {
            Ok(e) => e,
            Err(_) => continue,
        };
        let path = entry.path();
        if is_in_excluded_path(path) {
            continue;
        }
        if let Some(ref m) = gi {
            if m.matched_path_or_any_parents(
                path,
                entry.file_type().map(|ft| ft.is_dir()).unwrap_or(false),
            )
            .is_ignore()
            {
                continue;
            }
        }
        if entry.file_type().map(|ft| ft.is_file()).unwrap_or(false) {
            total += 1;
        }
    }
    log::debug!("Scan complete - found {} files", total);
    Ok(total)
}

/// Scan for source files (ignoring files under excluded directories).
fn scan_source_files(dir: &str, max_file_mb: u64) -> Result<(Vec<PathBuf>, usize), Box<dyn Error>> {
    log::debug!("Scanning for source files in '{}'...", dir);
    let mut source_files = Vec::new();
    let mut count = 0;
    let cap_bytes: u64 = max_file_mb.saturating_mul(1024).saturating_mul(1024);
    let gi = {
        let mut b = GitignoreBuilder::new(dir);
        let _ = b.add(Path::new(dir).join(".gitignore"));
        b.build().ok()
    };
    for result in IgnoreWalkBuilder::new(dir)
        .hidden(false)
        .git_ignore(true)
        .git_global(true)
        .git_exclude(true)
        .ignore(true)
        .build()
    {
        let entry = match result {
            Ok(e) => e,
            Err(_) => continue,
        };
        let path = entry.path();
        if is_in_excluded_path(path) {
            continue;
        }
        if entry.file_type().map(|ft| ft.is_file()).unwrap_or(false) {
            if let Some(ref m) = gi {
                if m.matched_path_or_any_parents(path, false).is_ignore() {
                    continue;
                }
            }
            if detect_file_type(path).is_some() {
                if let Ok(meta) = fs::metadata(path) {
                    if meta.len() > cap_bytes {
                        log::info!(
                            "Ignoring '{}' as larger than {} MB - use '--max-file-mb'",
                            path.display(),
                            max_file_mb
                        );
                        continue;
                    }
                }
                source_files.push(path.to_path_buf());
                count += 1;
            }
        }
    }
    log::debug!("{} source files found", count);
    Ok((source_files, count))
}

/// Add the provided source files to the Git index.
fn add_files_to_git(dir: &str, files: &[PathBuf], dry_run: bool) -> Result<usize, Box<dyn Error>> {
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
fn get_last_commit(repo: &Repository) -> Result<git2::Commit<'_>, Box<dyn Error>> {
    let obj = repo.head()?.resolve()?.peel(ObjectType::Commit)?;
    let commit = obj.into_commit().map_err(|_| "Couldn't find commit")?;
    Ok(commit)
}

/// Retrieve a commit by index (0 is most recent, 1 is next, etc.).
fn get_commit_by_index(repo: &Repository, idx: i32) -> Result<git2::Commit<'_>, Box<dyn Error>> {
    let mut revwalk = repo.revwalk()?;
    revwalk.push_head()?;
    revwalk.set_sorting(Sort::TIME)?;
    let commits: Vec<_> = revwalk.collect::<Result<Vec<_>, _>>()?;
    if (idx as usize) < commits.len() {
        repo.find_commit(commits[idx as usize])
            .map_err(|e| e.into())
    } else {
        Err("Index out of bounds".into())
    }
}

/// Resolve the Git signature (name/email) and describe its source for logging.
fn resolve_signature_with_source(
    repo: &Repository,
) -> Result<(Signature<'_>, String), Box<dyn Error>> {
    if let (Ok(name), Ok(email)) = (
        std::env::var("GIT_AUTHOR_NAME"),
        std::env::var("GIT_AUTHOR_EMAIL"),
    ) {
        return Ok((
            Signature::now(&name, &email)?,
            "env:GIT_AUTHOR_NAME/GIT_AUTHOR_EMAIL".into(),
        ));
    }
    if let (Ok(name), Ok(email)) = (
        std::env::var("GIT_COMMITTER_NAME"),
        std::env::var("GIT_COMMITTER_EMAIL"),
    ) {
        return Ok((
            Signature::now(&name, &email)?,
            "env:GIT_COMMITTER_NAME/GIT_COMMITTER_EMAIL".into(),
        ));
    }

    if let Ok(cfg) = repo.config() {
        let name = cfg.get_string("user.name").ok();
        let email = cfg.get_string("user.email").ok();
        if let (Some(name), Some(email)) = (name, email) {
            return Ok((
                Signature::now(&name, &email)?,
                "git config (repo/global)".into(),
            ));
        }
    }
    if let Ok(cfg) = git2::Config::open_default() {
        let name = cfg.get_string("user.name").ok();
        let email = cfg.get_string("user.email").ok();
        if let (Some(name), Some(email)) = (name, email) {
            return Ok((Signature::now(&name, &email)?, "git config (global)".into()));
        }
    }

    Ok((
        Signature::now("mdcode", "mdcode@example.com")?,
        "mdcode fallback".into(),
    ))
}

/// Retrieve the commit pointed to by the remote HEAD on GitHub.
fn get_remote_head_commit<'repo>(
    repo: &'repo Repository,
    dir: &str,
) -> Result<git2::Commit<'repo>, Box<dyn Error>> {
    // Ensure the remote exists.
    repo.find_remote("origin")
        .map_err(|_| "Remote 'origin' not found")?;

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

    // Try the symbolic origin/HEAD reference first.
    let head_ref = match repo.find_reference("refs/remotes/origin/HEAD") {
        Ok(r) => r,
        Err(_) => {
            // Fallback: determine the default branch via `git remote show origin`.
            let output = Command::new("git")
                .arg("-C")
                .arg(dir)
                .arg("remote")
                .arg("show")
                .arg("origin")
                .output()?;
            if !output.status.success() {
                return Err("git remote show origin failed".into());
            }
            let stdout = String::from_utf8_lossy(&output.stdout);
            let branch = stdout
                .lines()
                .find(|l| l.trim_start().starts_with("HEAD branch:"))
                .and_then(|l| l.split(':').nth(1))
                .map(|b| b.trim())
                .ok_or("Unable to determine default branch on origin")?;
            let ref_name = format!("refs/remotes/origin/{}", branch);
            repo.find_reference(&ref_name)?
        }
    };

    // origin/HEAD should normally be a symbolic ref to the default branch.
    // However some remotes may create it as a direct ref to a commit.
    // Try symbolic target first, falling back to the direct target if needed.
    if let Some(target) = head_ref.symbolic_target() {
        let branch_ref = repo.find_reference(target)?;
        let oid = branch_ref.target().ok_or("Remote HEAD has no target")?;
        repo.find_commit(oid).map_err(|e| e.into())
    } else if let Some(oid) = head_ref.target() {
        // origin/HEAD points directly to a commit
        repo.find_commit(oid).map_err(|e| e.into())
    } else {
        Err("origin/HEAD has no target".into())
    }
}

/// Diff commits based on provided version numbers.
fn diff_command(dir: &str, versions: &[String], dry_run: bool) -> Result<(), Box<dyn Error>> {
    let repo = Repository::open(dir)?;
    let before_commit = if (versions.len() == 2 && versions[0].eq_ignore_ascii_case("H"))
        || (versions.len() == 1 && versions[0].eq_ignore_ascii_case("L"))
    {
        get_remote_head_commit(&repo, dir)?
    } else {
        let idx = if versions.is_empty() {
            0
        } else {
            versions[0]
                .parse::<i32>()
                .map_err(|_| "invalid repo indexes specified")?
        };
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

    let (after_dir, after_timestamp_str) =
        if versions.len() == 1 && versions[0].to_uppercase() == "L" {
            (PathBuf::from(dir), "current".to_string())
        } else if versions.len() == 2 {
            if versions[0].to_uppercase() == "H" {
                let idx = versions[1]
                    .parse::<i32>()
                    .map_err(|_| "invalid repo indexes specified")?;
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
                let idx = versions[1]
                    .parse::<i32>()
                    .map_err(|_| "invalid repo indexes specified")?;
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

    log::info!(
        "{}Comparing {} with {}{}",
        YELLOW,
        before_timestamp,
        after_timestamp_str,
        RESET
    );

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
        }
        Err(e) => {
            log::warn!(
                "WinMergeU.exe failed to launch: {}. Trying windiff.exe...",
                e
            );
            match Command::new("windiff.exe").arg(before).arg(after).spawn() {
                Ok(_) => {
                    log::info!("Launched windiff.exe.");
                    Ok(())
                }
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
    // Recognize special filenames without extensions.
    if let Some(file_name) = file_path.file_name()?.to_str() {
        if file_name.eq_ignore_ascii_case("LICENSE") {
            return Some("License");
        }
        if file_name.eq_ignore_ascii_case("Dockerfile") {
            return Some("Build Script");
        }
        if file_name.eq_ignore_ascii_case("Makefile") {
            return Some("Build Script");
        }
        if file_name.eq_ignore_ascii_case("CMakeLists.txt") {
            return Some("CMake");
        }
    }

    let extension = file_path.extension()?.to_str()?.to_lowercase();
    match extension.as_str() {
        // Source Code
        "c" => Some("C"),
        "cpp" | "cc" | "cxx" => Some("C++"),
        "h" => Some("C/C++ Header"),
        "hpp" | "hh" | "hxx" => Some("C++ Header"),
        "java" => Some("Java"),
        "py" => Some("Python"),
        "rb" => Some("Ruby"),
        "cs" => Some("C#"),
        "go" => Some("Go"),
        "php" => Some("PHP"),
        "rs" => Some("Rust"),
        "swift" => Some("Swift"),
        "kt" | "kts" => Some("Kotlin"),
        "scala" => Some("Scala"),
        "js" | "jsx" => Some("JavaScript"),
        "ts" | "tsx" => Some("TypeScript"),
        "sh" | "bash" | "zsh" => Some("Shell Script"),
        "bat" => Some("Batch Script"),
        "ps1" => Some("PowerShell"),
        // Additional languages / build systems
        "r" => Some("R"),
        "jl" => Some("Julia"),
        "mm" => Some("Objective-C++"),
        "cmake" => Some("CMake"),
        // APIs / IDL
        "proto" => Some("Protobuf"),
        "graphql" | "gql" => Some("GraphQL"),
        "thrift" => Some("Thrift"),
        // Markup / Documentation
        "html" | "htm" => Some("HTML"),
        "css" | "scss" | "sass" | "less" => Some("CSS"),
        "xml" => Some("XML"),
        "json" => Some("JSON"),
        "yml" | "yaml" => Some("YAML"),
        "toml" => Some("TOML"),
        "lock" => Some("Lockfile"),
        "md" | "txt" | "rst" | "adoc" => Some("Documentation"),
        "ipynb" => Some("Notebook"),
        // Configuration / Build
        "ini" | "cfg" | "conf" => Some("Configuration"),
        "sln" => Some("Solution File"),
        "csproj" => Some("C# Project File"),
        "pom" => Some("Maven Project File"),
        "gradle" => Some("Gradle Build File"),
        // Installer scripts
        "iss" => Some("Installer Script"),
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
        // Audio
        "wav" | "mp3" | "flac" | "aac" | "m4a" | "ogg" | "opus" | "aiff" | "aif" | "wma"
        | "mid" | "midi" => Some("Audio"),
        // Fonts
        "ttf" | "otf" | "woff" | "woff2" => Some("Font"),
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
        if e.message()
            .contains("reference 'refs/heads/master' not found")
            || e.message()
                .contains("reference 'refs/heads/main' not found")
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
                    }
                    Delta::Deleted => {
                        if let Some(path) = delta.old_file().path() {
                            file_list.push(format!("{}{}{}", RED, path.to_string_lossy(), RESET));
                        }
                    }
                    _ => {
                        if let Some(path) = delta.new_file().path().or(delta.old_file().path()) {
                            file_list.push(path.to_string_lossy().to_string());
                        }
                    }
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
        log::info!(
            "{}{} {} | {}M:{} {} | {}F:{} {}{}",
            YELLOW,
            idx_str,
            formatted_time,
            BLUE,
            RESET,
            summary,
            BLUE,
            RESET,
            file_list.join(", "),
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
    let ignore_patterns = [
        // Rust/Cargo
        "target/",
        // CI builds for Rust that should never be checked in
        "target_ci/",
        // Generic build outputs and environments
        "bin/",
        "obj/",
        "venv/",
        ".venv/",
        "env/",
        // Common temporary/log files
        "*.tmp",
        "*.log",
    ];
    Ok(ignore_patterns.join("\n"))
}

/// Recursively check out a Git tree into the target directory.
fn checkout_tree_to_dir(
    repo: &Repository,
    tree: &git2::Tree,
    target: &Path,
) -> Result<(), Box<dyn Error>> {
    fs::create_dir_all(target)?;
    for entry in tree.iter() {
        let name = entry.name().ok_or("Invalid UTF-8 in filename")?;
        let entry_path = target.join(name);
        match entry.kind() {
            Some(git2::ObjectType::Tree) => {
                let subtree = repo.find_tree(entry.id())?;
                checkout_tree_to_dir(repo, &subtree, &entry_path)?;
            }
            Some(git2::ObjectType::Blob) => {
                let blob = repo.find_blob(entry.id())?;
                let mut file = File::create(&entry_path)?;
                file.write_all(blob.content())?;
            }
            _ => {}
        }
    }
    Ok(())
}

/// Create a temporary directory with the given prefix.
fn create_temp_dir(prefix: &str) -> Result<PathBuf, Box<dyn Error>> {
    let mut base = env::temp_dir();
    let unique = format!(
        "{}",
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)?
            .as_nanos()
    );
    base.push(format!("{}.{}", prefix, unique));
    fs::create_dir_all(&base)?;
    Ok(base)
}

/// Create a GitHub repository using the GitHub API.
///
/// Tries `GITHUB_TOKEN` then `GH_TOKEN`. If neither is set, returns a helpful error
/// suggesting to authenticate the GitHub CLI or set a token.
/// Returns the created repository.
async fn gh_create_api(
    name: &str,
    description: Option<String>,
) -> Result<octocrab::models::Repository, Box<dyn std::error::Error>> {
    let token = std::env::var("GITHUB_TOKEN")
        .or_else(|_| std::env::var("GH_TOKEN"))
        .map_err(|_| {
            "GitHub token not found. Install and authenticate GitHub CLI (`gh auth login`) \
or set GITHUB_TOKEN/GH_TOKEN with repo scope."
                .to_string()
        })?;
    let octocrab = octocrab::Octocrab::builder()
        .personal_token(token)
        .build()?;

    // Identify the GitHub user tied to the token without exposing the token.
    let me: serde_json::Value = octocrab.get("/user", None::<&()>).await?;
    let login = me
        .get("login")
        .and_then(|v| v.as_str())
        .unwrap_or("(unknown)");
    let email = me
        .get("email")
        .and_then(|v| v.as_str())
        .unwrap_or("(hidden or null)");
    println!(
        "GitHub auth: login '{}' (email: {}) via env:GITHUB_TOKEN",
        login, email
    );

    // POST to /user/repos with a JSON payload containing "name" and "description"
    let repo: octocrab::models::Repository = octocrab
        .post(
            "/user/repos",
            Some(&serde_json::json!( {
                "name": name,
                "description": description.unwrap_or_default()
            })),
        )
        .await?;
    println!("Created GitHub repository: {}", repo.html_url);
    Ok(repo)
}

/// Locate the GitHub CLI executable if available.
/// Returns a path to use when invoking the command.
fn gh_cli_path() -> Option<std::path::PathBuf> {
    use std::path::PathBuf;

    // 1) Try the name via PATH first.
    if let Ok(out) = Command::new("gh").arg("--version").output() {
        if out.status.success() {
            return Some(PathBuf::from("gh"));
        }
    }

    // 2) On Windows, try `where gh` and typical install directories.
    #[cfg(windows)]
    {
        if let Ok(out) = Command::new("where").arg("gh").output() {
            if out.status.success() {
                let txt = String::from_utf8_lossy(&out.stdout);
                if let Some(first) = txt.lines().find(|l| !l.trim().is_empty()) {
                    let p = Path::new(first.trim());
                    if p.exists() {
                        return Some(p.to_path_buf());
                    }
                }
            }
        }

        // Try LocalAppData user install: %LOCALAPPDATA%\Programs\GitHub CLI\gh.exe
        if let Ok(local) = std::env::var("LOCALAPPDATA") {
            let p = Path::new(&local)
                .join("Programs")
                .join("GitHub CLI")
                .join("gh.exe");
            if p.exists() {
                return Some(p);
            }
        }

        // Try Program Files (x86) and Program Files locations.
        for var in ["ProgramFiles(x86)", "ProgramFiles"] {
            if let Ok(base) = std::env::var(var) {
                let p = Path::new(&base).join("GitHub CLI").join("gh.exe");
                if p.exists() {
                    return Some(p);
                }
            }
        }

        // Fallback to the canonical Program Files path if env vars are missing.
        let default_path = Path::new("C:\\Program Files\\GitHub CLI\\gh.exe");
        if default_path.exists() {
            return Some(default_path.to_path_buf());
        }
    }

    None
}

/// Create a GitHub repository using GitHub CLI and the system's authenticated credentials.
/// This mirrors the existing flow by creating from the local directory, setting `origin`, and pushing.
fn gh_create_via_cli(
    gh_cmd: &std::path::Path,
    directory: &str,
    name: &str,
    description: Option<String>,
    visibility: RepoVisibility,
) -> Result<(), Box<dyn std::error::Error>> {
    let mut args = vec![
        "repo", "create", name, "--source", directory, "--remote", "origin", "--push",
    ];
    // Respect user default visibility; include description if provided.
    if let Some(desc) = description.as_deref() {
        args.push("--description");
        args.push(desc);
    }
    match visibility {
        RepoVisibility::Public => args.push("--public"),
        RepoVisibility::Private => args.push("--private"),
        RepoVisibility::Internal => args.push("--internal"),
    }
    let status = Command::new(gh_cmd).args(&args).status()?;
    if !status.success() {
        return Err("GitHub CLI 'gh repo create' failed".into());
    }
    println!("Created GitHub repository via GitHub CLI and pushed to 'origin'.");
    Ok(())
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
fn remote_branch_exists(
    directory: &str,
    remote: &str,
    branch: &str,
) -> Result<bool, Box<dyn Error>> {
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
    let (sig, src) = resolve_signature_with_source(&repo)?;
    let remote_url = repo
        .find_remote(remote)
        .ok()
        .and_then(|r| r.url().map(|s| s.to_string()))
        .unwrap_or_else(|| "(unknown)".into());
    println!(
        "Using Git author: {} <{}> (source: {}) | remote: {}",
        sig.name().unwrap_or("(unknown)"),
        sig.email().unwrap_or("(unknown)"),
        src,
        remote_url
    );
    let head = repo.head()?;
    let branch = head.shorthand().unwrap_or("master");

    // Check if the remote branch exists.
    let branch_exists = remote_branch_exists(directory, remote, branch)?;

    if branch_exists {
        println!(
            "Auto-pulling changes from remote '{}' for branch '{}'",
            remote, branch
        );
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

    println!(
        "Pushing local repository '{}' to remote '{}'",
        directory, remote
    );
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

/// Fetch changes from the remote and list commits not yet merged.
fn gh_fetch(directory: &str, remote: &str) -> Result<(), Box<dyn std::error::Error>> {
    let repo = Repository::open(directory)?;
    let (sig, src) = resolve_signature_with_source(&repo)?;
    let remote_url = repo
        .find_remote(remote)
        .ok()
        .and_then(|r| r.url().map(|s| s.to_string()))
        .unwrap_or_else(|| "(unknown)".into());
    println!(
        "Fetching from '{}' ({}) using Git author: {} <{}> (source: {})",
        remote,
        remote_url,
        sig.name().unwrap_or("(unknown)"),
        sig.email().unwrap_or("(unknown)"),
        src
    );
    let status = Command::new("git")
        .arg("-C")
        .arg(directory)
        .arg("fetch")
        .arg(remote)
        .status()?;
    if !status.success() {
        return Err("git fetch failed".into());
    }

    let head = repo.head()?;
    let branch = head.shorthand().ok_or("HEAD does not point to a branch")?;

    // Only show logs if the remote branch exists
    if !remote_branch_exists(directory, remote, branch)? {
        println!("Remote branch '{}/{}' does not exist.", remote, branch);
        return Ok(());
    }

    let output = Command::new("git")
        .arg("-C")
        .arg(directory)
        .arg("log")
        .arg("--oneline")
        .arg(format!("HEAD..{}/{}", remote, branch))
        .output()?;
    if !output.status.success() {
        return Err("git log failed".into());
    }

    let text = String::from_utf8_lossy(&output.stdout);
    if text.trim().is_empty() {
        println!("Local repository is up to date with remote.");
    } else {
        println!("Commits available on remote:");
        print!("{}", text);
    }
    Ok(())
}

/// Pull changes from the remote to synchronize the local repository.
fn gh_sync(directory: &str, remote: &str) -> Result<(), Box<dyn std::error::Error>> {
    let repo = Repository::open(directory)?;
    let (sig, src) = resolve_signature_with_source(&repo)?;
    let remote_url = repo
        .find_remote(remote)
        .ok()
        .and_then(|r| r.url().map(|s| s.to_string()))
        .unwrap_or_else(|| "(unknown)".into());
    println!(
        "Syncing with '{}' ({}) using Git author: {} <{}> (source: {})",
        remote,
        remote_url,
        sig.name().unwrap_or("(unknown)"),
        sig.email().unwrap_or("(unknown)"),
        src
    );
    let head = repo.head()?;
    let branch = head.shorthand().unwrap_or("master");

    let exists = remote_branch_exists(directory, remote, branch)?;
    if !exists {
        println!("Remote branch '{}' does not exist. Skipping sync.", branch);
        return Ok(());
    }

    println!(
        "Pulling changes from remote '{}' for branch '{}'",
        remote, branch
    );
    let status = Command::new("git")
        .arg("-C")
        .arg(directory)
        .arg("pull")
        .arg(remote)
        .arg(branch)
        .status()?;
    if status.success() {
        println!("Repository synchronized with remote.");
        Ok(())
    } else {
        Err("git pull failed".into())
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
        assert_eq!(
            detect_file_type(Path::new("test.bash")),
            Some("Shell Script")
        );
        assert_eq!(
            detect_file_type(Path::new("test.zsh")),
            Some("Shell Script")
        );
        assert_eq!(
            detect_file_type(Path::new("test.bat")),
            Some("Batch Script")
        );
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
        assert_eq!(
            detect_file_type(Path::new("README.md")),
            Some("Documentation")
        );
        assert_eq!(
            detect_file_type(Path::new("notes.txt")),
            Some("Documentation")
        );
        assert_eq!(
            detect_file_type(Path::new("manual.rst")),
            Some("Documentation")
        );
        assert_eq!(
            detect_file_type(Path::new("guide.adoc")),
            Some("Documentation")
        );

        // Configuration / Build
        assert_eq!(
            detect_file_type(Path::new("settings.ini")),
            Some("Configuration")
        );
        assert_eq!(
            detect_file_type(Path::new("config.cfg")),
            Some("Configuration")
        );
        assert_eq!(
            detect_file_type(Path::new("app.conf")),
            Some("Configuration")
        );
        assert_eq!(
            detect_file_type(Path::new("project.sln")),
            Some("Solution File")
        );
        assert_eq!(
            detect_file_type(Path::new("app.csproj")),
            Some("C# Project File")
        );
        assert_eq!(detect_file_type(Path::new("pom.xml")), Some("XML")); // Note: Maven's pom.xml is XML
        assert_eq!(
            detect_file_type(Path::new("build.gradle")),
            Some("Gradle Build File")
        );

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
        assert_eq!(
            detect_file_type(Path::new("vector.svg")),
            Some("Vector Image")
        );
        assert_eq!(detect_file_type(Path::new("icon.ico")), Some("Icon"));
        assert_eq!(detect_file_type(Path::new("cursor.cur")), Some("Cursor"));
        // Other asset
        assert_eq!(
            detect_file_type(Path::new("dialog.dlg")),
            Some("Dialog File")
        );
    }

    #[test]
    fn test_generate_gitignore_content() {
        let content = generate_gitignore_content(".").unwrap();
        let expected = "target/\ntarget_ci/\nbin/\nobj/\nvenv/\n.venv/\nenv/\n*.tmp\n*.log";
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
        new_repository(repo_str, false, 50).unwrap();
        assert!(
            Path::new(repo_str).join(".git").exists(),
            ".git directory should exist"
        );
        assert!(
            Path::new(repo_str).join(".gitignore").exists(),
            ".gitignore file should exist"
        );
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
        new_repository(repo_str, false, 50).unwrap();
        let file_path = repo_path.join("new_file.txt");
        fs::write(&file_path, "Hello, mdcode!").unwrap();
        // Provide a commit message to avoid hanging.
        update_repository(repo_str, false, Some("Test commit message"), 50).unwrap();
        let repo = Repository::open(repo_str).unwrap();
        let mut revwalk = repo.revwalk().unwrap();
        revwalk.push_head().unwrap();
        let commits: Vec<_> = revwalk.collect();
        assert!(
            commits.len() >= 2,
            "Repository should have at least two commits"
        );
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
        new_repository(repo_str, false, 50).unwrap();
        let file_path = repo_path.join("info_test.txt");
        fs::write(&file_path, "Test info output").unwrap();
        update_repository(repo_str, false, Some("Test commit message"), 50).unwrap();
        info_repository(repo_str).unwrap();
    }
}
