use anyhow::Result;
use globset::GlobSet;

use crate::rule::{ChangeContext, ChangeKind, CheckResult, Rule};

#[derive(Debug)]
pub struct ContentMatchRule {
    name: String,
    globset: GlobSet,
}

impl ContentMatchRule {
    pub fn new(name: String, globset: GlobSet) -> Self {
        Self { name, globset }
    }
}

impl Rule for ContentMatchRule {
    fn name(&self) -> &str {
        &self.name
    }

    fn check_change(&mut self, ctx: &ChangeContext) -> Result<Vec<CheckResult>> {
        if matches!(ctx.kind, ChangeKind::Deletion) {
            return Ok(vec![]);
        }
        if !ctx.entry_mode.is_blob() {
            return Ok(vec![]);
        }
        if !self.globset.is_match(&ctx.path) {
            return Ok(vec![]);
        }

        let blob = match ctx.repo.find_object(ctx.id) {
            Ok(obj) => obj,
            Err(e) => {
                return Ok(vec![CheckResult::Violation(format!(
                    "Content validation failed in {}: {}",
                    ctx.path, e
                ))]);
            }
        };
        let data = blob.data.as_slice();

        if ctx.path.ends_with(".json") {
            if let Err(e) = serde_json::from_slice::<serde_json::Value>(data) {
                return Ok(vec![CheckResult::Violation(format!(
                    "Content validation failed in {}: Invalid JSON in {}: {}",
                    ctx.path, ctx.path, e
                ))]);
            }
        } else if ctx.path.ends_with(".csv") {
            let mut reader = csv::Reader::from_reader(data);
            for result in reader.records() {
                if let Err(e) = result {
                    return Ok(vec![CheckResult::Violation(format!(
                        "Content validation failed in {}: Invalid CSV in {}: {}",
                        ctx.path, ctx.path, e
                    ))]);
                }
            }
        } else {
            return Ok(vec![CheckResult::Warning(format!(
                "content validation not supported for file: {}",
                ctx.path
            ))]);
        }
        Ok(vec![])
    }
}
