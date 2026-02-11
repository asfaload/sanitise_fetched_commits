mod config;
mod rule;
mod rules;

use anyhow::{anyhow, Context, Result};
use std::env;

fn main() -> Result<()> {
    let args: Vec<String> = env::args().collect();
    let repo_path = args.get(1).map(|s| s.as_str()).unwrap_or(".");
    let default_config = "validation_rules.json";
    let config_path = args.get(2).map(|s| s.as_str()).unwrap_or(default_config);

    let config = config::Config::from_file(config_path)
        .with_context(|| format!("Failed to load config from {}", config_path))?;

    let rule_count = config.rules.len();
    let mut rules = config.compile()?;

    println!(
        "Loaded {} validation rules from {}",
        rule_count, config_path
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
        if !validate_commit(&repo, &commit, &mut rules)? {
            all_passed = false;
        }
    }

    if all_passed {
        println!("\n\u{2705} All commits passed validation.");
        std::process::exit(0);
    } else {
        println!("\n\u{274c} Validation failed. Aborting.");
        std::process::exit(1);
    }
}

fn validate_commit(
    repo: &gix::Repository,
    commit: &gix::Object<'_>,
    rules: &mut [Box<dyn rule::Rule>],
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

    for rule in rules.iter_mut() {
        rule.reset();
    }

    parent_tree.changes()?.for_each_to_obtain_tree(
        &current_tree,
        |change: gix::object::tree::diff::Change<'_, '_, '_>| {
            let ctx = rule::ChangeContext::from_change(&change, repo);
            for rule in rules.iter_mut() {
                match rule.check_change(&ctx) {
                    Ok(results) => {
                        for result in results {
                            result.print(rule.name());
                            if result.is_violation() {
                                commit_passed = false;
                            }
                        }
                    }
                    Err(e) => {
                        eprintln!("   - Warning: {} error: {}", rule.name(), e);
                    }
                }
            }
            Ok::<_, anyhow::Error>(gix::object::tree::diff::Action::Continue)
        },
    )?;

    for rule in rules.iter_mut() {
        for result in rule.finalize()? {
            result.print(rule.name());
            if result.is_violation() {
                commit_passed = false;
            }
        }
    }

    Ok(commit_passed)
}
