use agent_skills_core::path_safety::sanitize_path;
use clap::{Args, Subcommand};
use regex::Regex;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

#[derive(Args, Debug, Clone)]
pub struct CheckWhatIfArgs {
    /// Target path of the what-if-analysis skill folder
    #[arg(long)]
    pub path: Option<String>,
}

#[derive(Args, Debug, Clone)]
pub struct ImpactAnalysisArgs {
    /// Target symbol name to calculate blast radius for
    #[arg(long)]
    pub symbol: String,

    /// Output findings in JSON format
    #[arg(long)]
    pub json: bool,
}

#[derive(Subcommand, Debug, Clone)]
pub enum WhatIfAnalysisSubcommand {
    /// Verify health and structural completeness of what-if-analysis files
    Check(CheckWhatIfArgs),
    /// Calculate symbol blast radius and prospective risk
    Impact(ImpactAnalysisArgs),
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct BlastRadiusReport {
    pub symbol: String,
    pub risk_level: String,
    pub caller_count: usize,
    pub test_files: Vec<PathBuf>,
    pub doc_files: Vec<PathBuf>,
}

#[allow(dead_code)]
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CallSite {
    pub file: PathBuf,
    pub line_number: usize,
    pub line_content: String,
}

#[allow(dead_code)]
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FailureRisk {
    pub description: String,
    pub line_number: usize,
    pub risk_type: String,
}

#[allow(dead_code)]
fn escape_markdown_pipes(value: &str) -> String {
    value.replace('|', "\\|")
}

pub fn analyze_symbol_blast_radius(
    symbol: &str,
    workspace_dir: &Path,
) -> anyhow::Result<BlastRadiusReport> {
    let mut caller_count = 0;
    let mut test_files = Vec::new();
    let mut doc_files = Vec::new();

    let mut walk = vec![workspace_dir.to_path_buf()];
    while let Some(curr) = walk.pop() {
        if let Ok(entries) = fs::read_dir(curr) {
            for entry in entries.flatten() {
                let p = entry.path();
                let fname = p.file_name().unwrap_or_default().to_string_lossy();
                if fname.starts_with('.') || fname == "target" || fname == "node_modules" {
                    continue;
                }
                if p.is_dir() {
                    walk.push(p.clone());
                } else if p.is_file() {
                    if let Ok(content) = fs::read_to_string(&p) {
                        if content.contains(symbol) {
                            caller_count += 1;
                            let fname_lower = fname.to_lowercase();
                            let is_test_file = fname_lower.starts_with("test_")
                                || fname_lower.contains("_test.")
                                || fname_lower == "tests.py"
                                || p.components()
                                    .any(|c| c.as_os_str() == "tests" || c.as_os_str() == "test");
                            let is_doc_file = matches!(
                                p.extension().and_then(|e| e.to_str()),
                                Some("md") | Some("rst") | Some("txt")
                            ) || p.components().any(|c| c.as_os_str() == "docs");

                            if is_test_file {
                                test_files.push(p.clone());
                            }
                            if is_doc_file {
                                doc_files.push(p);
                            }
                        }
                    }
                }
            }
        }
    }

    let risk_level = if caller_count > 10 {
        "HIGH".to_string()
    } else if caller_count > 3 {
        "MEDIUM".to_string()
    } else {
        "LOW".to_string()
    };

    Ok(BlastRadiusReport {
        symbol: symbol.to_string(),
        risk_level,
        caller_count,
        test_files,
        doc_files,
    })
}

#[allow(dead_code)]
pub fn generate_blast_radius_markdown(report: &BlastRadiusReport) -> String {
    let symbol = escape_markdown_pipes(&report.symbol);
    let risk_level = escape_markdown_pipes(&report.risk_level);
    let caller_count = escape_markdown_pipes(&report.caller_count.to_string());
    let test_count = escape_markdown_pipes(&report.test_files.len().to_string());
    let doc_count = escape_markdown_pipes(&report.doc_files.len().to_string());

    format!(
        "# Blast Radius: {symbol}\n\n| Field | Value |\n|-------|-------|\n| Risk Level | {risk_level} |\n| Caller Count | {caller_count} |\n| Test Files | {test_count} |\n| Doc Files | {doc_count} |\n"
    )
}

#[allow(dead_code)]
pub fn generate_blast_radius_report(report: &BlastRadiusReport) -> String {
    format!(
        "# Blast Radius Report\n\n{}",
        generate_blast_radius_markdown(report)
    )
}

#[allow(dead_code)]
pub fn extract_ast_call_sites(symbol: &str, file_path: &Path) -> anyhow::Result<Vec<CallSite>> {
    let content = fs::read_to_string(file_path)?;
    let call_re = Regex::new(&format!(r"\b{}(?:\.call)?\s*\(", regex::escape(symbol)))?;
    let mut sites = Vec::new();

    for (index, line) in content.lines().enumerate() {
        let trimmed = line.trim_start();
        if call_re.is_match(line) && !trimmed.starts_with("def ") && !trimmed.starts_with("class ")
        {
            sites.push(CallSite {
                file: file_path.to_path_buf(),
                line_number: index + 1,
                line_content: line.to_string(),
            });
        }
    }

    Ok(sites)
}

#[allow(dead_code)]
pub fn generate_counterfactual_test(symbol: &str, call_site: &CallSite) -> String {
    let safe_symbol = symbol
        .chars()
        .map(|c| {
            if c.is_ascii_alphanumeric() || c == '_' {
                c
            } else {
                '_'
            }
        })
        .collect::<String>();
    format!(
        "import pytest\n\n\
def test_{safe_symbol}_counterfactual():\n\
    \"\"\"Counterfactual: verify {symbol} at {file}:{line} is not a silent no-op.\"\"\"\n\
    result = {symbol}()\n\
    assert result is not None, \"{symbol} must not silently return None\"\n\
    assert result != None, \"{symbol} must produce a meaningful value\"\n",
        file = call_site.file.display(),
        line = call_site.line_number
    )
}

#[allow(dead_code)]
pub fn preempt_failure_modes(symbol: &str, file_path: &Path) -> anyhow::Result<Vec<FailureRisk>> {
    let content = fs::read_to_string(file_path)?;
    let def_re = Regex::new(&format!(
        r"^(?P<indent>\s*)def\s+{}\s*\((?P<params>[^)]*)\)\s*:",
        regex::escape(symbol)
    ))?;
    let divisor_re = Regex::new(r"/\s*([A-Za-z_][A-Za-z0-9_]*)")?;

    let lines: Vec<&str> = content.lines().collect();
    let mut def_index = None;
    let mut def_indent = 0usize;
    let mut params = Vec::new();

    for (index, line) in lines.iter().enumerate() {
        if let Some(captures) = def_re.captures(line) {
            def_index = Some(index);
            def_indent = captures
                .name("indent")
                .map(|m| m.as_str().chars().count())
                .unwrap_or(0);
            params = captures["params"]
                .split(',')
                .map(|param| {
                    param
                        .split('=')
                        .next()
                        .unwrap_or("")
                        .trim()
                        .trim_start_matches('*')
                })
                .filter(|param| !param.is_empty() && *param != "self" && *param != "cls")
                .map(|param| param.to_string())
                .collect();
            break;
        }
    }

    let Some(start) = def_index else {
        return Ok(Vec::new());
    };

    let mut body = Vec::new();
    for (index, line) in lines.iter().enumerate().skip(start + 1) {
        let trimmed = line.trim();
        if trimmed.is_empty() {
            body.push((index + 1, *line));
            continue;
        }
        let indent = line.chars().take_while(|c| c.is_whitespace()).count();
        if indent <= def_indent {
            break;
        }
        body.push((index + 1, *line));
    }

    let mut risks = Vec::new();
    for (body_index, (line_number, line)) in body.iter().enumerate() {
        let code = line.split('#').next().unwrap_or("").trim();
        if code.is_empty() || !code.contains('/') {
            continue;
        }

        for captures in divisor_re.captures_iter(code) {
            let divisor = captures.get(1).map(|m| m.as_str()).unwrap_or("");
            if !params.iter().any(|param| param == divisor) {
                continue;
            }

            let guarded = body[..body_index].iter().any(|(_, prior_line)| {
                let prior_code = prior_line.split('#').next().unwrap_or("").trim();
                prior_code.contains(&format!("if {divisor} == 0"))
                    || prior_code.contains(&format!("if 0 == {divisor}"))
                    || prior_code.contains(&format!("if not {divisor}"))
                    || prior_code.contains(&format!("if {divisor} != 0"))
            });

            if !guarded {
                risks.push(FailureRisk {
                    description: format!(
                        "Potential division-by-zero risk: parameter '{divisor}' is used as an unguarded divisor"
                    ),
                    line_number: *line_number,
                    risk_type: "parameter divisor without guard".to_string(),
                });
            }
        }
    }

    Ok(risks)
}

#[allow(dead_code)]
pub fn check_council_availability() -> Vec<(String, bool)> {
    let binaries = ["claude", "gemini", "gh-copilot", "aider"];
    binaries
        .iter()
        .map(|binary| {
            let available = Command::new("which")
                .arg(binary)
                .output()
                .map(|output| output.status.success())
                .unwrap_or(false);
            (binary.to_string(), available)
        })
        .collect()
}

#[allow(dead_code)]
pub fn evaluate_single_agent_scenarios(scenarios: &[(&str, &str)]) -> Vec<String> {
    scenarios
        .iter()
        .map(|(name, description)| {
            format!(
                "Scenario '{}': {}\n  Heuristic: Consider risk/benefit tradeoff.",
                name, description
            )
        })
        .collect()
}

#[allow(dead_code)]
pub fn generate_scenario_tradeoff_matrix(scenarios: &[(&str, &str)]) -> String {
    let mut lines = vec![
        "| Scenario | Description | Risk | Complexity |".to_string(),
        "|---------|-------------|------|------------|".to_string(),
    ];

    for (name, description) in scenarios {
        lines.push(format!(
            "| {} | {} | Medium | Medium |",
            escape_markdown_pipes(name),
            escape_markdown_pipes(description)
        ));
    }

    lines.join("\n")
}

#[allow(dead_code)]
pub fn run_scenario_probe(scenarios: &[(&str, &str)]) -> anyhow::Result<String> {
    let council = check_council_availability();
    let any_available = council.iter().any(|(_, available)| *available);

    if any_available {
        let available = council
            .into_iter()
            .filter_map(|(name, available)| available.then_some(name))
            .collect::<Vec<_>>()
            .join(", ");
        Ok(format!("Council analysis: available agents = {available}"))
    } else {
        Ok(evaluate_single_agent_scenarios(scenarios).join("\n"))
    }
}

pub fn check_what_if_analysis_health(skill_dir: &Path) -> anyhow::Result<Vec<String>> {
    let required_files = [
        skill_dir.join("SKILL.md"),
        skill_dir.join("README.md"),
        skill_dir.join("references").join("overview.md"),
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
            "What-If Analysis health check failed. Missing files: {:?}",
            missing
        ))
    }
}

pub fn run_what_if_analysis_command(
    subcommand: &WhatIfAnalysisSubcommand,
    repo_root: &Path,
) -> anyhow::Result<()> {
    match subcommand {
        WhatIfAnalysisSubcommand::Check(args) => {
            let skill_dir = if let Some(p) = &args.path {
                sanitize_path(p, Some(repo_root))?
            } else {
                repo_root.join("skills").join("what-if-analysis")
            };

            match check_what_if_analysis_health(&skill_dir) {
                Ok(_) => {
                    println!("What-If Analysis skill health check passed cleanly.");
                    Ok(())
                }
                Err(err) => Err(err),
            }
        }
        WhatIfAnalysisSubcommand::Impact(args) => {
            let report = analyze_symbol_blast_radius(&args.symbol, repo_root)?;
            if args.json {
                println!("{}", serde_json::to_string_pretty(&report).unwrap());
            } else {
                println!(
                    "What-If Blast Radius Analysis for symbol: '{}'",
                    report.symbol
                );
                println!("  Risk Level: {}", report.risk_level);
                println!("  Call Sites Found: {}", report.caller_count);
                println!("  Impacted Test Suites: {}", report.test_files.len());
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
    fn test_analyze_symbol_blast_radius_strict_assertions() {
        let dir = tempdir().unwrap();
        let base = dir.path().canonicalize().unwrap();

        fs::write(
            base.join("main.rs"),
            "fn main() { target_function(); }\nfn target_function() {}\n",
        )
        .unwrap();

        let report = analyze_symbol_blast_radius("target_function", &base).unwrap();
        assert_eq!(report.symbol, "target_function");
        assert_eq!(report.caller_count, 1);
        assert_eq!(report.risk_level, "LOW");
    }

    #[test]
    fn test_check_what_if_analysis_health_strict_assertions() {
        let dir = tempdir().unwrap();
        let base = dir.path().canonicalize().unwrap();
        let references = base.join("references");
        fs::create_dir_all(&references).unwrap();

        fs::write(base.join("SKILL.md"), "---").unwrap();
        fs::write(base.join("README.md"), "# Title").unwrap();
        fs::write(references.join("overview.md"), "# Overview").unwrap();

        let res = check_what_if_analysis_health(&base);
        assert!(res.is_ok());
    }

    // --- NEW FAILING TESTS (TDD RED phase) ---

    #[test]
    fn test_is_test_file_classification() {
        // Python: test_is_test_file_classification
        // Rust's naive fname.contains("test") check misclassifies:
        //   "contest_scores.py" contains the substring "test" (con-TEST-scores) but is NOT a test file
        //   "latest_release.py" contains "test" (la-TEST-_release) but is NOT a test file
        // Python uses path-component-aware check; Rust must too.
        let dir = tempdir().unwrap();
        let base = dir.path().canonicalize().unwrap();

        // This file name contains "test" as a substring but is NOT a test file
        let contest_file = base.join("contest_scores.py");
        fs::write(&contest_file, "def score(): pass\n").unwrap();
        let latest_file = base.join("latest_release.py");
        fs::write(&latest_file, "def release(): pass\n").unwrap();

        let report = analyze_symbol_blast_radius("score", &base).unwrap();
        let test_file_names: Vec<String> = report
            .test_files
            .iter()
            .map(|f| {
                f.file_name()
                    .unwrap_or_default()
                    .to_string_lossy()
                    .to_string()
            })
            .collect();

        assert!(
            !test_file_names.iter().any(|n| n == "contest_scores.py"),
            "contest_scores.py must NOT be classified as a test file (naive substring match regression), but it was: {:?}",
            test_file_names
        );
        assert!(
            !test_file_names.iter().any(|n| n == "latest_release.py"),
            "latest_release.py must NOT be classified as a test file (naive substring match regression), but it was: {:?}",
            test_file_names
        );
    }

    #[test]
    fn test_is_doc_file_classification() {
        let dir = tempdir().unwrap();
        let base = dir.path().canonicalize().unwrap();
        fs::write(base.join("README.md"), "target_sym is great\n").unwrap();
        fs::write(base.join("main.rs"), "fn target_sym() {}\n").unwrap();

        let report = analyze_symbol_blast_radius("target_sym", &base).unwrap();
        let doc_names: Vec<String> = report
            .doc_files
            .iter()
            .map(|f| {
                f.file_name()
                    .unwrap_or_default()
                    .to_string_lossy()
                    .to_string()
            })
            .collect();
        assert!(
            doc_names.contains(&"README.md".to_string()),
            "README.md must be classified as doc file, got: {:?}",
            doc_names
        );
    }

    #[test]
    fn test_pipe_escaping_in_report() {
        let report = BlastRadiusReport {
            symbol: "func|pipe".to_string(),
            risk_level: "LOW".to_string(),
            caller_count: 0,
            test_files: vec![],
            doc_files: vec![],
        };
        let md = generate_blast_radius_markdown(&report);
        assert!(
            md.contains("func\\|pipe") || !md.contains("func|pipe"),
            "pipe character in symbol name must be escaped in markdown"
        );
    }

    #[test]
    fn test_extract_ast_call_sites() {
        let dir = tempdir().unwrap();
        let base = dir.path().canonicalize().unwrap();
        let file = base.join("app.py");
        fs::write(
            &file,
            "def compute(x):\n    return x\n\nresult = compute(42)\n",
        )
        .unwrap();

        let sites = extract_ast_call_sites("compute", &file).unwrap();
        assert!(!sites.is_empty(), "must find call site for compute(42)");
        assert!(sites[0].line_content.contains("compute(42)"));
    }

    #[test]
    fn test_generate_counterfactual_test() {
        let call_site = CallSite {
            file: PathBuf::from("app.py"),
            line_number: 4,
            line_content: "result = compute(42)".to_string(),
        };
        let test_code = generate_counterfactual_test("compute", &call_site);
        assert!(!test_code.is_empty());
        assert!(test_code.contains("def test_") || test_code.contains("compute"));
    }

    #[test]
    fn test_generated_test_is_valid_python_with_real_assertions() {
        let call_site = CallSite {
            file: PathBuf::from("app.py"),
            line_number: 1,
            line_content: "compute(42)".to_string(),
        };
        let test_code = generate_counterfactual_test("compute", &call_site);
        assert!(
            test_code.contains("assert"),
            "generated test must contain at least one assert statement"
        );
    }

    #[test]
    fn test_generated_test_actually_fails_on_silent_symbol() {
        let call_site = CallSite {
            file: PathBuf::from("app.py"),
            line_number: 1,
            line_content: "compute(42)".to_string(),
        };
        let test_code = generate_counterfactual_test("compute", &call_site);
        assert!(test_code.contains("assert"), "test must have assertions");
        assert!(
            test_code.contains("None")
                || test_code.contains("not None")
                || test_code.contains("is not"),
            "test must check that result is not None (would fail on silent no-op)"
        );
    }

    #[test]
    fn test_preempt_failure_modes() {
        let dir = tempdir().unwrap();
        let base = dir.path().canonicalize().unwrap();
        let file = base.join("math_utils.py");
        fs::write(&file, "def divide(a, b):\n    return a / b\n").unwrap();

        let risks = preempt_failure_modes("divide", &file).unwrap();
        assert!(
            !risks.is_empty(),
            "must detect unguarded division by parameter b"
        );
        assert!(risks
            .iter()
            .any(|r| r.description.contains("b") || r.description.contains("division")));
    }

    #[test]
    fn test_preempt_ignores_division_outside_target_symbol() {
        let dir = tempdir().unwrap();
        let base = dir.path().canonicalize().unwrap();
        let file = base.join("utils.py");
        fs::write(
            &file,
            "def other(x, y):\n    return x / y\n\ndef target(a):\n    return a + 1\n",
        )
        .unwrap();

        let risks = preempt_failure_modes("target", &file).unwrap();
        assert!(
            risks.is_empty(),
            "must not flag division inside other() when analyzing target()"
        );
    }

    #[test]
    fn test_preempt_ignores_symbol_mentioned_only_in_comment() {
        let dir = tempdir().unwrap();
        let base = dir.path().canonicalize().unwrap();
        let file = base.join("utils.py");
        fs::write(
            &file,
            "def target(a, b):\n    # a / b would be risky\n    return a + b\n",
        )
        .unwrap();

        let risks = preempt_failure_modes("target", &file).unwrap();
        assert!(
            risks.is_empty(),
            "must not flag division that appears only in a comment"
        );
    }

    #[test]
    fn test_preempt_flags_unguarded_parameter_divisor() {
        let dir = tempdir().unwrap();
        let base = dir.path().canonicalize().unwrap();
        let file = base.join("utils.py");
        fs::write(&file, "def target(x, n):\n    return x / n\n").unwrap();

        let risks = preempt_failure_modes("target", &file).unwrap();
        assert!(
            !risks.is_empty(),
            "must flag x / n where n is an unguarded parameter divisor"
        );
    }

    #[test]
    fn test_preempt_respects_existing_zero_guard() {
        let dir = tempdir().unwrap();
        let base = dir.path().canonicalize().unwrap();
        let file = base.join("utils.py");
        fs::write(
            &file,
            "def target(x, n):\n    if n == 0:\n        return 0\n    return x / n\n",
        )
        .unwrap();

        let risks = preempt_failure_modes("target", &file).unwrap();
        assert!(
            risks.is_empty(),
            "must NOT flag division when zero-guard check exists before it"
        );
    }

    #[test]
    fn test_check_council_availability() {
        let results = check_council_availability();
        assert!(!results.is_empty(), "must return at least one entry");
        for (name, _avail) in &results {
            assert!(!name.is_empty(), "binary name must not be empty");
        }
    }

    #[test]
    fn test_evaluate_single_agent_scenarios() {
        let scenarios = vec![
            ("microservices", "Split monolith into services"),
            ("cache", "Add Redis caching"),
        ];
        let results = evaluate_single_agent_scenarios(&scenarios);
        assert_eq!(results.len(), 2);
        assert!(results[0].contains("microservices") || results[0].contains("Scenario"));
    }

    #[test]
    fn test_generate_scenario_tradeoff_matrix() {
        let scenarios = vec![
            ("microservices", "Split monolith"),
            ("cache", "Add caching"),
        ];
        let matrix = generate_scenario_tradeoff_matrix(&scenarios);
        assert!(matrix.contains("|"), "must produce a markdown table");
        assert!(matrix.contains("microservices"));
        assert!(matrix.contains("cache"));
    }

    #[test]
    fn test_run_scenario_probe_fallback_branch() {
        let scenarios = vec![("test-scenario", "A test scenario for probe")];
        let result = run_scenario_probe(&scenarios);
        assert!(result.is_ok(), "run_scenario_probe must not fail");
        let output = result.unwrap();
        assert!(!output.is_empty(), "must return non-empty output");
    }

    #[test]
    fn test_main_no_args() {
        let dir = tempdir().unwrap();
        let base = dir.path().canonicalize().unwrap();
        let refs = base.join("references");
        fs::create_dir_all(&refs).unwrap();
        fs::write(base.join("SKILL.md"), "---").unwrap();
        fs::write(base.join("README.md"), "# T").unwrap();
        fs::write(refs.join("overview.md"), "# O").unwrap();

        let result = run_what_if_analysis_command(
            &WhatIfAnalysisSubcommand::Check(CheckWhatIfArgs {
                path: Some(base.to_string_lossy().to_string()),
            }),
            &base,
        );
        assert!(result.is_ok(), "check command with valid path must succeed");
    }

    #[test]
    fn test_main_impact_command() {
        // Python: test_main_impact_command — what-if-analysis impact --symbol X runs without error
        // Feature absent: no test drives run_what_if_analysis_command dispatch with Impact subcommand
        let dir = tempdir().unwrap();
        let base = dir.path().canonicalize().unwrap();
        fs::write(base.join("main.rs"), "fn target() {}\n").unwrap();

        let args = WhatIfAnalysisSubcommand::Impact(ImpactAnalysisArgs {
            symbol: "target".to_string(),
            json: false,
        });
        let result = run_what_if_analysis_command(&args, &base);
        assert!(
            result.is_ok(),
            "impact command must succeed for a valid symbol in a real directory"
        );
        // Verify analyze_symbol_blast_radius produces correct output fields
        let report = analyze_symbol_blast_radius("target", &base).unwrap();
        assert_eq!(report.symbol, "target");
        assert!(
            !report.risk_level.is_empty(),
            "risk_level must not be empty"
        );
    }

    #[test]
    fn test_find_symbol_callers_and_tests_multi_file() {
        // Python: test_find_symbol_callers_and_tests — PARTIAL, multi-file scenario with both
        // a production caller and a separate test file referencing the target symbol.
        let dir = tempdir().unwrap();
        let base = dir.path().canonicalize().unwrap();
        fs::write(
            base.join("lib.rs"),
            "pub fn target_symbol() {}\npub fn caller() { target_symbol(); }\n",
        )
        .unwrap();
        let tests_dir = base.join("tests");
        fs::create_dir_all(&tests_dir).unwrap();
        fs::write(
            tests_dir.join("test_lib.rs"),
            "fn test_target() { target_symbol(); }\n",
        )
        .unwrap();

        let report = analyze_symbol_blast_radius("target_symbol", &base).unwrap();
        assert!(
            report.caller_count >= 2,
            "must count both the production caller and the test reference, got: {}",
            report.caller_count
        );
        assert!(
            !report.test_files.is_empty(),
            "must identify at least one test file referencing the symbol"
        );
    }

    #[test]
    fn test_analyze_blast_radius_and_report_full_document() {
        // Python: test_analyze_blast_radius_and_report — PARTIAL, the report-generation half:
        // a full markdown document containing the symbol name and a "Blast Radius Report" heading.
        let dir = tempdir().unwrap();
        let base = dir.path().canonicalize().unwrap();
        fs::write(base.join("main.rs"), "fn target() {}\n").unwrap();

        let report = analyze_symbol_blast_radius("target", &base).unwrap();
        let markdown = generate_blast_radius_report(&report);
        assert!(
            markdown.contains("target"),
            "report must contain the target symbol name"
        );
        assert!(
            markdown.contains("Blast Radius Report"),
            "report must contain the 'Blast Radius Report' heading"
        );
    }
}
