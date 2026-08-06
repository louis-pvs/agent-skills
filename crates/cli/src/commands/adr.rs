use agent_skills_core::config_safety::load_skill_config;
use agent_skills_core::path_safety::sanitize_path;
use clap::{Args, Subcommand, ValueEnum};
use regex::Regex;
use std::fs;
use std::path::{Path, PathBuf};

#[derive(ValueEnum, Clone, Debug, PartialEq, Eq, Default)]
pub enum AdrTemplate {
    #[default]
    Madr,
    Nygard,
}

#[derive(Args, Debug, Clone)]
pub struct NewAdrArgs {
    /// Title of the architectural decision record
    pub title: String,

    /// Template format (madr or nygard)
    #[arg(long, value_enum, default_value = "madr")]
    pub template: AdrTemplate,
}

#[derive(Args, Debug, Clone)]
pub struct SupersedeAdrArgs {
    /// ID or filename of the ADR being superseded
    #[arg(long)]
    pub old: String,

    /// ID or filename of the new ADR superseding the old ADR
    #[arg(long)]
    pub by: String,
}

#[derive(Args, Debug, Clone)]
pub struct StatusAdrArgs {
    /// ID or filename of the target ADR
    pub id: String,
}

#[derive(Subcommand, Debug, Clone)]
pub enum AdrSubcommand {
    /// Initialize docs/adr directory with 0000 ADR and README catalog index
    Init,
    /// Scaffold a new sequential Architectural Decision Record
    New(NewAdrArgs),
    /// Supersede an existing ADR with a new decision record
    Supersede(SupersedeAdrArgs),
    /// Set status of an ADR to Accepted
    Accept(StatusAdrArgs),
    /// Set status of an ADR to Rejected
    Reject(StatusAdrArgs),
    /// Rebuild docs/adr/README.md catalog index table
    Reindex,
    /// Validate ADR sequential IDs, headers, and link integrity
    Validate,
}

pub fn slugify(text: &str) -> String {
    let lower = text.to_lowercase();
    let reg_punct = Regex::new(r"[^\w\s-]").unwrap();
    let cleaned = reg_punct.replace_all(&lower, "");
    let reg_spaces = Regex::new(r"[\s_-]+").unwrap();
    let slug = reg_spaces.replace_all(&cleaned, "-");
    slug.trim_matches('-').to_string()
}

pub fn resolve_adr_directory(repo_root: &Path) -> Result<(PathBuf, String), String> {
    let skill_dir = repo_root
        .join("skills")
        .join("architecture-decision-records");
    let cfg = load_skill_config(
        "architecture-decision-records",
        Some(&skill_dir),
        Some(repo_root),
        None,
    );

    let mut adr_rel_path = "docs/adr".to_string();
    let mut adr_prefix = "ADR".to_string();

    if let Some(map) = cfg.as_mapping() {
        if let Some(rel) = map.get("adr_dir").and_then(|v| v.as_str()) {
            adr_rel_path = rel.to_string();
        }
        if let Some(pfx) = map.get("prefix").and_then(|v| v.as_str()) {
            adr_prefix = pfx.to_string();
        }
    }

    let adr_dir = sanitize_path(&adr_rel_path, Some(repo_root))?;
    Ok((adr_dir, adr_prefix))
}

pub fn find_highest_adr_id(adr_dir: &Path) -> u32 {
    if !adr_dir.exists() {
        return 0;
    }
    let mut highest = 0;
    let pattern = Regex::new(r"^(\d{4})-[a-z0-9-]+\.md$").unwrap();

    if let Ok(entries) = fs::read_dir(adr_dir) {
        for entry in entries.flatten() {
            let fname = entry.file_name().to_string_lossy().to_lowercase();
            if fname == "readme.md" || fname == "index.md" {
                continue;
            }
            if let Some(caps) = pattern.captures(&fname) {
                if let Ok(id) = caps.get(1).unwrap().as_str().parse::<u32>() {
                    if id > highest {
                        highest = id;
                    }
                }
            }
        }
    }
    highest
}

pub fn get_all_adr_files(adr_dir: &Path) -> Vec<PathBuf> {
    if !adr_dir.exists() {
        return Vec::new();
    }
    let pattern = Regex::new(r"^\d{4}-[a-z0-9-]+\.md$").unwrap();
    let mut adr_files = Vec::new();

    if let Ok(entries) = fs::read_dir(adr_dir) {
        for entry in entries.flatten() {
            let p = entry.path();
            if let Some(fname) = p.file_name().and_then(|f| f.to_str()) {
                if pattern.is_match(fname) {
                    adr_files.push(p);
                }
            }
        }
    }
    adr_files.sort();
    adr_files
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AdrMetadata {
    pub id: String,
    pub title: String,
    pub status: String,
    pub date: String,
    pub filename: String,
}

pub fn parse_adr_metadata(file_path: &Path) -> AdrMetadata {
    let filename = file_path
        .file_name()
        .map(|n| n.to_string_lossy().to_string())
        .unwrap_or_default();

    let mut id = "0000".to_string();
    let pattern_id = Regex::new(r"^(\d{4})").unwrap();
    if let Some(caps) = pattern_id.captures(&filename) {
        id = caps.get(1).unwrap().as_str().to_string();
    }

    let mut metadata = AdrMetadata {
        id,
        title: "Unknown Title".to_string(),
        status: "Unknown".to_string(),
        date: "Unknown".to_string(),
        filename,
    };

    let content = match fs::read_to_string(file_path) {
        Ok(c) => c,
        Err(_) => return metadata,
    };

    let mut in_status = false;
    let date_pattern = Regex::new(r"\b(\d{4}-\d{2}-\d{2})\b").unwrap();
    let title_prefix_pattern = Regex::new(r"^\d+[\.\:]\s*").unwrap();
    let status_inline = Regex::new(r"(?i)^-?\s*\*\*Status\*\*:\s*(.+)").unwrap();

    for line in content.lines() {
        let stripped = line.trim();

        if stripped.starts_with("# ") && metadata.title == "Unknown Title" {
            let raw_title = stripped[2..].trim();
            metadata.title = title_prefix_pattern.replace(raw_title, "").to_string();
        }

        if metadata.status == "Unknown" {
            if let Some(caps) = status_inline.captures(stripped) {
                metadata.status = caps.get(1).unwrap().as_str().trim().to_string();
            }
        }

        if stripped.to_lowercase() == "## status" {
            in_status = true;
            continue;
        }

        if in_status {
            if stripped.starts_with("## ") && stripped.to_lowercase() != "## status" {
                in_status = false;
            } else if !stripped.is_empty() {
                metadata.status = stripped.to_string();
                in_status = false;
            }
        }

        if metadata.date == "Unknown" {
            if let Some(caps) = date_pattern.captures(line) {
                metadata.date = caps.get(1).unwrap().as_str().to_string();
            }
        }
    }

    metadata
}

pub fn scaffold_adr_file(
    adr_dir: &Path,
    title: &str,
    template: AdrTemplate,
) -> Result<PathBuf, String> {
    fs::create_dir_all(adr_dir).map_err(|e| format!("Failed to create adr_dir: {e}"))?;

    let next_id = find_highest_adr_id(adr_dir) + 1;
    let id_str = format!("{next_id:04}");
    let slug = slugify(title);
    let filename = format!("{id_str}-{slug}.md");
    let file_path = adr_dir.join(&filename);

    let date_str = chrono::Local::now().format("%Y-%m-%d").to_string();

    let content = match template {
        AdrTemplate::Madr => format!(
            "# {next_id}. {title}\n\nDate: {date_str}\n\n## Status\n\nProposed\n\n## Context & Problem Statement\n\nDescribe context and problem statement.\n\n## Decision Drivers\n\n* Driver 1\n* Driver 2\n\n## Considered Options\n\n* Option 1\n* Option 2\n\n## Decision Outcome\n\nChosen Option: Option 1\n\n### Consequences\n\n* Positive consequence 1\n* Negative consequence 1\n"
        ),
        AdrTemplate::Nygard => format!(
            "# {next_id}. {title}\n\nDate: {date_str}\n\n## Status\n\nProposed\n\n## Context\n\nContext and decision drivers.\n\n## Decision\n\nDecision description.\n\n## Consequences\n\nConsequences description.\n"
        ),
    };

    fs::write(&file_path, content).map_err(|e| format!("Failed to write ADR file: {e}"))?;
    Ok(file_path)
}

pub fn generate_index_markdown(adr_dir: &Path) -> String {
    let adr_files = get_all_adr_files(adr_dir);
    let mut lines = vec![
        "# Architectural Decision Records".to_string(),
        "".to_string(),
        "This directory contains all Architectural Decision Records (ADRs) for this project."
            .to_string(),
        "".to_string(),
        "| ID | Title | Status | Date |".to_string(),
        "| :--- | :--- | :--- | :--- |".to_string(),
    ];

    for file_path in adr_files {
        let meta = parse_adr_metadata(&file_path);
        let title_link = format!("[{}]({})", meta.title, meta.filename);
        lines.push(format!(
            "| `{}` | {} | {} | {} |",
            meta.id, title_link, meta.status, meta.date
        ));
    }

    lines.push("".to_string());
    lines.join("\n")
}

pub fn reindex_adr_catalog(adr_dir: &Path) -> Result<PathBuf, String> {
    fs::create_dir_all(adr_dir).map_err(|e| format!("Failed to create adr_dir: {e}"))?;
    let readme_path = adr_dir.join("README.md");
    let content = generate_index_markdown(adr_dir);
    fs::write(&readme_path, content).map_err(|e| format!("Failed to write README.md: {e}"))?;
    Ok(readme_path)
}

pub fn update_adr_status(adr_file: &Path, new_status: &str) -> Result<(), String> {
    if !adr_file.exists() {
        return Err(format!("File does not exist: {}", adr_file.display()));
    }

    let content = fs::read_to_string(adr_file).map_err(|e| format!("Failed to read file: {e}"))?;
    let mut new_lines = Vec::new();
    let mut in_status = false;
    let mut status_updated = false;

    for line in content.lines() {
        let stripped = line.trim();
        if stripped.to_lowercase() == "## status" {
            in_status = true;
            new_lines.push(line.to_string());
            continue;
        }

        if in_status {
            if stripped.starts_with("## ") {
                in_status = false;
                new_lines.push(line.to_string());
            } else if !status_updated {
                new_lines.push(new_status.to_string());
                status_updated = true;
            }
            continue;
        }

        new_lines.push(line.to_string());
    }

    if !status_updated && in_status {
        new_lines.push(new_status.to_string());
    }

    fs::write(adr_file, new_lines.join("\n") + "\n")
        .map_err(|e| format!("Failed to write status to file: {e}"))?;
    Ok(())
}

pub fn run_adr_command(subcommand: &AdrSubcommand, repo_root: &Path) -> Result<(), String> {
    let (adr_dir, _) = resolve_adr_directory(repo_root)?;

    match subcommand {
        AdrSubcommand::Init => {
            fs::create_dir_all(&adr_dir).map_err(|e| format!("Failed to create adr_dir: {e}"))?;
            let init_adr = adr_dir.join("0000-use-markdown-architectural-decision-records.md");
            if !init_adr.exists() {
                let date_str = chrono::Local::now().format("%Y-%m-%d").to_string();
                let init_content = format!(
                    "# 0. Record Architecture Decisions\n\nDate: {date_str}\n\n## Status\n\nAccepted\n\n## Context\n\nWe need to record architectural decisions.\n\n## Decision\n\nWe will use MADR/Nygard formatted records in version control.\n"
                );
                let _ = fs::write(&init_adr, init_content);
            }
            let index_path = reindex_adr_catalog(&adr_dir)?;
            println!("🎉 Initialized ADR directory at: {}", adr_dir.display());
            println!("📖 Generated catalog index: {}", index_path.display());
            Ok(())
        }
        AdrSubcommand::New(args) => {
            let scaffolded = scaffold_adr_file(&adr_dir, &args.title, args.template.clone())?;
            let _ = reindex_adr_catalog(&adr_dir);
            println!("🎉 Scaffolded new ADR: {}", scaffolded.display());
            Ok(())
        }
        AdrSubcommand::Supersede(args) => {
            let files = get_all_adr_files(&adr_dir);
            let old_file = files
                .iter()
                .find(|p| p.name_matches(&args.old))
                .ok_or_else(|| format!("Could not find ADR matching old ID/name '{}'", args.old))?;
            let by_file = files.iter().find(|p| p.name_matches(&args.by));

            let by_ref = by_file
                .map(|p| p.file_name().unwrap().to_string_lossy().to_string())
                .unwrap_or_else(|| args.by.clone());

            let status_text = format!("Superseded by [{by_ref}]({by_ref})");
            update_adr_status(old_file, &status_text)?;
            let _ = reindex_adr_catalog(&adr_dir);
            println!("🔄 Set status of {} to '{status_text}'", old_file.display());
            Ok(())
        }
        AdrSubcommand::Accept(args) => {
            let files = get_all_adr_files(&adr_dir);
            let target = files
                .iter()
                .find(|p| p.name_matches(&args.id))
                .ok_or_else(|| format!("Could not find ADR matching '{}'", args.id))?;

            update_adr_status(target, "Accepted")?;
            let _ = reindex_adr_catalog(&adr_dir);
            println!("✅ Set status of {} to Accepted", target.display());
            Ok(())
        }
        AdrSubcommand::Reject(args) => {
            let files = get_all_adr_files(&adr_dir);
            let target = files
                .iter()
                .find(|p| p.name_matches(&args.id))
                .ok_or_else(|| format!("Could not find ADR matching '{}'", args.id))?;

            update_adr_status(target, "Rejected")?;
            let _ = reindex_adr_catalog(&adr_dir);
            println!("❌ Set status of {} to Rejected", target.display());
            Ok(())
        }
        AdrSubcommand::Reindex => {
            let index_path = reindex_adr_catalog(&adr_dir)?;
            println!("📖 Reindexed ADR catalog at: {}", index_path.display());
            Ok(())
        }
        AdrSubcommand::Validate => {
            let files = get_all_adr_files(&adr_dir);
            println!(
                "Auditing {} ADR file(s) in {}...",
                files.len(),
                adr_dir.display()
            );
            let mut errors = Vec::new();

            for file in files {
                let meta = parse_adr_metadata(&file);
                if meta.title == "Unknown Title" {
                    errors.push(format!("{}: Missing H1 title header.", file.display()));
                }
                if meta.status == "Unknown" {
                    errors.push(format!("{}: Missing status section.", file.display()));
                }
            }

            if errors.is_empty() {
                println!("✅ All ADR files are valid!");
                Ok(())
            } else {
                for err in &errors {
                    eprintln!("  - {err}");
                }
                Err("ADR validation failed.".to_string())
            }
        }
    }
}

trait MatchName {
    fn name_matches(&self, query: &str) -> bool;
}

impl MatchName for PathBuf {
    fn name_matches(&self, query: &str) -> bool {
        let fname = self.file_name().unwrap_or_default().to_string_lossy();
        fname.contains(query) || fname.starts_with(query)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn test_slugify_strict_assertions() {
        assert_eq!(slugify("Adopt Rust for CLI!"), "adopt-rust-for-cli");
        assert_eq!(slugify("Use  PostgreSQL 15--DB"), "use-postgresql-15-db");
    }

    #[test]
    fn test_scaffold_and_reindex_adr_strict_assertions() {
        let dir = tempdir().unwrap();
        let base = dir.path().canonicalize().unwrap();
        let adr_dir = base.join("docs/adr");

        let file_path = scaffold_adr_file(&adr_dir, "Test Decision", AdrTemplate::Madr).unwrap();
        assert!(file_path.exists());
        assert_eq!(
            file_path.file_name().unwrap().to_str().unwrap(),
            "0001-test-decision.md"
        );

        let index_path = reindex_adr_catalog(&adr_dir).unwrap();
        assert!(index_path.exists());

        let index_content = fs::read_to_string(index_path).unwrap();
        assert!(index_content.contains("0001"));
        assert!(index_content.contains("Test Decision"));
    }
}
