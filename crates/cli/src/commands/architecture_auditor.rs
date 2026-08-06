use agent_skills_core::path_safety::sanitize_path;
use clap::{Args, Subcommand};
use regex::Regex;
use std::fs;
use std::path::{Path, PathBuf};

#[derive(Args, Debug, Clone)]
pub struct CheckAuditorArgs {
    /// Target path of the architecture-auditor skill folder
    #[arg(long)]
    pub path: Option<String>,
}

#[derive(Args, Debug, Clone)]
pub struct AnalyzeFileArgs {
    /// Path of the source file to analyze for structural metrics
    #[arg(long)]
    pub file: String,
}

#[derive(Subcommand, Debug, Clone)]
pub enum ArchitectureAuditorSubcommand {
    /// Verify health and structural completeness of architecture-auditor files
    Check(CheckAuditorArgs),
    /// Analyze a single source file for metrics (lines, classes, functions)
    Analyze(AnalyzeFileArgs),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FileMetrics {
    pub file: PathBuf,
    pub lines: usize,
    pub classes: usize,
    pub functions: usize,
}

pub fn analyze_file_metrics(file_path: &Path) -> Result<FileMetrics, String> {
    if !file_path.exists() || !file_path.is_file() {
        return Err(format!("File not found: {}", file_path.display()));
    }

    let content = fs::read_to_string(file_path).map_err(|e| format!("Failed to read file: {e}"))?;
    let lines = content.lines().count();

    let class_re = Regex::new(r"(?m)^\s*(class|struct|enum|trait|interface)\s+\w+").unwrap();
    let func_re = Regex::new(r"(?m)^\s*(def|fn|async\s+fn|function)\s+\w+").unwrap();

    let classes = class_re.find_iter(&content).count();
    let functions = func_re.find_iter(&content).count();

    Ok(FileMetrics {
        file: file_path.to_path_buf(),
        lines,
        classes,
        functions,
    })
}

pub fn check_architecture_auditor_health(skill_dir: &Path) -> Result<Vec<String>, String> {
    let required_files = [
        skill_dir.join("SKILL.md"),
        skill_dir.join("README.md"),
        skill_dir.join("references").join("overview.md"),
        skill_dir.join("references").join("solid.md"),
        skill_dir.join("references").join("dry-yagni.md"),
        skill_dir.join("references").join("cupid.md"),
        skill_dir.join("references").join("kiss.md"),
        skill_dir.join("references").join("principle-tensions.md"),
        skill_dir.join("references").join("audit-report.md"),
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
            "Architecture Auditor health check failed. Missing files: {:?}",
            missing
        ))
    }
}

pub fn run_architecture_auditor_command(
    subcommand: &ArchitectureAuditorSubcommand,
    repo_root: &Path,
) -> Result<(), String> {
    match subcommand {
        ArchitectureAuditorSubcommand::Check(args) => {
            let skill_dir = if let Some(p) = &args.path {
                sanitize_path(p, Some(repo_root))?
            } else {
                repo_root.join("skills").join("architecture-auditor")
            };

            match check_architecture_auditor_health(&skill_dir) {
                Ok(_) => {
                    println!("Architecture Auditor skill health check passed cleanly.");
                    Ok(())
                }
                Err(err) => Err(err),
            }
        }
        ArchitectureAuditorSubcommand::Analyze(args) => {
            let safe_file = sanitize_path(&args.file, Some(repo_root))?;
            let metrics = analyze_file_metrics(&safe_file)?;
            println!(
                "File metrics for {}: lines={}, classes={}, functions={}",
                metrics.file.display(),
                metrics.lines,
                metrics.classes,
                metrics.functions
            );
            Ok(())
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn test_analyze_file_metrics_strict_assertions() {
        let dir = tempdir().unwrap();
        let base = dir.path().canonicalize().unwrap();
        let sample = base.join("sample.rs");
        fs::write(&sample, "struct Sample {}\nfn foo() {}\nfn bar() {}\n").unwrap();

        let metrics = analyze_file_metrics(&sample).unwrap();
        assert_eq!(metrics.lines, 3);
        assert_eq!(metrics.classes, 1);
        assert_eq!(metrics.functions, 2);
    }

    #[test]
    fn test_check_architecture_auditor_health_strict_assertions() {
        let dir = tempdir().unwrap();
        let base = dir.path().canonicalize().unwrap();
        let references = base.join("references");
        fs::create_dir_all(&references).unwrap();

        fs::write(base.join("SKILL.md"), "---").unwrap();
        fs::write(base.join("README.md"), "# Title").unwrap();
        fs::write(references.join("overview.md"), "# Overview").unwrap();
        fs::write(references.join("solid.md"), "# SOLID").unwrap();
        fs::write(references.join("dry-yagni.md"), "# DRY").unwrap();
        fs::write(references.join("cupid.md"), "# CUPID").unwrap();
        fs::write(references.join("kiss.md"), "# KISS").unwrap();
        fs::write(references.join("principle-tensions.md"), "# Tensions").unwrap();
        fs::write(references.join("audit-report.md"), "# Report").unwrap();

        let res = check_architecture_auditor_health(&base);
        assert!(res.is_ok());
    }
}
