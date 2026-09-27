use chrono::{Duration, Utc};
use reposhelf_core::RepoInfo;
use serde::Serialize;
#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub enum Indicator {
    Dirty,
    Ahead,
    Behind,
    NoRemote,
    Stale,
    LargeGenerated,
}
pub fn indicators(repo: &RepoInfo, stale_days: i64) -> Vec<Indicator> {
    let mut out = Vec::new();
    if repo.dirty {
        out.push(Indicator::Dirty)
    }
    if repo.ahead.unwrap_or(0) > 0 {
        out.push(Indicator::Ahead)
    }
    if repo.behind.unwrap_or(0) > 0 {
        out.push(Indicator::Behind)
    }
    if repo.remotes.is_empty() {
        out.push(Indicator::NoRemote)
    }
    if repo
        .last_commit_at
        .is_some_and(|d| d < Utc::now() - Duration::days(stale_days))
    {
        out.push(Indicator::Stale)
    }
    if repo.disk.generated.iter().any(|d| d.bytes > 0) {
        out.push(Indicator::LargeGenerated)
    }
    out
}
