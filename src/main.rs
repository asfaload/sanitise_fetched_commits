use anyhow::{anyhow, Context, Result};

fn main() -> Result<()> {
    // 1. Open the repository
    let repo = gix::open(".").context("Failed to open git repository")?;

    // 2. Resolve the range: HEAD..origin/main
    let head_id = repo.head()?.id().ok_or_else(|| anyhow!("HEAD not found"))?;
    let remote_ref = repo
        .find_reference("refs/remotes/origin/main")
        .context("Could not find origin/main. Did you run 'git fetch'?")?;
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
