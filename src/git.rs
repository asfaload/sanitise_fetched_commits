use anyhow::{anyhow, Context, Result};
use std::path::PathBuf;
use std::process::Command;

/// A handle to a git repository directory.
pub struct GitRepo {
    path: PathBuf,
}

/// A single tree-level change in a commit diff.
pub struct TreeChange {
    pub path: String,
    pub kind: TreeChangeKind,
    pub old_mode: String,
    pub new_mode: String,
    pub old_oid: String,
    pub new_oid: String,
}

#[derive(Debug, Clone, Copy)]
pub enum TreeChangeKind {
    Addition,
    Deletion,
    Modification,
    Rename,
}

impl GitRepo {
    /// Open a git repository at the given path.
    pub fn open(path: &str) -> Result<Self> {
        let path = PathBuf::from(path);
        let output = Command::new("git")
            .args(["-C", &path.to_string_lossy(), "rev-parse", "--git-dir"])
            .output()
            .context("Failed to execute git")?;
        if !output.status.success() {
            return Err(anyhow!("Not a git repository: {}", path.display()));
        }
        Ok(Self { path })
    }

    fn run(&self, args: &[&str]) -> Result<String> {
        let output = Command::new("git")
            .arg("-C")
            .arg(&self.path)
            .args(args)
            .output()
            .with_context(|| format!("Failed to execute: git {}", args.join(" ")))?;
        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr).trim().to_string();
            return Err(anyhow!("git {} failed: {}", args.join(" "), stderr));
        }
        Ok(String::from_utf8_lossy(&output.stdout)
            .trim_end()
            .to_string())
    }

    /// Resolve a refspec to a hex OID string.
    pub fn resolve_ref(&self, refspec: &str) -> Result<String> {
        self.run(&["rev-parse", refspec])
            .with_context(|| format!("Could not resolve ref '{}'", refspec))
    }

    /// Get the current branch name (e.g. "master"). Errors on detached HEAD.
    pub fn head_branch(&self) -> Result<String> {
        self.run(&["symbolic-ref", "--short", "HEAD"])
            .map_err(|_| anyhow!("HEAD is detached or not on a branch"))
    }

    /// Get HEAD OID.
    pub fn head_oid(&self) -> Result<String> {
        self.run(&["rev-parse", "HEAD"])
    }

    /// List commits from `from` (exclusive) to `to` (inclusive), first-parent only.
    /// Returns commit OIDs in reverse chronological order (newest first).
    pub fn list_commits(&self, from: &str, to: &str) -> Result<Vec<String>> {
        let range = format!("{}..{}", from, to);
        let output = self.run(&["rev-list", "--first-parent", &range])?;
        if output.is_empty() {
            return Ok(vec![]);
        }
        Ok(output.lines().map(String::from).collect())
    }

    /// Get the tree diff for a commit against its first parent.
    pub fn diff_tree(&self, commit_oid: &str) -> Result<Vec<TreeChange>> {
        // Try to resolve first parent; if it fails, this is a root commit
        let first_parent = self.run(&["rev-parse", &format!("{}^1", commit_oid)]);

        let output = match first_parent {
            Ok(parent) => self.run(&[
                "diff-tree",
                "-r",
                "-M",
                "--no-commit-id",
                &parent,
                commit_oid,
            ])?,
            Err(_) => {
                self.run(&["diff-tree", "-r", "--root", "--no-commit-id", commit_oid])?
            }
        };

        if output.is_empty() {
            return Ok(vec![]);
        }
        parse_diff_tree_output(&output)
    }

    /// Read a blob object's raw content.
    pub fn read_blob(&self, oid: &str) -> Result<Vec<u8>> {
        let output = Command::new("git")
            .arg("-C")
            .arg(&self.path)
            .args(["cat-file", "-p", oid])
            .output()
            .with_context(|| format!("Failed to read blob {}", oid))?;
        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(anyhow!("Failed to read blob {}: {}", oid, stderr));
        }
        Ok(output.stdout)
    }

    /// Get the first line of a commit's message.
    pub fn commit_subject(&self, oid: &str) -> Result<String> {
        self.run(&["log", "-1", "--format=%s", oid])
    }
}

fn is_null_oid(oid: &str) -> bool {
    oid.chars().all(|c| c == '0')
}

/// Check if a git file mode represents a blob (regular file, executable, symlink).
pub fn is_blob_mode(mode: &str) -> bool {
    matches!(mode, "100644" | "100755" | "120000")
}

fn parse_diff_tree_output(output: &str) -> Result<Vec<TreeChange>> {
    let mut changes = Vec::new();
    for line in output.lines() {
        if let Some(change) = parse_diff_tree_line(line)? {
            changes.push(change);
        }
    }
    Ok(changes)
}

fn parse_diff_tree_line(line: &str) -> Result<Option<TreeChange>> {
    // Format: :<old-mode> <new-mode> <old-oid> <new-oid> <status>\t<path>[\t<new-path>]
    if !line.starts_with(':') {
        return Ok(None);
    }

    // Split on tab to separate metadata from path(s)
    let tab_parts: Vec<&str> = line[1..].split('\t').collect();
    if tab_parts.len() < 2 {
        return Err(anyhow!("Invalid diff-tree line: {}", line));
    }

    // Parse space-separated metadata
    let meta_parts: Vec<&str> = tab_parts[0].split_whitespace().collect();
    if meta_parts.len() < 5 {
        return Err(anyhow!("Invalid diff-tree metadata: {}", line));
    }

    let old_mode = meta_parts[0].to_string();
    let new_mode = meta_parts[1].to_string();
    let old_oid = meta_parts[2].to_string();
    let new_oid = meta_parts[3].to_string();
    let status = meta_parts[4];

    let (kind, path) = if status.starts_with('A') {
        (TreeChangeKind::Addition, tab_parts[1].to_string())
    } else if status.starts_with('D') {
        (TreeChangeKind::Deletion, tab_parts[1].to_string())
    } else if status.starts_with('M') || status.starts_with('T') {
        (TreeChangeKind::Modification, tab_parts[1].to_string())
    } else if status.starts_with('R') {
        // Rename: tab_parts[1] = old path, tab_parts[2] = new path
        let new_path = if tab_parts.len() > 2 {
            tab_parts[2]
        } else {
            tab_parts[1]
        };
        (TreeChangeKind::Rename, new_path.to_string())
    } else {
        return Ok(None);
    };

    Ok(Some(TreeChange {
        path,
        kind,
        old_mode,
        new_mode,
        old_oid,
        new_oid,
    }))
}

/// Check if an OID is the null OID (all zeros).
pub fn oid_is_null(oid: &str) -> bool {
    is_null_oid(oid)
}
