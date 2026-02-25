use anyhow::{anyhow, Context, Result};
use rhai::{CallFnOptions, Dynamic, Engine, Map, Scope, AST};

use crate::rule::{ChangeContext, ChangeKind, CheckResult, Rule};

pub struct ScriptRule {
    name: String,
    engine: Engine,
    ast: AST,
    scope: Scope<'static>,
    state: Dynamic,
    has_finalize: bool,
}

impl std::fmt::Debug for ScriptRule {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ScriptRule")
            .field("name", &self.name)
            .field("has_finalize", &self.has_finalize)
            .finish()
    }
}

impl ScriptRule {
    pub fn new(name: String, script_path: &str) -> Result<Self> {
        let script_content = std::fs::read_to_string(script_path)
            .with_context(|| format!("Failed to read script file: {}", script_path))?;

        let mut engine = Engine::new();
        register_helpers(&mut engine);

        let ast = engine
            .compile(&script_content)
            .map_err(|e| anyhow!("Failed to compile script '{}': {}", script_path, e))?;

        // Validate that check_change function exists with 1 parameter
        let has_check_change = ast
            .iter_functions()
            .any(|f| f.name == "check_change" && f.params.len() == 1);
        if !has_check_change {
            return Err(anyhow!(
                "Script '{}' must define a 'check_change' function with 1 parameter",
                script_path
            ));
        }

        let has_finalize = ast
            .iter_functions()
            .any(|f| f.name == "finalize" && f.params.is_empty());

        let scope = Scope::new();
        let state = Dynamic::from(Map::new());

        Ok(Self {
            name,
            engine,
            ast,
            scope,
            state,
            has_finalize,
        })
    }
}

/// Register helper functions into the Rhai engine.
fn register_helpers(engine: &mut Engine) {
    // Result constructors - return Maps with "type" and "message" keys
    engine.register_fn("violation", |msg: String| -> Map {
        let mut map = Map::new();
        map.insert("type".into(), Dynamic::from("violation".to_string()));
        map.insert("message".into(), Dynamic::from(msg));
        map
    });

    engine.register_fn("pass", |msg: String| -> Map {
        let mut map = Map::new();
        map.insert("type".into(), Dynamic::from("pass".to_string()));
        map.insert("message".into(), Dynamic::from(msg));
        map
    });

    engine.register_fn("warning", |msg: String| -> Map {
        let mut map = Map::new();
        map.insert("type".into(), Dynamic::from("warning".to_string()));
        map.insert("message".into(), Dynamic::from(msg));
        map
    });

    // Glob matching
    engine.register_fn("glob_match", |pattern: String, path: String| -> bool {
        match globset::Glob::new(&pattern) {
            Ok(glob) => glob.compile_matcher().is_match(&path),
            Err(_) => false,
        }
    });

    // Format validators
    engine.register_fn("validate_json", |text: String| -> bool {
        serde_json::from_str::<serde_json::Value>(&text).is_ok()
    });

    engine.register_fn("validate_csv", |text: String| -> bool {
        let mut reader = csv::Reader::from_reader(text.as_bytes());
        reader.records().all(|r| r.is_ok())
    });

    engine.register_fn("validate_yaml", |text: String| -> bool {
        serde_yaml::from_str::<serde_yaml::Value>(&text).is_ok()
    });

    engine.register_fn("validate_toml", |text: String| -> bool {
        text.parse::<toml::Value>().is_ok()
    });
}

/// Map a Rhai Dynamic return value to an optional CheckResult.
fn map_result(value: Dynamic) -> Option<CheckResult> {
    if value.is_unit() {
        return None;
    }

    if let Some(map) = value.try_cast::<Map>() {
        let result_type = map
            .get("type")
            .and_then(|v: &Dynamic| v.clone().into_string().ok())
            .unwrap_or_default();
        let message = map
            .get("message")
            .and_then(|v: &Dynamic| v.clone().into_string().ok())
            .unwrap_or_default();

        match result_type.as_str() {
            "violation" => Some(CheckResult::Violation(message)),
            "pass" => Some(CheckResult::Pass(message)),
            "warning" => Some(CheckResult::Warning(message)),
            _ => None,
        }
    } else {
        None
    }
}

/// Compute the diff information for a change context.
/// Returns (added_lines, deleted_lines, content).
fn compute_diff(ctx: &ChangeContext) -> Result<(Vec<Dynamic>, Vec<Dynamic>, String)> {
    match ctx.kind {
        ChangeKind::Addition => {
            if !ctx.entry_mode.is_blob() {
                return Ok((vec![], vec![], String::new()));
            }
            let blob = ctx
                .repo
                .find_object(ctx.id)
                .with_context(|| format!("Failed to find added blob for '{}'", ctx.path))?;
            let content = String::from_utf8_lossy(blob.data.as_slice()).to_string();
            let added_lines: Vec<Dynamic> = content
                .lines()
                .map(|l| Dynamic::from(l.to_string()))
                .collect();
            Ok((added_lines, vec![], content))
        }
        ChangeKind::Deletion => {
            if !ctx.entry_mode.is_blob() {
                return Ok((vec![], vec![], String::new()));
            }
            // For deletion, previous_id contains the old blob, but id is the deleted entry
            // The id in a Deletion change refers to the deleted blob
            let blob = ctx
                .repo
                .find_object(ctx.id)
                .with_context(|| format!("Failed to find deleted blob for '{}'", ctx.path))?;
            let old_content = String::from_utf8_lossy(blob.data.as_slice()).to_string();
            let deleted_lines: Vec<Dynamic> = old_content
                .lines()
                .map(|l| Dynamic::from(l.to_string()))
                .collect();
            Ok((vec![], deleted_lines, String::new()))
        }
        ChangeKind::Modification | ChangeKind::Rewrite => {
            if !ctx.entry_mode.is_blob() {
                return Ok((vec![], vec![], String::new()));
            }
            let previous_id = match ctx.previous_id {
                Some(id) => id,
                None => return Ok((vec![], vec![], String::new())),
            };

            let old_blob = ctx
                .repo
                .find_object(previous_id)
                .with_context(|| format!("Failed to find previous blob for '{}'", ctx.path))?;
            let new_blob = ctx
                .repo
                .find_object(ctx.id)
                .with_context(|| format!("Failed to find new blob for '{}'", ctx.path))?;

            let old_content = String::from_utf8_lossy(old_blob.data.as_slice());
            let new_content = String::from_utf8_lossy(new_blob.data.as_slice()).to_string();

            let diff = similar::TextDiff::from_lines(old_content.as_ref(), &new_content);
            let mut added_lines = Vec::new();
            let mut deleted_lines = Vec::new();

            for change in diff.iter_all_changes() {
                match change.tag() {
                    similar::ChangeTag::Insert => {
                        added_lines.push(Dynamic::from(change.value().trim_end_matches('\n').to_string()));
                    }
                    similar::ChangeTag::Delete => {
                        deleted_lines.push(Dynamic::from(change.value().trim_end_matches('\n').to_string()));
                    }
                    similar::ChangeTag::Equal => {}
                }
            }

            Ok((added_lines, deleted_lines, new_content))
        }
    }
}

impl Rule for ScriptRule {
    fn name(&self) -> &str {
        &self.name
    }

    fn check_change(&mut self, ctx: &ChangeContext) -> Result<Vec<CheckResult>> {
        // Map ChangeKind to string
        let kind_str = match ctx.kind {
            ChangeKind::Addition => "addition",
            ChangeKind::Deletion => "deletion",
            ChangeKind::Modification => "modification",
            ChangeKind::Rewrite => "rewrite",
        };

        // Compute diff
        let (added_lines, deleted_lines, content) = compute_diff(ctx)?;

        // Build context map for the script
        let mut ctx_map = Map::new();
        ctx_map.insert("path".into(), Dynamic::from(ctx.path.clone()));
        ctx_map.insert("kind".into(), Dynamic::from(kind_str.to_string()));
        ctx_map.insert("added_lines".into(), Dynamic::from(added_lines));
        ctx_map.insert("deleted_lines".into(), Dynamic::from(deleted_lines));
        ctx_map.insert("content".into(), Dynamic::from(content));

        // Call script function
        let options = CallFnOptions::new()
            .eval_ast(false)
            .rewind_scope(true)
            .bind_this_ptr(&mut self.state);

        let result: Dynamic = self
            .engine
            .call_fn_with_options(
                options,
                &mut self.scope,
                &self.ast,
                "check_change",
                (Dynamic::from(ctx_map),),
            )
            .map_err(|e| anyhow!("Script '{}' check_change failed: {}", self.name, e))?;

        // Map result
        match map_result(result) {
            Some(cr) => Ok(vec![cr]),
            None => Ok(vec![]),
        }
    }

    fn finalize(&mut self) -> Result<Vec<CheckResult>> {
        if !self.has_finalize {
            return Ok(vec![]);
        }

        let options = CallFnOptions::new()
            .eval_ast(false)
            .rewind_scope(true)
            .bind_this_ptr(&mut self.state);

        let result: Dynamic = self
            .engine
            .call_fn_with_options(options, &mut self.scope, &self.ast, "finalize", ())
            .map_err(|e| anyhow!("Script '{}' finalize failed: {}", self.name, e))?;

        match map_result(result) {
            Some(cr) => Ok(vec![cr]),
            None => Ok(vec![]),
        }
    }

    fn reset(&mut self) {
        self.state = Dynamic::from(Map::new());
        self.scope = Scope::new();
    }
}
