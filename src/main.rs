mod config;

use anyhow::{anyhow, Context, Result};
use std::env;

fn main() -> Result<()> {
    let args: Vec<String> = env::args().collect();
    let repo_path = args.get(1).map(|s| s.as_str()).unwrap_or(".");
    let default_config = "validation_rules.json";
    let config_path = args.get(2).map(|s| s.as_str()).unwrap_or(default_config);

    let config = config::Config::from_file(config_path)
        .with_context(|| format!("Failed to load config from {}", config_path))?;
    let rules = config.compile()?;

    println!(
        "Loaded {} validation rules from {}",
        config.rules.len(),
        config_path
    );

    let repo = gix::open(repo_path).context("Failed to open git repository")?;

    let head = repo.head()?;
    let referent_name = head
        .referent_name()
        .ok_or_else(|| anyhow!("HEAD is detached or not on a branch"))?;
    let branch_name = referent_name.shorten().to_string();

    let remote_ref_path = format!("refs/remotes/origin/{}", branch_name);
    println!("Using remote reference: {}", remote_ref_path);

    let head_id = head.id().ok_or_else(|| anyhow!("HEAD not found"))?;
    let remote_ref = repo.find_reference(&remote_ref_path).with_context(|| {
        format!(
            "Could not find {}. Did you run 'git fetch'?",
            remote_ref_path
        )
    })?;
    let remote_id = remote_ref.id();

    println!(
        "Validating commits from {} down to {}...",
        remote_id, head_id
    );

    let mut all_passed = true;

    for commit_info in remote_id.ancestors().first_parent_only().all()? {
        let commit_id: gix::Id<'_> = commit_info?.id();
        if commit_id == head_id {
            break;
        }

        let commit = commit_id.object()?;
        if !validate_commit(&repo, &commit, &rules)? {
            all_passed = false;
        }
    }

    if all_passed {
        println!("\n✅ All commits passed validation.");
        std::process::exit(0);
    } else {
        println!("\n❌ Validation failed. Aborting.");
        std::process::exit(1);
    }
}

fn validate_commit(
    repo: &gix::Repository,
    commit: &gix::Object<'_>,
    rules: &config::CompiledRules,
) -> Result<bool> {
    let mut commit_passed = true;
    let current_tree = commit.clone().into_commit().tree()?;

    let parent_tree = match commit.clone().into_commit().parent_ids().next() {
        Some(parent_id) => parent_id.object()?.into_commit().tree()?,
        None => repo
            .find_object(gix::hash::ObjectId::empty_tree(repo.object_hash()))?
            .into_tree(),
    };

    println!("Checking commit: {}", commit.id);

    parent_tree.changes()?.for_each_to_obtain_tree(
        &current_tree,
        |change: gix::object::tree::diff::Change<'_, '_, '_>| {
            let path = change.location().to_string();

            match change {
                gix::object::tree::diff::Change::Deletion { .. } => {
                    if let Some(rule_name) = &rules.content_deletion {
                        println!("   - ❌ {} - Deletion forbidden: {}", rule_name, path);
                        commit_passed = false;
                    }
                }
                gix::object::tree::diff::Change::Addition { entry_mode, id, .. }
                | gix::object::tree::diff::Change::Modification { entry_mode, id, .. } => {
                    for rule in &rules.filename_match {
                        if rule.globset.is_match(&path) {
                            match rule.action {
                                config::Action::Forbid => {
                                    println!(
                                        "   - ❌ {} - Path matches forbidden pattern: {}",
                                        rule.name, path
                                    );
                                    commit_passed = false;
                                }
                                config::Action::Require => {
                                    println!(
                                        "   - ✅ {} - Path matches required pattern: {}",
                                        rule.name, path
                                    );
                                }
                            }
                        }
                    }

                    for rule in &rules.depth_limit {
                        let parts: Vec<&str> = path.split('/').collect();
                        for (i, segment) in parts.iter().enumerate() {
                            if rule.patterns.contains(&segment.to_string()) && i >= rule.max_depth {
                                println!(
                                    "   - ❌ {} - Folder '{}' too deep (depth {}): {}",
                                    rule.name,
                                    segment,
                                    i + 1,
                                    path
                                );
                                commit_passed = false;
                            }
                        }
                    }

                    if entry_mode.is_blob() {
                        for rule in &rules.content_match {
                            if rule.globset.is_match(&path) {
                                if let Err(e) = validate_blob_content(repo, id, &path) {
                                    println!(
                                        "   - ❌ {} - Content validation failed in {}: {}",
                                        rule.name, path, e
                                    );
                                    commit_passed = false;
                                }
                            }
                        }
                    }
                }
                gix::object::tree::diff::Change::Rewrite { .. } => {}
            }
            Ok::<_, anyhow::Error>(gix::object::tree::diff::Action::Continue)
        },
    )?;

    Ok(commit_passed)
}

fn validate_blob_content(_repo: &gix::Repository, id: gix::Id<'_>, path: &str) -> Result<()> {
    let blob = id.object()?;
    let data = blob.data.as_slice();

    if path.ends_with(".json") {
        serde_json::from_slice::<serde_json::Value>(data)
            .map_err(|e| anyhow!("Invalid JSON in {}: {}", path, e))?;
    } else if path.ends_with(".csv") {
        let mut reader = csv::Reader::from_reader(data);
        for result in reader.records() {
            result.map_err(|e| anyhow!("Invalid CSV in {}: {}", path, e))?;
        }
    }
    Ok(())
}
