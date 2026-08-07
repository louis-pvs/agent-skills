use agent_skills_core::path_safety::sanitize_path;
use clap::{Args, Subcommand};
use regex::Regex;
use serde::{Deserialize, Serialize};
use std::collections::HashSet;
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

pub fn format_smells(smells: &[CodeSmell]) -> String {
    if smells.is_empty() {
        return "No issues found.".to_string();
    }
    smells
        .iter()
        .map(|smell| {
            let icon = match smell.severity.as_str() {
                "ERROR" => "🔴 ERROR",
                "WARNING" => "⚠️ WARNING",
                _ => "💡 ADVISORY",
            };
            format!(
                "{icon} [{}] {}:{}: {}",
                smell.smell_category,
                smell.file.display(),
                smell.line_number,
                smell.description
            )
        })
        .collect::<Vec<_>>()
        .join("\n")
}

#[allow(dead_code)]
pub fn format_smells_report(smells: &[CodeSmell]) -> String {
    format_smells(smells)
}

pub fn scan_file_for_smells(
    file_path: &Path,
    max_function_lines: usize,
    max_parameters: usize,
    max_nesting_depth: usize,
) -> anyhow::Result<Vec<CodeSmell>> {
    if !file_path.exists() || !file_path.is_file() {
        return Err(anyhow::anyhow!("File not found: {}", file_path.display()));
    }

    let content =
        fs::read_to_string(file_path).map_err(|e| anyhow::anyhow!("Failed to read file: {e}"))?;
    let mut smells = Vec::new();
    let is_python = file_path.extension().and_then(|ext| ext.to_str()) == Some("py");

    let func_def_re =
        Regex::new(r"(?m)^\s*(?:def|fn|async\s+fn|function)\s+(\w+)\s*\(([^)]*)\)").unwrap();
    let todo_re = Regex::new(r"(?i)\b(TODO|FIXME)\b").unwrap();
    let import_re = Regex::new(r"^\s*import\s+(\w+)(?:\s+as\s+(\w+))?").unwrap();
    let from_import_re = Regex::new(r"^\s*from\s+[\w\.]+\s+import\s+(.+)").unwrap();
    let complexity_re = Regex::new(r"\b(if|elif|for|while|except|and|or)\b").unwrap();

    let lines: Vec<&str> = content.lines().collect();

    for (idx, line) in lines.iter().enumerate() {
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

    if is_python {
        let mut imported_symbols = Vec::new();
        for (idx, line) in lines.iter().enumerate() {
            if let Some(caps) = import_re.captures(line) {
                let package = caps.get(1).map_or("", |m| m.as_str()).trim();
                let alias = caps.get(2).map_or("", |m| m.as_str()).trim();
                let symbol = if alias.is_empty() { package } else { alias };
                if !symbol.is_empty() {
                    imported_symbols.push((idx + 1, symbol.to_string()));
                }
            } else if let Some(caps) = from_import_re.captures(line) {
                let names = caps.get(1).map_or("", |m| m.as_str());
                for import_part in names.split(',') {
                    let segment = import_part.trim();
                    if segment.is_empty() {
                        continue;
                    }
                    let tokens: Vec<&str> = segment.split_whitespace().collect();
                    let base_name = tokens.first().copied().unwrap_or("").trim();
                    let alias = if tokens.get(1).copied() == Some("as") {
                        tokens.get(2).copied().unwrap_or("").trim()
                    } else {
                        ""
                    };
                    let symbol = if alias.is_empty() { base_name } else { alias };
                    if !symbol.is_empty() {
                        imported_symbols.push((idx + 1, symbol.to_string()));
                    }
                }
            }
        }

        let non_import_lines: Vec<&str> = lines
            .iter()
            .copied()
            .filter(|line| {
                !import_re.is_match(line)
                    && !from_import_re.is_match(line)
                    && !line.trim_start().starts_with('#')
            })
            .collect();

        let mut seen = HashSet::new();
        for (line_number, symbol) in imported_symbols {
            if !seen.insert((line_number, symbol.clone())) {
                continue;
            }
            let symbol_re = Regex::new(&format!(r"\b{}\b", regex::escape(&symbol))).unwrap();
            if !non_import_lines.iter().any(|line| symbol_re.is_match(line)) {
                smells.push(CodeSmell {
                    file: file_path.to_path_buf(),
                    line_number,
                    smell_category: "Unused Code".to_string(),
                    description: format!("Unused import: '{symbol}'"),
                    severity: "WARNING".to_string(),
                });
            }
        }
    }

    for (idx, line) in lines.iter().enumerate() {
        if let Some(caps) = func_def_re.captures(line) {
            let func_name = caps.get(1).map_or("", |m| m.as_str());
            let params_str = caps.get(2).map_or("", |m| m.as_str());
            let def_indent = line.len() - line.trim_start().len();
            let body_indent = def_indent + 4;

            let param_count = params_str
                .split(',')
                .map(|param| param.trim())
                .filter(|param| !param.is_empty() && *param != "self" && *param != "cls")
                .count();

            if param_count > max_parameters {
                smells.push(CodeSmell {
                    file: file_path.to_path_buf(),
                    line_number: idx + 1,
                    smell_category: "Bloaters".to_string(),
                    description: format!(
                        "Function '{}' has excessive parameters ({param_count} > {max_parameters})",
                        func_name
                    ),
                    severity: "WARNING".to_string(),
                });
            }

            if is_python && line.trim_start().starts_with("def ") {
                let mut first_body_line = None;
                for &candidate in lines.iter().skip(idx + 1) {
                    let trimmed = candidate.trim();
                    if trimmed.is_empty() {
                        continue;
                    }
                    let candidate_indent = candidate.len() - candidate.trim_start().len();
                    if candidate_indent <= def_indent {
                        break;
                    }
                    first_body_line = Some(trimmed);
                    break;
                }

                if !matches!(
                    first_body_line,
                    Some(body) if body.starts_with("\"\"\"") || body.starts_with("'''")
                ) {
                    smells.push(CodeSmell {
                        file: file_path.to_path_buf(),
                        line_number: idx + 1,
                        smell_category: "Documentation".to_string(),
                        description: format!("Function '{func_name}' missing docstring"),
                        severity: "ADVISORY".to_string(),
                    });
                }

                if !line.contains("->") {
                    smells.push(CodeSmell {
                        file: file_path.to_path_buf(),
                        line_number: idx + 1,
                        smell_category: "Maintainability".to_string(),
                        description: format!("Function '{func_name}' missing type annotations"),
                        severity: "ADVISORY".to_string(),
                    });
                }
            }

            let mut func_lines = 0;
            let mut complexity = 1;
            let mut nesting_flagged = false;
            let mut dead_code_pending = false;

            for (curr_idx, &curr_line) in lines.iter().enumerate().skip(idx + 1) {
                let trimmed = curr_line.trim();
                let curr_indent = curr_line.len() - curr_line.trim_start().len();

                if !trimmed.is_empty() && curr_indent <= def_indent {
                    break;
                }
                if trimmed.is_empty() {
                    continue;
                }

                func_lines += 1;
                complexity += complexity_re.find_iter(trimmed).count();

                if is_python {
                    let relative_depth = if curr_indent >= body_indent {
                        (curr_indent - body_indent) / 4
                    } else {
                        0
                    };
                    if !nesting_flagged && relative_depth >= max_nesting_depth {
                        smells.push(CodeSmell {
                            file: file_path.to_path_buf(),
                            line_number: curr_idx + 1,
                            smell_category: "Bloaters".to_string(),
                            description: format!(
                                "Deep nesting detected (depth {} > max {})",
                                relative_depth, max_nesting_depth
                            ),
                            severity: "WARNING".to_string(),
                        });
                        nesting_flagged = true;
                    }
                }

                if dead_code_pending && !trimmed.starts_with('#') {
                    if curr_indent == body_indent
                        && !trimmed.starts_with("def ")
                        && !trimmed.starts_with("class ")
                    {
                        smells.push(CodeSmell {
                            file: file_path.to_path_buf(),
                            line_number: curr_idx + 1,
                            smell_category: "Dead Code".to_string(),
                            description: "Unreachable code after return/raise".to_string(),
                            severity: "WARNING".to_string(),
                        });
                    }
                    dead_code_pending = false;
                }

                if curr_indent == body_indent
                    && (trimmed == "return"
                        || trimmed.starts_with("return ")
                        || trimmed == "raise"
                        || trimmed.starts_with("raise "))
                {
                    dead_code_pending = true;
                }
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

            if complexity > 10 {
                smells.push(CodeSmell {
                    file: file_path.to_path_buf(),
                    line_number: idx + 1,
                    smell_category: "Complexity".to_string(),
                    description: format!(
                        "Function '{}' has high cyclomatic complexity ({complexity})",
                        func_name
                    ),
                    severity: "WARNING".to_string(),
                });
            }
        }
    }

    Ok(smells)
}

pub fn check_code_janitor_health(skill_dir: &Path) -> anyhow::Result<Vec<String>> {
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
        Err(anyhow::anyhow!(
            "Code Janitor health check failed. Missing files: {:?}",
            missing
        ))
    }
}

pub fn run_code_janitor_command(
    subcommand: &CodeJanitorSubcommand,
    repo_root: &Path,
) -> anyhow::Result<()> {
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
            if args.file.is_none() && args.dir.is_none() {
                return Err(anyhow::anyhow!("scan requires --file or --dir"));
            }

            let mut files_to_scan = Vec::new();
            if let Some(f) = &args.file {
                let p = sanitize_path(f, Some(repo_root))?;
                if !p.exists() || !p.is_file() {
                    return Err(anyhow::anyhow!("File not found: {}", p.display()));
                }
                files_to_scan.push(p);
            } else if let Some(d) = &args.dir {
                let p = sanitize_path(d, Some(repo_root))?;
                if !p.exists() || !p.is_dir() {
                    return Err(anyhow::anyhow!("Directory not found: {}", p.display()));
                }
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

            let mut all_smells = Vec::new();
            for file in files_to_scan {
                let smells = scan_file_for_smells(
                    &file,
                    args.max_function_lines,
                    args.max_parameters,
                    args.max_nesting_depth,
                )?;
                all_smells.extend(smells);
            }

            if args.json {
                println!("{}", serde_json::to_string_pretty(&all_smells).unwrap());
            } else {
                let report = format_smells(&all_smells);
                println!("{report}");
            }
            Ok(())
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    // Fixture: a Python function with >10 branches for test_detects_high_complexity.
    // Kept here (inside #[cfg(test)]) so it is only compiled in test builds and
    // doesn't trigger the dead_code lint in release/clippy runs.
    const HIGH_COMPLEXITY_PY: &str = "def f(x):\n    if x == 1:\n        return 1\n    elif x == 2:\n        return 2\n    elif x == 3:\n        return 3\n    elif x == 4:\n        return 4\n    elif x == 5:\n        return 5\n    elif x == 6:\n        return 6\n    elif x == 7:\n        return 7\n    elif x == 8:\n        return 8\n    elif x == 9:\n        return 9\n    elif x == 10:\n        return 10\n    for item in [x]:\n        if item and x or item:\n            return item\n    return 0\n";

    #[test]
    fn test_scan_file_for_smells_strict_assertions() {
        let dir = tempdir().unwrap();
        let base = dir.path().canonicalize().unwrap();
        let sample = base.join("sample.py");
        let mut content = String::from("def oversized_function() -> None:\n    \"\"\"Doc.\"\"\"\n");
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

    #[test]
    fn test_detects_excessive_params() {
        let dir = tempdir().unwrap();
        let sample = dir.path().join("params.py");
        fs::write(&sample, "def long_params(a, b, c, d, e, f, g):\n    pass\n").unwrap();

        let smells = scan_file_for_smells(&sample, 50, 4, 4).unwrap();
        let param_smell = smells
            .iter()
            .find(|s| s.description.contains("excessive parameters"));
        assert!(param_smell.is_some());
    }

    #[test]
    fn test_json_output() {
        let dir = tempdir().unwrap();
        let base = dir.path().canonicalize().unwrap();
        let sample = base.join("sample.py");
        fs::write(&sample, "def test_func():\n    # TODO: fix me\n    pass\n").unwrap();

        let args = ScanJanitorArgs {
            file: Some(sample.display().to_string()),
            dir: None,
            max_function_lines: 50,
            max_parameters: 5,
            max_nesting_depth: 4,
            json: true,
        };

        let res = run_code_janitor_command(&CodeJanitorSubcommand::Scan(args), &base);
        assert!(res.is_ok());
    }

    #[test]
    fn test_detects_unused_import() {
        let dir = tempdir().unwrap();
        let sample = dir.path().join("unused.py");
        fs::write(&sample, "import os\n\ndef f():\n    return 42\n").unwrap();

        let smells = scan_file_for_smells(&sample, 50, 5, 4).unwrap();
        let unused = smells.iter().find(|s| s.description.contains("os"));
        assert!(unused.is_some(), "Unused import 'os' must be detected");
    }

    #[test]
    fn test_detects_code_after_return() {
        let dir = tempdir().unwrap();
        let sample = dir.path().join("dead.py");
        fs::write(&sample, "def f():\n    return 42\n    x = 1\n").unwrap();

        let smells = scan_file_for_smells(&sample, 50, 5, 4).unwrap();
        let dead = smells.iter().find(|s| s.smell_category == "Dead Code");
        assert!(dead.is_some(), "Dead code after return must be detected");
    }

    #[test]
    fn test_excludes_self_from_count() {
        let dir = tempfile::tempdir().unwrap();
        let sample = dir.path().join("cls.py");
        std::fs::write(
            &sample,
            "def method(self, a, b, c, d, e):\n    \"\"\"Doc.\"\"\"\n    return 1\n",
        )
        .unwrap();
        let smells = scan_file_for_smells(&sample, 50, 5, 4).unwrap();
        let param_smell = smells.iter().find(|s| {
            s.description.contains("too many parameters") || s.description.contains("excessive")
        });
        assert!(
            param_smell.is_none(),
            "self must not count toward param limit, got: {:?}",
            param_smell
        );
    }

    #[test]
    fn test_detects_deep_nesting() {
        let dir = tempdir().unwrap();
        let sample = dir.path().join("nested.py");
        fs::write(
            &sample,
            "def f():\n    if True:\n        if True:\n            if True:\n                if True:\n                    x = 1\n",
        )
        .unwrap();

        let smells = scan_file_for_smells(&sample, 50, 5, 4).unwrap();
        let nested = smells
            .iter()
            .find(|s| s.smell_category == "Bloaters" && s.description.contains("nesting"));
        assert!(nested.is_some(), "Deep nesting must be detected");
    }

    #[test]
    fn test_detects_high_complexity() {
        let dir = tempdir().unwrap();
        let sample = dir.path().join("complex.py");
        fs::write(&sample, HIGH_COMPLEXITY_PY).unwrap();

        let smells = scan_file_for_smells(&sample, 50, 5, 4).unwrap();
        let complex = smells.iter().find(|s| s.smell_category == "Complexity");
        assert!(complex.is_some(), "High complexity must be detected");
    }

    #[test]
    fn test_detects_missing_docstring() {
        let dir = tempdir().unwrap();
        let sample = dir.path().join("no_doc.py");
        fs::write(&sample, "def f():\n    x = 1\n    return x\n").unwrap();

        let smells = scan_file_for_smells(&sample, 50, 5, 4).unwrap();
        let doc = smells.iter().find(|s| s.smell_category == "Documentation");
        assert!(doc.is_some(), "Missing docstring must be detected");
    }

    #[test]
    fn test_detects_missing_annotation() {
        let dir = tempdir().unwrap();
        let sample = dir.path().join("no_types.py");
        fs::write(&sample, "def f(x):\n    return x\n").unwrap();

        let smells = scan_file_for_smells(&sample, 50, 5, 4).unwrap();
        let ann = smells
            .iter()
            .find(|s| s.smell_category == "Maintainability");
        assert!(ann.is_some(), "Missing type annotations must be detected");
    }

    #[test]
    fn test_no_false_positive_on_normal_comment() {
        let dir = tempfile::tempdir().unwrap();
        let sample = dir.path().join("clean.py");
        std::fs::write(
            &sample,
            "def f():\n    # This is a regular comment\n    return 42\n",
        )
        .unwrap();
        let smells = scan_file_for_smells(&sample, 50, 5, 4).unwrap();
        let todo_smell = smells.iter().find(|s| s.smell_category == "Maintenance");
        assert!(
            todo_smell.is_none(),
            "regular comment must not be flagged as TODO/FIXME, got: {:?}",
            todo_smell
        );
    }

    #[test]
    fn test_empty_findings_shows_clean() {
        let result = format_smells(&[]);
        assert!(result.to_lowercase().contains("no issues"));
    }

    #[test]
    fn test_findings_show_severity_icons() {
        let smell = CodeSmell {
            file: PathBuf::from("foo.py"),
            line_number: 1,
            smell_category: "Bloaters".to_string(),
            description: "oversized".to_string(),
            severity: "WARNING".to_string(),
        };
        let report = format_smells(&[smell]);
        assert!(report.contains("⚠️"));
    }

    #[test]
    fn test_to_dict_returns_correct_keys() {
        let smell = CodeSmell {
            file: PathBuf::from("sample.py"),
            line_number: 3,
            smell_category: "Bloaters".to_string(),
            description: "too long".to_string(),
            severity: "WARNING".to_string(),
        };
        let value = serde_json::to_value(&smell).unwrap();
        let obj = value.as_object().unwrap();
        for key in [
            "file",
            "line_number",
            "smell_category",
            "description",
            "severity",
        ] {
            assert!(
                obj.contains_key(key),
                "serialized CodeSmell must contain key '{key}'"
            );
        }
        assert_eq!(obj["line_number"], 3);
        assert_eq!(obj["severity"], "WARNING");
    }

    #[test]
    fn test_repr_contains_key_info() {
        let smell = CodeSmell {
            file: PathBuf::from("sample.py"),
            line_number: 7,
            smell_category: "Bloaters".to_string(),
            description: "oversized".to_string(),
            severity: "WARNING".to_string(),
        };
        let repr = format!("{:?}", smell);
        assert!(repr.contains("sample.py"));
        assert!(repr.contains('7'));
        assert!(repr.contains("Bloaters"));
    }

    #[test]
    fn test_no_false_positive_for_used_import() {
        let dir = tempdir().unwrap();
        let sample = dir.path().join("used.py");
        fs::write(&sample, "import os\n\ndef f():\n    return os.getcwd()\n").unwrap();

        let smells = scan_file_for_smells(&sample, 50, 5, 4).unwrap();
        let unused = smells.iter().find(|s| s.description.contains("os"));
        assert!(unused.is_none(), "Used import must not be flagged");
    }

    #[test]
    fn test_detects_unused_from_import() {
        let dir = tempdir().unwrap();
        let sample = dir.path().join("from_import.py");
        fs::write(
            &sample,
            "from os import path, getcwd\n\ndef f():\n    return path.join('a','b')\n",
        )
        .unwrap();

        let smells = scan_file_for_smells(&sample, 50, 5, 4).unwrap();
        let unused_getcwd = smells.iter().find(|s| s.description.contains("getcwd"));
        assert!(
            unused_getcwd.is_some(),
            "Unused imported symbol 'getcwd' must be flagged"
        );
    }

    #[test]
    fn test_handles_aliased_import() {
        let dir = tempdir().unwrap();
        let sample = dir.path().join("alias.py");
        fs::write(&sample, "import numpy as np\n\ndef f():\n    return 42\n").unwrap();

        let smells = scan_file_for_smells(&sample, 50, 5, 4).unwrap();
        let unused = smells
            .iter()
            .find(|s| s.description.contains("np") || s.description.contains("numpy"));
        assert!(unused.is_some(), "Unused aliased import must be flagged");
    }

    #[test]
    fn test_detects_code_after_raise() {
        let dir = tempdir().unwrap();
        let sample = dir.path().join("raise.py");
        fs::write(
            &sample,
            "def f():\n    raise ValueError(\"err\")\n    x = 1\n",
        )
        .unwrap();

        let smells = scan_file_for_smells(&sample, 50, 5, 4).unwrap();
        let dead = smells.iter().find(|s| s.smell_category == "Dead Code");
        assert!(dead.is_some(), "Dead code after raise must be detected");
    }

    #[test]
    fn test_no_false_positive_for_conditional_return() {
        let dir = tempdir().unwrap();
        let sample = dir.path().join("cond.py");
        fs::write(
            &sample,
            "def f(x):\n    if x > 0:\n        return x\n    return -x\n",
        )
        .unwrap();

        let smells = scan_file_for_smells(&sample, 50, 5, 4).unwrap();
        let dead = smells.iter().find(|s| s.smell_category == "Dead Code");
        assert!(
            dead.is_none(),
            "Conditional returns must not trigger dead code warning"
        );
    }

    #[test]
    fn test_accepts_short_function() {
        // Python: test_accepts_short_function — PARTIAL, short function under threshold, zero findings
        let dir = tempdir().unwrap();
        let sample = dir.path().join("short.py");
        fs::write(
            &sample,
            "def small() -> int:\n    \"\"\"Doc.\"\"\"\n    return 1\n",
        )
        .unwrap();

        let smells = scan_file_for_smells(&sample, 30, 5, 4).unwrap();
        assert!(
            smells.is_empty(),
            "a short function under threshold with no other smells must produce zero findings, got: {smells:?}"
        );
    }

    #[test]
    fn test_accepts_shallow_nesting() {
        // Python: test_accepts_shallow_nesting — feature absent, will trivially pass until
        // detect_deep_nesting exists; kept as a regression guard once it's implemented.
        let dir = tempdir().unwrap();
        let sample = dir.path().join("shallow.py");
        fs::write(
            &sample,
            "def f(x):\n    if x:\n        return 1\n    return 0\n",
        )
        .unwrap();

        let smells = scan_file_for_smells(&sample, 50, 5, 4).unwrap();
        let nesting_smell = smells.iter().find(|s| s.description.contains("nest"));
        assert!(
            nesting_smell.is_none(),
            "shallow nesting must not be flagged"
        );
    }

    #[test]
    fn test_accepts_simple_function() {
        // Python: test_accepts_simple_function — feature absent, will trivially pass until
        // calculate_cyclomatic_complexity exists; kept as a regression guard once it's implemented.
        let dir = tempdir().unwrap();
        let sample = dir.path().join("simple.py");
        fs::write(&sample, "def f(x):\n    return x + 1\n").unwrap();

        let smells = scan_file_for_smells(&sample, 50, 5, 4).unwrap();
        let complexity_smell = smells.iter().find(|s| s.description.contains("complexity"));
        assert!(
            complexity_smell.is_none(),
            "a simple function must not be flagged as high complexity"
        );
    }

    #[test]
    fn test_accepts_function_with_docstring() {
        // Python: test_accepts_function_with_docstring — feature absent, will trivially pass
        // until detect_missing_docstring exists; kept as a regression guard once implemented.
        let dir = tempdir().unwrap();
        let sample = dir.path().join("documented.py");
        fs::write(
            &sample,
            "def f():\n    \"\"\"Does a thing.\"\"\"\n    return 1\n",
        )
        .unwrap();

        let smells = scan_file_for_smells(&sample, 50, 5, 4).unwrap();
        let docstring_smell = smells.iter().find(|s| s.description.contains("docstring"));
        assert!(
            docstring_smell.is_none(),
            "a documented function must not be flagged"
        );
    }

    #[test]
    fn test_accepts_annotated_function() {
        // Python: test_accepts_annotated_function — feature absent, will trivially pass until
        // detect_missing_annotation exists; kept as a regression guard once implemented.
        let dir = tempdir().unwrap();
        let sample = dir.path().join("annotated.py");
        fs::write(&sample, "def f(x: int) -> int:\n    return x + 1\n").unwrap();

        let smells = scan_file_for_smells(&sample, 50, 5, 4).unwrap();
        let annotation_smell = smells.iter().find(|s| s.description.contains("annotation"));
        assert!(
            annotation_smell.is_none(),
            "a fully-annotated function must not be flagged"
        );
    }

    #[test]
    fn test_detects_todo() {
        // Python: test_detects_todo — existing todo_re regex, direct test
        let dir = tempdir().unwrap();
        let sample = dir.path().join("todo.py");
        fs::write(
            &sample,
            "def f():\n    # TODO: fix this later\n    return 1\n",
        )
        .unwrap();

        let smells = scan_file_for_smells(&sample, 50, 5, 4).unwrap();
        let todo_smell = smells.iter().find(|s| s.smell_category == "Maintenance");
        assert!(todo_smell.is_some(), "TODO marker must be detected");
        assert_eq!(todo_smell.unwrap().line_number, 2);
    }

    #[test]
    fn test_detects_fixme() {
        // Python: test_detects_fixme — same regex as TODO
        let dir = tempdir().unwrap();
        let sample = dir.path().join("fixme.py");
        fs::write(
            &sample,
            "def f():\n    # FIXME: broken edge case\n    return 1\n",
        )
        .unwrap();

        let smells = scan_file_for_smells(&sample, 50, 5, 4).unwrap();
        let fixme_smell = smells.iter().find(|s| s.smell_category == "Maintenance");
        assert!(fixme_smell.is_some(), "FIXME marker must be detected");
    }

    #[test]
    fn test_scan_file_returns_findings_multiple_types() {
        // Python: test_scan_file_returns_findings — PARTIAL, multiple concurrent smell types
        // in one file (TODO marker + excessive params, the two detectors that both exist today).
        let dir = tempdir().unwrap();
        let sample = dir.path().join("multi.py");
        fs::write(
            &sample,
            "def messy(a, b, c, d, e, f, g):\n    # TODO: refactor this\n    pass\n",
        )
        .unwrap();

        let smells = scan_file_for_smells(&sample, 50, 5, 4).unwrap();
        let categories: std::collections::HashSet<_> =
            smells.iter().map(|s| s.smell_category.as_str()).collect();
        assert!(
            categories.contains("Maintenance"),
            "must detect the TODO marker"
        );
        assert!(
            categories.contains("Bloaters"),
            "must detect the excessive params"
        );
    }

    #[test]
    fn test_no_args_returns_error() {
        let dir = tempdir().unwrap();
        let base = dir.path().canonicalize().unwrap();

        let args = ScanJanitorArgs {
            file: None,
            dir: None,
            max_function_lines: 30,
            max_parameters: 5,
            max_nesting_depth: 4,
            json: false,
        };
        let res = run_code_janitor_command(&CodeJanitorSubcommand::Scan(args), &base);
        let err = res.expect_err("scan without --file/--dir must error");
        assert_eq!(err.to_string(), "scan requires --file or --dir");
    }

    #[test]
    fn test_nonexistent_file_returns_error() {
        let dir = tempdir().unwrap();
        let base = dir.path().canonicalize().unwrap();

        let args = ScanJanitorArgs {
            file: Some("does-not-exist.py".to_string()),
            dir: None,
            max_function_lines: 30,
            max_parameters: 5,
            max_nesting_depth: 4,
            json: false,
        };
        let res = run_code_janitor_command(&CodeJanitorSubcommand::Scan(args), &base);
        let err = res.expect_err("scan with nonexistent --file must error");
        assert!(err.to_string().contains("File not found"));
    }

    #[test]
    fn test_scan_real_file_succeeds() {
        // Python: test_scan_real_file_succeeds — first CLI-dispatch-level test of Scan
        let dir = tempdir().unwrap();
        let base = dir.path().canonicalize().unwrap();
        let sample = base.join("clean.py");
        fs::write(&sample, "def f():\n    return 1\n").unwrap();

        let args = ScanJanitorArgs {
            file: Some(sample.display().to_string()),
            dir: None,
            max_function_lines: 30,
            max_parameters: 5,
            max_nesting_depth: 4,
            json: false,
        };
        let res = run_code_janitor_command(&CodeJanitorSubcommand::Scan(args), &base);
        assert!(res.is_ok());
    }
}
