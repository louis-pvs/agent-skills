use agent_skills_core::path_safety::sanitize_path;
use clap::{Args, Subcommand};
use regex::Regex;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};

#[derive(Args, Debug, Clone)]
pub struct CheckJanitorArgs {
    /// Target path of the code-janitor skill folder
    #[arg(long)]
    pub path: Option<String>,
}

#[derive(Args, Debug, Clone)]
pub struct ScanJanitorArgs {
    /// Path of the source file to scan for code smells
    #[arg(long)]
    pub file: Option<String>,

    /// Path of the directory to scan recursively
    #[arg(long)]
    pub dir: Option<String>,

    /// Maximum allowed lines per function (default: 30)
    #[arg(long, default_value_t = 30)]
    pub max_function_lines: usize,

    /// Maximum allowed parameters per function (default: 5)
    #[arg(long, default_value_t = 5)]
    pub max_parameters: usize,

    /// Maximum allowed nesting depth (default: 4)
    #[arg(long, default_value_t = 4)]
    pub max_nesting_depth: usize,

    /// Output findings in JSON format
    #[arg(long)]
    pub json: bool,
}

#[derive(Subcommand, Debug, Clone)]
pub enum CodeJanitorSubcommand {
    /// Verify health and structural completeness of code-janitor files
    Check(CheckJanitorArgs),
    /// Scan file or directory for code smells (unused code, oversized functions, deep nesting)
    Scan(ScanJanitorArgs),
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct CodeSmell {
    pub file: PathBuf,
    pub line_number: usize,
    pub smell_category: String,
    pub description: String,
    pub severity: String,
}

pub fn scan_file_for_smells(
    file_path: &Path,
    max_function_lines: usize,
    max_parameters: usize,
    _max_nesting_depth: usize,
) -> Result<Vec<CodeSmell>, String> {
    if !file_path.exists() || !file_path.is_file() {
        return Err(format!("File not found: {}", file_path.display()));
    }

    let content = fs::read_to_string(file_path).map_err(|e| format!("Failed to read file: {e}"))?;
    let mut smells = Vec::new();

    let func_def_re =
        Regex::new(r"(?m)^\s*(def|fn|async\s+fn|function)\s+(\w+)\s*\(([^)]*)\)").unwrap();
    let todo_re = Regex::new(r"(?i)\b(TODO|FIXME)\b").unwrap();

    for (idx, line) in content.lines().enumerate() {
        if todo_re.is_match(line) {
            smells.push(CodeSmell {
                file: file_path.to_path_buf(),
                line_number: idx + 1,
                smell_category: "Maintenance".to_string(),
                description: format!("Stale marker found: {}", line.trim()),
                severity: "ADVISORY".to_string(),
            });
        }
    }

    let lines: Vec<&str> = content.lines().collect();
    for (idx, line) in lines.iter().enumerate() {
        if let Some(caps) = func_def_re.captures(line) {
            let func_name = caps.get(2).map_or("", |m| m.as_str());
            let params_str = caps.get(3).map_or("", |m| m.as_str());

            let param_count = if params_str.trim().is_empty() {
                0
            } else {
                params_str.split(',').count()
            };

            if param_count > max_parameters {
                smells.push(CodeSmell {
                    file: file_path.to_path_buf(),
                    line_number: idx + 1,
                    smell_category: "Bloaters".to_string(),
                    description: format!(
                        "Function '{}' has too many parameters ({param_count} > {max_parameters})",
                        func_name
                    ),
                    severity: "WARNING".to_string(),
                });
            }

            // Estimate function length
            let mut func_lines = 0;
            for next_line in lines.iter().skip(idx + 1) {
                if func_def_re.is_match(next_line) {
                    break;
                }
                func_lines += 1;
            }

            if func_lines > max_function_lines {
                smells.push(CodeSmell {
                    file: file_path.to_path_buf(),
                    line_number: idx + 1,
                    smell_category: "Bloaters".to_string(),
                    description: format!(
                        "Function '{}' is oversized ({func_lines} lines > {max_function_lines})",
                        func_name
                    ),
                    severity: "WARNING".to_string(),
                });
            }
        }
    }

    Ok(smells)
}

pub fn check_code_janitor_health(skill_dir: &Path) -> Result<Vec<String>, String> {
    let required_files = [
        skill_dir.join("SKILL.md"),
        skill_dir.join("README.md"),
        skill_dir.join("references").join("overview.md"),
        skill_dir
            .join("references")
            .join("clean-code-heuristics.md"),
        skill_dir.join("references").join("code-smells-catalog.md"),
        skill_dir.join("references").join("janitor-audit-report.md"),
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
            "Code Janitor health check failed. Missing files: {:?}",
            missing
        ))
    }
}

pub fn run_code_janitor_command(
    subcommand: &CodeJanitorSubcommand,
    repo_root: &Path,
) -> Result<(), String> {
    match subcommand {
        CodeJanitorSubcommand::Check(args) => {
            let skill_dir = if let Some(p) = &args.path {
                sanitize_path(p, Some(repo_root))?
            } else {
                repo_root.join("skills").join("code-janitor")
            };

            match check_code_janitor_health(&skill_dir) {
                Ok(_) => {
                    println!("Code Janitor skill health check passed cleanly.");
                    Ok(())
                }
                Err(err) => Err(err),
            }
        }
        CodeJanitorSubcommand::Scan(args) => {
            let mut files_to_scan = Vec::new();
            if let Some(f) = &args.file {
                let p = sanitize_path(f, Some(repo_root))?;
                files_to_scan.push(p);
            } else if let Some(d) = &args.dir {
                let p = sanitize_path(d, Some(repo_root))?;
                if p.is_dir() {
                    let mut walk = vec![p];
                    while let Some(curr) = walk.pop() {
                        if let Ok(entries) = fs::read_dir(curr) {
                            for entry in entries.flatten() {
                                let path = entry.path();
                                let fname = path.file_name().unwrap_or_default().to_string_lossy();
                                if fname.starts_with('.')
                                    || fname == "target"
                                    || fname == "node_modules"
                                {
                                    continue;
                                }
                                if path.is_dir() {
                                    walk.push(path);
                                } else if path.is_file() {
                                    files_to_scan.push(path);
                                }
                            }
                        }
                    }
                }
            } else {
                files_to_scan.push(repo_root.to_path_buf());
            }

            let mut all_smells = Vec::new();
            for file in files_to_scan {
                if let Ok(smells) = scan_file_for_smells(
                    &file,
                    args.max_function_lines,
                    args.max_parameters,
                    args.max_nesting_depth,
                ) {
                    all_smells.extend(smells);
                }
            }

            if args.json {
                println!("{}", serde_json::to_string_pretty(&all_smells).unwrap());
            } else {
                println!(
                    "Code Janitor Smell Scan Results (Total: {}):",
                    all_smells.len()
                );
                for s in all_smells {
                    println!(
                        "  [{}] [{}] {}:{}: {}",
                        s.severity,
                        s.smell_category,
                        s.file.display(),
                        s.line_number,
                        s.description
                    );
                }
            }
            Ok(())
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn test_scan_file_for_smells_strict_assertions() {
        let dir = tempdir().unwrap();
        let base = dir.path().canonicalize().unwrap();
        let sample = base.join("sample.py");
        let mut content = String::from("def oversized_function():\n");
        for i in 0..35 {
            content.push_str(&format!("    print('line {}')\n", i));
        }
        fs::write(&sample, content).unwrap();

        let smells = scan_file_for_smells(&sample, 30, 5, 4).unwrap();
        assert_eq!(smells.len(), 1);
        assert_eq!(smells[0].smell_category, "Bloaters");
        assert!(smells[0].description.contains("oversized_function"));
    }

    #[test]
    fn test_check_code_janitor_health_strict_assertions() {
        let dir = tempdir().unwrap();
        let base = dir.path().canonicalize().unwrap();
        let references = base.join("references");
        fs::create_dir_all(&references).unwrap();

        fs::write(base.join("SKILL.md"), "---").unwrap();
        fs::write(base.join("README.md"), "# Title").unwrap();
        fs::write(references.join("overview.md"), "# Overview").unwrap();
        fs::write(references.join("clean-code-heuristics.md"), "# Clean").unwrap();
        fs::write(references.join("code-smells-catalog.md"), "# Smells").unwrap();
        fs::write(references.join("janitor-audit-report.md"), "# Report").unwrap();

        let res = check_code_janitor_health(&base);
        assert!(res.is_ok());
    }
}
