use reposhelf_core::RepoInfo;
/// Case-insensitive subsequence matcher; deterministic and allocation-light for local inventories.
pub fn filter<'a>(repos: impl IntoIterator<Item = &'a RepoInfo>, query: &str) -> Vec<&'a RepoInfo> {
    let q = query.to_lowercase();
    repos
        .into_iter()
        .filter(|r| {
            matches(&r.name, &q)
                || matches(&r.path.to_string_lossy(), &q)
                || r.branch.as_deref().is_some_and(|b| matches(b, &q))
                || r.remotes.iter().any(|x| matches(x, &q))
        })
        .collect()
}
fn matches(value: &str, q: &str) -> bool {
    let normalized = value.to_lowercase();
    let mut chars = normalized.chars();
    q.chars().all(|c| chars.by_ref().any(|x| x == c))
}
