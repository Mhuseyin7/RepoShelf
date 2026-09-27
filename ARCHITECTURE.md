# Architecture

The workspace separates the stable inspection core (`crates/core`) from focused adapters: discovery, Git, disk, health, search, SQLite storage, and platform actions. `apps/cli` and `apps/desktop` consume the same core API. The core only uses filesystem reads and libgit2 inspection, except its explicit `fetch` API.

Discovery prunes expensive generated directories by default, does not follow symbolic links, deduplicates canonical repository paths, and recognizes `.git` directories or files (worktrees). Disk traversal also does not follow links.

SQLite is strictly application state: scan snapshots, settings, groups, tags, and notes. It never becomes the source of Git truth and never stores credentials. The desktop process only exposes typed Tauri commands; opening a folder uses direct process arguments rather than a shell.
