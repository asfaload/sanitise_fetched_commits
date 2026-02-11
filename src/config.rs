use anyhow::{Context, Result};
use globset::{Glob, GlobSetBuilder};
use serde::Deserialize;
use std::collections::HashMap;

use crate::rule::Rule;
use crate::rules::*;

#[derive(Debug, Deserialize)]
pub struct Config {
    pub rules: Vec<RuleConfig>,
}

#[derive(Debug, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum RuleConfig {
    FilenameMatch {
        name: String,
        enabled: bool,
        patterns: Vec<String>,
        action: Action,
    },
    DepthLimit {
        name: String,
        enabled: bool,
        patterns: Vec<String>,
        max_depth: usize,
    },
    ContentMatch {
        name: String,
        enabled: bool,
        patterns: Vec<String>,
    },
    ContentDeletion {
        name: String,
        enabled: bool,
    },
    LineDeletion {
        name: String,
        enabled: bool,
        patterns: Vec<String>,
    },
}

#[derive(Debug, Deserialize, Clone)]
#[serde(rename_all = "lowercase")]
pub enum Action {
    Forbid,
    Require,
}

#[derive(Debug)]
pub enum ConfigSeverity {
    Warning,
    Error,
}

#[derive(Debug)]
pub struct ConfigIssue {
    pub severity: ConfigSeverity,
    pub rule_name: Option<String>,
    pub message: String,
}

impl std::fmt::Display for ConfigIssue {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let level = match self.severity {
            ConfigSeverity::Warning => "Warning",
            ConfigSeverity::Error => "Error",
        };
        match &self.rule_name {
            Some(name) => write!(f, "{}: [{}] {}", level, name, self.message),
            None => write!(f, "{}: {}", level, self.message),
        }
    }
}

impl RuleConfig {
    pub fn name(&self) -> &str {
        match self {
            RuleConfig::FilenameMatch { name, .. }
            | RuleConfig::DepthLimit { name, .. }
            | RuleConfig::ContentMatch { name, .. }
            | RuleConfig::ContentDeletion { name, .. }
            | RuleConfig::LineDeletion { name, .. } => name,
        }
    }

    pub fn patterns(&self) -> &[String] {
        match self {
            RuleConfig::FilenameMatch { patterns, .. }
            | RuleConfig::DepthLimit { patterns, .. }
            | RuleConfig::ContentMatch { patterns, .. }
            | RuleConfig::LineDeletion { patterns, .. } => patterns,
            RuleConfig::ContentDeletion { .. } => &[],
        }
    }

}

impl Config {
    pub fn from_file(path: &str) -> Result<Self> {
        let content = std::fs::read_to_string(path)
            .with_context(|| format!("Failed to read config file: {}", path))?;
        serde_json::from_str(&content)
            .with_context(|| format!("Failed to parse config file: {}", path))
    }

    pub fn validate(&self) -> Vec<ConfigIssue> {
        let mut issues = Vec::new();

        // 1. Empty rules array
        if self.rules.is_empty() {
            issues.push(ConfigIssue {
                severity: ConfigSeverity::Warning,
                rule_name: None,
                message: "No rules defined".to_string(),
            });
        }

        // 2. Duplicate rule names
        let mut name_counts: HashMap<&str, usize> = HashMap::new();
        for rule in &self.rules {
            *name_counts.entry(rule.name()).or_insert(0) += 1;
        }
        for (name, count) in &name_counts {
            if *count > 1 {
                issues.push(ConfigIssue {
                    severity: ConfigSeverity::Warning,
                    rule_name: Some(name.to_string()),
                    message: format!("Duplicate rule name (appears {} times)", count),
                });
            }
        }

        // 3. Invalid glob patterns
        for rule in &self.rules {
            for pattern in rule.patterns() {
                if Glob::new(pattern).is_err() {
                    issues.push(ConfigIssue {
                        severity: ConfigSeverity::Error,
                        rule_name: Some(rule.name().to_string()),
                        message: format!("Invalid glob pattern: '{}'", pattern),
                    });
                }
            }
        }

        // 4. max_depth == 0
        for rule in &self.rules {
            if let RuleConfig::DepthLimit { max_depth, .. } = rule {
                if *max_depth == 0 {
                    issues.push(ConfigIssue {
                        severity: ConfigSeverity::Error,
                        rule_name: Some(rule.name().to_string()),
                        message: "max_depth must be greater than 0".to_string(),
                    });
                }
            }
        }

        // 5. Conflicting FilenameMatch rules (same pattern with both Forbid and Require)
        let mut forbid_patterns: HashMap<&str, &str> = HashMap::new();
        let mut require_patterns: HashMap<&str, &str> = HashMap::new();
        for rule in &self.rules {
            if let RuleConfig::FilenameMatch {
                patterns, action, ..
            } = rule
            {
                for pattern in patterns {
                    match action {
                        Action::Forbid => {
                            forbid_patterns.insert(pattern, rule.name());
                        }
                        Action::Require => {
                            require_patterns.insert(pattern, rule.name());
                        }
                    }
                }
            }
        }
        for (pattern, forbid_rule) in &forbid_patterns {
            if let Some(require_rule) = require_patterns.get(pattern) {
                issues.push(ConfigIssue {
                    severity: ConfigSeverity::Warning,
                    rule_name: None,
                    message: format!(
                        "Pattern '{}' is both forbidden (in '{}') and required (in '{}')",
                        pattern, forbid_rule, require_rule
                    ),
                });
            }
        }

        issues
    }

    pub fn compile(self) -> Result<Vec<Box<dyn Rule>>> {
        self.rules
            .into_iter()
            .map(|r| r.compile())
            .filter_map(|r| r.transpose())
            .collect()
    }
}

fn build_globset(patterns: &[String]) -> Result<globset::GlobSet> {
    let mut builder = GlobSetBuilder::new();
    for pattern in patterns {
        let glob = Glob::new(pattern)
            .with_context(|| format!("Invalid glob pattern '{}'", pattern))?;
        builder.add(glob);
    }
    Ok(builder.build()?)
}

impl RuleConfig {
    fn compile(self) -> Result<Option<Box<dyn Rule>>> {
        match self {
            RuleConfig::FilenameMatch {
                name,
                enabled,
                patterns,
                action,
            } => {
                if !enabled {
                    return Ok(None);
                }
                let globset = build_globset(&patterns)?;
                Ok(Some(Box::new(FilenameMatchRule::new(
                    name, globset, patterns, action,
                ))))
            }
            RuleConfig::DepthLimit {
                name,
                enabled,
                patterns,
                max_depth,
            } => {
                if !enabled {
                    return Ok(None);
                }
                Ok(Some(Box::new(DepthLimitRule::new(name, patterns, max_depth))))
            }
            RuleConfig::ContentMatch {
                name,
                enabled,
                patterns,
            } => {
                if !enabled {
                    return Ok(None);
                }
                let globset = build_globset(&patterns)?;
                Ok(Some(Box::new(ContentMatchRule::new(name, globset))))
            }
            RuleConfig::ContentDeletion { name, enabled } => {
                if !enabled {
                    return Ok(None);
                }
                Ok(Some(Box::new(ContentDeletionRule::new(name))))
            }
            RuleConfig::LineDeletion {
                name,
                enabled,
                patterns,
            } => {
                if !enabled {
                    return Ok(None);
                }
                let globset = build_globset(&patterns)?;
                Ok(Some(Box::new(LineDeletionRule::new(name, globset))))
            }
        }
    }
}
