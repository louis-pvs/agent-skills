use agent_skills_core::path_safety::sanitize_path;
use clap::{Args, Subcommand};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;

#[derive(Args, Debug, Clone)]
pub struct CheckProgressArgs {
    /// Target path of the self-progress skill folder
    #[arg(long)]
    pub path: Option<String>,
}

#[derive(Args, Debug, Clone)]
pub struct AnalyzeTranscriptArgs {
    /// Path of the session transcript JSONL file to parse
    #[arg(long)]
    pub transcript: String,

    /// Output findings in JSON format
    #[arg(long)]
    pub json: bool,
}

#[derive(Subcommand, Debug, Clone)]
pub enum SelfProgressSubcommand {
    /// Verify health and structural completeness of self-progress files
    Check(CheckProgressArgs),
    /// Parse transcript signals and generate retrospective summary
    Analyze(AnalyzeTranscriptArgs),
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct TranscriptSummary {
    pub total_steps: usize,
    pub user_inputs: usize,
    pub tool_calls: usize,
    pub errors: usize,
    pub failed_tool_calls: Vec<String>,
    pub user_corrections: Vec<String>,
    pub research_queries: Vec<String>,
}

pub fn parse_transcript_signals(transcript_path: &Path) -> anyhow::Result<TranscriptSummary> {
    if !transcript_path.exists() || !transcript_path.is_file() {
        return Err(anyhow::anyhow!(
            "Transcript file not found: {}",
            transcript_path.display()
        ));
    }

    let content = fs::read_to_string(transcript_path)
        .map_err(|e| anyhow::anyhow!("Failed to read file: {e}"))?;

    let mut total_steps = 0;
    let mut user_inputs = 0;
    let mut tool_calls = 0;
    let mut errors = 0;
    let mut failed_tool_calls = Vec::new();
    let mut user_corrections = Vec::new();
    let mut research_queries = Vec::new();

    for line in content.lines() {
        let line_str = line.trim();
        if line_str.is_empty() {
            continue;
        }

        total_steps += 1;
        if let Ok(v) = serde_json::from_str::<serde_json::Value>(line_str) {
            if let Some(t) = v.get("type").and_then(|s| s.as_str()) {
                if t == "USER_INPUT" {
                    user_inputs += 1;
                    if let Some(c) = v.get("content").and_then(|s| s.as_str()) {
                        if c.contains("No,") || c.contains("instead of") || c.contains("should") {
                            user_corrections.push(c.to_string());
                        }
                    }
                }
            }
            if let Some(status) = v.get("status").and_then(|s| s.as_str()) {
                if status == "ERROR" {
                    errors += 1;
                    if let Some(tn) = v.get("tool_name").and_then(|s| s.as_str()) {
                        failed_tool_calls.push(tn.to_string());
                    }
                }
            }
            if v.get("tool_calls").is_some() || v.get("tool_name").is_some() {
                tool_calls += 1;
                if let Some(tn) = v.get("tool_name").and_then(|s| s.as_str()) {
                    if tn == "web_search" {
                        if let Some(q) = v.get("query").and_then(|s| s.as_str()) {
                            research_queries.push(q.to_string());
                        }
                    }
                }
            }
        }
    }

    Ok(TranscriptSummary {
        total_steps,
        user_inputs,
        tool_calls,
        errors,
        failed_tool_calls,
        user_corrections,
        research_queries,
    })
}

pub fn check_self_progress_health(skill_dir: &Path) -> anyhow::Result<Vec<String>> {
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
            "Self Progress health check failed. Missing files: {:?}",
            missing
        ))
    }
}

pub fn run_self_progress_command(
    subcommand: &SelfProgressSubcommand,
    repo_root: &Path,
) -> anyhow::Result<()> {
    match subcommand {
        SelfProgressSubcommand::Check(args) => {
            let skill_dir = if let Some(p) = &args.path {
                sanitize_path(p, Some(repo_root))?
            } else {
                repo_root.join("skills").join("self-progress")
            };

            match check_self_progress_health(&skill_dir) {
                Ok(_) => {
                    println!("Self Progress skill health check passed cleanly.");
                    Ok(())
                }
                Err(err) => Err(err),
            }
        }
        SelfProgressSubcommand::Analyze(args) => {
            let safe_transcript = sanitize_path(&args.transcript, Some(repo_root))?;
            let summary = parse_transcript_signals(&safe_transcript)?;

            if args.json {
                println!("{}", serde_json::to_string_pretty(&summary).unwrap());
            } else {
                println!(
                    "Self Progress Retrospective Analysis for {}",
                    summary.total_steps
                );
                println!("  User Inputs: {}", summary.user_inputs);
                println!("  Tool Calls: {}", summary.tool_calls);
                println!("  Errors Encountered: {}", summary.errors);
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
    fn test_parse_transcript_signals_strict_assertions() {
        let dir = tempdir().unwrap();
        let base = dir.path().canonicalize().unwrap();
        let sample = base.join("transcript.jsonl");
        let content = "{\"type\":\"USER_INPUT\",\"content\":\"fix bug\"}\n{\"type\":\"PLANNER_RESPONSE\",\"status\":\"ERROR\"}\n";
        fs::write(&sample, content).unwrap();

        let summary = parse_transcript_signals(&sample).unwrap();
        assert_eq!(summary.total_steps, 2);
        assert_eq!(summary.user_inputs, 1);
        assert_eq!(summary.errors, 1);
    }

    #[test]
    fn test_check_self_progress_health_strict_assertions() {
        let dir = tempdir().unwrap();
        let base = dir.path().canonicalize().unwrap();
        let references = base.join("references");
        fs::create_dir_all(&references).unwrap();

        fs::write(base.join("SKILL.md"), "---").unwrap();
        fs::write(base.join("README.md"), "# Title").unwrap();
        fs::write(references.join("overview.md"), "# Overview").unwrap();

        let res = check_self_progress_health(&base);
        assert!(res.is_ok());
    }

    #[test]
    fn test_extract_failed_tool_calls() {
        let dir = tempdir().unwrap();
        let sample = dir.path().join("t.jsonl");
        fs::write(
            &sample,
            r#"{"type":"TOOL_CALL","tool_name":"bash","status":"ERROR","error":"command not found"}
{"type":"TOOL_CALL","tool_name":"grep","status":"OK"}
"#,
        )
        .unwrap();

        let summary = parse_transcript_signals(&sample).unwrap();
        assert!(
            summary.failed_tool_calls.contains(&"bash".to_string()),
            "failed_tool_calls must contain 'bash', got: {:?}",
            summary.failed_tool_calls
        );
    }

    #[test]
    fn test_extract_user_corrections() {
        let dir = tempdir().unwrap();
        let sample = dir.path().join("t.jsonl");
        fs::write(
            &sample,
            r#"{"type":"USER_INPUT","content":"No, you should use grep instead of find"}
"#,
        )
        .unwrap();

        let summary = parse_transcript_signals(&sample).unwrap();
        assert!(
            summary
                .user_corrections
                .iter()
                .any(|c| c.contains("No, you should")),
            "user_corrections must capture correction, got: {:?}",
            summary.user_corrections
        );
    }

    #[test]
    fn test_extract_research_patterns() {
        let dir = tempdir().unwrap();
        let sample = dir.path().join("t.jsonl");
        fs::write(
            &sample,
            r#"{"type":"TOOL_CALL","tool_name":"web_search","query":"how to parse YAML in Rust"}
"#,
        )
        .unwrap();

        let summary = parse_transcript_signals(&sample).unwrap();
        assert!(
            summary
                .research_queries
                .contains(&"how to parse YAML in Rust".to_string()),
            "research_queries must contain query, got: {:?}",
            summary.research_queries
        );
    }
}
