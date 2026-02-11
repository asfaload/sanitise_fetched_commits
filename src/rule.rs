use anyhow::Result;

use crate::cli::RunOptions;

#[derive(Debug, Clone)]
pub enum ChangeKind {
    Addition,
    Deletion,
    Modification,
    Rewrite,
}

/// All info a rule needs about a single tree change.
pub struct ChangeContext<'repo> {
    pub repo: &'repo gix::Repository,
    pub path: String,
    pub kind: ChangeKind,
    pub entry_mode: gix::object::tree::EntryMode,
    pub id: gix::ObjectId,
    pub previous_id: Option<gix::ObjectId>,
}

impl<'repo> ChangeContext<'repo> {
    pub fn from_change(
        change: &gix::object::tree::diff::Change<'_, '_, '_>,
        repo: &'repo gix::Repository,
    ) -> Self {
        let path = change.location().to_string();
        let (kind, entry_mode, id, previous_id) = match change {
            gix::object::tree::diff::Change::Deletion {
                entry_mode, id, ..
            } => (ChangeKind::Deletion, *entry_mode, id.detach(), None),
            gix::object::tree::diff::Change::Addition {
                entry_mode, id, ..
            } => (ChangeKind::Addition, *entry_mode, id.detach(), None),
            gix::object::tree::diff::Change::Modification {
                entry_mode,
                id,
                previous_id,
                ..
            } => (
                ChangeKind::Modification,
                *entry_mode,
                id.detach(),
                Some(previous_id.detach()),
            ),
            gix::object::tree::diff::Change::Rewrite {
                entry_mode,
                id,
                source_id,
                ..
            } => (
                ChangeKind::Rewrite,
                *entry_mode,
                id.detach(),
                Some(source_id.detach()),
            ),
        };
        Self {
            repo,
            path,
            kind,
            entry_mode,
            id,
            previous_id,
        }
    }
}

/// The result of a rule check on a single change.
pub enum CheckResult {
    Violation(String),
    Pass(String),
    Warning(String),
}

impl CheckResult {
    pub fn is_violation(&self) -> bool {
        matches!(self, Self::Violation(_))
    }

    pub fn status_str(&self) -> &str {
        match self {
            Self::Violation(_) => "violation",
            Self::Pass(_) => "pass",
            Self::Warning(_) => "warning",
        }
    }

    pub fn message(&self) -> &str {
        match self {
            Self::Violation(msg) | Self::Pass(msg) | Self::Warning(msg) => msg,
        }
    }

    pub fn print(&self, rule_name: &str, opts: &RunOptions) {
        match self {
            Self::Violation(msg) => println!("   - \u{274c} {} - {}", rule_name, msg),
            Self::Pass(msg) => {
                if opts.verbose {
                    println!("   - \u{2705} {} - {}", rule_name, msg);
                }
            }
            Self::Warning(msg) => println!("   - \u{26a0}\u{fe0f}  {} - {}", rule_name, msg),
        }
    }
}

pub trait Rule: std::fmt::Debug {
    /// Human-readable rule name (from config).
    fn name(&self) -> &str;

    /// Check a single tree change. Return violations/passes/warnings.
    fn check_change(&mut self, ctx: &ChangeContext) -> Result<Vec<CheckResult>>;

    /// Called after all changes in a commit. For rules that need
    /// post-commit validation (e.g. Require pattern). Default: no-op.
    fn finalize(&mut self) -> Result<Vec<CheckResult>> {
        Ok(vec![])
    }

    /// Reset per-commit state. Called before each new commit.
    fn reset(&mut self) {}
}
