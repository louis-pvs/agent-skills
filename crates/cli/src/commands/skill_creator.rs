use agent_skills_core::path_safety::sanitize_path;
use clap::{Args, Subcommand, ValueEnum};
use std::fs;
use std::path::{Path, PathBuf};

#[derive(ValueEnum, Clone, Debug, PartialEq, Eq)]
pub enum SkillType {
    Simple,
    Complex,
    Router,
}

#[derive(Args, Debug, Clone)]
pub struct ScaffoldArgs {
    /// Name of the skill to scaffold (e.g. 'my-skill')
    #[arg(long)]
    pub name: Option<String>,

    /// Description of when to trigger the skill
    #[arg(long)]
    pub description: Option<String>,

    /// Target domain category for dynamic capability mapping
    #[arg(long)]
    pub domain: Option<String>,

    /// Comma-separated list of tags for taxonomy classification
    #[arg(long)]
    pub tags: Option<String>,

    /// Directory to scaffold skill inside
    #[arg(long, default_value = "skills")]
    pub target_dir: String,

    /// Type of skill template to scaffold
    #[arg(long, value_enum, default_value = "simple")]
    pub r#type: SkillType,

    /// Mark skill as user-invoked (disable-model-invocation: true)
    #[arg(long)]
    pub user_invoked: bool,

    /// Alias for --type complex
    #[arg(long)]
    pub complex: bool,

    /// Preview scaffolding without writing files to disk
    #[arg(long)]
    pub dry_run: bool,
}

#[derive(Args, Debug, Clone)]
pub struct ValidateArgs {
    /// Validate an existing skill directory path
    #[arg(long, short)]
    pub path: String,

    /// Preview validation action without modifying files
    #[arg(long)]
    pub dry_run: bool,
}

#[derive(Subcommand, Debug, Clone)]
pub enum SkillCreatorSubcommand {
    /// Scaffold a new Agent Skill according to agentskills.io standard
    Scaffold(ScaffoldArgs),
    /// Validate an existing Agent Skill directory against specification and design rules
    Validate(ValidateArgs),
}

const MAX_NAME_LEN: usize = 64;
const MAX_DESC_LEN: usize = 1024;

pub fn validate_skill_metadata(name: &str, description: &str) -> Vec<String> {
    let mut errors = Vec::new();

    if name.is_empty() {
        errors.push("Skill name cannot be empty.".to_string());
    } else if name.len() > MAX_NAME_LEN {
        errors.push(format!(
            "Skill name exceeds maximum length of {MAX_NAME_LEN} characters ({} chars).",
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
                "Skill name must contain only lowercase alphanumeric characters and single hyphens (e.g. 'my-new-skill'). Cannot start or end with a hyphen.".to_string()
            );
        }
    }

    if description.is_empty() {
        errors.push("Skill description cannot be empty.".to_string());
    } else if description.len() > MAX_DESC_LEN {
        errors.push(format!(
            "Skill description exceeds maximum length of {MAX_DESC_LEN} characters ({} chars).",
            description.len()
        ));
    }

    errors
}

pub fn scaffold_skill(args: &ScaffoldArgs, base_dir: Option<&Path>) -> anyhow::Result<PathBuf> {
    let name = args.name.as_deref().ok_or_else(|| {
        anyhow::anyhow!(
            "Error: Both --name and --description are required when scaffolding a skill."
        )
    })?;
    let description = args.description.as_deref().ok_or_else(|| {
        anyhow::anyhow!(
            "Error: Both --name and --description are required when scaffolding a skill."
        )
    })?;

    let meta_errors = validate_skill_metadata(name, description);
    if !meta_errors.is_empty() {
        return Err(anyhow::anyhow!(
            "Invalid skill metadata:\n- {}",
            meta_errors.join("\n- ")
        ));
    }

    let skill_type = if args.complex {
        SkillType::Complex
    } else {
        args.r#type.clone()
    };

    let target_base = sanitize_path(&args.target_dir, base_dir)?;
    let skill_path = target_base.join(name);

    if skill_path.exists() {
        return Err(anyhow::anyhow!(
            "Skill directory already exists: {}",
            skill_path.display()
        ));
    }

    if args.dry_run {
        println!(
            "[DRY-RUN] Would scaffold skill '{name}' ({skill_type:?}) at: {}",
            skill_path.display()
        );
        return Ok(skill_path);
    }

    fs::create_dir_all(&skill_path)
        .map_err(|e| anyhow::anyhow!("Failed to create directory: {e}"))?;

    let title_words: Vec<String> = name
        .split('-')
        .map(|w| {
            let mut c = w.chars();
            match c.next() {
                None => String::new(),
                Some(f) => f.to_uppercase().collect::<String>() + c.as_str(),
            }
        })
        .collect();
    let skill_title = title_words.join(" ");

    let desc_str = description.trim();
    let formatted_desc = if desc_str.contains(':')
        || desc_str.contains('\'')
        || desc_str.contains('"')
        || desc_str.contains('#')
    {
        let escaped = desc_str.replace('\'', "''");
        format!("'{escaped}'")
    } else {
        desc_str.to_string()
    };

    let mut frontmatter_lines = vec![
        "---".to_string(),
        format!("name: {name}"),
        format!("description: {formatted_desc}"),
    ];

    if let Some(dom) = &args.domain {
        let dom_trimmed = dom.trim();
        let formatted_dom =
            if dom_trimmed.contains(':') || dom_trimmed.contains('\'') || dom_trimmed.contains('"')
            {
                format!("'{dom_trimmed}'")
            } else {
                dom_trimmed.to_string()
            };
        frontmatter_lines.push(format!("domain: {formatted_dom}"));
    }

    if let Some(tags_raw) = &args.tags {
        let clean_tags: Vec<&str> = tags_raw
            .split(',')
            .map(|t| t.trim())
            .filter(|t| !t.is_empty())
            .collect();
        if !clean_tags.is_empty() {
            frontmatter_lines.push(format!("tags: [{}]", clean_tags.join(", ")));
        }
    }

    if args.user_invoked {
        frontmatter_lines.push("disable-model-invocation: true".to_string());
    }

    frontmatter_lines.push("---".to_string());
    let frontmatter = frontmatter_lines.join("\n");

    let skill_md_content = match skill_type {
        SkillType::Router => format!(
            "{frontmatter}\n\n# {skill_title}\n\nRouter skill for dispatching task requests to specialized sub-skills or workflows.\n\n## Sub-Skill Directory & Routing Map\n\n| Intent / Task | Skill / Resource | Description |\n| :--- | :--- | :--- |\n| Task Category A | `sub-skill-a` | Description of when to use sub-skill A. |\n| Task Category B | `sub-skill-b` | Description of when to use sub-skill B. |\n| Task Category C | [reference-doc.md](references/reference-doc.md) | Reference documentation. |\n\n## Guidance for Agent\n\n1. Inspect the user request against the routing map above.\n2. Direct the workflow to the appropriate sub-skill or reference file.\n3. If no matching sub-skill applies, request clarification from the user.\n\n## Completion Criteria\n\n- [ ] Matched user request to target sub-skill or reference document cleanly.\n- [ ] Direct routing advice provided to user.\n"
        ),
        SkillType::Complex => format!(
            "{frontmatter}\n\n# {skill_title}\n\nOverview of {skill_title}.\n\n## Workflow\n\n1. **Initialization**: Read configuration or set up environment.\n2. **Execution**: Run automation CLI:\n\n   ```bash\n   cargo run -p agent-skills -- {name} --check\n   ```\n\n3. **Synthesis**: Process output and report results to user.\n\n## Completion Criteria\n\n- [ ] Automation CLI executes cleanly with exit code 0.\n- [ ] Unit tests pass without error.\n- [ ] Output complies with required schema.\n\n## References\n\n- [overview.md](references/overview.md) — Extended reference documentation.\n"
        ),
        SkillType::Simple => format!(
            "{frontmatter}\n\n# {skill_title}\n\nOverview of {skill_title}.\n\n## Guidelines & Workflow\n\n1. Step 1: Initial workflow step.\n2. Step 2: Main task execution.\n3. Step 3: Verification and output.\n\n## Rules & Constraints\n\n- Rule 1: Key guideline to follow.\n\n## Completion Criteria\n\n- [ ] Task execution completed without errors or unresolved exceptions.\n- [ ] Output produced matches requested structure and parameters.\n"
        ),
    };

    fs::write(skill_path.join("SKILL.md"), skill_md_content)
        .map_err(|e| anyhow::anyhow!("Failed to write SKILL.md: {e}"))?;

    let mut readme_content = format!(
        "# {skill_title}\n\n{desc_str}\n\n---\n\n## Documentation Entry Points\n\n- **AI Agent Protocol**: See [SKILL.md](SKILL.md) (used automatically by Antigravity, Claude Code, and Copilot).\n"
    );
    if skill_type == SkillType::Complex {
        readme_content.push_str("- **Architecture & References**: See [references/overview.md](references/overview.md).\n");
    }

    fs::write(skill_path.join("README.md"), readme_content)
        .map_err(|e| anyhow::anyhow!("Failed to write README.md: {e}"))?;

    if skill_type == SkillType::Complex {
        let ref_dir = skill_path.join("references");
        let tmpl_dir = skill_path.join("templates");
        let ex_dir = skill_path.join("examples");
        let scripts_dir = skill_path.join("scripts");
        let tests_dir = scripts_dir.join("tests");

        fs::create_dir_all(&ref_dir)
            .map_err(|e| anyhow::anyhow!("Failed to create references dir: {e}"))?;
        fs::create_dir_all(&tmpl_dir)
            .map_err(|e| anyhow::anyhow!("Failed to create templates dir: {e}"))?;
        fs::create_dir_all(&ex_dir)
            .map_err(|e| anyhow::anyhow!("Failed to create examples dir: {e}"))?;
        fs::create_dir_all(&tests_dir)
            .map_err(|e| anyhow::anyhow!("Failed to create tests dir: {e}"))?;

        fs::write(
            ref_dir.join("overview.md"),
            format!("# {skill_title} Overview\n\nExtended reference documentation.\n"),
        )
        .map_err(|e| anyhow::anyhow!("Failed to write overview.md: {e}"))?;

        fs::write(
            scripts_dir.join("main.py"),
            format!(
                "#!/usr/bin/env python3\n\"\"\"{skill_title} main script.\"\"\"\nimport sys\n\ndef main() -> int:\n    print(\"{skill_title} script running\")\n    return 0\n\nif __name__ == '__main__':\n    sys.exit(main())\n"
            ),
        )
        .map_err(|e| anyhow::anyhow!("Failed to write main.py: {e}"))?;
    }

    Ok(skill_path)
}

#[allow(dead_code)]
pub fn parse_skill_frontmatter_fields(
    skill_md_path: &Path,
) -> anyhow::Result<(Option<String>, Vec<String>)> {
    let content = fs::read_to_string(skill_md_path)?;
    if !content.starts_with("---") {
        return Ok((None, Vec::new()));
    }
    let parts: Vec<&str> = content.splitn(3, "---").collect();
    if parts.len() < 3 {
        return Ok((None, Vec::new()));
    }
    let yaml_block = parts[1];
    let val: serde_yaml::Value = serde_yaml::from_str(yaml_block)?;
    let domain = val
        .get("domain")
        .and_then(|v| v.as_str())
        .map(|s| s.to_string());
    let mut tags = Vec::new();
    if let Some(t_seq) = val.get("tags").and_then(|v| v.as_sequence()) {
        for t in t_seq {
            if let Some(ts) = t.as_str() {
                tags.push(ts.to_string());
            }
        }
    }
    Ok((domain, tags))
}

pub fn validate_skill(args: &ValidateArgs, base_dir: Option<&Path>) -> (bool, Vec<String>) {
    let mut issues = Vec::new();

    let target_dir = match sanitize_path(&args.path, base_dir) {
        Ok(p) => p,
        Err(e) => return (false, vec![e.to_string()]),
    };

    if !target_dir.is_dir() {
        return (
            false,
            vec![format!(
                "Target path is not a directory: {}",
                target_dir.display()
            )],
        );
    }

    if args.dry_run {
        println!(
            "[DRY-RUN] Would validate skill at: {}",
            target_dir.display()
        );
        return (true, Vec::new());
    }

    let skill_md = target_dir.join("SKILL.md");
    if !skill_md.exists() {
        return (
            false,
            vec![format!(
                "Missing required SKILL.md at: {}",
                skill_md.display()
            )],
        );
    }

    let content = match fs::read_to_string(&skill_md) {
        Ok(c) => c,
        Err(e) => return (false, vec![format!("Failed to read SKILL.md: {e}")]),
    };

    let lines: Vec<&str> = content.lines().collect();

    // Context Load Audit
    if lines.len() > 500 {
        issues.push(format!(
            "Context Load warning: SKILL.md line count ({}) exceeds limit of 500 lines.",
            lines.len()
        ));
    }

    // YAML Frontmatter check
    if !content.starts_with("---") {
        issues.push("SKILL.md must start with YAML frontmatter delimiter '---'.".to_string());
    } else {
        let parts: Vec<&str> = content.splitn(3, "---").collect();
        if parts.len() < 3 {
            issues.push("SKILL.md YAML frontmatter is not closed with '---'.".to_string());
        } else {
            let yaml_block = parts[1];
            if let Err(e) = serde_yaml::from_str::<serde_yaml::Value>(yaml_block) {
                issues.push(format!("Invalid YAML frontmatter syntax: {e}"));
            } else if let Ok(val) = serde_yaml::from_str::<serde_yaml::Value>(yaml_block) {
                if let Some(map) = val.as_mapping() {
                    let name = map
                        .get(serde_yaml::Value::String("name".to_string()))
                        .and_then(|v| v.as_str())
                        .unwrap_or("");
                    let desc = map
                        .get(serde_yaml::Value::String("description".to_string()))
                        .and_then(|v| v.as_str())
                        .unwrap_or("");

                    let meta_errs = validate_skill_metadata(name, desc);
                    issues.extend(meta_errs);

                    if !desc.is_empty() {
                        let lower_desc = desc.to_lowercase();
                        let compounds = [
                            " and also ",
                            " as well as ",
                            " along with ",
                            " and additionally ",
                        ];
                        let has_compound = compounds.iter().any(|c| lower_desc.contains(c));
                        let and_count = lower_desc.matches(" and ").count();
                        if has_compound || and_count >= 3 {
                            issues.push(format!(
                                "Scope Creep warning: Skill description contains compound task connectors ('{desc}')."
                            ));
                        }
                    }

                    if let Some(dir_name) = target_dir.file_name().and_then(|n| n.to_str()) {
                        if !name.is_empty() && name != dir_name {
                            issues.push(format!(
                                "Frontmatter name ('{name}') does not match directory name ('{dir_name}')."
                            ));
                        }
                    }
                }
            }
        }
    }

    // Structure Audit
    let scripts_dir = target_dir.join("scripts");
    let references_dir = target_dir.join("references");
    let is_complex = scripts_dir.is_dir() || references_dir.is_dir();

    if is_complex {
        let overview_md = references_dir.join("overview.md");
        if !overview_md.exists() {
            issues.push(format!(
                "Complex Skill Structure Warning: Missing mandatory 'references/overview.md' at: {}",
                overview_md.display()
            ));
        }

        if scripts_dir.is_dir() {
            let has_orchestrator = scripts_dir.join("main.py").exists()
                || scripts_dir.join("council.py").exists()
                || scripts_dir.join("scaffold_skill.py").exists()
                || scripts_dir.join("tdd_runner.py").exists()
                || scripts_dir.join("anneal_runner.py").exists()
                || scripts_dir.join("doc_auditor.py").exists()
                || scripts_dir.join("adr_cli.py").exists();

            if !has_orchestrator {
                issues.push(format!(
                    "Complex Skill Structure Warning: Missing main CLI orchestrator script in {}",
                    scripts_dir.display()
                ));
            }

            // Non-stdlib import & line count check
            if let Ok(entries) = fs::read_dir(&scripts_dir) {
                for entry in entries.flatten() {
                    let p = entry.path();
                    if p.is_file() {
                        if let Ok(script_content) = fs::read_to_string(&p) {
                            if script_content.lines().count() > 500 {
                                issues.push(format!(
                                    "Context Load warning: Script '{}' line count exceeds 500 lines.",
                                    p.display()
                                ));
                            }
                            if p.extension().and_then(|e| e.to_str()) == Some("py") {
                                for line in script_content.lines() {
                                    let stripped = line.trim();
                                    if (stripped.starts_with("import ")
                                        || stripped.starts_with("from "))
                                        && (stripped.contains("requests")
                                            || stripped.contains("pydantic")
                                            || stripped.contains("pandas")
                                            || stripped.contains("numpy"))
                                    {
                                        issues.push(format!(
                                            "Non-stdlib import detected in '{}': '{}'",
                                            p.display(),
                                            stripped
                                        ));
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }

    // Completion Criteria Audit
    let body = if content.contains("---") {
        content.splitn(3, "---").nth(2).unwrap_or("")
    } else {
        &content
    };

    let has_completion = body.contains("## Completion Criteria")
        || body.contains("## Verification")
        || body.contains("- [ ]")
        || body.contains("- [x]");
    if !has_completion {
        issues.push(
            "Missing checkable completion criteria or verification section ('## Completion Criteria' or '## Verification').".to_string(),
        );
    }

    let is_valid = issues.is_empty();
    (is_valid, issues)
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn test_validate_skill_metadata_strict_assertions() {
        let errs = validate_skill_metadata("valid-skill-name", "A valid skill description.");
        assert!(errs.is_empty());

        let errs_invalid = validate_skill_metadata("Invalid_Name", "");
        assert_eq!(errs_invalid.len(), 2);
        assert!(errs_invalid[0].contains("lowercase alphanumeric"));
        assert!(errs_invalid[1].contains("cannot be empty"));
    }

    #[test]
    fn test_scaffold_and_validate_simple_skill_strict_assertions() {
        let dir = tempdir().unwrap();
        let base = dir.path().canonicalize().unwrap();

        let args = ScaffoldArgs {
            name: Some("test-scaffold".to_string()),
            description: Some("Test scaffold description.".to_string()),
            domain: Some("testing".to_string()),
            tags: Some("test, unit".to_string()),
            target_dir: "skills".to_string(),
            r#type: SkillType::Simple,
            user_invoked: true,
            complex: false,
            dry_run: false,
        };

        let scaffolded = scaffold_skill(&args, Some(&base)).unwrap();
        assert!(scaffolded.exists());
        assert_eq!(
            scaffolded.file_name().unwrap().to_str().unwrap(),
            "test-scaffold"
        );

        let skill_md_content = fs::read_to_string(scaffolded.join("SKILL.md")).unwrap();
        assert!(skill_md_content.contains("disable-model-invocation: true"));
        assert!(skill_md_content.contains("domain: testing"));
        assert!(skill_md_content.contains("tags: [test, unit]"));

        let val_args = ValidateArgs {
            path: scaffolded.to_string_lossy().to_string(),
            dry_run: false,
        };

        let (is_valid, issues) = validate_skill(&val_args, Some(&base));
        assert_eq!(issues, Vec::<String>::new());
        assert!(is_valid);
    }

    #[test]
    fn test_scaffold_and_validate_complex_skill_strict_assertions() {
        let dir = tempdir().unwrap();
        let base = dir.path().canonicalize().unwrap();

        let args = ScaffoldArgs {
            name: Some("test-complex".to_string()),
            description: Some("Test complex description.".to_string()),
            domain: None,
            tags: None,
            target_dir: "skills".to_string(),
            r#type: SkillType::Complex,
            user_invoked: false,
            complex: false,
            dry_run: false,
        };

        let scaffolded = scaffold_skill(&args, Some(&base)).unwrap();
        assert!(scaffolded.join("references/overview.md").is_file());
        assert!(scaffolded.join("scripts/main.py").is_file());

        let val_args = ValidateArgs {
            path: scaffolded.to_string_lossy().to_string(),
            dry_run: false,
        };

        let (is_valid, issues) = validate_skill(&val_args, Some(&base));
        assert_eq!(issues, Vec::<String>::new());
        assert!(is_valid);
    }

    #[test]
    fn test_validate_skill_metadata_too_long() {
        // Python: test_validate_skill_metadata_too_long — name exceeding MAX_NAME_LEN is rejected
        let long_name = "a".repeat(MAX_NAME_LEN + 1);
        let errs = validate_skill_metadata(&long_name, "A valid description.");
        assert!(
            !errs.is_empty(),
            "name exceeding {MAX_NAME_LEN} chars must produce validation errors"
        );
        assert!(
            errs.iter().any(|e| e.contains("exceeds maximum length")),
            "error must mention 'exceeds maximum length', got: {:?}",
            errs
        );
    }

    #[test]
    fn test_scaffold_and_validate_user_invoked_router_skill() {
        // Python: test_scaffold_and_validate_user_invoked_router_skill
        // SkillType::Router branch is never tested
        let dir = tempdir().unwrap();
        let base = dir.path().canonicalize().unwrap();

        let args = ScaffoldArgs {
            name: Some("my-router".to_string()),
            description: Some("A router skill for testing.".to_string()),
            domain: None,
            tags: None,
            target_dir: "skills".to_string(),
            r#type: SkillType::Router,
            user_invoked: true,
            complex: false,
            dry_run: false,
        };

        let scaffolded = scaffold_skill(&args, Some(&base)).unwrap();
        assert!(
            scaffolded.exists(),
            "Router skill directory must be created"
        );

        let skill_md = fs::read_to_string(scaffolded.join("SKILL.md")).unwrap();
        assert!(
            skill_md.contains("router") || skill_md.contains("Router"),
            "Router skill SKILL.md must mention router type, got: {skill_md}"
        );
    }

    #[test]
    fn test_validate_skill_missing_completion_criteria() {
        // Python: test_validate_skill_missing_completion_criteria — validate detects absent section
        let dir = tempdir().unwrap();
        let base = dir.path().canonicalize().unwrap();
        let skill_dir = base.join("skills").join("bad-skill");
        fs::create_dir_all(&skill_dir).unwrap();

        // SKILL.md without Completion Criteria section
        fs::write(
            skill_dir.join("SKILL.md"),
            "---
name: bad-skill
---
# Bad Skill

## Context
No completion criteria here.
",
        )
        .unwrap();

        let val_args = ValidateArgs {
            path: skill_dir.to_string_lossy().to_string(),
            dry_run: false,
        };
        let (is_valid, issues) = validate_skill(&val_args, Some(&base));
        assert!(
            !is_valid || !issues.is_empty(),
            "SKILL.md without Completion Criteria must fail validation"
        );
        assert!(
            issues
                .iter()
                .any(|i| i.to_lowercase().contains("completion")),
            "validation issue must mention 'completion criteria', got: {:?}",
            issues
        );
    }

    #[test]
    fn test_validate_skill_detects_scope_creep() {
        let dir = tempdir().unwrap();
        let base = dir.path().canonicalize().unwrap();
        let skill_dir = base.join("skills").join("creepy-skill");
        fs::create_dir_all(&skill_dir).unwrap();
        fs::write(
            skill_dir.join("SKILL.md"),
            "---\nname: creepy-skill\ndescription: \"Use when analyzing code and also generating reports and also fixing bugs and also running tests.\"\n---\n\n## Completion Criteria\n- [ ] done\n",
        )
        .unwrap();

        let val_args = ValidateArgs {
            path: skill_dir.to_string_lossy().to_string(),
            dry_run: false,
        };
        let (is_valid, issues) = validate_skill(&val_args, Some(&base));
        assert!(
            !is_valid || !issues.is_empty(),
            "scope creep must be detected"
        );
        assert!(
            issues.iter().any(
                |i| i.to_lowercase().contains("scope") || i.to_lowercase().contains("compound")
            ),
            "issue must mention scope or compound, got: {:?}",
            issues
        );
    }

    #[test]
    fn test_validate_skill_detects_non_stdlib_imports() {
        let dir = tempdir().unwrap();
        let base = dir.path().canonicalize().unwrap();
        let skill_dir = base.join("skills").join("import-skill");
        let scripts_dir = skill_dir.join("scripts");
        let refs_dir = skill_dir.join("references");
        fs::create_dir_all(&scripts_dir).unwrap();
        fs::create_dir_all(&refs_dir).unwrap();
        fs::write(
            skill_dir.join("SKILL.md"),
            "---\nname: import-skill\ndescription: A skill.\n---\n\n## Completion Criteria\n- [ ] done\n",
        )
        .unwrap();
        fs::write(refs_dir.join("overview.md"), "# Overview\n").unwrap();
        fs::write(
            scripts_dir.join("main.py"),
            "import requests\nimport os\n\ndef main():\n    pass\n",
        )
        .unwrap();

        let val_args = ValidateArgs {
            path: skill_dir.to_string_lossy().to_string(),
            dry_run: false,
        };
        let (is_valid, issues) = validate_skill(&val_args, Some(&base));
        assert!(!is_valid || !issues.is_empty());
        assert!(
            issues.iter().any(|i| i.to_lowercase().contains("requests")
                || i.to_lowercase().contains("non-stdlib")),
            "non-stdlib import must be detected, got: {:?}",
            issues
        );
    }

    #[test]
    fn test_validate_skill_detects_excessive_lines() {
        let dir = tempdir().unwrap();
        let base = dir.path().canonicalize().unwrap();
        let skill_dir = base.join("skills").join("long-skill");
        let scripts_dir = skill_dir.join("scripts");
        let refs_dir = skill_dir.join("references");
        fs::create_dir_all(&scripts_dir).unwrap();
        fs::create_dir_all(&refs_dir).unwrap();

        let mut content =
            "---\nname: long-skill\ndescription: A very long skill.\n---\n\n## Completion Criteria\n- [ ] done\n"
                .to_string();
        for i in 0..510 {
            content.push_str(&format!("# Line {}\n", i));
        }
        fs::write(skill_dir.join("SKILL.md"), &content).unwrap();
        fs::write(refs_dir.join("overview.md"), "# Overview\n").unwrap();
        fs::write(scripts_dir.join("main.py"), "def main():\n    pass\n").unwrap();

        let val_args = ValidateArgs {
            path: skill_dir.to_string_lossy().to_string(),
            dry_run: false,
        };
        let (is_valid, issues) = validate_skill(&val_args, Some(&base));
        assert!(
            !is_valid || !issues.is_empty(),
            "excessive line count must be detected"
        );
        assert!(
            issues
                .iter()
                .any(|i| i.to_lowercase().contains("line") || i.to_lowercase().contains("context")),
            "must mention line count issue, got: {:?}",
            issues
        );
    }

    #[test]
    fn test_validate_skill_detects_invalid_yaml_frontmatter() {
        // Python: test_validate_skill_detects_invalid_yaml_frontmatter — broken YAML in SKILL.md flagged
        let dir = tempdir().unwrap();
        let base = dir.path().canonicalize().unwrap();
        let skill_dir = base.join("skills").join("broken-skill");
        fs::create_dir_all(&skill_dir).unwrap();

        // Write SKILL.md with malformed YAML frontmatter (unclosed bracket)
        fs::write(
            skill_dir.join("SKILL.md"),
            "---
name: [unclosed
---
# Broken
",
        )
        .unwrap();

        let val_args = ValidateArgs {
            path: skill_dir.to_string_lossy().to_string(),
            dry_run: false,
        };
        let (is_valid, issues) = validate_skill(&val_args, Some(&base));
        assert!(
            !is_valid || !issues.is_empty(),
            "invalid YAML frontmatter must cause validate_skill to fail"
        );
        assert!(
            issues.iter().any(|i| i.to_lowercase().contains("yaml")
                || i.to_lowercase().contains("frontmatter")
                || i.to_lowercase().contains("parse")),
            "issue must mention YAML/frontmatter parse error, got: {:?}",
            issues
        );
    }

    #[test]
    fn test_scaffold_skill_quotes_colon_description() {
        // Python: test_scaffold_skill_quotes_colon_description — descriptions with ':' are properly quoted
        let dir = tempdir().unwrap();
        let base = dir.path().canonicalize().unwrap();

        let args = ScaffoldArgs {
            name: Some("colon-skill".to_string()),
            description: Some("Use when: you need colon handling.".to_string()),
            domain: None,
            tags: None,
            target_dir: "skills".to_string(),
            r#type: SkillType::Simple,
            user_invoked: false,
            complex: false,
            dry_run: false,
        };

        let scaffolded = scaffold_skill(&args, Some(&base)).unwrap();
        let skill_md = fs::read_to_string(scaffolded.join("SKILL.md")).unwrap();

        // The description containing ':' must be quoted in YAML to prevent parse errors
        let unquoted = "description: Use when: you need colon handling.";
        assert!(
            !skill_md.contains(unquoted),
            "description with colon must be YAML-quoted (e.g. wrapped in quotes), not left as bare YAML value"
        );
    }

    #[test]
    fn test_scaffold_and_validate_complex_skill_full_structure() {
        // Python: test_scaffold_and_validate_complex_skill — PARTIAL, the existing strict_assertions
        // test only checks references/ and scripts/main.py; scaffold_skill also creates
        // templates/, examples/, and scripts/tests/ which were never asserted.
        let dir = tempdir().unwrap();
        let base = dir.path().canonicalize().unwrap();

        let args = ScaffoldArgs {
            name: Some("full-structure-skill".to_string()),
            description: Some("Full structure description.".to_string()),
            domain: None,
            tags: None,
            target_dir: "skills".to_string(),
            r#type: SkillType::Complex,
            user_invoked: false,
            complex: false,
            dry_run: false,
        };

        let scaffolded = scaffold_skill(&args, Some(&base)).unwrap();
        assert!(
            scaffolded.join("templates").is_dir(),
            "complex skill must create templates/"
        );
        assert!(
            scaffolded.join("examples").is_dir(),
            "complex skill must create examples/"
        );
        assert!(
            scaffolded.join("scripts").join("tests").is_dir(),
            "complex skill must create scripts/tests/"
        );
    }

    #[test]
    fn test_parse_frontmatter_domain_tags_roundtrip() {
        // Python: test_parse_frontmatter — domain/tags fields are written by scaffold_skill but
        // never read back by validate_skill; no isolated frontmatter-parsing unit exists.
        let dir = tempdir().unwrap();
        let base = dir.path().canonicalize().unwrap();

        let args = ScaffoldArgs {
            name: Some("tagged-skill".to_string()),
            description: Some("Has domain and tags.".to_string()),
            domain: Some("testing".to_string()),
            tags: Some("test,unit".to_string()),
            target_dir: "skills".to_string(),
            r#type: SkillType::Simple,
            user_invoked: false,
            complex: false,
            dry_run: false,
        };

        let scaffolded = scaffold_skill(&args, Some(&base)).unwrap();
        let (domain, tags) = parse_skill_frontmatter_fields(&scaffolded.join("SKILL.md")).unwrap();
        assert_eq!(domain, Some("testing".to_string()));
        assert_eq!(tags, vec!["test".to_string(), "unit".to_string()]);
    }
}
