use agent_skills_core::path_safety::sanitize_path;
use clap::{Args, Subcommand};
use regex::Regex;
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

#[derive(Args, Debug, Clone)]
pub struct GitCouplingArgs {
    /// Target file to find co-changed coupled files for
    #[arg(long)]
    pub file: String,

    /// Minimum co-commit count threshold
    #[arg(long, default_value_t = 1)]
    pub min_commits: usize,

    /// Maximum number of coupled files to return
    #[arg(long, default_value_t = 10)]
    pub limit: usize,
}

#[derive(Args, Debug, Clone)]
pub struct SymbolNavArgs {
    /// Target symbol name to search for
    #[arg(long)]
    pub symbol: String,

    /// Path to directory or file to search in (defaults to cwd)
    #[arg(long, default_value = ".")]
    pub path: String,

    /// Symbol search filter type (all, definition, reference)
    #[arg(long, default_value = "all")]
    pub r#type: String,
}

#[derive(Args, Debug, Clone)]
pub struct AstSearchArgs {
    /// Search pattern to match
    #[arg(long)]
    pub pattern: String,

    /// Path to directory or file to search in (defaults to cwd)
    #[arg(long, default_value = ".")]
    pub path: String,
}

#[derive(Subcommand, Debug, Clone)]
pub enum ContextGathererSubcommand {
    /// Analyze git log history to surface files that change together
    GitCoupling(GitCouplingArgs),
    /// Search for symbol definitions and references across code files
    SymbolNav(SymbolNavArgs),
    /// Search for code patterns and structure matches across source files
    AstSearch(AstSearchArgs),
}

pub fn calculate_git_coupling(
    file_path: &str,
    min_commits: usize,
    limit: usize,
    repo_root: &Path,
) -> Result<Vec<(String, usize)>, String> {
    let output = Command::new("git")
        .args([
            "log",
            "--name-only",
            "--format=format:---COMMIT---",
            "--follow",
            "--",
            file_path,
        ])
        .current_dir(repo_root)
        .output()
        .map_err(|e| format!("Failed to run git log: {e}"))?;

    if !output.status.success() {
        return Err(format!(
            "Git command failed: {}",
            String::from_utf8_lossy(&output.stderr)
        ));
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    let mut co_change_counts: HashMap<String, usize> = HashMap::new();

    let commits = stdout.split("---COMMIT---");
    for commit in commits {
        let lines: Vec<&str> = commit
            .lines()
            .map(|l| l.trim())
            .filter(|l| !l.is_empty())
            .collect();
        if lines.contains(&file_path) {
            for f in lines {
                if f != file_path {
                    *co_change_counts.entry(f.to_string()).or_insert(0) += 1;
                }
            }
        }
    }

    let mut results: Vec<(String, usize)> = co_change_counts
        .into_iter()
        .filter(|(_, count)| *count >= min_commits)
        .collect();

    results.sort_by(|a, b| b.1.cmp(&a.1).then_with(|| a.0.cmp(&b.0)));
    if results.len() > limit {
        results.truncate(limit);
    }

    Ok(results)
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SymbolMatch {
    pub file_path: PathBuf,
    pub line_number: usize,
    pub line_content: String,
    pub match_type: String,
}

pub fn search_symbols(
    symbol: &str,
    target_path: &Path,
    match_type_filter: &str,
) -> Result<Vec<SymbolMatch>, String> {
    let mut matches = Vec::new();
    let is_definition_pattern = Regex::new(&format!(
        r"(?i)\b(def|fn|class|struct|enum|interface|trait|type|const|let|var|function)\s+{}\b",
        regex::escape(symbol)
    ))
    .unwrap();
    let is_any_pattern = Regex::new(&format!(r"\b{}\b", regex::escape(symbol))).unwrap();

    let mut files_to_scan = Vec::new();
    if target_path.is_file() {
        files_to_scan.push(target_path.to_path_buf());
    } else if target_path.is_dir() {
        let mut walk = vec![target_path.to_path_buf()];
        while let Some(dir) = walk.pop() {
            if let Ok(entries) = fs::read_dir(dir) {
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
    }

    for file in files_to_scan {
        if let Ok(content) = fs::read_to_string(&file) {
            for (idx, line) in content.lines().enumerate() {
                if is_any_pattern.is_match(line) {
                    let is_def = is_definition_pattern.is_match(line);
                    let mtype = if is_def { "definition" } else { "reference" };

                    if match_type_filter == "all" || match_type_filter == mtype {
                        matches.push(SymbolMatch {
                            file_path: file.clone(),
                            line_number: idx + 1,
                            line_content: line.trim().to_string(),
                            match_type: mtype.to_string(),
                        });
                    }
                }
            }
        }
    }

    Ok(matches)
}

pub fn search_ast_patterns(
    pattern_str: &str,
    target_path: &Path,
) -> Result<Vec<SymbolMatch>, String> {
    let clean_pattern = pattern_str.replace('*', ".*");
    let regex = Regex::new(&clean_pattern).map_err(|e| format!("Invalid pattern regex: {e}"))?;

    let mut matches = Vec::new();
    let mut files_to_scan = Vec::new();

    if target_path.is_file() {
        files_to_scan.push(target_path.to_path_buf());
    } else if target_path.is_dir() {
        let mut walk = vec![target_path.to_path_buf()];
        while let Some(dir) = walk.pop() {
            if let Ok(entries) = fs::read_dir(dir) {
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
    }

    for file in files_to_scan {
        if let Ok(content) = fs::read_to_string(&file) {
            for (idx, line) in content.lines().enumerate() {
                if regex.is_match(line) {
                    matches.push(SymbolMatch {
                        file_path: file.clone(),
                        line_number: idx + 1,
                        line_content: line.trim().to_string(),
                        match_type: "pattern_match".to_string(),
                    });
                }
            }
        }
    }

    Ok(matches)
}

pub fn run_context_gatherer_command(
    subcommand: &ContextGathererSubcommand,
    repo_root: &Path,
) -> Result<(), String> {
    match subcommand {
        ContextGathererSubcommand::GitCoupling(args) => {
            let results =
                calculate_git_coupling(&args.file, args.min_commits, args.limit, repo_root)?;
            println!("Git Temporal Coupling for '{}':", args.file);
            if results.is_empty() {
                println!("No coupled files found matching criteria.");
            } else {
                for (file, count) in results {
                    println!("  - {file} (co-changed {count} times)");
                }
            }
            Ok(())
        }
        ContextGathererSubcommand::SymbolNav(args) => {
            let target_path = sanitize_path(&args.path, Some(repo_root))?;
            let results = search_symbols(&args.symbol, &target_path, &args.r#type)?;
            println!("Symbol Search Results for '{}':", args.symbol);
            if results.is_empty() {
                println!("No matching symbol occurrences found.");
            } else {
                for m in results {
                    println!(
                        "  [{}] {}:{}: {}",
                        m.match_type,
                        m.file_path.display(),
                        m.line_number,
                        m.line_content
                    );
                }
            }
            Ok(())
        }
        ContextGathererSubcommand::AstSearch(args) => {
            let target_path = sanitize_path(&args.path, Some(repo_root))?;
            let results = search_ast_patterns(&args.pattern, &target_path)?;
            println!("AST Pattern Search Results for '{}':", args.pattern);
            if results.is_empty() {
                println!("No matching pattern occurrences found.");
            } else {
                for m in results {
                    println!(
                        "  {}:{}: {}",
                        m.file_path.display(),
                        m.line_number,
                        m.line_content
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
    fn test_symbol_nav_definitions_strict_assertions() {
        let dir = tempdir().unwrap();
        let base = dir.path().canonicalize().unwrap();
        let file_path = base.join("sample.rs");
        fs::write(
            &file_path,
            "pub fn calculate_total() -> u32 {\n    let total = 42;\n    total\n}\n",
        )
        .unwrap();

        let matches = search_symbols("calculate_total", &base, "definition").unwrap();
        assert_eq!(matches.len(), 1);
        assert_eq!(matches[0].file_path, file_path);
        assert_eq!(matches[0].line_number, 1);
        assert_eq!(matches[0].match_type, "definition");
        assert!(matches[0].line_content.contains("pub fn calculate_total"));
    }

    #[test]
    fn test_ast_search_patterns_strict_assertions() {
        let dir = tempdir().unwrap();
        let base = dir.path().canonicalize().unwrap();
        let file_path = base.join("test_example.py");
        fs::write(
            &file_path,
            "def test_foo():\n    pass\n\ndef test_bar():\n    pass\n",
        )
        .unwrap();

        let matches = search_ast_patterns("def test_.*", &base).unwrap();
        assert_eq!(matches.len(), 2);
        assert_eq!(matches[0].line_number, 1);
        assert_eq!(matches[1].line_number, 4);
    }
}
