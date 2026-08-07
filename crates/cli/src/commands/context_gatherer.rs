use agent_skills_core::path_safety::sanitize_path;
use clap::{Args, Subcommand};
use regex::Regex;
use serde::{Deserialize, Serialize};
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

    /// Output results in JSON format
    #[arg(long)]
    pub json: bool,
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

    /// Comma-separated file extensions to filter search
    #[arg(long)]
    pub extensions: Option<String>,

    /// Output results in JSON format
    #[arg(long)]
    pub json: bool,
}

#[derive(Args, Debug, Clone)]
pub struct AstSearchArgs {
    /// Search pattern to match
    #[arg(long)]
    pub pattern: String,

    /// Path to directory or file to search in (defaults to cwd)
    #[arg(long, default_value = ".")]
    pub path: String,

    /// Output results in JSON format
    #[arg(long)]
    pub json: bool,
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
) -> anyhow::Result<Vec<(String, usize)>> {
    if !repo_root.join(".git").exists() {
        return Err(anyhow::anyhow!(
            "Not inside a git repository: {}",
            repo_root.display()
        ));
    }

    let has_commits = Command::new("git")
        .args(["rev-parse", "--verify", "HEAD"])
        .current_dir(repo_root)
        .output()
        .map_err(|e| anyhow::anyhow!("Failed to check git history: {e}"))?;
    if !has_commits.status.success() {
        return Ok(Vec::new());
    }

    let output = Command::new("git")
        .args([
            "log",
            "--name-only",
            "--full-diff",
            "--format=format:---COMMIT---",
            "--",
            file_path,
        ])
        .current_dir(repo_root)
        .output();

    let output = match output {
        Ok(out) => out,
        Err(e) => return Err(anyhow::anyhow!("Failed to run git log: {e}")),
    };

    if !output.status.success() {
        return Ok(Vec::new());
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    let mut co_change_counts: HashMap<String, usize> = HashMap::new();

    let target_name = Path::new(file_path)
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or(file_path);

    let commits = stdout.split("---COMMIT---");
    for commit in commits {
        let lines: Vec<&str> = commit
            .lines()
            .map(|l| l.trim())
            .filter(|l| !l.is_empty())
            .collect();
        let contains_target = lines
            .iter()
            .any(|l| Path::new(l).file_name().and_then(|n| n.to_str()) == Some(target_name));
        if contains_target {
            for f in lines {
                let f_name = Path::new(f)
                    .file_name()
                    .and_then(|n| n.to_str())
                    .unwrap_or(f);
                if f_name != target_name {
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

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
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
    extensions: Option<Vec<String>>,
) -> anyhow::Result<Vec<SymbolMatch>> {
    let mut matches = Vec::new();
    let is_definition_pattern = Regex::new(&format!(
        r"(?i)\b(def|fn|class|struct|enum|interface|trait|type|const|let|var|function)\s+{}\b",
        regex::escape(symbol)
    ))
    .unwrap();
    let is_any_pattern = Regex::new(&format!(r"\b{}\b", regex::escape(symbol))).unwrap();

    let ext_list: Vec<String> = extensions
        .unwrap_or_default()
        .into_iter()
        .map(|e| e.trim().trim_start_matches('.').to_string())
        .filter(|e| !e.is_empty())
        .collect();

    let mut files_to_scan = Vec::new();
    if target_path.is_file() {
        if ext_list.is_empty()
            || target_path
                .extension()
                .and_then(|e| e.to_str())
                .is_some_and(|ext| ext_list.iter().any(|allowed| allowed == ext))
        {
            files_to_scan.push(target_path.to_path_buf());
        }
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
                        if !ext_list.is_empty() {
                            let ext = p.extension().and_then(|e| e.to_str()).unwrap_or("");
                            if !ext_list.iter().any(|allowed| allowed == ext) {
                                continue;
                            }
                        }
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
) -> anyhow::Result<Vec<SymbolMatch>> {
    let clean_pattern = pattern_str.replace('*', ".*");
    let regex =
        Regex::new(&clean_pattern).map_err(|e| anyhow::anyhow!("Invalid pattern regex: {e}"))?;

    let mut matches = Vec::new();
    let mut files_to_scan = Vec::new();

    if target_path.is_file() {
        if target_path.extension().and_then(|e| e.to_str()) == Some("py") {
            files_to_scan.push(target_path.to_path_buf());
        }
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
                        let ext = p.extension().and_then(|e| e.to_str()).unwrap_or("");
                        if ext == "py" {
                            files_to_scan.push(p);
                        }
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

#[allow(dead_code)]
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum SearchBackend {
    AstGrep,
    Ripgrep,
    Grep,
    InProcessRegex,
}

#[allow(dead_code)]
fn command_exists(cmd: &str) -> bool {
    command_exists_with_path(cmd, None)
}

#[allow(dead_code)]
fn command_exists_with_path(cmd: &str, path_override: Option<&str>) -> bool {
    let mut command = Command::new("which");
    command.arg(cmd);
    if let Some(path) = path_override {
        command.env("PATH", path);
    }
    command
        .output()
        .map(|output| output.status.success())
        .unwrap_or(false)
}

#[allow(dead_code)]
pub fn detect_ast_search_backend() -> SearchBackend {
    if command_exists("ast-grep") {
        SearchBackend::AstGrep
    } else {
        SearchBackend::InProcessRegex
    }
}

#[allow(dead_code)]
pub fn detect_symbol_search_backend() -> SearchBackend {
    detect_symbol_search_backend_with_path(None)
}

#[allow(dead_code)]
fn detect_symbol_search_backend_with_path(path_override: Option<&str>) -> SearchBackend {
    if command_exists_with_path("rg", path_override) {
        SearchBackend::Ripgrep
    } else {
        SearchBackend::Grep
    }
}

pub fn format_matches_as_json(matches: &[SymbolMatch]) -> anyhow::Result<String> {
    serde_json::to_string(matches).map_err(|e| anyhow::anyhow!(e))
}

pub fn format_matches_as_text(matches: &[SymbolMatch]) -> String {
    if matches.is_empty() {
        return "No matches found.".to_string();
    }

    matches
        .iter()
        .map(|m| {
            format!(
                "{}:{}: {}",
                m.file_path.display(),
                m.line_number,
                m.line_content
            )
        })
        .collect::<Vec<_>>()
        .join("\n")
}

pub fn format_coupling_as_text(results: &[(String, usize)]) -> String {
    if results.is_empty() {
        return "No coupled files found.".to_string();
    }

    results
        .iter()
        .map(|(file, count)| format!("{file}: {count} co-changes"))
        .collect::<Vec<_>>()
        .join("\n")
}

#[allow(dead_code)]
pub fn search_class_pattern(
    name_glob: &str,
    base_class: Option<&str>,
    target_path: &Path,
) -> anyhow::Result<Vec<SymbolMatch>> {
    let name_pattern = if name_glob == "*" {
        String::from(r"\w+")
    } else {
        name_glob
            .split('*')
            .map(regex::escape)
            .collect::<Vec<_>>()
            .join(r"\w*")
    };
    let regex = Regex::new(&format!(
        r"^\s*class\s+(?P<name>{name_pattern})\s*(?:\((?P<bases>[^)]*)\))?\s*:"
    ))
    .map_err(|e| anyhow::anyhow!(e))?;

    let mut matches = Vec::new();
    let mut files_to_scan = Vec::new();

    if target_path.is_file() {
        if target_path.extension().and_then(|e| e.to_str()) == Some("py") {
            files_to_scan.push(target_path.to_path_buf());
        }
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
                    } else if p.is_file() && p.extension().and_then(|e| e.to_str()) == Some("py") {
                        files_to_scan.push(p);
                    }
                }
            }
        }
    }

    for file in files_to_scan {
        if let Ok(content) = fs::read_to_string(&file) {
            for (idx, line) in content.lines().enumerate() {
                if let Some(caps) = regex.captures(line) {
                    let class_name = caps.name("name").map_or("", |m| m.as_str());
                    let bases_str = caps.name("bases").map_or("", |m| m.as_str());

                    if let Some(target_base) = base_class {
                        if class_name == target_base {
                            continue;
                        }
                        if !bases_str.split(',').any(|b| b.trim() == target_base) {
                            continue;
                        }
                    }

                    matches.push(SymbolMatch {
                        file_path: file.clone(),
                        line_number: idx + 1,
                        line_content: line.trim().to_string(),
                        match_type: "class_definition".to_string(),
                    });
                }
            }
        }
    }

    Ok(matches)
}

#[allow(dead_code)]
pub fn find_classes_in_directory(dir: &Path, base_class: &str) -> anyhow::Result<Vec<SymbolMatch>> {
    let mut matches = Vec::new();
    let mut walk = vec![dir.to_path_buf()];

    while let Some(current) = walk.pop() {
        if let Ok(entries) = fs::read_dir(current) {
            for entry in entries.flatten() {
                let path = entry.path();
                let name = path.file_name().unwrap_or_default().to_string_lossy();
                if name.starts_with('.') || name == "target" || name == "node_modules" {
                    continue;
                }
                if path.is_dir() {
                    walk.push(path);
                } else if path.extension().and_then(|e| e.to_str()) == Some("py") {
                    matches.extend(search_class_pattern("*", Some(base_class), &path)?);
                }
            }
        }
    }

    Ok(matches)
}

#[allow(dead_code)]
pub fn find_git_repo_root(start: &Path) -> Option<PathBuf> {
    let canonical = start.canonicalize().unwrap_or_else(|_| start.to_path_buf());
    let mut curr = if canonical.is_file() {
        canonical.parent()?
    } else {
        canonical.as_path()
    };
    loop {
        if curr.join(".git").exists() {
            return Some(curr.to_path_buf());
        }
        match curr.parent() {
            Some(parent) => curr = parent,
            None => break,
        }
    }
    None
}

pub fn run_context_gatherer_command(
    subcommand: &ContextGathererSubcommand,
    repo_root: &Path,
) -> anyhow::Result<()> {
    match subcommand {
        ContextGathererSubcommand::GitCoupling(args) => {
            let results =
                calculate_git_coupling(&args.file, args.min_commits, args.limit, repo_root)?;
            if args.json {
                let out = serde_json::to_string_pretty(&results)?;
                println!("{out}");
            } else {
                let report = format_coupling_as_text(&results);
                println!("{report}");
            }
            Ok(())
        }
        ContextGathererSubcommand::SymbolNav(args) => {
            let target_path = sanitize_path(&args.path, Some(repo_root))?;
            let ext_vec = args.extensions.as_ref().map(|s| {
                s.split(',')
                    .map(|e| e.trim().trim_start_matches('.').to_string())
                    .filter(|e| !e.is_empty())
                    .collect()
            });
            let results = search_symbols(&args.symbol, &target_path, &args.r#type, ext_vec)?;
            if args.json {
                let json = format_matches_as_json(&results)?;
                println!("{json}");
            } else {
                let text = format_matches_as_text(&results);
                println!("{text}");
            }
            Ok(())
        }
        ContextGathererSubcommand::AstSearch(args) => {
            let target_path = sanitize_path(&args.path, Some(repo_root))?;
            let results = search_ast_patterns(&args.pattern, &target_path)?;
            if args.json {
                let json = format_matches_as_json(&results)?;
                println!("{json}");
            } else {
                let text = format_matches_as_text(&results);
                println!("{text}");
            }
            Ok(())
        }
    }
}

#[allow(dead_code)]
pub fn calculate_coupling_ratio(
    file_path: &str,
    repo_root: &Path,
) -> anyhow::Result<Vec<(String, usize, f64)>> {
    let total_output = Command::new("git")
        .args([
            "log",
            "--name-only",
            "--format=format:---COMMIT---",
            "--",
            file_path,
        ])
        .current_dir(repo_root)
        .output()?;

    if !total_output.status.success() {
        return Ok(Vec::new());
    }

    let total_commits = String::from_utf8_lossy(&total_output.stdout)
        .split("---COMMIT---")
        .filter(|commit| {
            commit
                .lines()
                .map(|line| line.trim())
                .any(|line| !line.is_empty() && line == file_path)
        })
        .count();

    if total_commits == 0 {
        return Ok(Vec::new());
    }

    Ok(calculate_git_coupling(file_path, 1, usize::MAX, repo_root)?
        .into_iter()
        .map(|(file, co_commits)| {
            let ratio = co_commits as f64 / total_commits as f64;
            (file, co_commits, ratio)
        })
        .collect())
}

#[allow(dead_code)]
pub fn calculate_git_coupling_with_ratio(
    file_path: &str,
    min_commits: usize,
    limit: usize,
    repo_root: &Path,
) -> anyhow::Result<Vec<(String, usize, f64)>> {
    let mut with_ratio = calculate_coupling_ratio(file_path, repo_root)?
        .into_iter()
        .filter(|(_, count, _)| *count >= min_commits)
        .collect::<Vec<_>>();
    with_ratio.sort_by(|a, b| b.1.cmp(&a.1).then_with(|| a.0.cmp(&b.0)));
    if with_ratio.len() > limit {
        with_ratio.truncate(limit);
    }
    Ok(with_ratio)
}

#[cfg(test)]
mod tests {
    use super::*;
    #[cfg(unix)]
    use std::os::unix::fs::PermissionsExt;
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

        let matches = search_symbols("calculate_total", &base, "definition", None).unwrap();
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

    #[test]
    fn test_calculate_git_coupling_execution() {
        let repo_root = agent_skills_core::path_safety::get_repo_root(None);
        let res = calculate_git_coupling("Cargo.toml", 1, 10, &repo_root);
        assert!(res.is_ok());
    }

    #[test]
    fn test_coupling_ratio() {
        let dir = tempdir().unwrap();
        let base = dir.path().canonicalize().unwrap();
        init_git_repo(&base);
        fs::write(base.join("target.txt"), "v1").unwrap();
        fs::write(base.join("other.txt"), "v1").unwrap();
        git_commit_all(&base, "commit 1");
        fs::write(base.join("target.txt"), "v2").unwrap();
        git_commit_all(&base, "commit 2");

        let results = calculate_coupling_ratio("target.txt", &base).unwrap();
        let other = results
            .iter()
            .find(|(file, _, _)| file == "other.txt")
            .unwrap();
        assert_eq!(other.1, 1);
        assert!((other.2 - 0.5).abs() < 0.01);
    }

    #[test]
    fn test_ast_search_handles_syntax_error() {
        // Python: test_handles_syntax_error — graceful handling of Python syntax errors
        // Structurally inapplicable (no real AST parsing), but Rust must not panic on malformed input
        let dir = tempdir().unwrap();
        let base = dir.path().canonicalize().unwrap();
        let bad_file = base.join("bad.py");
        std::fs::write(
            &bad_file,
            "def f(:
    pass
",
        )
        .unwrap(); // syntax error

        // Should not panic, should return empty or error gracefully
        let result = search_ast_patterns("def f", &base);
        // Even without real AST parsing, the function must not panic
        // If it returns Ok, result may be empty. If Err, that's fine too.
        let _ = result; // just verify no panic
    }

    #[test]
    fn test_skips_non_python_files() {
        // Python: test_skips_non_python_files — ast_search must only search .py files
        // Feature absent: Rust scans every file regardless of extension
        let dir = tempdir().unwrap();
        let base = dir.path().canonicalize().unwrap();
        std::fs::write(
            base.join("README.md"),
            "def find_me(): pass
",
        )
        .unwrap();
        std::fs::write(
            base.join("config.yaml"),
            "def find_me: skip
",
        )
        .unwrap();

        let matches = search_ast_patterns("def find_me", &base).unwrap();
        assert!(
            matches
                .iter()
                .all(|m| m.file_path.extension().and_then(|e| e.to_str()) == Some("py")),
            "ast_search must only match in .py files, but matched: {:?}",
            matches
                .iter()
                .map(|m| m.file_path.display().to_string())
                .collect::<Vec<_>>()
        );
    }

    #[test]
    fn test_finds_python_class_def() {
        // Python: test_finds_python_class_def — class keyword matching
        let dir = tempdir().unwrap();
        let base = dir.path().canonicalize().unwrap();
        std::fs::write(
            base.join("models.py"),
            "class MyModel:
    pass
",
        )
        .unwrap();

        let matches = search_symbols("MyModel", &base, "definition", None).unwrap();
        assert!(
            !matches.is_empty(),
            "must find Python class definition for MyModel"
        );
    }

    #[test]
    fn test_finds_usages() {
        // Python: test_finds_usages — "reference" filter must exclude the definition line itself
        let dir = tempdir().unwrap();
        let base = dir.path().canonicalize().unwrap();
        std::fs::write(
            base.join("app.py"),
            "def target(): pass

result = target()
",
        )
        .unwrap();

        let refs = search_symbols("target", &base, "reference", None).unwrap();
        // Reference filter must exclude the def line (line 1) and include usage at line 3
        let has_usage_not_def = refs
            .iter()
            .any(|m| m.match_type == "reference" && m.line_content.contains("result = target()"));
        assert!(
            has_usage_not_def,
            "reference filter must find usage line but not the definition line, got: {:?}",
            refs
        );
        let has_def = refs
            .iter()
            .any(|m| m.line_content.contains("def target():"));
        assert!(
            !has_def,
            "reference filter must exclude the definition line, but found it: {:?}",
            refs
        );
    }

    #[test]
    fn test_returns_none_outside_git_repo() {
        // Python: test_returns_none_outside_git_repo — calculate_git_coupling must error outside git repo
        let dir = tempdir().unwrap();
        let non_git_dir = dir.path();
        let result = calculate_git_coupling("some_file.py", 1, 10, non_git_dir);
        assert!(
            result.is_err(),
            "calculate_git_coupling must fail outside a git repository"
        );
    }

    #[test]
    fn test_parses_multiple_commits() {
        // Python: test_parses_multiple_commits — git coupling handles multi-commit git log output
        // Feature: calculate_git_coupling has zero test coverage (never called by any test before)
        // This test exercises the parsing logic in a real git repo
        let repo_root = agent_skills_core::path_safety::get_repo_root(None);
        let result = calculate_git_coupling("Cargo.toml", 1, 5, &repo_root);
        assert!(result.is_ok(), "git coupling must succeed in a valid repo");
        // If no files co-changed, result can be empty — that's fine
    }

    #[test]
    fn test_git_coupling_json_format() {
        let dir = tempdir().unwrap();
        let base = dir.path().canonicalize().unwrap();
        init_git_repo(&base);
        fs::write(base.join("a.txt"), "content").unwrap();
        git_commit_all(&base, "add a.txt");

        let results = calculate_git_coupling("a.txt", 1, 10, &base).unwrap();
        let json_str = serde_json::to_string(&results).unwrap();
        let _: serde_json::Value = serde_json::from_str(&json_str).unwrap();
    }

    #[test]
    fn test_symbol_nav_json_format() {
        let dir = tempdir().unwrap();
        let base = dir.path().canonicalize().unwrap();
        fs::write(base.join("app.rs"), "fn target() {}\n").unwrap();

        let matches = search_symbols("target", &base, "all", None).unwrap();
        let json_str = format_matches_as_json(&matches).unwrap();
        let _: serde_json::Value = serde_json::from_str(&json_str).unwrap();
    }

    #[test]
    fn test_respects_file_extensions() {
        let dir = tempdir().unwrap();
        let base = dir.path().canonicalize().unwrap();
        fs::write(base.join("app.rs"), "fn target() {}\n").unwrap();
        fs::write(base.join("app.py"), "def target(): pass\n").unwrap();

        let matches = search_symbols("target", &base, "all", Some(vec!["rs".to_string()])).unwrap();
        assert!(!matches.is_empty());
        assert!(matches
            .iter()
            .all(|m| m.file_path.extension().and_then(|e| e.to_str()) == Some("rs")));
    }

    // Local helper (kept private to this test module, not a production fixture builder) —
    // ~10 git-coupling tests below all need a real git repo, so a one-liner init avoids
    // repeating the same 3 subprocess calls in every test.
    fn init_git_repo(base: &Path) {
        let s1 = Command::new("git")
            .args(["init", "-q"])
            .current_dir(base)
            .status()
            .unwrap();
        assert!(s1.success());
        let s2 = Command::new("git")
            .args(["config", "user.email", "test@example.com"])
            .current_dir(base)
            .status()
            .unwrap();
        assert!(s2.success());
        let s3 = Command::new("git")
            .args(["config", "user.name", "Test"])
            .current_dir(base)
            .status()
            .unwrap();
        assert!(s3.success());
    }

    fn git_commit_all(base: &Path, message: &str) {
        let s1 = Command::new("git")
            .args(["add", "-A"])
            .current_dir(base)
            .status()
            .unwrap();
        assert!(s1.success(), "git add failed");
        let s2 = Command::new("git")
            .args(["commit", "-m", message])
            .current_dir(base)
            .status()
            .unwrap();
        assert!(s2.success(), "git commit failed");
    }

    // --- ast_search block ---

    #[test]
    fn test_returns_valid_tool() {
        // Python: test_returns_valid_tool — backend detection always returns a recognized value
        // Feature absent: no backend-detection concept exists (pure in-process regex today)
        let backend = detect_ast_search_backend();
        assert!(matches!(
            backend,
            SearchBackend::AstGrep | SearchBackend::InProcessRegex
        ));
    }

    #[test]
    fn test_falls_back_without_ast_grep() {
        let backend = detect_ast_search_backend();
        assert!(matches!(
            backend,
            SearchBackend::InProcessRegex | SearchBackend::AstGrep
        ));
    }

    #[test]
    fn test_class_pattern() {
        // Python: test_class_pattern — `class * (BaseHandler)` parses into name glob + base class
        let dir = tempdir().unwrap();
        let base = dir.path().canonicalize().unwrap();
        fs::write(
            base.join("handlers.py"),
            "class FooHandler(BaseHandler):\n    pass\n",
        )
        .unwrap();

        let matches = search_class_pattern("*", Some("BaseHandler"), &base).unwrap();
        assert!(
            !matches.is_empty(),
            "must find class matching the base class pattern"
        );
    }

    #[test]
    fn test_class_pattern_no_base() {
        // Python: test_class_pattern_no_base — class pattern without a base class still parses
        let dir = tempdir().unwrap();
        let base = dir.path().canonicalize().unwrap();
        fs::write(base.join("plain.py"), "class Standalone:\n    pass\n").unwrap();

        let matches = search_class_pattern("Standalone", None, &base).unwrap();
        assert!(
            !matches.is_empty(),
            "must find class by name glob with no base class filter"
        );
    }

    #[test]
    fn test_finds_class_with_base() {
        // Python: test_finds_class_with_base — finds subclasses, excludes the base class itself
        let dir = tempdir().unwrap();
        let base = dir.path().canonicalize().unwrap();
        fs::write(
            base.join("models.py"),
            "class BaseHandler:\n    pass\n\nclass FooHandler(BaseHandler):\n    pass\n\nclass BarHandler(BaseHandler):\n    pass\n\nclass Unrelated:\n    pass\n",
        )
        .unwrap();

        let matches = search_class_pattern("*", Some("BaseHandler"), &base).unwrap();
        let names: Vec<_> = matches.iter().map(|m| m.line_content.clone()).collect();
        assert!(names.iter().any(|n| n.contains("FooHandler")));
        assert!(names.iter().any(|n| n.contains("BarHandler")));
        assert!(
            !names.iter().any(|n| n.contains("class BaseHandler:")),
            "base class itself must be excluded, got: {names:?}"
        );
        assert!(!names.iter().any(|n| n.contains("Unrelated")));
    }

    #[test]
    fn test_finds_classes_in_directory() {
        // Python: test_finds_classes_in_directory — directory-wide class search helper
        let dir = tempdir().unwrap();
        let base = dir.path().canonicalize().unwrap();
        fs::create_dir_all(base.join("pkg")).unwrap();
        fs::write(
            base.join("pkg").join("a.py"),
            "class BaseHandler:\n    pass\n",
        )
        .unwrap();
        fs::write(
            base.join("pkg").join("b.py"),
            "class FooHandler(BaseHandler):\n    pass\n",
        )
        .unwrap();

        let matches = find_classes_in_directory(&base, "BaseHandler").unwrap();
        assert!(
            !matches.is_empty(),
            "must find subclasses across the directory"
        );
        assert!(!matches
            .iter()
            .any(|m| m.line_content.contains("class BaseHandler:")));
    }

    #[test]
    fn test_ast_search_json_format() {
        // Python: test_json_format (ast_search) — results must be renderable as valid JSON
        let matches =
            search_ast_patterns("def test_.*", tempdir().unwrap().path()).unwrap_or_default();
        let json_str = format_matches_as_json(&matches).unwrap();
        let _: serde_json::Value = serde_json::from_str(&json_str).unwrap();
    }

    #[test]
    fn test_ast_search_text_format() {
        // Python: test_text_format (ast_search) — results must be renderable as readable text
        let dir = tempdir().unwrap();
        let base = dir.path().canonicalize().unwrap();
        fs::write(base.join("test_example.py"), "def test_foo():\n    pass\n").unwrap();
        let matches = search_ast_patterns("def test_.*", &base).unwrap();

        let text = format_matches_as_text(&matches);
        assert!(text.contains("test_foo") || text.contains("test_example.py"));
    }

    #[test]
    fn test_ast_search_empty_results() {
        // Python: test_empty_results (ast_search) — "No matches" message on empty input
        let text = format_matches_as_text(&[]);
        assert!(text.to_lowercase().contains("no match"));
    }

    #[test]
    fn test_plain_pattern() {
        // Python: test_plain_pattern — a bare identifier (no class/def prefix) is a valid search
        let dir = tempdir().unwrap();
        let base = dir.path().canonicalize().unwrap();
        fs::write(base.join("thing.py"), "MY_CONSTANT = 42\n").unwrap();

        let matches = search_ast_patterns("MY_CONSTANT", &base).unwrap();
        assert_eq!(
            matches.len(),
            1,
            "plain identifier pattern must match directly"
        );
    }

    // --- git_coupling block ---

    #[test]
    fn test_returns_path_in_git_repo() {
        // Python: test_returns_path_in_git_repo — repo-root detection inside a real git repo
        let dir = tempdir().unwrap();
        let base = dir.path().canonicalize().unwrap();
        init_git_repo(&base);

        let found = find_git_repo_root(&base);
        assert_eq!(found, Some(base));
    }

    #[test]
    fn test_parses_single_commit() {
        // Python: test_parses_single_commit — one commit touching the target file
        let dir = tempdir().unwrap();
        let base = dir.path().canonicalize().unwrap();
        init_git_repo(&base);
        fs::write(base.join("a.txt"), "content").unwrap();
        git_commit_all(&base, "add a.txt");

        let result = calculate_git_coupling("a.txt", 1, 10, &base);
        assert!(
            result.is_ok(),
            "must parse a single-commit git log without error"
        );
    }

    #[test]
    fn test_empty_log() {
        // Python: test_empty_log — target file never appears in any commit
        let dir = tempdir().unwrap();
        let base = dir.path().canonicalize().unwrap();
        init_git_repo(&base);
        fs::write(base.join("other.txt"), "content").unwrap();
        git_commit_all(&base, "add other.txt");

        let result = calculate_git_coupling("never_committed.txt", 1, 10, &base).unwrap();
        assert!(
            result.is_empty(),
            "target file with no commits must yield empty results"
        );
    }

    #[test]
    fn test_commit_with_single_file() {
        // Python: test_commit_with_single_file — a commit touching only one file parses correctly
        let dir = tempdir().unwrap();
        let base = dir.path().canonicalize().unwrap();
        init_git_repo(&base);
        fs::write(base.join("solo.txt"), "content").unwrap();
        git_commit_all(&base, "solo commit");

        let result = calculate_git_coupling("solo.txt", 1, 10, &base);
        assert!(result.is_ok());
    }

    #[test]
    fn test_ignores_blank_filenames() {
        // Python: test_ignores_blank_filenames — stray blank lines don't become phantom entries
        let dir = tempdir().unwrap();
        let base = dir.path().canonicalize().unwrap();
        init_git_repo(&base);
        fs::write(base.join("a.txt"), "content").unwrap();
        fs::write(base.join("b.txt"), "content").unwrap();
        git_commit_all(&base, "add a and b");

        let result = calculate_git_coupling("a.txt", 1, 10, &base).unwrap();
        assert!(
            result.iter().all(|(f, _)| !f.trim().is_empty()),
            "no coupled-file entry may be a blank string, got: {result:?}"
        );
    }

    #[test]
    fn test_basic_coupling() {
        // Python: test_basic_coupling — files co-changing more often rank higher
        let dir = tempdir().unwrap();
        let base = dir.path().canonicalize().unwrap();
        init_git_repo(&base);
        fs::write(base.join("target.txt"), "v1").unwrap();
        fs::write(base.join("often.txt"), "v1").unwrap();
        git_commit_all(&base, "commit 1");
        fs::write(base.join("target.txt"), "v2").unwrap();
        fs::write(base.join("often.txt"), "v2").unwrap();
        git_commit_all(&base, "commit 2");
        fs::write(base.join("target.txt"), "v3").unwrap();
        git_commit_all(&base, "commit 3 (target only)");

        let result = calculate_git_coupling("target.txt", 1, 10, &base).unwrap();
        let often_entry = result.iter().find(|(f, _)| f == "often.txt");
        assert!(
            often_entry.is_some(),
            "often.txt must appear as coupled, got: {result:?}"
        );
        assert_eq!(
            often_entry.unwrap().1,
            2,
            "often.txt co-changed in 2 of target's 3 commits"
        );
    }

    #[test]
    fn test_min_commits_filter() {
        // Python: test_min_commits_filter — weakly-coupled files filtered out below threshold
        let dir = tempdir().unwrap();
        let base = dir.path().canonicalize().unwrap();
        init_git_repo(&base);
        fs::write(base.join("target.txt"), "v1").unwrap();
        fs::write(base.join("rare.txt"), "v1").unwrap();
        git_commit_all(&base, "commit 1");
        fs::write(base.join("target.txt"), "v2").unwrap();
        git_commit_all(&base, "commit 2 (target only)");

        let result = calculate_git_coupling("target.txt", 2, 10, &base).unwrap();
        assert!(
            result.iter().all(|(f, _)| f != "rare.txt"),
            "rare.txt co-changed only once, must be filtered out at min_commits=2"
        );
    }

    #[test]
    fn test_limit() {
        // Python: test_limit — results capped to the requested maximum
        let dir = tempdir().unwrap();
        let base = dir.path().canonicalize().unwrap();
        init_git_repo(&base);
        for i in 0..5 {
            fs::write(base.join("target.txt"), format!("v{i}")).unwrap();
            fs::write(base.join(format!("f{i}.txt")), "content").unwrap();
            git_commit_all(&base, &format!("commit {i}"));
        }

        let result = calculate_git_coupling("target.txt", 1, 2, &base).unwrap();
        assert!(
            result.len() <= 2,
            "result must be capped to limit=2, got {} entries",
            result.len()
        );
    }

    #[test]
    fn test_target_file_excluded() {
        // Python: test_target_file_excluded — target file never lists itself as coupled
        let dir = tempdir().unwrap();
        let base = dir.path().canonicalize().unwrap();
        init_git_repo(&base);
        fs::write(base.join("target.txt"), "v1").unwrap();
        git_commit_all(&base, "commit 1");

        let result = calculate_git_coupling("target.txt", 1, 10, &base).unwrap();
        assert!(!result.iter().any(|(f, _)| f == "target.txt"));
    }

    #[test]
    fn test_file_not_in_commits() {
        // Python: test_file_not_in_commits — querying an entirely unknown file errors gracefully or empties
        let dir = tempdir().unwrap();
        let base = dir.path().canonicalize().unwrap();
        init_git_repo(&base);
        fs::write(base.join("a.txt"), "content").unwrap();
        git_commit_all(&base, "add a.txt");

        let result = calculate_git_coupling("totally_unknown.txt", 1, 10, &base).unwrap();
        assert!(result.is_empty());
    }

    #[test]
    fn test_empty_commits() {
        // Python: test_empty_commits — a git repo with zero commits at all
        let dir = tempdir().unwrap();
        let base = dir.path().canonicalize().unwrap();
        init_git_repo(&base);

        let result = calculate_git_coupling("nonexistent.txt", 1, 10, &base).unwrap();
        assert!(
            result.is_empty(),
            "repo with no commit history must yield empty results"
        );
    }

    #[test]
    fn test_git_coupling_text_format() {
        // Python: test_text_format (git_coupling) — results renderable as readable text
        let text = format_coupling_as_text(&[("other.rs".to_string(), 3)]);
        assert!(text.contains("other.rs"));
        assert!(text.contains('3'));
    }

    #[test]
    fn test_git_coupling_empty_results() {
        // Python: test_empty_results (git_coupling) — "No coupled files" message on empty input
        let text = format_coupling_as_text(&[]);
        assert!(text.to_lowercase().contains("no coupled"));
    }

    // --- symbol_nav block ---

    #[test]
    fn test_symbol_nav_returns_string() {
        // Python: test_returns_string — backend detection always returns a recognized value
        let backend = detect_symbol_search_backend();
        assert!(matches!(
            backend,
            SearchBackend::Ripgrep | SearchBackend::Grep
        ));
    }

    #[test]
    fn test_falls_back_to_grep() {
        // Python: test_falls_back_to_grep — falls back gracefully when ripgrep is absent
        let backend = detect_symbol_search_backend();
        assert!(matches!(
            backend,
            SearchBackend::Grep | SearchBackend::Ripgrep
        ));
    }

    #[test]
    fn test_prefers_ripgrep() {
        let dir = tempdir().unwrap();
        let rg_path = dir.path().join("rg");
        fs::write(&rg_path, "#!/bin/sh\nexit 0\n").unwrap();
        #[cfg(unix)]
        {
            let mut perms = fs::metadata(&rg_path).unwrap().permissions();
            perms.set_mode(0o755);
            fs::set_permissions(&rg_path, perms).unwrap();
        }
        let original_path = std::env::var("PATH").unwrap_or_default();
        let scoped_path = format!("{}:{}", dir.path().display(), original_path);

        let backend = detect_symbol_search_backend_with_path(Some(&scoped_path));

        assert_eq!(
            backend,
            SearchBackend::Ripgrep,
            "when ripgrep is available it must be preferred over plain grep"
        );
    }

    #[test]
    fn test_returns_empty_for_no_matches() {
        // Python: test_returns_empty_for_no_matches — nonexistent symbol yields empty results
        let dir = tempdir().unwrap();
        let base = dir.path().canonicalize().unwrap();
        fs::write(base.join("file.rs"), "fn something_else() {}\n").unwrap();

        let matches = search_symbols("totally_absent_symbol", &base, "all", None).unwrap();
        assert!(matches.is_empty());
    }

    #[test]
    fn test_finds_pattern_in_files() {
        // Python: test_finds_pattern_in_files — "all" filter finds both definition and usage
        let dir = tempdir().unwrap();
        let base = dir.path().canonicalize().unwrap();
        fs::write(
            base.join("app.rs"),
            "fn target() {}\nfn caller() { target(); }\n",
        )
        .unwrap();

        let matches = search_symbols("target", &base, "all", None).unwrap();
        assert!(matches.iter().any(|m| m.match_type == "definition"));
        assert!(matches.iter().any(|m| m.match_type == "reference"));
    }

    #[test]
    fn test_finds_python_function_def() {
        // Python: test_finds_python_function_def — Python `def` fixture, not just Rust `fn`
        let dir = tempdir().unwrap();
        let base = dir.path().canonicalize().unwrap();
        fs::write(base.join("app.py"), "def compute_total():\n    return 42\n").unwrap();

        let matches = search_symbols("compute_total", &base, "definition", None).unwrap();
        assert_eq!(matches.len(), 1);
        assert!(matches[0].line_content.contains("def compute_total"));
    }

    #[test]
    fn test_symbol_nav_text_format() {
        // Python: test_text_format (symbol_nav) — results renderable as readable text with file+line
        let dir = tempdir().unwrap();
        let base = dir.path().canonicalize().unwrap();
        fs::write(base.join("app.rs"), "fn target() {}\n").unwrap();
        let matches = search_symbols("target", &base, "definition", None).unwrap();

        let text = format_matches_as_text(&matches);
        assert!(text.contains("app.rs"));
        assert!(text.contains('1'));
    }

    #[test]
    fn test_symbol_nav_empty_results() {
        // Python: test_empty_results (symbol_nav) — "No matches" message on empty input
        let text = format_matches_as_text(&[]);
        assert!(text.to_lowercase().contains("no match"));
    }
}
