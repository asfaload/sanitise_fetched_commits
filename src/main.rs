mod cli;
mod config;
mod rule;
mod rules;

use anyhow::{anyhow, Context, Result};
use clap::Parser;
use gix::prelude::ObjectIdExt;

use cli::{OutputFormat, RunOptions};

fn resolve_ref(repo: &gix::Repository, refspec: &str) -> Result<gix::ObjectId> {
    repo.rev_parse_single(refspec)
        .map(|id| id.detach())
        .with_context(|| format!("Could not resolve ref '{}'", refspec))
}

fn main() -> Result<()> {
    let args = cli::Cli::parse();
    let opts = RunOptions::from(&args);

    let config = config::Config::from_file(&args.config)
        .with_context(|| format!("Failed to load config from {}", args.config))?;

    let issues = config.validate();
    let has_errors = issues
        .iter()
        .any(|i| matches!(i.severity, config::ConfigSeverity::Error));
    if !issues.is_empty() {
        for issue in &issues {
            eprintln!("{}", issue);
        }
        if has_errors {
            std::process::exit(1);
        }
    }

    let rule_count = config.rules.len();
    let mut rules = config.compile()?;

    if matches!(opts.format, OutputFormat::Plain) {
        println!(
            "Loaded {} validation rules from {}",
            rule_count, args.config
        );
    }

    let repo = gix::open(&args.repo).context("Failed to open git repository")?;

    let (start_id, stop_id) = match (&args.to, &args.from) {
        (None, None) => {
            let head = repo.head()?;
            let referent_name = head
                .referent_name()
                .ok_or_else(|| anyhow!("HEAD is detached or not on a branch"))?;
            let branch_name = referent_name.shorten().to_string();
            let remote_ref_path = format!("refs/remotes/{}/{}", opts.remote, branch_name);
            if matches!(opts.format, OutputFormat::Plain) {
                println!("Using remote reference: {}", remote_ref_path);
            }
            let head_id = head
                .id()
                .ok_or_else(|| anyhow!("HEAD not found"))?
                .detach();
            let remote_ref = repo.find_reference(&remote_ref_path).with_context(|| {
                format!(
                    "Could not find {}. Did you run 'git fetch'?",
                    remote_ref_path
                )
            })?;
            let remote_id = remote_ref.id().detach();
            (remote_id, head_id)
        }
        (Some(to), None) => {
            let start = resolve_ref(&repo, to)?;
            let head = repo.head()?;
            let head_id = head
                .id()
                .ok_or_else(|| anyhow!("HEAD not found"))?
                .detach();
            (start, head_id)
        }
        (None, Some(from)) => {
            let stop = resolve_ref(&repo, from)?;
            let head = repo.head()?;
            let referent_name = head
                .referent_name()
                .ok_or_else(|| anyhow!("HEAD is detached or not on a branch"))?;
            let branch_name = referent_name.shorten().to_string();
            let remote_ref_path = format!("refs/remotes/{}/{}", opts.remote, branch_name);
            if matches!(opts.format, OutputFormat::Plain) {
                println!("Using remote reference: {}", remote_ref_path);
            }
            let remote_ref = repo.find_reference(&remote_ref_path).with_context(|| {
                format!(
                    "Could not find {}. Did you run 'git fetch'?",
                    remote_ref_path
                )
            })?;
            let remote_id = remote_ref.id().detach();
            (remote_id, stop)
        }
        (Some(to), Some(from)) => {
            let start = resolve_ref(&repo, to)?;
            let stop = resolve_ref(&repo, from)?;
            (start, stop)
        }
    };

    if matches!(opts.format, OutputFormat::Plain) {
        println!(
            "Validating commits from {} down to {}...",
            start_id, stop_id
        );
    }

    let mut all_passed = true;
    let mut commits_checked: usize = 0;
    let mut total_violations: usize = 0;
    let mut commit_reports: Vec<output::CommitReport> = Vec::new();

    for commit_info in start_id.attach(&repo).ancestors().first_parent_only().all()? {
        let commit_id = commit_info?.id().detach();
        if commit_id == stop_id {
            break;
        }

        let commit = commit_id.attach(&repo).object()?;
        let report = validate_commit(&repo, &commit, &mut rules, &opts)?;
        if !report.passed {
            all_passed = false;
        }
        total_violations += report.violation_count;
        commits_checked += 1;
        commit_reports.push(report);
    }

    let warning = if commits_checked == 0 {
        Some("No commits found in the specified range.".to_string())
    } else {
        None
    };

    match opts.format {
        OutputFormat::Plain => {
            if let Some(ref w) = warning {
                println!("Warning: {}", w);
            }
            println!(
                "\nChecked {} commit(s), {} violation(s) found.",
                commits_checked, total_violations
            );
            if all_passed {
                println!("\u{2705} All commits passed validation.");
            } else if opts.dry_run {
                println!("\u{274c} Validation failed. (dry-run)");
            } else {
                println!("\u{274c} Validation failed. Aborting.");
            }
        }
        OutputFormat::Json => {
            let report = output::Report {
                commits_checked,
                total_violations,
                passed: all_passed,
                commits: commit_reports,
                warning,
            };
            println!("{}", serde_json::to_string_pretty(&report)?);
        }
    }

    if all_passed || opts.dry_run {
        std::process::exit(0);
    } else {
        std::process::exit(1);
    }
}

mod output {
    use serde::Serialize;

    #[derive(Serialize)]
    pub struct Report {
        pub commits_checked: usize,
        pub total_violations: usize,
        pub passed: bool,
        pub commits: Vec<CommitReport>,
        #[serde(skip_serializing_if = "Option::is_none")]
        pub warning: Option<String>,
    }

    #[derive(Serialize)]
    pub struct CommitReport {
        pub hash: String,
        pub message: String,
        pub passed: bool,
        pub violation_count: usize,
        pub results: Vec<ResultEntry>,
    }

    #[derive(Serialize)]
    pub struct ResultEntry {
        pub rule: String,
        pub status: String,
        pub message: String,
    }
}

fn validate_commit(
    repo: &gix::Repository,
    commit: &gix::Object<'_>,
    rules: &mut [Box<dyn rule::Rule>],
    opts: &RunOptions,
) -> Result<output::CommitReport> {
    let commit_obj = commit.clone().into_commit();
    let current_tree = commit_obj.tree()?;
    let message_raw = commit_obj
        .message_raw()
        .map(|m| m.to_string())
        .unwrap_or_default();
    let first_line = message_raw.lines().next().unwrap_or("").to_string();
    let short_hash = &commit.id.to_string()[..8];

    let parent_tree = match commit.clone().into_commit().parent_ids().next() {
        Some(parent_id) => parent_id.object()?.into_commit().tree()?,
        None => repo
            .find_object(gix::hash::ObjectId::empty_tree(repo.object_hash()))?
            .into_tree(),
    };

    if matches!(opts.format, OutputFormat::Plain) {
        println!("Checking commit: {} {}", short_hash, first_line);
    }

    for rule in rules.iter_mut() {
        rule.reset();
    }

    let mut results: Vec<output::ResultEntry> = Vec::new();
    let mut violation_count: usize = 0;

    parent_tree.changes()?.for_each_to_obtain_tree(
        &current_tree,
        |change: gix::object::tree::diff::Change<'_, '_, '_>| {
            let ctx = rule::ChangeContext::from_change(&change, repo);
            for rule in rules.iter_mut() {
                match rule.check_change(&ctx) {
                    Ok(check_results) => {
                        for cr in check_results {
                            if matches!(opts.format, OutputFormat::Plain) {
                                cr.print(rule.name(), opts);
                            }
                            if cr.is_violation() {
                                violation_count += 1;
                            }
                            results.push(output::ResultEntry {
                                rule: rule.name().to_string(),
                                status: cr.status_str().to_string(),
                                message: cr.message().to_string(),
                            });
                        }
                    }
                    Err(e) => {
                        if matches!(opts.format, OutputFormat::Plain) {
                            eprintln!("   - Warning: {} error: {}", rule.name(), e);
                        }
                    }
                }
            }
            Ok::<_, anyhow::Error>(gix::object::tree::diff::Action::Continue)
        },
    )?;

    for rule in rules.iter_mut() {
        for cr in rule.finalize()? {
            if matches!(opts.format, OutputFormat::Plain) {
                cr.print(rule.name(), opts);
            }
            if cr.is_violation() {
                violation_count += 1;
            }
            results.push(output::ResultEntry {
                rule: rule.name().to_string(),
                status: cr.status_str().to_string(),
                message: cr.message().to_string(),
            });
        }
    }

    Ok(output::CommitReport {
        hash: short_hash.to_string(),
        message: first_line,
        passed: violation_count == 0,
        violation_count,
        results,
    })
}
