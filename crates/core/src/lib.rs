//! Read-only repository discovery and Git metadata collection for RepoShelf.
//! All inspection APIs deliberately avoid changing repositories or accessing the network.

use anyhow::{Context, Result};
use chrono::{DateTime, Utc};
use git2::{Repository, Status, StatusOptions};
use serde::Serialize;
use std::{
    collections::HashSet,
    fs,
    path::{Path, PathBuf},
};
use walkdir::{DirEntry, WalkDir};

const DEFAULT_EXCLUSIONS: &[&str] = &[
    "node_modules",
    "vendor",
    "target",
    ".next",
    "dist",
    "build",
    ".cache",
    ".git",
];
const GENERATED_DIRS: &[&str] = &[
    "node_modules",
    "vendor",
    "target",
    ".next",
    "dist",
    "build",
    "coverage",
    ".venv",
    "venv",
    "__pycache__",
    ".turbo",
];

#[derive(Debug, Clone, Serialize)]
pub struct ScanOptions {
    pub include_generated: bool,
    pub max_depth: usize,
}

impl Default for ScanOptions {
    fn default() -> Self {
        Self {
            include_generated: false,
            max_depth: 32,
        }
    }
}

#[derive(Debug, Clone, Default, Serialize)]
pub struct ChangeCounts {
    pub modified: usize,
    pub added: usize,
    pub deleted: usize,
    pub untracked: usize,
    pub staged: usize,
}

#[derive(Debug, Clone, Serialize)]
pub struct DiskBreakdown {
    pub total_bytes: u64,
    pub git_bytes: u64,
    pub generated: Vec<DirectorySize>,
}

#[derive(Debug, Clone, Serialize)]
pub struct DirectorySize {
    pub name: String,
    pub bytes: u64,
}

#[derive(Debug, Clone, Serialize)]
pub struct RepoInfo {
    pub name: String,
    pub path: PathBuf,
    pub is_bare: bool,
    pub branch: Option<String>,
    pub detached_head: bool,
    pub head_commit: Option<String>,
    pub last_commit_at: Option<DateTime<Utc>>,
    pub remotes: Vec<String>,
    pub default_remote: Option<String>,
    pub dirty: bool,
    pub changes: ChangeCounts,
    pub ahead: Option<usize>,
    pub behind: Option<usize>,
    pub disk: DiskBreakdown,
    pub technologies: Vec<String>,
}

pub fn discover(roots: &[PathBuf], options: &ScanOptions) -> Result<Vec<PathBuf>> {
    let mut repos = HashSet::new();
    for root in roots {
        let root = root
            .canonicalize()
            .with_context(|| format!("cannot access scan root {}", root.display()))?;
        if is_repo(&root) {
            repos.insert(root.clone());
        }
        let walker = WalkDir::new(&root)
            .follow_links(false)
            .max_depth(options.max_depth)
            .into_iter()
            .filter_entry(|e| should_descend(e, options));
        for entry in walker.filter_map(Result::ok) {
            let p = entry.path();
            // Check a directory itself rather than yielding `.git`: `.git` remains
            // pruned below, which avoids walking every repository's object database.
            if entry.file_type().is_dir() && ((has_git_marker(p) && is_repo(p)) || is_bare_repo(p))
            {
                repos.insert(p.to_path_buf());
            }
        }
    }
    let mut repos: Vec<_> = repos.into_iter().collect();
    repos.sort();
    Ok(repos)
}

fn should_descend(entry: &DirEntry, options: &ScanOptions) -> bool {
    if entry.depth() == 0 || !entry.file_type().is_dir() {
        return true;
    }
    let name = entry.file_name().to_string_lossy();
    options.include_generated || !DEFAULT_EXCLUSIONS.iter().any(|x| *x == name)
}

fn is_repo(path: &Path) -> bool {
    Repository::open(path).is_ok()
}
fn has_git_marker(path: &Path) -> bool {
    let marker = path.join(".git");
    marker.is_dir() || marker.is_file()
}
fn is_bare_repo(path: &Path) -> bool {
    path.join("HEAD").is_file()
        && path.join("objects").is_dir()
        && Repository::open_bare(path).is_ok()
}

pub fn inspect(path: &Path) -> Result<RepoInfo> {
    let repo = Repository::open(path)
        .with_context(|| format!("cannot open repository {}", path.display()))?;
    let workdir = repo.workdir().unwrap_or(path);
    let head = repo.head().ok();
    let detached_head = repo.head_detached().unwrap_or(false);
    let branch = head
        .as_ref()
        .and_then(|h| h.shorthand())
        .map(str::to_owned)
        .filter(|_| !detached_head);
    let head_commit = head
        .as_ref()
        .and_then(|h| h.target())
        .map(|id| id.to_string());
    let last_commit_at = repo
        .head()
        .ok()
        .and_then(|h| h.peel_to_commit().ok())
        .and_then(|c| DateTime::from_timestamp(c.time().seconds(), 0));
    let remotes = repo
        .remotes()?
        .iter()
        .flatten()
        .map(str::to_owned)
        .collect::<Vec<_>>();
    let default_remote = if remotes.iter().any(|r| r == "origin") {
        Some("origin".into())
    } else {
        remotes.first().cloned()
    };
    let changes = changes(&repo)?;
    let (ahead, behind) = upstream_distance(&repo, head.as_ref());
    Ok(RepoInfo {
        name: workdir
            .file_name()
            .or_else(|| path.file_name())
            .unwrap_or_default()
            .to_string_lossy()
            .to_string(),
        path: workdir.to_path_buf(),
        is_bare: repo.is_bare(),
        branch,
        detached_head,
        head_commit,
        last_commit_at,
        remotes,
        default_remote,
        dirty: changes.modified + changes.added + changes.deleted + changes.untracked > 0,
        changes,
        ahead,
        behind,
        disk: disk_usage(workdir, repo.path()),
        technologies: technologies(workdir),
    })
}

fn changes(repo: &Repository) -> Result<ChangeCounts> {
    let mut options = StatusOptions::new();
    options
        .include_untracked(true)
        .recurse_untracked_dirs(true)
        .renames_head_to_index(true);
    let statuses = repo.statuses(Some(&mut options))?;
    let mut c = ChangeCounts::default();
    for s in statuses.iter() {
        let x = s.status();
        if x.intersects(Status::WT_MODIFIED | Status::INDEX_MODIFIED) {
            c.modified += 1;
        }
        if x.contains(Status::INDEX_NEW) {
            c.added += 1;
        }
        if x.intersects(Status::WT_DELETED | Status::INDEX_DELETED) {
            c.deleted += 1;
        }
        if x.contains(Status::WT_NEW) {
            c.untracked += 1;
        }
        if x.intersects(
            Status::INDEX_NEW
                | Status::INDEX_MODIFIED
                | Status::INDEX_DELETED
                | Status::INDEX_RENAMED
                | Status::INDEX_TYPECHANGE,
        ) {
            c.staged += 1;
        }
    }
    Ok(c)
}

fn upstream_distance(
    repo: &Repository,
    head: Option<&git2::Reference<'_>>,
) -> (Option<usize>, Option<usize>) {
    let Some(local) = head.and_then(|h| h.target()) else {
        return (None, None);
    };
    let Ok(upstream) = repo.branch_upstream_name(head.and_then(|h| h.name()).unwrap_or("")) else {
        return (None, None);
    };
    let Ok(reference) = repo.find_reference(upstream.as_str().unwrap_or("")) else {
        return (None, None);
    };
    let Some(remote) = reference.target() else {
        return (None, None);
    };
    repo.graph_ahead_behind(local, remote)
        .map(|(a, b)| (Some(a), Some(b)))
        .unwrap_or((None, None))
}

fn disk_usage(workdir: &Path, gitdir: &Path) -> DiskBreakdown {
    let total_bytes = tree_size(workdir);
    let git_bytes = tree_size(gitdir);
    let generated = GENERATED_DIRS
        .iter()
        .filter_map(|name| {
            let p = workdir.join(name);
            p.is_dir().then(|| DirectorySize {
                name: (*name).into(),
                bytes: tree_size(&p),
            })
        })
        .collect();
    DiskBreakdown {
        total_bytes,
        git_bytes,
        generated,
    }
}

fn tree_size(path: &Path) -> u64 {
    WalkDir::new(path)
        .follow_links(false)
        .into_iter()
        .filter_map(Result::ok)
        .filter_map(|e| e.metadata().ok())
        .filter(|m| m.is_file())
        .map(|m| m.len())
        .sum()
}
fn technologies(path: &Path) -> Vec<String> {
    [
        ("package.json", "Node.js / TypeScript"),
        ("Cargo.toml", "Rust"),
        ("pyproject.toml", "Python"),
        ("composer.json", "PHP"),
        ("go.mod", "Go"),
    ]
    .iter()
    .filter(|(f, _)| path.join(f).is_file())
    .map(|(_, n)| (*n).into())
    .collect()
}

pub fn format_bytes(bytes: u64) -> String {
    const UNITS: &[&str] = &["B", "KB", "MB", "GB", "TB"];
    let mut n = bytes as f64;
    let mut i = 0;
    while n >= 1024.0 && i < UNITS.len() - 1 {
        n /= 1024.0;
        i += 1;
    }
    format!("{n:.1} {}", UNITS[i])
}

pub fn fetch(path: &Path) -> Result<()> {
    let repo = Repository::open(path)?;
    let names = repo.remotes()?;
    for name in names.iter().flatten() {
        let mut remote = repo.find_remote(name)?;
        remote.fetch(&[] as &[&str], None, None)?;
    }
    Ok(())
}

pub fn path_exists(path: &Path) -> bool {
    fs::metadata(path).is_ok()
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::{
        fs,
        time::{SystemTime, UNIX_EPOCH},
    };

    fn temp_dir(label: &str) -> PathBuf {
        let nonce = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        // Keep fixtures inside the workspace target directory: sandboxed test
        // runners may deny access to the operating-system temporary folder.
        let workspace = Path::new(env!("CARGO_MANIFEST_DIR"))
            .parent()
            .and_then(Path::parent)
            .unwrap();
        let path = workspace
            .join("target")
            .join("reposhelf-tests")
            .join(format!("reposhelf-{label}-{nonce}"));
        fs::create_dir_all(&path).unwrap();
        path
    }

    #[test]
    fn discovers_and_inspects_a_dirty_repository_without_following_git() {
        let root = temp_dir("discovery");
        let repo_path = root.join("project");
        Repository::init(&repo_path).unwrap();
        fs::write(repo_path.join("untracked.txt"), "content").unwrap();

        let found = discover(std::slice::from_ref(&root), &ScanOptions::default()).unwrap();
        assert_eq!(found, vec![repo_path.canonicalize().unwrap()]);
        let info = inspect(&repo_path).unwrap();
        assert!(info.dirty);
        assert_eq!(info.changes.untracked, 1);
        assert_eq!(info.changes.added, 0);
        assert_eq!(info.name, "project");
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn reports_generated_directory_size() {
        let root = temp_dir("disk");
        Repository::init(&root).unwrap();
        let generated = root.join("target");
        fs::create_dir(&generated).unwrap();
        fs::write(generated.join("artifact"), vec![0u8; 4096]).unwrap();
        let info = inspect(&root).unwrap();
        assert!(info
            .disk
            .generated
            .iter()
            .any(|d| d.name == "target" && d.bytes == 4096));
        fs::remove_dir_all(root).unwrap();
    }
}
