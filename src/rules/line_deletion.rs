use anyhow::{anyhow, Result};
use globset::GlobSet;

use crate::rule::{ChangeContext, ChangeKind, CheckResult, Rule};

#[derive(Debug)]
pub struct LineDeletionRule {
    name: String,
    globset: GlobSet,
}

impl LineDeletionRule {
    pub fn new(name: String, globset: GlobSet) -> Self {
        Self { name, globset }
    }
}

impl Rule for LineDeletionRule {
    fn name(&self) -> &str {
        &self.name
    }

    fn check_change(&mut self, ctx: &ChangeContext) -> Result<Vec<CheckResult>> {
        match ctx.kind {
            ChangeKind::Deletion => {
                if self.globset.is_match(&ctx.path) {
                    Ok(vec![CheckResult::Violation(format!(
                        "File deleted (all lines removed): {}",
                        ctx.path
                    ))])
                } else {
                    Ok(vec![])
                }
            }
            ChangeKind::Modification | ChangeKind::Rewrite => {
                if !ctx.entry_mode.is_blob() {
                    return Ok(vec![]);
                }
                if !self.globset.is_match(&ctx.path) {
                    return Ok(vec![]);
                }
                let previous_id = match ctx.previous_id {
                    Some(id) => id,
                    None => return Ok(vec![]),
                };

                let old_blob = ctx.repo.find_object(previous_id)?;
                let new_blob = ctx.repo.find_object(ctx.id)?;

                let old_content = std::str::from_utf8(old_blob.data.as_slice())
                    .map_err(|_| anyhow!("Binary file, skipping line deletion check"))?;
                let new_content = std::str::from_utf8(new_blob.data.as_slice())
                    .map_err(|_| anyhow!("Binary file, skipping line deletion check"))?;

                let diff = similar::TextDiff::from_lines(old_content, new_content);
                let has_deletions = diff
                    .iter_all_changes()
                    .any(|c| c.tag() == similar::ChangeTag::Delete);

                if has_deletions {
                    Ok(vec![CheckResult::Violation(format!(
                        "Lines deleted in protected file: {}",
                        ctx.path
                    ))])
                } else {
                    Ok(vec![])
                }
            }
            _ => Ok(vec![]),
        }
    }
}
