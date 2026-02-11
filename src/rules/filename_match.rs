use anyhow::Result;
use globset::GlobSet;

use crate::config::Action;
use crate::rule::{ChangeContext, ChangeKind, CheckResult, Rule};

#[derive(Debug)]
pub struct FilenameMatchRule {
    name: String,
    globset: GlobSet,
    action: Action,
    matched: bool,
}

impl FilenameMatchRule {
    pub fn new(name: String, globset: GlobSet, action: Action) -> Self {
        Self {
            name,
            globset,
            action,
            matched: false,
        }
    }
}

impl Rule for FilenameMatchRule {
    fn name(&self) -> &str {
        &self.name
    }

    fn check_change(&mut self, ctx: &ChangeContext) -> Result<Vec<CheckResult>> {
        if matches!(ctx.kind, ChangeKind::Deletion) {
            return Ok(vec![]);
        }
        if self.globset.is_match(&ctx.path) {
            match self.action {
                Action::Forbid => Ok(vec![CheckResult::Violation(format!(
                    "Path matches forbidden pattern: {}",
                    ctx.path
                ))]),
                Action::Require => {
                    self.matched = true;
                    Ok(vec![CheckResult::Pass(format!(
                        "Path matches required pattern: {}",
                        ctx.path
                    ))])
                }
            }
        } else {
            Ok(vec![])
        }
    }

    fn finalize(&mut self) -> Result<Vec<CheckResult>> {
        if matches!(self.action, Action::Require) && !self.matched {
            Ok(vec![CheckResult::Violation(
                "Required file pattern not found in commit".to_string(),
            )])
        } else {
            Ok(vec![])
        }
    }

    fn reset(&mut self) {
        self.matched = false;
    }
}
