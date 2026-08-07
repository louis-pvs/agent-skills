use agent_skills_core::depgraph::{generate_lockfile, verify_graph};
use clap::Args;
use serde_json::json;
use std::path::Path;

#[derive(Args, Debug, Clone)]
pub struct DepgraphArgs {
    /// Generate or update skills.lock file
    #[arg(long)]
    pub generate_lock: bool,

    /// Verify graph integrity and lockfile synchronization
    #[arg(long)]
    pub verify: bool,

    /// Output results in JSON format
    #[arg(long)]
    pub json: bool,

    /// Preview actions without writing lockfile to disk
    #[arg(long)]
    pub dry_run: bool,
}

pub fn run_depgraph(args: &DepgraphArgs, repo_root: &Path) -> anyhow::Result<()> {
    let skills_dir = repo_root.join("skills");
    let lockfile_path = repo_root.join("skills.lock");

    if args.dry_run {
        println!(
            "[DRY-RUN] Would analyze dependency graph for skills at: {}",
            skills_dir.display()
        );
        return Ok(());
    }

    if args.generate_lock {
        let lockfile = generate_lockfile(&skills_dir, &lockfile_path)?;
        if args.json {
            let json_out = serde_json::to_string_pretty(&lockfile)
                .map_err(|e| anyhow::anyhow!("Failed to format JSON output: {e}"))?;
            println!("{json_out}");
        } else {
            println!(
                "🎉 Successfully generated 'skills.lock' with {} skill(s).",
                lockfile.skills.len()
            );
        }
        return Ok(());
    }

    let (is_valid, errors, warnings) = verify_graph(&skills_dir, &lockfile_path);

    if args.json {
        let output = json!({
            "valid": is_valid,
            "errors": errors,
            "warnings": warnings,
        });
        println!("{}", serde_json::to_string_pretty(&output).unwrap());
    } else if is_valid {
        println!("✅ Skill dependency graph is VALID and in sync with skills.lock!");
        for w in &warnings {
            println!("  ⚠️ {w}");
        }
    } else {
        eprintln!("❌ Skill dependency graph verification FAILED:");
        for err in &errors {
            eprintln!("  ❌ {err}");
        }
        return Err(anyhow::anyhow!("Dependency graph verification failed."));
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn test_run_depgraph_dry_run_strict_assertions() {
        let dir = tempdir().unwrap();
        let root = dir.path().canonicalize().unwrap();

        let args = DepgraphArgs {
            generate_lock: false,
            verify: true,
            json: false,
            dry_run: true,
        };

        let res = run_depgraph(&args, &root);
        assert!(res.is_ok());
    }
}
