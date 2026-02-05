use anyhow::{Context, Result};
use globset::{Glob, GlobSetBuilder};
use serde::Deserialize;

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
            .with_context(|| format!(" Failed to parse config file: {}", path))
    }

    pub fn compile(&self) -> Result<CompiledRules> {
        let mut compiled = CompiledRules {
            filename_match: vec![],
            depth_limit: vec![],
            content_match: vec![],
            content_deletion: None,
        };

        for rule in &self.rules {
            match rule {
                RuleConfig::FilenameMatch {
                    name,
                    enabled,
                    patterns,
                    action,
                } => {
                    if !enabled {
                        continue;
                    }
                    let mut glob_builder = GlobSetBuilder::new();
                    for pattern in patterns {
                        let glob = Glob::new(pattern)
                            .with_context(|| format!("Invalid glob pattern '{}'", pattern))?;
                        glob_builder.add(glob);
                    }
                    let globset = glob_builder.build()?;
                    compiled.filename_match.push(CompiledFilenameRule {
                        name: name.clone(),
                        globset,
                        action: action.clone(),
                    });
                }
                RuleConfig::DepthLimit {
                    name,
                    enabled,
                    patterns,
                    max_depth,
                } => {
                    if !enabled {
                        continue;
                    }
                    compiled.depth_limit.push(CompiledDepthRule {
                        name: name.clone(),
                        patterns: patterns.clone(),
                        max_depth: *max_depth,
                    });
                }
                RuleConfig::ContentMatch {
                    name,
                    enabled,
                    patterns,
                } => {
                    if !enabled {
                        continue;
                    }
                    let mut glob_builder = GlobSetBuilder::new();
                    for pattern in patterns {
                        let glob = Glob::new(pattern)
                            .with_context(|| format!("Invalid glob pattern '{}'", pattern))?;
                        glob_builder.add(glob);
                    }
                    let globset = glob_builder.build()?;
                    compiled.content_match.push(CompiledContentMatchRule {
                        name: name.clone(),
                        globset,
                    });
                }
                RuleConfig::ContentDeletion { name, enabled } => {
                    if !enabled {
                        continue;
                    }
                    compiled.content_deletion = Some(name.clone());
                }
            }
        }

        Ok(compiled)
    }
}

#[derive(Debug)]
pub struct CompiledRules {
    pub filename_match: Vec<CompiledFilenameRule>,
    pub depth_limit: Vec<CompiledDepthRule>,
    pub content_match: Vec<CompiledContentMatchRule>,
    pub content_deletion: Option<String>,
}

#[derive(Debug)]
pub struct CompiledFilenameRule {
    pub name: String,
    pub globset: globset::GlobSet,
    pub action: Action,
}

#[derive(Debug)]
pub struct CompiledDepthRule {
    pub name: String,
    pub patterns: Vec<String>,
    pub max_depth: usize,
}

#[derive(Debug)]
pub struct CompiledContentMatchRule {
    pub name: String,
    pub globset: globset::GlobSet,
}
