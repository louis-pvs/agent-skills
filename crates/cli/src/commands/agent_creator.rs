use agent_skills_core::path_safety::sanitize_path;
use clap::{Args, Subcommand};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};

#[derive(Args, Debug, Clone)]
pub struct ScaffoldArgs {
    /// Name of the custom agent (e.g. 'qa-engineer')
    #[arg(long)]
    pub name: Option<String>,

    /// Description of the agent's role and specialization
    #[arg(long)]
    pub description: Option<String>,

    /// Target model tier (e.g. 'gemini-3.7-flash', 'gemini-3.7-flash-thinking')
    #[arg(long, default_value = "gemini-3.7-flash-thinking")]
    pub model: String,

    /// Enable agent as primary session (mainAgent: true)
    #[arg(long, default_value_t = true)]
    pub main_agent: bool,

    /// Enable agent as background subagent (subagent: true)
    #[arg(long, default_value_t = true)]
    pub subagent: bool,

    /// Comma-separated list of tools to grant to the agent
    #[arg(long)]
    pub tools: Option<String>,

    /// Comma-separated list of skills to pre-attach to the agent
    #[arg(long)]
    pub skills: Option<String>,

    /// Target directory to scaffold the agent definition
    #[arg(long, default_value = ".agents/agents")]
    pub target_dir: String,

    /// Preview scaffolding without writing files to disk
    #[arg(long)]
    pub dry_run: bool,
}

#[derive(Args, Debug, Clone)]
pub struct ValidateArgs {
    /// Path to an agent .md file or directory containing agent definitions
    #[arg(long, short)]
    pub path: String,

    /// Output validation results in JSON format
    #[arg(long)]
    pub json: bool,

    /// Preview validation action without modifying files
    #[arg(long)]
    pub dry_run: bool,
}

#[derive(Subcommand, Debug, Clone)]
pub enum AgentCreatorSubcommand {
    /// Scaffold a new Antigravity Custom Agent markdown file
    Scaffold(ScaffoldArgs),
    /// Validate an existing custom agent markdown file or directory
    Validate(ValidateArgs),
}

const MAX_NAME_LEN: usize = 64;
const MAX_DESC_LEN: usize = 1024;

#[derive(Serialize, Deserialize, Debug)]
pub struct AgentValidationResult {
    pub file: String,
    pub valid: bool,
    pub issues: Vec<String>,
    pub warnings: Vec<String>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct AgentValidationSummary {
    pub valid: bool,
    pub results: Vec<AgentValidationResult>,
}

pub fn validate_agent_metadata(name: &str, description: &str) -> Vec<String> {
    let mut errors = Vec::new();

    if name.is_empty() {
        errors.push("Agent name cannot be empty.".to_string());
    } else if name.len() > MAX_NAME_LEN {
        errors.push(format!(
            "Agent name exceeds maximum length of {MAX_NAME_LEN} characters ({} chars).",
            name.len()
        ));
    } else {
        let is_valid_name = name
            .chars()
            .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '-')
            && !name.starts_with('-')
            && !name.ends_with('-')
            && !name.contains("--");
        if !is_valid_name {
            errors.push(
                "Agent name must contain only lowercase alphanumeric characters and single hyphens (e.g. 'qa-engineer'). Cannot start or end with a hyphen.".to_string()
            );
        }
    }

    if description.is_empty() {
        errors.push("Agent description cannot be empty.".to_string());
    } else if description.len() < 10 {
        errors.push("Agent description is too short (must be at least 10 characters).".to_string());
    } else if description.len() > MAX_DESC_LEN {
        errors.push(format!(
            "Agent description exceeds maximum length of {MAX_DESC_LEN} characters ({} chars).",
            description.len()
        ));
    }

    errors
}

pub fn validate_agent_content(path: &Path, content: &str) -> AgentValidationResult {
    let mut issues = Vec::new();
    let mut warnings = Vec::new();

    let normalized = content.replace("\r\n", "\n");
    let trimmed = normalized.trim_start();

    if !trimmed.starts_with("---") {
        issues.push("Missing opening frontmatter delimiter '---'.".to_string());
        return AgentValidationResult {
            file: path.display().to_string(),
            valid: false,
            issues,
            warnings,
        };
    }

    let rest = &trimmed[3..];
    let closing_pos = rest.find("\n---");
    if closing_pos.is_none() {
        issues.push("Missing closing frontmatter delimiter '---'.".to_string());
        return AgentValidationResult {
            file: path.display().to_string(),
            valid: false,
            issues,
            warnings,
        };
    }

    let frontmatter_str = &rest[..closing_pos.unwrap()];
    let body_str = &rest[closing_pos.unwrap() + 4..].trim();

    let mut name = String::new();
    let mut desc = String::new();

    for line in frontmatter_str.lines() {
        let line_trimmed = line.trim();
        if let Some(stripped) = line_trimmed.strip_prefix("name:") {
            name = stripped
                .trim()
                .trim_matches('"')
                .trim_matches('\'')
                .to_string();
        } else if let Some(stripped) = line_trimmed.strip_prefix("description:") {
            desc = stripped
                .trim()
                .trim_matches('"')
                .trim_matches('\'')
                .to_string();
        }
    }

    let meta_issues = validate_agent_metadata(&name, &desc);
    issues.extend(meta_issues);

    if body_str.is_empty() {
        issues.push("Agent system prompt body is empty.".to_string());
    } else {
        if !body_str.contains("## Completion Criteria") {
            warnings.push(
                "Missing recommended '## Completion Criteria' section in prompt body.".to_string(),
            );
        }
    }

    let valid = issues.is_empty();
    AgentValidationResult {
        file: path.display().to_string(),
        valid,
        issues,
        warnings,
    }
}

pub fn scaffold_agent(args: &ScaffoldArgs, repo_root: Option<&Path>) -> anyhow::Result<PathBuf> {
    let name = args
        .name
        .clone()
        .unwrap_or_else(|| "custom-agent".to_string());
    let description = args
        .description
        .clone()
        .unwrap_or_else(|| format!("Specialized custom agent for {name} tasks."));

    let meta_issues = validate_agent_metadata(&name, &description);
    if !meta_issues.is_empty() {
        anyhow::bail!(
            "Agent metadata validation failed:\n{}",
            meta_issues
                .iter()
                .map(|e| format!("  - {e}"))
                .collect::<Vec<_>>()
                .join("\n")
        );
    }

    let target_dir_path = if Path::new(&args.target_dir).is_absolute() {
        PathBuf::from(&args.target_dir)
    } else {
        sanitize_path(&args.target_dir, repo_root)?
    };

    let target_file_path = target_dir_path.join(format!("{name}.md"));

    let tools_yaml = if let Some(ref t) = args.tools {
        let tools_list = t
            .split(',')
            .map(|s| format!("  - {}", s.trim()))
            .collect::<Vec<_>>()
            .join("\n");
        format!("tools:\n{tools_list}\n")
    } else {
        "tools:\n  - view_file\n  - grep_search\n  - ask_question\n".to_string()
    };

    let skills_yaml = if let Some(ref s) = args.skills {
        let skills_list = s
            .split(',')
            .map(|item| format!("  - {}", item.trim()))
            .collect::<Vec<_>>()
            .join("\n");
        format!("skills:\n{skills_list}\n")
    } else {
        "".to_string()
    };

    let title = name
        .split('-')
        .map(|w| {
            let mut c = w.chars();
            match c.next() {
                None => String::new(),
                Some(f) => f.to_uppercase().collect::<String>() + c.as_str(),
            }
        })
        .collect::<Vec<_>>()
        .join(" ");

    let content = format!(
        r#"---
name: {name}
description: "{description}"
model: {model}
mainAgent: {main_agent}
subagent: {subagent}
{tools_yaml}{skills_yaml}---

# Role: {title}

You are a specialized custom agent configured for {name} workflows.

## Operational Workflow

1. **Phase 1: Input & Clarification**:
   - Inspect context and parameters.
   - If requirements or design choices are ambiguous, call the `ask_question` tool with `(Recommended)` first-choice options.
2. **Phase 2: Execution**:
   - Perform role-specific actions strictly abiding by the Principle of Least Privilege.
3. **Phase 3: Synthesis & Verification**:
   - Verify outcomes and report results cleanly.

## Interactive Decision-Making Protocol

Whenever you encounter ambiguous requirements, architectural forks, or configuration choices:
- Call the `ask_question` tool instead of asking open-ended text questions.
- Format options with the `(Recommended)` prefix on the top best-practice option.
- Phrase options from the user's perspective as direct actions.
- Never include redundant "Other" options (provided by the UI).
- Resolve decisions one step at a time down the decision tree.

## Rules & Constraints

- Follow least-privilege tool usage.
- Do not make out-of-scope edits or destructive modifications without confirmation.

## Completion Criteria

- [ ] Interactive choices (if any) resolved using the `ask_question` tool protocol.
- [ ] Task objectives completed cleanly without errors.
- [ ] Output complies with required schema and project standards.
"#,
        model = args.model,
        main_agent = args.main_agent,
        subagent = args.subagent,
    );

    if args.dry_run {
        println!(
            "[DRY RUN] Would create agent file at: {}",
            target_file_path.display()
        );
        return Ok(target_file_path);
    }

    fs::create_dir_all(&target_dir_path)?;
    fs::write(&target_file_path, content)?;

    Ok(target_file_path)
}

pub fn validate_agent(args: &ValidateArgs, repo_root: Option<&Path>) -> (bool, Vec<String>) {
    let target_path = if Path::new(&args.path).is_absolute() {
        PathBuf::from(&args.path)
    } else {
        match sanitize_path(&args.path, repo_root) {
            Ok(p) => p,
            Err(e) => return (false, vec![format!("Invalid path: {e}")]),
        }
    };

    let mut files = Vec::new();
    if target_path.is_file() {
        files.push(target_path.clone());
    } else if target_path.is_dir() {
        if let Ok(entries) = fs::read_dir(&target_path) {
            for entry in entries.flatten() {
                let p = entry.path();
                if p.is_file() && p.extension().and_then(|ext| ext.to_str()) == Some("md") {
                    files.push(p);
                }
            }
        }
    } else {
        return (
            false,
            vec![format!("Path does not exist: {}", target_path.display())],
        );
    }

    files.sort();

    if files.is_empty() {
        return (
            false,
            vec![format!(
                "No markdown agent files found in: {}",
                target_path.display()
            )],
        );
    }

    let mut results = Vec::new();
    let mut all_issues = Vec::new();
    let mut all_valid = true;

    for file in &files {
        match fs::read_to_string(file) {
            Ok(content) => {
                let res = validate_agent_content(file, &content);
                if !res.valid {
                    all_valid = false;
                    for issue in &res.issues {
                        all_issues.push(format!("{}: {}", file.display(), issue));
                    }
                }
                results.push(res);
            }
            Err(err) => {
                all_valid = false;
                let issue = format!("{}: Failed to read file: {err}", file.display());
                all_issues.push(issue.clone());
                results.push(AgentValidationResult {
                    file: file.display().to_string(),
                    valid: false,
                    issues: vec![issue],
                    warnings: Vec::new(),
                });
            }
        }
    }

    if args.json {
        let summary = AgentValidationSummary {
            valid: all_valid,
            results,
        };
        if let Ok(json_str) = serde_json::to_string_pretty(&summary) {
            println!("{json_str}");
        }
    }

    (all_valid, all_issues)
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn test_validate_agent_metadata_valid() {
        let errors = validate_agent_metadata("qa-engineer", "A valid description for QA engineer.");
        assert!(errors.is_empty());
    }

    #[test]
    fn test_validate_agent_metadata_invalid_name() {
        let errors = validate_agent_metadata("INVALID_NAME!", "Valid description here.");
        assert!(!errors.is_empty());
        assert!(errors[0].contains("lowercase alphanumeric"));
    }

    #[test]
    fn test_validate_agent_content_valid() {
        let content = r#"---
name: test-agent
description: "A valid test agent specialized in verification."
model: gemini-3.7-flash
mainAgent: true
subagent: true
tools:
  - view_file
---

# Role: Test Agent

System prompt content.

## Completion Criteria

- [ ] Task completed.
"#;
        let res = validate_agent_content(Path::new("test-agent.md"), content);
        assert!(res.valid);
        assert!(res.issues.is_empty());
    }

    #[test]
    fn test_validate_agent_content_missing_frontmatter() {
        let content = "# Role: No Frontmatter\n\nBody only.";
        let res = validate_agent_content(Path::new("test-agent.md"), content);
        assert!(!res.valid);
        assert!(res
            .issues
            .iter()
            .any(|i| i.contains("Missing opening frontmatter")));
    }

    #[test]
    fn test_scaffold_and_validate_agent_strict_assertions() {
        let dir = tempdir().unwrap();
        let target_dir = dir.path().join(".agents/agents");

        let args = ScaffoldArgs {
            name: Some("qa-tdd-engineer".to_string()),
            description: Some(
                "Specialized in Test-Driven Development and unit test suites.".to_string(),
            ),
            model: "gemini-3.7-flash-thinking".to_string(),
            main_agent: true,
            subagent: true,
            tools: Some("view_file,write_to_file,run_command,ask_question".to_string()),
            skills: Some("tdd,self-annealer".to_string()),
            target_dir: target_dir.to_string_lossy().to_string(),
            dry_run: false,
        };

        let result_path = scaffold_agent(&args, Some(dir.path())).unwrap();
        assert!(result_path.exists());

        let val_args = ValidateArgs {
            path: result_path.to_string_lossy().to_string(),
            json: false,
            dry_run: false,
        };

        let (is_valid, issues) = validate_agent(&val_args, Some(dir.path()));
        assert!(is_valid, "Validation failed with issues: {:?}", issues);
        assert!(issues.is_empty());
    }
}
