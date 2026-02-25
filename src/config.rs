use anyhow::{Context, Result};
use serde::Deserialize;
use std::collections::HashMap;

use crate::rule::Rule;
use crate::rules::ScriptRule;

#[derive(Debug, Deserialize)]
pub struct Config {
    pub rules: Vec<RuleConfig>,
    #[serde(skip)]
    pub config_dir: String,
}

#[derive(Debug, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum RuleConfig {
    Script {
        name: String,
        enabled: bool,
        script: String,
    },
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
            RuleConfig::Script { name, .. } => name,
        }
    }
}

impl Config {
    pub fn from_file(path: &str) -> Result<Self> {
        let content = std::fs::read_to_string(path)
            .with_context(|| format!("Failed to read config file: {}", path))?;
        let mut config: Config = serde_json::from_str(&content)
            .with_context(|| format!("Failed to parse config file: {}", path))?;

        // Compute config_dir from the config file path
        let config_path = std::path::Path::new(path);
        let config_dir = config_path
            .parent()
            .map(|p| p.to_string_lossy().to_string())
            .unwrap_or_else(|| ".".to_string());
        config.config_dir = config_dir;

        Ok(config)
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

        issues
    }

    pub fn compile(self) -> Result<Vec<Box<dyn Rule>>> {
        let config_dir = self.config_dir.clone();
        self.rules
            .into_iter()
            .map(|r| r.compile(&config_dir))
            .filter_map(|r| r.transpose())
            .collect()
    }
}

impl RuleConfig {
    fn compile(self, config_dir: &str) -> Result<Option<Box<dyn Rule>>> {
        match self {
            RuleConfig::Script {
                name,
                enabled,
                script,
            } => {
                if !enabled {
                    return Ok(None);
                }
                let script_path = std::path::Path::new(config_dir).join(&script);
                let script_path_str = script_path.to_string_lossy().to_string();
                Ok(Some(Box::new(ScriptRule::new(name, &script_path_str)?)))
            }
        }
    }
}
