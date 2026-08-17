use agent_skills_core::depgraph::{generate_lockfile, verify_graph};
use clap::Args;
use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};

#[cfg(unix)]
fn install_skill_dir(src: &Path, dst: &Path) -> std::io::Result<()> {
    std::os::unix::fs::symlink(src, dst)
}

#[cfg(windows)]
fn copy_dir_all(src: &Path, dst: &Path) -> std::io::Result<()> {
    fs::create_dir_all(dst)?;
    for entry in fs::read_dir(src)? {
        let entry = entry?;
        let ty = entry.file_type()?;
        let from = entry.path();
        let to = dst.join(entry.file_name());
        if ty.is_dir() {
            copy_dir_all(&from, &to)?;
        } else {
            fs::copy(&from, &to)?;
        }
    }
    Ok(())
}

#[cfg(windows)]
fn install_skill_dir(src: &Path, dst: &Path) -> std::io::Result<()> {
    copy_dir_all(src, dst)
}

/// Recursively compares two directory trees by relative path and file content.
/// Used to detect a stale copy-installed skill (Windows) where only comparing
/// SKILL.md would miss changes to scripts/references files (#1122-style drift).
fn dirs_equal(a: &Path, b: &Path) -> bool {
    fn collect_files(
        root: &Path,
        prefix: &Path,
        out: &mut BTreeMap<PathBuf, PathBuf>,
    ) -> std::io::Result<()> {
        for entry in fs::read_dir(root)? {
            let entry = entry?;
            let ty = entry.file_type()?;
            let rel = prefix.join(entry.file_name());
            if ty.is_dir() {
                collect_files(&entry.path(), &rel, out)?;
            } else {
                out.insert(rel, entry.path());
            }
        }
        Ok(())
    }

    let mut files_a = BTreeMap::new();
    let mut files_b = BTreeMap::new();
    if collect_files(a, Path::new(""), &mut files_a).is_err()
        || collect_files(b, Path::new(""), &mut files_b).is_err()
    {
        return false;
    }

    if files_a.keys().ne(files_b.keys()) {
        return false;
    }

    files_a.iter().all(|(rel, path_a)| {
        let path_b = &files_b[rel];
        matches!((fs::read(path_a), fs::read(path_b)), (Ok(ca), Ok(cb)) if ca == cb)
    })
}

#[derive(Args, Debug, Clone)]
pub struct InstallArgs {
    /// Remove global symlinks instead of creating them
    #[arg(long)]
    pub unlink: bool,

    /// Show planned actions without modifying filesystem
    #[arg(long)]
    pub dry_run: bool,
}

pub fn get_default_global_targets() -> BTreeMap<String, PathBuf> {
    let home = dirs::home_dir().unwrap_or_else(|| PathBuf::from("~"));
    let mut map = BTreeMap::new();
    map.insert(
        "Gemini / Antigravity".to_string(),
        home.join(".gemini").join("config").join("skills"),
    );
    map.insert(
        "Claude Code".to_string(),
        home.join(".claude").join("skills"),
    );
    map.insert(
        "GitHub Copilot".to_string(),
        home.join(".copilot").join("skills"),
    );
    map
}

pub fn find_repo_skills(skills_dir: &Path) -> Vec<PathBuf> {
    if !skills_dir.exists() {
        return Vec::new();
    }
    let mut skills = Vec::new();
    if let Ok(entries) = fs::read_dir(skills_dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() && path.join("SKILL.md").exists() {
                skills.push(path);
            }
        }
    }
    skills.sort();
    skills
}

pub fn install_skill_symlink(skill_dir: &Path, target_base_dir: &Path, dry_run: bool) -> String {
    let skill_name = match skill_dir.file_name() {
        Some(name) => name.to_string_lossy().to_string(),
        None => return "[ERROR] Invalid skill path".to_string(),
    };

    let target_link = target_base_dir.join(&skill_name);

    if !dry_run {
        if let Err(e) = fs::create_dir_all(target_base_dir) {
            return format!(
                "[ERROR] Failed to create dir '{}': {e}",
                target_base_dir.display()
            );
        }
    }

    if target_link.is_symlink() || target_link.exists() {
        let normalize = |p: &Path| -> PathBuf {
            let s = p.to_string_lossy();
            let stripped = s.strip_prefix(r"\\?\").unwrap_or(&s);
            PathBuf::from(stripped)
        };

        let skill_canonical = skill_dir
            .canonicalize()
            .unwrap_or_else(|_| skill_dir.to_path_buf());
        let norm_skill_canonical = normalize(&skill_canonical);
        let norm_skill_orig = normalize(skill_dir);

        let is_same = if target_link.is_symlink() {
            if let Ok(current_target) = fs::read_link(&target_link) {
                let norm_target = normalize(&current_target);
                norm_target == norm_skill_canonical || norm_target == norm_skill_orig
            } else {
                false
            }
        } else if target_link.is_dir() && target_link.join("SKILL.md").exists() {
            dirs_equal(skill_dir, &target_link)
        } else {
            false
        };

        if is_same {
            return format!(
                "[EXISTS] {skill_name} already linked in {}",
                target_base_dir.display()
            );
        }

        if !dry_run {
            let _ = fs::remove_dir_all(&target_link);
            let _ = fs::remove_file(&target_link);
            let _ = fs::remove_dir(&target_link);
        }
    }

    if dry_run {
        return format!(
            "[DRY-RUN] Would link {skill_name} -> {}",
            target_link.display()
        );
    }

    match install_skill_dir(skill_dir, &target_link) {
        Ok(_) => format!("[LINKED] {skill_name} -> {}", target_link.display()),
        Err(e) => format!(
            "[ERROR] Failed to link {skill_name} in {}: {e}",
            target_base_dir.display()
        ),
    }
}

pub fn uninstall_skill_symlink(skill_dir: &Path, target_base_dir: &Path, dry_run: bool) -> String {
    let skill_name = match skill_dir.file_name() {
        Some(name) => name.to_string_lossy().to_string(),
        None => return "[ERROR] Invalid skill path".to_string(),
    };

    let target_link = target_base_dir.join(&skill_name);

    if !target_link.exists() && !target_link.is_symlink() {
        return format!(
            "[SKIP] {skill_name} not found in {}",
            target_base_dir.display()
        );
    }

    if dry_run {
        return format!("[DRY-RUN] Would remove {}", target_link.display());
    }

    if target_link.is_symlink() || target_link.is_file() {
        if let Err(_e) = fs::remove_file(&target_link) {
            if let Err(e2) = fs::remove_dir(&target_link) {
                return format!(
                    "[ERROR] Failed to remove link {}: {e2}",
                    target_link.display()
                );
            }
        }
    } else if target_link.is_dir() {
        if let Err(_e) = fs::remove_dir(&target_link) {
            if let Err(e2) = fs::remove_dir_all(&target_link) {
                return format!(
                    "[ERROR] Failed to remove dir {}: {e2}",
                    target_link.display()
                );
            }
        }
    }

    format!("[REMOVED] {}", target_link.display())
}

pub fn configure_file_with_cargo_path(
    path: &Path,
    cargo_line: &str,
    dry_run: bool,
) -> Option<String> {
    if path.exists() {
        if let Ok(content) = fs::read_to_string(path) {
            if content.contains(".cargo\\bin") || content.contains(".cargo/bin") {
                return Some(format!(
                    "[EXISTS] Cargo bin PATH configured in {}",
                    path.display()
                ));
            }
        }
        if dry_run {
            Some(format!(
                "[DRY-RUN] Would append Cargo PATH to {}",
                path.display()
            ))
        } else if let Ok(mut content) = fs::read_to_string(path) {
            if !content.ends_with('\n') && !content.is_empty() {
                content.push('\n');
            }
            content.push_str(cargo_line);
            content.push('\n');
            if fs::write(path, content).is_ok() {
                Some(format!(
                    "[CONFIGURED] Appended Cargo bin PATH to {}",
                    path.display()
                ))
            } else {
                Some(format!(
                    "[ERROR] Failed to write Cargo PATH to {}",
                    path.display()
                ))
            }
        } else {
            None
        }
    } else if !dry_run {
        if let Some(parent) = path.parent() {
            let _ = fs::create_dir_all(parent);
        }
        let content = format!("{cargo_line}\n");
        if fs::write(path, content).is_ok() {
            Some(format!(
                "[CONFIGURED] Created profile with Cargo bin PATH at {}",
                path.display()
            ))
        } else {
            Some(format!(
                "[ERROR] Failed to create profile at {}",
                path.display()
            ))
        }
    } else {
        Some(format!(
            "[DRY-RUN] Would create profile with Cargo bin PATH at {}",
            path.display()
        ))
    }
}

pub fn ensure_shell_profile_environment(dry_run: bool) -> Vec<String> {
    let mut messages = Vec::new();
    let home = match dirs::home_dir() {
        Some(h) => h,
        None => {
            return vec![
                "[SKIP] Could not determine home directory for shell profiles.".to_string(),
            ]
        }
    };

    #[cfg(windows)]
    {
        let cargo_line = "$env:PATH = \"$env:USERPROFILE\\.cargo\\bin;$env:PATH\"";
        let mut profile_paths = vec![
            home.join("Documents")
                .join("PowerShell")
                .join("Microsoft.PowerShell_profile.ps1"),
            home.join("Documents")
                .join("WindowsPowerShell")
                .join("Microsoft.PowerShell_profile.ps1"),
        ];

        if let Ok(entries) = fs::read_dir(&home) {
            for entry in entries.flatten() {
                let name = entry.file_name().to_string_lossy().to_string();
                if name.starts_with("OneDrive") && entry.path().is_dir() {
                    profile_paths.push(
                        entry
                            .path()
                            .join("Documents")
                            .join("PowerShell")
                            .join("Microsoft.PowerShell_profile.ps1"),
                    );
                    profile_paths.push(
                        entry
                            .path()
                            .join("Documents")
                            .join("WindowsPowerShell")
                            .join("Microsoft.PowerShell_profile.ps1"),
                    );
                }
            }
        }

        for path in profile_paths {
            if let Some(msg) = configure_file_with_cargo_path(&path, cargo_line, dry_run) {
                messages.push(msg);
            }
        }
    }

    #[cfg(unix)]
    {
        let cargo_line = "export PATH=\"$HOME/.cargo/bin:$PATH\"";
        let profile_paths = vec![
            home.join(".bashrc"),
            home.join(".zshrc"),
            home.join(".profile"),
        ];

        for path in profile_paths {
            if let Some(msg) = configure_file_with_cargo_path(&path, cargo_line, dry_run) {
                messages.push(msg);
            }
        }
    }

    messages
}

pub fn run_installer(args: &InstallArgs, repo_root: &Path) -> anyhow::Result<()> {
    let skills_dir = repo_root.join("skills");
    let lockfile_path = repo_root.join("skills.lock");

    let skills = find_repo_skills(&skills_dir);
    if skills.is_empty() {
        return Err(anyhow::anyhow!(
            "No valid skills found in {}",
            skills_dir.display()
        ));
    }

    if !args.unlink {
        println!("Validating skill dependency graph...");
        let (is_valid, _errors, warnings) = verify_graph(&skills_dir, &lockfile_path);
        for w in &warnings {
            println!("  ⚠️ {w}");
        }

        if !is_valid {
            println!("  Regenerating lockfile...");
            if let Err(e) = generate_lockfile(&skills_dir, &lockfile_path) {
                return Err(anyhow::anyhow!(
                    "Skill dependency graph verification failed: {e}"
                ));
            }
        }
        println!("✅ Skill dependency graph validated successfully.\n");
    }

    let action_str = if args.unlink {
        "Uninstalling"
    } else {
        "Installing"
    };
    println!("{action_str} {} skill(s) globally...\n", skills.len());

    let targets = get_default_global_targets();
    for (agent_name, target_dir) in &targets {
        println!("=== {agent_name} ({}) ===", target_dir.display());
        for skill in &skills {
            let msg = if args.unlink {
                uninstall_skill_symlink(skill, target_dir, args.dry_run)
            } else {
                install_skill_symlink(skill, target_dir, args.dry_run)
            };
            println!("  {msg}");
        }
        println!();
    }

    if !args.unlink {
        println!("=== Shell Profile Environment Configuration ===");
        let profile_msgs = ensure_shell_profile_environment(args.dry_run);
        for msg in profile_msgs {
            println!("  {msg}");
        }
        println!();
    }

    println!("Done!");
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn test_install_skill_symlink_strict_assertions() {
        let dir = tempdir().unwrap();
        let base = dir.path().canonicalize().unwrap();

        let skill_dir = base.join("skills").join("test-skill");
        fs::create_dir_all(&skill_dir).unwrap();
        fs::write(skill_dir.join("SKILL.md"), "# Test").unwrap();

        let target_dir = base.join("global_target");

        let result = install_skill_symlink(&skill_dir, &target_dir, false);
        assert!(result.contains("[LINKED]"));

        let link_path = target_dir.join("test-skill");
        assert!(link_path.exists());
        #[cfg(unix)]
        assert!(link_path.is_symlink());

        let uninst_res = uninstall_skill_symlink(&skill_dir, &target_dir, false);
        assert!(uninst_res.contains("[REMOVED]"));
        assert!(!link_path.exists());
    }

    #[test]
    fn test_find_repo_skills() {
        let dir = tempdir().unwrap();
        let skills_dir = dir.path().join("skills");
        fs::create_dir_all(skills_dir.join("skill-1")).unwrap();
        fs::write(skills_dir.join("skill-1").join("SKILL.md"), "# Skill 1").unwrap();
        fs::create_dir_all(skills_dir.join("not-a-skill")).unwrap();

        let found = find_repo_skills(&skills_dir);
        assert_eq!(found.len(), 1);
        assert_eq!(found[0].file_name().unwrap(), "skill-1");
    }

    #[test]
    fn test_install_dry_run() {
        let dir = tempdir().unwrap();
        let base = dir.path().canonicalize().unwrap();
        let skill_dir = base.join("skills").join("test-skill");
        fs::create_dir_all(&skill_dir).unwrap();
        fs::write(skill_dir.join("SKILL.md"), "# Test").unwrap();
        let target_dir = base.join("global_target");

        let res = install_skill_symlink(&skill_dir, &target_dir, true);
        assert!(res.contains("[DRY-RUN]"));
        assert!(!target_dir.join("test-skill").exists());
    }

    #[test]
    fn test_install_idempotency_exists() {
        let dir = tempdir().unwrap();
        let base = dir.path().canonicalize().unwrap();
        let skill_dir = base.join("skills").join("test-skill");
        fs::create_dir_all(&skill_dir).unwrap();
        fs::write(skill_dir.join("SKILL.md"), "# Test").unwrap();
        let target_dir = base.join("global_target");

        install_skill_symlink(&skill_dir, &target_dir, false);
        let res2 = install_skill_symlink(&skill_dir, &target_dir, false);
        assert!(res2.contains("[EXISTS]"));
    }

    #[test]
    fn test_configure_file_with_cargo_path_creation_and_idempotency() {
        let dir = tempdir().unwrap();
        let profile_file = dir.path().join("sub").join("profile.ps1");
        let cargo_line = "$env:PATH = \"$env:USERPROFILE\\.cargo\\bin;$env:PATH\"";

        // 1. Dry run on non-existent file
        let dry_res = configure_file_with_cargo_path(&profile_file, cargo_line, true);
        assert!(dry_res.unwrap().contains("[DRY-RUN]"));
        assert!(!profile_file.exists());

        // 2. Real creation
        let create_res = configure_file_with_cargo_path(&profile_file, cargo_line, false);
        assert!(create_res.unwrap().contains("[CONFIGURED]"));
        assert!(profile_file.exists());
        let content = fs::read_to_string(&profile_file).unwrap();
        assert!(content.contains(cargo_line));

        // 3. Idempotent re-run
        let exists_res = configure_file_with_cargo_path(&profile_file, cargo_line, false);
        assert!(exists_res.unwrap().contains("[EXISTS]"));
    }

    #[test]
    fn test_configure_file_with_cargo_path_appends_cleanly_to_existing_content() {
        let dir = tempdir().unwrap();
        let profile_file = dir.path().join("profile.ps1");
        fs::write(&profile_file, "# existing comment without newline").unwrap();

        let cargo_line = "$env:PATH = \"$env:USERPROFILE\\.cargo\\bin;$env:PATH\"";
        let res = configure_file_with_cargo_path(&profile_file, cargo_line, false);
        assert!(res.unwrap().contains("[CONFIGURED]"));

        let content = fs::read_to_string(&profile_file).unwrap();
        assert!(content.starts_with("# existing comment without newline\n"));
        assert!(content.contains(cargo_line));
    }

    #[test]
    fn test_configure_file_with_cargo_path_skips_unix_path() {
        let dir = tempdir().unwrap();
        let profile_file = dir.path().join(".bashrc");
        fs::write(&profile_file, "export PATH=\"$HOME/.cargo/bin:$PATH\"\n").unwrap();

        let cargo_line = "export PATH=\"$HOME/.cargo/bin:$PATH\"";
        let res = configure_file_with_cargo_path(&profile_file, cargo_line, false);
        assert!(res.unwrap().contains("[EXISTS]"));
    }
}
