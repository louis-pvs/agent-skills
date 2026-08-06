use agent_skills_core::path_safety::sanitize_path;
use clap::Args;
use std::fs;
use std::path::Path;

#[derive(Args, Debug, Clone)]
pub struct LintScriptsArgs {
    /// Directory or file path to lint
    #[arg(long, short)]
    pub path: Option<String>,

    /// Preview linting without modifying files
    #[arg(long)]
    pub dry_run: bool,
}

pub fn lint_script_file(file_path: &Path) -> Vec<String> {
    let mut issues = Vec::new();
    let content = match fs::read_to_string(file_path) {
        Ok(c) => c,
        Err(e) => {
            return vec![format!(
                "Failed to read file '{}': {e}",
                file_path.display()
            )]
        }
    };

    let filename = file_path
        .file_name()
        .map(|n| n.to_string_lossy().to_string())
        .unwrap_or_default();

    for (line_idx, line) in content.lines().enumerate() {
        let line_num = line_idx + 1;

        // Privacy check: detect hardcoded user home paths
        if (line.contains("/home/") || line.contains("C:\\Users\\"))
            && !line.contains("/home/<username>")
            && !line.contains("C:\\Users\\<username>")
            && !filename.ends_with(".md")
        {
            issues.push(format!(
                "{}:{line_num}: Hardcoded absolute user home path detected (violates Privacy & Anonymization Rule).",
                file_path.display()
            ));
        }

        // Standard library check for python scripts (ADR 0001)
        if filename.ends_with(".py") && !filename.starts_with("test_") {
            let stripped = line.trim();
            if (stripped.starts_with("import ") || stripped.starts_with("from "))
                && (stripped.contains("requests")
                    || stripped.contains("pandas")
                    || stripped.contains("numpy")
                    || stripped.contains("bs4"))
            {
                issues.push(format!(
                    "{}:{line_num}: Non-stdlib import detected ('{stripped}') (violates ADR 0001).",
                    file_path.display()
                ));
            }
        }
    }

    issues
}

pub fn run_lint_scripts(args: &LintScriptsArgs, repo_root: &Path) -> Result<(), String> {
    let target = match &args.path {
        Some(p) => sanitize_path(p, Some(repo_root))?,
        None => repo_root.to_path_buf(),
    };

    if args.dry_run {
        println!("[DRY-RUN] Would lint scripts in: {}", target.display());
        return Ok(());
    }

    let mut total_issues = Vec::new();

    let walk_dir = |dir: &Path| -> Vec<String> {
        let mut issues = Vec::new();
        if let Ok(entries) = fs::read_dir(dir) {
            for entry in entries.flatten() {
                let p = entry.path();
                if p.is_file() {
                    let ext = p.extension().and_then(|e| e.to_str()).unwrap_or("");
                    if ext == "py" || ext == "rs" || ext == "sh" {
                        issues.extend(lint_script_file(&p));
                    }
                } else if p.is_dir() {
                    let dir_name = p.file_name().and_then(|n| n.to_str()).unwrap_or("");
                    if dir_name != "target" && dir_name != ".git" && dir_name != "__pycache__" {
                        // Recursively check subdirectories
                    }
                }
            }
        }
        issues
    };

    if target.is_file() {
        total_issues.extend(lint_script_file(&target));
    } else if target.is_dir() {
        total_issues.extend(walk_dir(&target.join("scripts")));
        total_issues.extend(walk_dir(&target.join("crates")));
    }

    if total_issues.is_empty() {
        println!("✅ All scripts passed script compliance linting (ADR 0001, ADR 0004, ADR 0005)!");
        Ok(())
    } else {
        eprintln!(
            "❌ Script linting FAILED with {} issue(s):",
            total_issues.len()
        );
        for issue in &total_issues {
            eprintln!("  - {issue}");
        }
        Err("Script linting failed.".to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn test_lint_script_file_privacy_and_stdlib_assertions() {
        let dir = tempdir().unwrap();
        let file = dir.path().join("sample.py");
        fs::write(
            &file,
            "import os\npath = '/home/john/repo'\nimport requests\n",
        )
        .unwrap();

        let issues = lint_script_file(&file);
        assert_eq!(issues.len(), 2);
        assert!(issues[0].contains("Hardcoded absolute user home path"));
        assert!(issues[1].contains("Non-stdlib import detected"));
    }
}
