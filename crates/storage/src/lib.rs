//! SQLite cache and user-owned local metadata. Git itself remains the source of truth.
use anyhow::Result;
use chrono::Utc;
use reposhelf_core::RepoInfo;
use rusqlite::{params, Connection, OptionalExtension};
use std::path::Path;

pub struct Store {
    conn: Connection,
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct SavedRepo {
    pub path: String,
    pub group_name: Option<String>,
    pub note: Option<String>,
    pub tags: Vec<String>,
}

impl Store {
    pub fn open(path: impl AsRef<Path>) -> Result<Self> {
        let conn = Connection::open(path)?;
        conn.execute_batch("PRAGMA foreign_keys = ON; PRAGMA journal_mode = WAL;
            CREATE TABLE IF NOT EXISTS repositories (path TEXT PRIMARY KEY, snapshot_json TEXT NOT NULL, scanned_at TEXT NOT NULL);
            CREATE TABLE IF NOT EXISTS groups (name TEXT PRIMARY KEY);
            CREATE TABLE IF NOT EXISTS repo_groups (repo_path TEXT PRIMARY KEY, group_name TEXT NOT NULL REFERENCES groups(name) ON DELETE CASCADE);
            CREATE TABLE IF NOT EXISTS tags (name TEXT PRIMARY KEY);
            CREATE TABLE IF NOT EXISTS repo_tags (repo_path TEXT NOT NULL, tag_name TEXT NOT NULL REFERENCES tags(name) ON DELETE CASCADE, PRIMARY KEY(repo_path, tag_name));
            CREATE TABLE IF NOT EXISTS notes (repo_path TEXT PRIMARY KEY, body TEXT NOT NULL);
            CREATE TABLE IF NOT EXISTS settings (key TEXT PRIMARY KEY, value TEXT NOT NULL);")?;
        Ok(Self { conn })
    }
    pub fn cache_repo(&self, repo: &RepoInfo) -> Result<()> {
        self.conn.execute("INSERT INTO repositories(path,snapshot_json,scanned_at) VALUES(?1,?2,?3) ON CONFLICT(path) DO UPDATE SET snapshot_json=excluded.snapshot_json, scanned_at=excluded.scanned_at", params![repo.path.to_string_lossy(), serde_json::to_string(repo)?, Utc::now().to_rfc3339()])?;
        Ok(())
    }
    pub fn set_group(&self, path: &str, group: Option<&str>) -> Result<()> {
        if let Some(group) = group {
            self.conn
                .execute("INSERT OR IGNORE INTO groups(name) VALUES(?1)", [group])?;
            self.conn.execute("INSERT INTO repo_groups(repo_path,group_name) VALUES(?1,?2) ON CONFLICT(repo_path) DO UPDATE SET group_name=excluded.group_name", params![path,group])?;
        } else {
            self.conn
                .execute("DELETE FROM repo_groups WHERE repo_path=?1", [path])?;
        }
        Ok(())
    }
    pub fn set_tags(&self, path: &str, tags: &[String]) -> Result<()> {
        let tx = self.conn.unchecked_transaction()?;
        tx.execute("DELETE FROM repo_tags WHERE repo_path=?1", [path])?;
        for tag in tags {
            tx.execute("INSERT OR IGNORE INTO tags(name) VALUES(?1)", [tag])?;
            tx.execute(
                "INSERT INTO repo_tags(repo_path,tag_name) VALUES(?1,?2)",
                params![path, tag],
            )?;
        }
        tx.commit()?;
        Ok(())
    }
    pub fn set_note(&self, path: &str, note: Option<&str>) -> Result<()> {
        if let Some(note) = note {
            self.conn.execute("INSERT INTO notes(repo_path,body) VALUES(?1,?2) ON CONFLICT(repo_path) DO UPDATE SET body=excluded.body", params![path,note])?;
        } else {
            self.conn
                .execute("DELETE FROM notes WHERE repo_path=?1", [path])?;
        }
        Ok(())
    }
    pub fn metadata(&self, path: &str) -> Result<SavedRepo> {
        let group_name = self
            .conn
            .query_row(
                "SELECT group_name FROM repo_groups WHERE repo_path=?1",
                [path],
                |r| r.get(0),
            )
            .optional()?;
        let note = self
            .conn
            .query_row("SELECT body FROM notes WHERE repo_path=?1", [path], |r| {
                r.get(0)
            })
            .optional()?;
        let mut q = self
            .conn
            .prepare("SELECT tag_name FROM repo_tags WHERE repo_path=?1 ORDER BY tag_name")?;
        let tags = q
            .query_map([path], |r| r.get(0))?
            .collect::<std::result::Result<_, _>>()?;
        Ok(SavedRepo {
            path: path.into(),
            group_name,
            note,
            tags,
        })
    }
    pub fn setting(&self, key: &str) -> Result<Option<String>> {
        Ok(self
            .conn
            .query_row("SELECT value FROM settings WHERE key=?1", [key], |r| {
                r.get(0)
            })
            .optional()?)
    }
    pub fn set_setting(&self, key: &str, value: &str) -> Result<()> {
        self.conn.execute("INSERT INTO settings(key,value) VALUES(?1,?2) ON CONFLICT(key) DO UPDATE SET value=excluded.value", params![key,value])?;
        Ok(())
    }
}
