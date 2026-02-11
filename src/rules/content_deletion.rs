use anyhow::Result;

use crate::rule::{ChangeContext, ChangeKind, CheckResult, Rule};

#[derive(Debug)]
pub struct ContentDeletionRule {
    name: String,
}

impl ContentDeletionRule {
    pub fn new(name: String) -> Self {
        Self { name }
    }
}

impl Rule for ContentDeletionRule {
    fn name(&self) -> &str {
        &self.name
    }

    fn check_change(&mut self, ctx: &ChangeContext) -> Result<Vec<CheckResult>> {
        if matches!(ctx.kind, ChangeKind::Deletion) {
            Ok(vec![CheckResult::Violation(format!(
                "Deletion forbidden: {}",
                ctx.path
            ))])
        } else {
            Ok(vec![])
        }
    }
}
