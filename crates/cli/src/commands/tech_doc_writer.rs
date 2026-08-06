use agent_skills_core::path_safety::sanitize_path;
use clap::{Args, Subcommand};
use regex::Regex;
use std::collections::BTreeSet;
use std::fs;
use std::path::{Path, PathBuf};

#[derive(Args, Debug, Clone)]
pub struct AuditArgs {
    /// Markdown file(s) or directory paths to audit
    #[arg(long, short, num_args = 1..)]
    pub path: Vec<String>,

    /// Preview audit without returning non-zero exit code
    #[arg(long)]
    pub dry_run: bool,
}

#[derive(Subcommand, Debug, Clone)]
pub enum TechDocWriterSubcommand {
    /// Audit Markdown documents against GFM conventions and quality directives
    Audit(AuditArgs),
}

pub fn audit_markdown_file(file_path: &Path) -> (bool, Vec<String>) {
    if !file_path.exists() {
        return (
            false,
            vec![format!("File not found: {}", file_path.display())],
        );
    }

    let content = match fs::read_to_string(file_path) {
        Ok(c) => c,
        Err(e) => {
            return (
                false,
                vec![format!("Failed to read file {}: {e}", file_path.display())],
            )
        }
    };

    let lines: Vec<&str> = content.lines().collect();
    let mut findings = Vec::new();
    let mut is_valid = true;

    let mut in_code_block = false;
    let mut code_block_fence_len = 0;
    let mut code_block_lang = String::new();

    let mut h1_matches: Vec<(usize, String)> = Vec::new();
    let ascii_art_patterns = [
        Regex::new(r"\+---\+").unwrap(),
        Regex::new(r"\|   \|").unwrap(),
        Regex::new(r"/\s*\\\s*/").unwrap(),
        Regex::new(r"\[\s*\]--->").unwrap(),
        Regex::new(r"<===>").unwrap(),
    ];

    let privacy_pattern = Regex::new(r"(/home/[a-zA-Z0-9_-]+|C:\\Users\\[a-zA-Z0-9_-]+)").unwrap();
    let alert_pattern = Regex::new(r"^>\s*\[!(.*?)\]").unwrap();

    let mut valid_alerts = BTreeSet::new();
    valid_alerts.insert("NOTE");
    valid_alerts.insert("TIP");
    valid_alerts.insert("IMPORTANT");
    valid_alerts.insert("WARNING");
    valid_alerts.insert("CAUTION");

    let fence_regex = Regex::new(r"^(~{3,}|`{3,})(.*)$").unwrap();

    for (idx, line) in lines.iter().enumerate().map(|(i, l)| (i + 1, l)) {
        let stripped = line.trim();

        if let Some(caps) = fence_regex.captures(stripped) {
            let fence_chars = caps.get(1).unwrap().as_str();
            let fence_len = fence_chars.len();
            if !in_code_block {
                in_code_block = true;
                code_block_fence_len = fence_len;
                code_block_lang = caps
                    .get(2)
                    .map(|m| m.as_str().trim().to_lowercase())
                    .unwrap_or_default();
            } else if in_code_block && fence_len >= code_block_fence_len {
                in_code_block = false;
                code_block_fence_len = 0;
                code_block_lang.clear();
            }
            continue;
        }

        // H1 Title Check (outside code blocks)
        if !in_code_block && stripped.starts_with("# ") && !stripped.starts_with("##") {
            h1_matches.push((idx, line.to_string()));
        }

        // ASCII Art Check inside non-diagram code blocks
        if in_code_block
            && !["mermaid", "json", "yaml", "toml", "bash", "sh"]
                .contains(&code_block_lang.as_str())
        {
            for pat in &ascii_art_patterns {
                if pat.is_match(line) {
                    findings.push(format!(
                        "WARNING: Line {idx} appears to contain ASCII art. Use 'mermaid' diagrams instead."
                    ));
                    break;
                }
            }
        }

        // Privacy Check for hardcoded absolute paths
        if let Some(caps) = privacy_pattern.captures(line) {
            let matched_path = caps.get(0).unwrap().as_str();
            let lower_line = line.to_lowercase();
            if !lower_line.contains("username")
                && !lower_line.contains("user")
                && !lower_line.contains("path/to")
            {
                is_valid = false;
                findings.push(format!(
                    "ERROR: Line {idx} contains hardcoded personal user path ('{matched_path}'). Use relative or generic paths."
                ));
            }
        }

        // GFM Alert Callout Check (outside code blocks)
        if !in_code_block {
            if let Some(caps) = alert_pattern.captures(stripped) {
                let tag = caps.get(1).unwrap().as_str().to_uppercase();
                if !valid_alerts.contains(tag.as_str()) {
                    is_valid = false;
                    let valid_str: Vec<&str> = valid_alerts.iter().copied().collect();
                    findings.push(format!(
                        "ERROR: Line {idx} has invalid GFM alert tag '[!{tag}]'. Must be one of {}.",
                        valid_str.join(", ")
                    ));
                }
            }
        }
    }

    if h1_matches.is_empty() {
        findings.push("WARNING: No H1 title ('# Title') found in document.".to_string());
    } else if h1_matches.len() > 1 {
        is_valid = false;
        let line_numbers: Vec<String> = h1_matches.iter().map(|(idx, _)| idx.to_string()).collect();
        findings.push(format!(
            "ERROR: Multiple H1 titles found on lines [{}]. GFM requires exactly one H1 per document.",
            line_numbers.join(", ")
        ));
    }

    (is_valid, findings)
}

pub fn run_tech_doc_writer_audit(args: &AuditArgs, repo_root: &Path) -> Result<(), String> {
    let mut overall_success = true;
    let mut target_files: Vec<PathBuf> = Vec::new();

    for raw_path in &args.path {
        let sanitized = sanitize_path(raw_path, Some(repo_root))?;
        if sanitized.is_file() {
            target_files.push(sanitized);
        } else if sanitized.is_dir() {
            let walk = |dir: &Path| -> Vec<PathBuf> {
                let mut files = Vec::new();
                if let Ok(entries) = fs::read_dir(dir) {
                    for entry in entries.flatten() {
                        let p = entry.path();
                        if p.is_file() && p.extension().and_then(|e| e.to_str()) == Some("md") {
                            files.push(p);
                        }
                    }
                }
                files
            };
            target_files.extend(walk(&sanitized));
        }
    }

    if target_files.is_empty() {
        println!("No Markdown files found to audit.");
        return Ok(());
    }

    for file in &target_files {
        println!("Auditing: {}", file.display());
        let (passed, findings) = audit_markdown_file(file);

        if findings.is_empty() {
            println!("  ✓ All checks passed cleanly.");
        } else {
            for finding in &findings {
                println!("  - {finding}");
            }
        }

        if !passed && !args.dry_run {
            overall_success = false;
        }
    }

    if overall_success {
        Ok(())
    } else {
        Err("Technical documentation audit failed.".to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn test_audit_markdown_file_valid_doc_strict_assertions() {
        let dir = tempdir().unwrap();
        let md_file = dir.path().join("README.md");
        fs::write(
            &md_file,
            "# Valid Title\n\n> [!NOTE]\n> A valid note alert box.\n\n## Section\n",
        )
        .unwrap();

        let (passed, findings) = audit_markdown_file(&md_file);
        assert!(passed);
        assert_eq!(findings, Vec::<String>::new());
    }

    #[test]
    fn test_audit_markdown_file_invalid_multiple_h1_and_alert_strict_assertions() {
        let dir = tempdir().unwrap();
        let md_file = dir.path().join("BAD.md");
        fs::write(&md_file, "# Title 1\n\n# Title 2\n\n> [!INVALID_TAG]\n").unwrap();

        let (passed, findings) = audit_markdown_file(&md_file);
        assert!(!passed);
        assert_eq!(findings.len(), 2);
        assert!(findings[0].contains("invalid GFM alert tag"));
        assert!(findings[1].contains("Multiple H1 titles found"));
    }
}
