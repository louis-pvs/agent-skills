use agent_skills_core::path_safety::sanitize_path;
use clap::{Args, Subcommand};
use regex::Regex;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};

#[derive(Args, Debug, Clone)]
pub struct CheckConflictArgs {
    /// Target path of the git-conflict-resolver skill folder
    #[arg(long)]
    pub path: Option<String>,
}

#[derive(Args, Debug, Clone)]
pub struct AnalyzeConflictArgs {
    /// Path of the source file to analyze or verify conflict markers
    #[arg(long)]
    pub file: Option<String>,

    /// Verify zero conflict markers across the workspace
    #[arg(long)]
    pub verify: bool,

    /// Output findings in JSON format
    #[arg(long)]
    pub json: bool,
}

#[derive(Subcommand, Debug, Clone)]
pub enum GitConflictResolverSubcommand {
    /// Verify health and structural completeness of git-conflict-resolver files
    Check(CheckConflictArgs),
    /// Analyze active Git conflict markers or verify clean resolution
    Analyze(AnalyzeConflictArgs),
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ConflictMarkerLocation {
    pub file: PathBuf,
    pub line_number: usize,
    pub marker_type: String,
    pub line_content: String,
}

pub fn detect_conflict_markers(file_path: &Path) -> anyhow::Result<Vec<ConflictMarkerLocation>> {
    if !file_path.exists() || !file_path.is_file() {
        return Err(anyhow::anyhow!("File not found: {}", file_path.display()));
    }

    let content =
        fs::read_to_string(file_path).map_err(|e| anyhow::anyhow!("Failed to read file: {e}"))?;
    let mut markers = Vec::new();

    let start_re = Regex::new(r"^<{7}\s*(.*)$").unwrap();
    let base_re = Regex::new(r"^\|{7}\s*(.*)$").unwrap();
    let sep_re = Regex::new(r"^={7}\s*$").unwrap();
    let end_re = Regex::new(r"^>{7}\s*(.*)$").unwrap();

    for (idx, line) in content.lines().enumerate() {
        let line_number = idx + 1;
        if start_re.is_match(line) {
            markers.push(ConflictMarkerLocation {
                file: file_path.to_path_buf(),
                line_number,
                marker_type: "START".to_string(),
                line_content: line.to_string(),
            });
        } else if base_re.is_match(line) {
            markers.push(ConflictMarkerLocation {
                file: file_path.to_path_buf(),
                line_number,
                marker_type: "BASE".to_string(),
                line_content: line.to_string(),
            });
        } else if sep_re.is_match(line) {
            markers.push(ConflictMarkerLocation {
                file: file_path.to_path_buf(),
                line_number,
                marker_type: "SEP".to_string(),
                line_content: line.to_string(),
            });
        } else if end_re.is_match(line) {
            markers.push(ConflictMarkerLocation {
                file: file_path.to_path_buf(),
                line_number,
                marker_type: "END".to_string(),
                line_content: line.to_string(),
            });
        }
    }

    Ok(markers)
}

#[allow(dead_code)]
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ConflictSection {
    pub ours_label: String,
    pub ours_content: String,
    pub theirs_label: String,
    pub theirs_content: String,
}

#[allow(dead_code)]
pub fn group_conflict_sections(
    _markers: &[ConflictMarkerLocation],
    file_content: &str,
) -> Vec<ConflictSection> {
    let mut sections = Vec::new();
    let mut in_conflict = false;
    let mut in_theirs = false;
    let mut ours_label = String::new();
    let mut ours_content = String::new();
    let mut theirs_content = String::new();

    for line in file_content.lines() {
        if line.starts_with("<<<<<<<") {
            in_conflict = true;
            in_theirs = false;
            ours_label = line.trim_start_matches('<').trim().to_string();
            ours_content.clear();
            theirs_content.clear();
        } else if line.starts_with("=======") {
            in_theirs = true;
        } else if line.starts_with(">>>>>>>") {
            if in_conflict {
                let theirs_label = line.trim_start_matches('>').trim().to_string();
                sections.push(ConflictSection {
                    ours_label: ours_label.clone(),
                    ours_content: ours_content.trim().to_string(),
                    theirs_label,
                    theirs_content: theirs_content.trim().to_string(),
                });
                in_conflict = false;
                in_theirs = false;
            }
        } else if in_conflict {
            if !in_theirs {
                if !ours_content.is_empty() {
                    ours_content.push('\n');
                }
                ours_content.push_str(line);
            } else {
                if !theirs_content.is_empty() {
                    theirs_content.push('\n');
                }
                theirs_content.push_str(line);
            }
        }
    }

    sections
}

pub fn check_git_conflict_resolver_health(skill_dir: &Path) -> anyhow::Result<Vec<String>> {
    let required_files = [
        skill_dir.join("SKILL.md"),
        skill_dir.join("README.md"),
        skill_dir.join("references").join("overview.md"),
        skill_dir
            .join("references")
            .join("resolution-strategies.md"),
        skill_dir.join("templates").join("resolution_report.md"),
    ];

    let mut missing = Vec::new();
    for req in &required_files {
        if !req.is_file() {
            missing.push(
                req.strip_prefix(skill_dir)
                    .unwrap_or(req)
                    .display()
                    .to_string(),
            );
        }
    }

    if missing.is_empty() {
        Ok(Vec::new())
    } else {
        Err(anyhow::anyhow!(
            "Git Conflict Resolver health check failed. Missing files: {:?}",
            missing
        ))
    }
}

pub fn run_git_conflict_resolver_command(
    subcommand: &GitConflictResolverSubcommand,
    repo_root: &Path,
) -> anyhow::Result<()> {
    match subcommand {
        GitConflictResolverSubcommand::Check(args) => {
            let skill_dir = if let Some(p) = &args.path {
                sanitize_path(p, Some(repo_root))?
            } else {
                repo_root.join("skills").join("git-conflict-resolver")
            };

            match check_git_conflict_resolver_health(&skill_dir) {
                Ok(_) => {
                    println!("Git Conflict Resolver skill health check passed cleanly.");
                    Ok(())
                }
                Err(err) => Err(err),
            }
        }
        GitConflictResolverSubcommand::Analyze(args) => {
            let mut files_to_scan = Vec::new();
            if let Some(f) = &args.file {
                let safe_path = sanitize_path(f, Some(repo_root))?;
                files_to_scan.push(safe_path);
            } else {
                // Scan workspace files
                let mut walk = vec![repo_root.to_path_buf()];
                while let Some(curr) = walk.pop() {
                    if let Ok(entries) = fs::read_dir(curr) {
                        for entry in entries.flatten() {
                            let p = entry.path();
                            let fname = p.file_name().unwrap_or_default().to_string_lossy();
                            if fname.starts_with('.')
                                || fname == "target"
                                || fname == "node_modules"
                            {
                                continue;
                            }
                            if p.is_dir() {
                                walk.push(p);
                            } else if p.is_file() {
                                files_to_scan.push(p);
                            }
                        }
                    }
                }
            }

            let mut all_markers = Vec::new();
            for file in files_to_scan {
                if let Ok(markers) = detect_conflict_markers(&file) {
                    all_markers.extend(markers);
                }
            }

            if args.verify {
                if all_markers.is_empty() {
                    println!("✅ Verification PASSED: Zero conflict markers detected.");
                    Ok(())
                } else {
                    Err(anyhow::anyhow!(
                        "❌ Verification FAILED: Found {} conflict markers in workspace.",
                        all_markers.len()
                    ))
                }
            } else if args.json {
                println!("{}", serde_json::to_string_pretty(&all_markers).unwrap());
                Ok(())
            } else {
                println!(
                    "Conflict Analysis Results (Total Markers: {}):",
                    all_markers.len()
                );
                for m in all_markers {
                    println!(
                        "  [{}] {}:{}: {}",
                        m.marker_type,
                        m.file.display(),
                        m.line_number,
                        m.line_content
                    );
                }
                Ok(())
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn test_detect_conflict_markers_strict_assertions() {
        let dir = tempdir().unwrap();
        let base = dir.path().canonicalize().unwrap();
        let sample = base.join("conflicted.rs");
        let content =
            "<<<<<<< HEAD\nprintln!(\"ours\");\n=======\nprintln!(\"theirs\");\n>>>>>>> incoming\n";
        fs::write(&sample, content).unwrap();

        let markers = detect_conflict_markers(&sample).unwrap();
        assert_eq!(markers.len(), 3);
        assert_eq!(markers[0].marker_type, "START");
        assert_eq!(markers[1].marker_type, "SEP");
        assert_eq!(markers[2].marker_type, "END");
    }

    #[test]
    fn test_check_git_conflict_resolver_health_strict_assertions() {
        let dir = tempdir().unwrap();
        let base = dir.path().canonicalize().unwrap();
        let references = base.join("references");
        let templates = base.join("templates");
        fs::create_dir_all(&references).unwrap();
        fs::create_dir_all(&templates).unwrap();

        fs::write(base.join("SKILL.md"), "---").unwrap();
        fs::write(base.join("README.md"), "# Title").unwrap();
        fs::write(references.join("overview.md"), "# Overview").unwrap();
        fs::write(references.join("resolution-strategies.md"), "# Strategies").unwrap();
        fs::write(templates.join("resolution_report.md"), "# Report").unwrap();

        let res = check_git_conflict_resolver_health(&base);
        assert!(res.is_ok());
    }

    #[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
    pub enum GitState {
        Clean,
        Rebasing,
        MergeInProgress,
        DetachedHead,
        NotAGitRepo,
    }

    pub fn detect_git_state(path: &Path) -> GitState {
        let git_dir = path.join(".git");
        if !git_dir.exists() {
            return GitState::NotAGitRepo;
        }
        if git_dir.join("rebase-apply").exists() || git_dir.join("rebase-merge").exists() {
            return GitState::Rebasing;
        }
        if git_dir.join("MERGE_HEAD").exists() {
            return GitState::MergeInProgress;
        }
        if let Ok(head) = fs::read_to_string(git_dir.join("HEAD")) {
            if !head.starts_with("ref:") {
                return GitState::DetachedHead;
            }
        }
        GitState::Clean
    }

    #[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
    pub struct ConflictReport {
        pub total_conflicts: usize,
        pub conflicted_files: Vec<PathBuf>,
        pub markers: Vec<ConflictMarkerLocation>,
    }

    pub fn analyze_repository_conflicts(base_path: &Path) -> anyhow::Result<ConflictReport> {
        let mut files_to_scan = Vec::new();
        let mut walk = vec![base_path.to_path_buf()];
        while let Some(curr) = walk.pop() {
            if let Ok(entries) = fs::read_dir(curr) {
                for entry in entries.flatten() {
                    let p = entry.path();
                    let fname = p.file_name().unwrap_or_default().to_string_lossy();
                    if fname.starts_with('.') || fname == "target" || fname == "node_modules" {
                        continue;
                    }
                    if p.is_dir() {
                        walk.push(p);
                    } else if p.is_file() {
                        files_to_scan.push(p);
                    }
                }
            }
        }

        let mut markers = Vec::new();
        let mut conflicted_files = Vec::new();

        for file in files_to_scan {
            if let Ok(m) = detect_conflict_markers(&file) {
                if !m.is_empty() {
                    conflicted_files.push(file);
                    markers.extend(m);
                }
            }
        }

        Ok(ConflictReport {
            total_conflicts: markers.len(),
            conflicted_files,
            markers,
        })
    }

    #[test]
    fn test_detect_git_state_temporary_dir() {
        let dir = tempdir().unwrap();
        let state = detect_git_state(dir.path());
        assert!(
            state == GitState::NotAGitRepo || state == GitState::Clean,
            "detect_git_state must return valid state for temp dir, got: {:?}",
            state
        );
    }

    #[test]
    fn test_analyze_repository_conflicts_structure() {
        let dir = tempdir().unwrap();
        let base = dir.path().canonicalize().unwrap();
        fs::write(
            base.join("conflicted.rs"),
            "<<<<<<< HEAD\nours\n=======\ntheirs\n>>>>>>> incoming\n",
        )
        .unwrap();

        let report = analyze_repository_conflicts(&base).unwrap();
        assert_eq!(report.conflicted_files.len(), 1);
        assert_eq!(report.total_conflicts, 3);
    }

    #[test]
    fn test_conflict_marker_json_output() {
        // Python: test_json_flag — analyze subcommand with --json must output a JSON array
        // of ConflictMarkerLocation objects with file/line_number/marker_type fields.
        let dir = tempdir().unwrap();
        let base = dir.path().canonicalize().unwrap();
        let sample = base.join("conflicted.rs");
        let content =
            "<<<<<<< HEAD\nprintln!(\"ours\");\n=======\nprintln!(\"theirs\");\n>>>>>>> incoming\n";
        fs::write(&sample, content).unwrap();

        let markers = detect_conflict_markers(&sample).unwrap();
        let json_str = serde_json::to_string(&markers).unwrap();
        let parsed: serde_json::Value = serde_json::from_str(&json_str).unwrap();

        // Each entry must have file, line_number, marker_type fields
        let entry = &parsed[0];
        assert!(
            entry.get("file").is_some(),
            "JSON output must include 'file' field"
        );
        assert!(
            entry.get("line_number").is_some(),
            "JSON output must include 'line_number' field"
        );
        assert!(
            entry.get("marker_type").is_some(),
            "JSON output must include 'marker_type' field"
        );
        // Must also have content field
        assert!(
            entry.get("line_content").is_some(),
            "JSON output must include 'line_content' field"
        );
    }

    #[test]
    fn test_parse_conflict_markers_two_way_grouping() {
        // Python: test_parse_conflict_markers_two_way — a two-way conflict must be grouped into
        // a ConflictSection with distinct ours/theirs content and labels, not just flat marker
        // positions (detect_conflict_markers only locates markers, doesn't group them).
        let dir = tempdir().unwrap();
        let base = dir.path().canonicalize().unwrap();
        let sample = base.join("conflicted.rs");
        let content =
            "<<<<<<< HEAD\nprintln!(\"ours\");\n=======\nprintln!(\"theirs\");\n>>>>>>> incoming\n";
        fs::write(&sample, content).unwrap();

        let markers = detect_conflict_markers(&sample).unwrap();
        let sections = group_conflict_sections(&markers, content);
        assert_eq!(sections.len(), 1);
        assert_eq!(sections[0].ours_label, "HEAD");
        assert!(sections[0].ours_content.contains("ours"));
        assert_eq!(sections[0].theirs_label, "incoming");
        assert!(sections[0].theirs_content.contains("theirs"));
    }

    #[test]
    fn test_parse_conflict_markers_three_way() {
        // Python: test_parse_conflict_markers_three_way — a three-way conflict (with a BASE
        // section) must be detected including the common-ancestor marker.
        let dir = tempdir().unwrap();
        let base = dir.path().canonicalize().unwrap();
        let sample = base.join("three_way.rs");
        let content = "<<<<<<< HEAD\nours\n||||||| base\ncommon ancestor\n=======\ntheirs\n>>>>>>> incoming\n";
        fs::write(&sample, content).unwrap();

        let markers = detect_conflict_markers(&sample).unwrap();
        let types: Vec<_> = markers.iter().map(|m| m.marker_type.as_str()).collect();
        assert!(types.contains(&"START"));
        assert!(types.contains(&"BASE"));
        assert!(types.contains(&"SEP"));
        assert!(types.contains(&"END"));
    }

    #[test]
    fn test_verify_zero_markers_clean_and_dirty() {
        // Python: test_verify_zero_markers_clean_and_dirty — the --verify CLI path must
        // distinguish a clean file (zero markers, passes) from a dirty one (markers remain, fails).
        let dir = tempdir().unwrap();
        let base = dir.path().canonicalize().unwrap();
        fs::write(base.join("clean.rs"), "fn main() {}\n").unwrap();

        let clean_result = run_git_conflict_resolver_command(
            &GitConflictResolverSubcommand::Analyze(AnalyzeConflictArgs {
                file: None,
                verify: true,
                json: false,
            }),
            &base,
        );
        assert!(
            clean_result.is_ok(),
            "verify must pass when no conflict markers exist"
        );

        fs::write(
            base.join("dirty.rs"),
            "<<<<<<< HEAD\nours\n=======\ntheirs\n>>>>>>> incoming\n",
        )
        .unwrap();

        let dirty_result = run_git_conflict_resolver_command(
            &GitConflictResolverSubcommand::Analyze(AnalyzeConflictArgs {
                file: None,
                verify: true,
                json: false,
            }),
            &base,
        );
        assert!(
            dirty_result.is_err(),
            "verify must fail when conflict markers remain"
        );
    }
}
