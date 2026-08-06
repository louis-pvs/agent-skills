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

pub fn detect_conflict_markers(file_path: &Path) -> Result<Vec<ConflictMarkerLocation>, String> {
    if !file_path.exists() || !file_path.is_file() {
        return Err(format!("File not found: {}", file_path.display()));
    }

    let content = fs::read_to_string(file_path).map_err(|e| format!("Failed to read file: {e}"))?;
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

pub fn check_git_conflict_resolver_health(skill_dir: &Path) -> Result<Vec<String>, String> {
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
        Err(format!(
            "Git Conflict Resolver health check failed. Missing files: {:?}",
            missing
        ))
    }
}

pub fn run_git_conflict_resolver_command(
    subcommand: &GitConflictResolverSubcommand,
    repo_root: &Path,
) -> Result<(), String> {
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
                    Err(format!(
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
}
