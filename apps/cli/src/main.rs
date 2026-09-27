use anyhow::{bail, Result};
use clap::{Parser, Subcommand};
use reposhelf_core::{discover, fetch, format_bytes, inspect, path_exists, RepoInfo, ScanOptions};
use std::path::PathBuf;

#[derive(Parser)]
#[command(name = "reposhelf", about = "A local-first Git repository inventory")]
struct Cli {
    #[command(subcommand)]
    command: Command,
}
#[derive(Subcommand)]
enum Command {
    Scan {
        #[arg(default_value = ".")]
        roots: Vec<PathBuf>,
        #[arg(long)]
        include_generated: bool,
    },
    List {
        #[arg(default_value = ".")]
        roots: Vec<PathBuf>,
        #[arg(long)]
        json: bool,
    },
    Status {
        #[arg(default_value = ".")]
        roots: Vec<PathBuf>,
    },
    Dirty {
        #[arg(default_value = ".")]
        roots: Vec<PathBuf>,
    },
    Stale {
        #[arg(default_value = ".")]
        roots: Vec<PathBuf>,
        #[arg(long, default_value_t = 180)]
        days: i64,
    },
    Large {
        #[arg(default_value = ".")]
        roots: Vec<PathBuf>,
        #[arg(long, default_value_t = 1024)]
        min_mb: u64,
    },
    Fetch {
        path: PathBuf,
    },
    Repo {
        name: String,
        #[arg(default_value = ".")]
        root: PathBuf,
    },
    Doctor {
        #[arg(default_value = ".")]
        root: PathBuf,
    },
}
fn main() -> Result<()> {
    let cli = Cli::parse();
    match cli.command {
        Command::Scan {
            roots,
            include_generated,
        } => {
            let repos = find(&roots, include_generated)?;
            let info = inspect_all(&repos);
            summary(&info);
        }
        Command::List { roots, json } => {
            let info = inspect_all(&find(&roots, false)?);
            if json {
                println!("{}", serde_json::to_string_pretty(&info)?);
            } else {
                for r in info {
                    print_repo(&r);
                }
            }
        }
        Command::Status { roots } => summary(&inspect_all(&find(&roots, false)?)),
        Command::Dirty { roots } => inspect_all(&find(&roots, false)?)
            .iter()
            .filter(|r| r.dirty)
            .for_each(print_repo),
        Command::Stale { roots, days } => {
            let cutoff = chrono::Utc::now() - chrono::Duration::days(days);
            inspect_all(&find(&roots, false)?)
                .iter()
                .filter(|r| r.last_commit_at.is_some_and(|d| d < cutoff))
                .for_each(print_repo);
        }
        Command::Large { roots, min_mb } => inspect_all(&find(&roots, false)?)
            .iter()
            .filter(|r| r.disk.total_bytes >= min_mb * 1024 * 1024)
            .for_each(print_repo),
        Command::Fetch { path } => {
            fetch(&path)?;
            println!("Fetched remotes for {}", safe(&path.display().to_string()));
        }
        Command::Repo { name, root } => {
            let r = inspect_all(&find(&[root], false)?)
                .into_iter()
                .find(|r| r.name == name)
                .ok_or_else(|| anyhow::anyhow!("repository not found: {}", safe(&name)))?;
            print_repo(&r);
        }
        Command::Doctor { root } => {
            if !path_exists(&root) {
                bail!(
                    "scan root does not exist: {}",
                    safe(&root.display().to_string())
                );
            }
            let found = find(&[root], false)?;
            println!(
                "Scan root is accessible. Found {} repository/repositories.",
                found.len()
            );
        }
    };
    Ok(())
}
fn find(roots: &[PathBuf], include_generated: bool) -> Result<Vec<PathBuf>> {
    discover(
        roots,
        &ScanOptions {
            include_generated,
            ..Default::default()
        },
    )
}
fn inspect_all(paths: &[PathBuf]) -> Vec<RepoInfo> {
    paths
        .iter()
        .filter_map(|p| {
            inspect(p)
                .map_err(|e| eprintln!("Skipped {}: {e:#}", safe(&p.display().to_string())))
                .ok()
        })
        .collect()
}
fn summary(repos: &[RepoInfo]) {
    let total: u64 = repos.iter().map(|r| r.disk.total_bytes).sum();
    println!("Found {} repositories.\n\nClean:             {}\nDirty:              {}\nAhead of remote:    {}\nBehind remote:      {}\nNo remote:          {}\nStale >180 days:   {}\n\nDisk usage:\n{}", repos.len(), repos.iter().filter(|r| !r.dirty).count(), repos.iter().filter(|r| r.dirty).count(), repos.iter().filter(|r| r.ahead.unwrap_or(0)>0).count(), repos.iter().filter(|r| r.behind.unwrap_or(0)>0).count(), repos.iter().filter(|r| r.remotes.is_empty()).count(), repos.iter().filter(|r| r.last_commit_at.is_some_and(|d| d < chrono::Utc::now()-chrono::Duration::days(180))).count(), format_bytes(total));
}
fn print_repo(r: &RepoInfo) {
    println!(
        "{}\n  {}\n  {} | {} | ↑{} ↓{} | {}\n  {}",
        safe(&r.name),
        safe(&r.path.display().to_string()),
        r.branch.as_deref().unwrap_or(if r.detached_head {
            "detached HEAD"
        } else {
            "bare"
        }),
        if r.dirty {
            format!(
                "Dirty: {} modified, {} untracked",
                r.changes.modified, r.changes.untracked
            )
        } else {
            "Clean".into()
        },
        r.ahead.unwrap_or(0),
        r.behind.unwrap_or(0),
        r.last_commit_at
            .map(|d| d.format("Last commit: %Y-%m-%d").to_string())
            .unwrap_or_else(|| "No commits".into()),
        format_bytes(r.disk.total_bytes)
    );
}
fn safe(value: &str) -> String {
    value.chars().filter(|c| !c.is_control()).collect()
}
