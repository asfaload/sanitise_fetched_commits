use anyhow::{anyhow, Context, Result};
use std::env;

fn main() -> Result<()> {
    // 1. Parse command line arguments
    let args: Vec<String> = env::args().collect();
    let repo_path = args.get(1).map(|s| s.as_str()).unwrap_or(".");

    // 2. Open the repository
    let repo = gix::open(repo_path).context("Failed to open git repository")?;

    // 3. Get the current branch name from HEAD
    let head = repo.head()?;
    let referent_name = head
        .referent_name()
        .ok_or_else(|| anyhow!("HEAD is detached or not on a branch"))?;
    let branch_name = referent_name.shorten().to_string();

    // 4. Construct the remote reference path
    let remote_ref_path = format!("refs/remotes/origin/{}", branch_name);
    println!("Using remote reference: {}", remote_ref_path);

    // 5. Resolve the range: HEAD..origin/<branch>
    let head_id = head.id().ok_or_else(|| anyhow!("HEAD not found"))?;
    let remote_ref = repo.find_reference(&remote_ref_path).with_context(|| {
        format!(
            "Could not find {}. Did you run 'git fetch'?",
            remote_ref_path
        )
    })?;
    let remote_id = remote_ref.id();

    // 3. Prepare to walk the commits from remote back to HEAD
    println!(
        "Validating commits from {} down to {}...",
        remote_id, head_id
    );

    let mut all_passed = true;

    for commit_info in remote_id.ancestors().first_parent_only().all()? {
        let commit_id: gix::Id<'_> = commit_info?.id();
        if commit_id == head_id {
            break;
        } // Stop once we reach our local HEAD

        let commit = commit_id.object()?;
        if !validate_commit(&repo, &commit)? {
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

fn validate_commit(repo: &gix::Repository, commit: &gix::Object<'_>) -> Result<bool> {
    let mut commit_passed = true;
    let current_tree = commit.clone().into_commit().tree()?;

    // Get parent tree (handling the case of the first commit if necessary)
    let parent_tree = match commit.clone().into_commit().parent_ids().next() {
        Some(parent_id) => parent_id.object()?.into_commit().tree()?,
        None => repo
            .find_object(gix::hash::ObjectId::empty_tree(repo.object_hash()))?
            .into_tree(),
    };

    println!("Checking commit: {}", commit.id);

    // Diff current tree against parent
    parent_tree.changes()?.for_each_to_obtain_tree(
        &current_tree,
        |change: gix::object::tree::diff::Change<'_, '_, '_>| {
            let path = change.location().to_string();
            let forbidden_names = ["my-dir", "my-dir-pending"];

            match change {
                // Rule 1: No Deletions
                gix::object::tree::diff::Change::Deletion { .. } => {
                    println!("   - ❌ Deletion forbidden: {}", path);
                    commit_passed = false;
                }
                // Rule 2 & 3: Additions
                gix::object::tree::diff::Change::Addition { entry_mode, id, .. }
                // Rule 2 & 3: Modifications
                | gix::object::tree::diff::Change::Modification { entry_mode, id, .. } => {
                    // Rule 2: Check Depth
                    let parts: Vec<&str> = path.split('/').collect();
                    for (i, segment) in parts.iter().enumerate() {
                        if forbidden_names.contains(segment) && i >= 3 {
                            println!(
                                "   - ❌ Folder '{}' too deep (depth {}): {}",
                                segment,
                                i + 1,
                                path
                            );
                            commit_passed = false;
                        }
                    }

                    // Rule 3: Content Validation
                    if entry_mode.is_blob() {
                        if let Err(e) = validate_blob_content(repo, id, &path) {
                            println!("   - ❌ Content Error: {}", e);
                            commit_passed = false;
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
