use anyhow::{Context, Result};
use globset::{Glob, GlobSetBuilder};
use serde::Deserialize;

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

impl Config {
    pub fn from_file(path: &str) -> Result<Self> {
        let content = std::fs::read_to_string(path)
            .with_context(|| format!("Failed to read config file: {}", path))?;
        serde_json::from_str(&content)
            .with_context(|| format!("Failed to parse config file: {}", path))
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
                Ok(Some(Box::new(FilenameMatchRule::new(name, globset, action))))
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
