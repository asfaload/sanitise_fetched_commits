mod cli;
mod config;
mod git;
mod rule;
mod rules;

use anyhow::{Context, Result};
use clap::Parser;

use cli::{OutputFormat, RunOptions};
use git::GitRepo;
use rule::{ChangeContext, ChangeKind};

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

    let repo = GitRepo::open(&args.repo).context("Failed to open git repository")?;

    let (start_id, stop_id) = match (&args.to, &args.from) {
        (None, None) => {
            let branch_name = repo.head_branch()?;
            let remote_ref_path = format!("refs/remotes/{}/{}", opts.remote, branch_name);
            if matches!(opts.format, OutputFormat::Plain) {
                println!("Using remote reference: {}", remote_ref_path);
            }
            let head_id = repo.head_oid()?;
            let remote_id = repo.resolve_ref(&remote_ref_path).with_context(|| {
                format!(
                    "Could not find {}. Did you run 'git fetch'?",
                    remote_ref_path
                )
            })?;
            (remote_id, head_id)
        }
        (Some(to), None) => {
            let start = repo
                .resolve_ref(to)
                .with_context(|| format!("Could not resolve ref '{}'", to))?;
            let head_id = repo.head_oid()?;
            (start, head_id)
        }
        (None, Some(from)) => {
            let stop = repo
                .resolve_ref(from)
                .with_context(|| format!("Could not resolve ref '{}'", from))?;
            let branch_name = repo.head_branch()?;
            let remote_ref_path = format!("refs/remotes/{}/{}", opts.remote, branch_name);
            if matches!(opts.format, OutputFormat::Plain) {
                println!("Using remote reference: {}", remote_ref_path);
            }
            let remote_id = repo.resolve_ref(&remote_ref_path).with_context(|| {
                format!(
                    "Could not find {}. Did you run 'git fetch'?",
                    remote_ref_path
                )
            })?;
            (remote_id, stop)
        }
        (Some(to), Some(from)) => {
            let start = repo
                .resolve_ref(to)
                .with_context(|| format!("Could not resolve ref '{}'", to))?;
            let stop = repo
                .resolve_ref(from)
                .with_context(|| format!("Could not resolve ref '{}'", from))?;
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

    // Walk commits from stop_id (exclusive) to start_id (inclusive), first-parent only.
    // start_id is the newer tip (e.g. remote tracking branch after fetch),
    // stop_id is the older base (e.g. HEAD before fetch).
    let commits = repo.list_commits(&stop_id, &start_id)?;

    for commit_oid in &commits {
        let report = validate_commit(&repo, commit_oid, &mut rules, &opts)?;
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
    repo: &GitRepo,
    commit_oid: &str,
    rules: &mut [Box<dyn rule::Rule>],
    opts: &RunOptions,
) -> Result<output::CommitReport> {
    let first_line = repo.commit_subject(commit_oid)?;
    let short_hash = &commit_oid[..8.min(commit_oid.len())];

    if matches!(opts.format, OutputFormat::Plain) {
        println!("Checking commit: {} {}", short_hash, first_line);
    }

    for rule in rules.iter_mut() {
        rule.reset();
    }

    let mut results: Vec<output::ResultEntry> = Vec::new();
    let mut violation_count: usize = 0;

    let changes = repo.diff_tree(commit_oid)?;

    for change in &changes {
        let is_blob =
            git::is_blob_mode(&change.new_mode) || git::is_blob_mode(&change.old_mode);

        let content = if is_blob && !git::oid_is_null(&change.new_oid) {
            repo.read_blob(&change.new_oid)
                .with_context(|| format!("Failed to read new blob for '{}'", change.path))?
        } else {
            vec![]
        };

        let previous_content = if is_blob && !git::oid_is_null(&change.old_oid) {
            repo.read_blob(&change.old_oid)
                .with_context(|| format!("Failed to read previous blob for '{}'", change.path))?
        } else {
            vec![]
        };

        let kind = match change.kind {
            git::TreeChangeKind::Addition => ChangeKind::Addition,
            git::TreeChangeKind::Deletion => ChangeKind::Deletion,
            git::TreeChangeKind::Modification => ChangeKind::Modification,
            git::TreeChangeKind::Rename => ChangeKind::Rewrite,
        };

        let ctx = ChangeContext {
            path: change.path.clone(),
            kind,
            is_blob,
            content,
            previous_content,
        };

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
    }

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
