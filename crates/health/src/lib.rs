use chrono::{Duration, Utc};
use reposhelf_core::RepoInfo;
use serde::Serialize;

pub const DEFAULT_LARGE_GENERATED_BYTES: u64 = 100 * 1024 * 1024;
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
    indicators_with_threshold(repo, stale_days, DEFAULT_LARGE_GENERATED_BYTES)
}

pub fn indicators_with_threshold(
    repo: &RepoInfo,
    stale_days: i64,
    generated_threshold_bytes: u64,
) -> Vec<Indicator> {
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
    if repo
        .disk
        .generated
        .iter()
        .any(|d| d.bytes >= generated_threshold_bytes)
    {
        out.push(Indicator::LargeGenerated)
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use reposhelf_core::{ChangeCounts, DirectorySize, DiskBreakdown, RepoInfo};

    fn repo(bytes: u64) -> RepoInfo {
        RepoInfo {
            name: "fixture".into(),
            path: "fixture".into(),
            is_bare: false,
            branch: Some("main".into()),
            detached_head: false,
            head_commit: None,
            last_commit_at: None,
            remotes: vec!["origin".into()],
            default_remote: Some("origin".into()),
            dirty: false,
            changes: ChangeCounts::default(),
            ahead: Some(0),
            behind: Some(0),
            disk: DiskBreakdown {
                total_bytes: bytes,
                git_bytes: 0,
                generated: vec![DirectorySize {
                    name: "target".into(),
                    bytes,
                }],
            },
            technologies: vec![],
        }
    }

    #[test]
    fn only_large_generated_directories_are_flagged() {
        assert!(!indicators(&repo(1024), 180).contains(&Indicator::LargeGenerated));
        assert!(indicators(&repo(DEFAULT_LARGE_GENERATED_BYTES), 180)
            .contains(&Indicator::LargeGenerated));
    }
}
