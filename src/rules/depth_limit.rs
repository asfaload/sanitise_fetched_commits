use anyhow::Result;

use crate::rule::{ChangeContext, ChangeKind, CheckResult, Rule};

#[derive(Debug)]
pub struct DepthLimitRule {
    name: String,
    patterns: Vec<String>,
    max_depth: usize,
}

impl DepthLimitRule {
    pub fn new(name: String, patterns: Vec<String>, max_depth: usize) -> Self {
        Self {
            name,
            patterns,
            max_depth,
        }
    }
}

impl Rule for DepthLimitRule {
    fn name(&self) -> &str {
        &self.name
    }

    fn check_change(&mut self, ctx: &ChangeContext) -> Result<Vec<CheckResult>> {
        if matches!(ctx.kind, ChangeKind::Deletion) {
            return Ok(vec![]);
        }
        let parts: Vec<&str> = ctx.path.split('/').collect();
        let mut results = vec![];
        for (i, segment) in parts.iter().enumerate() {
            if self.patterns.contains(&segment.to_string()) && i >= self.max_depth {
                results.push(CheckResult::Violation(format!(
                    "Folder '{}' too deep (depth {}): {}",
                    segment,
                    i + 1,
                    ctx.path
                )));
            }
        }
        Ok(results)
    }
}
