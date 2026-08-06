use agent_skills_core::path_safety::sanitize_path;
use clap::{Args, Subcommand};
use std::fs;
use std::path::{Path, PathBuf};

#[derive(Args, Debug, Clone)]
pub struct CheckDomainArgs {
    /// Target path of the domain-modeling skill folder (defaults to skills/domain-modeling)
    #[arg(long)]
    pub path: Option<String>,
}

#[derive(Args, Debug, Clone)]
pub struct ScaffoldEntityArgs {
    /// Name of the entity or aggregate root
    pub name: String,

    /// Target directory to scaffold entity stub into (defaults to cwd)
    #[arg(long, default_value = ".")]
    pub path: String,
}

#[derive(Subcommand, Debug, Clone)]
pub enum DomainModelingSubcommand {
    /// Verify health and structural completeness of domain-modeling files
    Check(CheckDomainArgs),
    /// Scaffold a pure domain entity / aggregate root stub
    ScaffoldEntity(ScaffoldEntityArgs),
}

pub fn check_domain_modeling_health(skill_dir: &Path) -> Result<Vec<String>, String> {
    let required_files = [
        skill_dir.join("SKILL.md"),
        skill_dir.join("README.md"),
        skill_dir.join("references").join("overview.md"),
        skill_dir.join("references").join("ddd-patterns.md"),
        skill_dir.join("references").join("state-machines.md"),
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
        Err(format!(
            "Domain Modeling health check failed. Missing files: {:?}",
            missing
        ))
    }
}

pub fn scaffold_entity_stub(name: &str, target_dir: &Path) -> Result<PathBuf, String> {
    fs::create_dir_all(target_dir)
        .map_err(|e| format!("Failed to create target directory: {e}"))?;

    let filename = format!("{}.rs", name.to_lowercase());
    let file_path = target_dir.join(&filename);

    let stub_content = format!(
        r#"/// Pure domain entity aggregate root representing {name}.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct {name} {{
    id: String,
    status: String,
}}

impl {name} {{
    pub fn new(id: impl Into<String>) -> Self {{
        Self {{
            id: id.into(),
            status: "Draft".to_string(),
        }}
    }}

    pub fn id(&self) -> &str {{
        &self.id
    }}

    pub fn status(&self) -> &str {{
        &self.status
    }}
}}
"#
    );

    fs::write(&file_path, stub_content)
        .map_err(|e| format!("Failed to write entity stub file: {e}"))?;
    Ok(file_path)
}

pub fn run_domain_modeling_command(
    subcommand: &DomainModelingSubcommand,
    repo_root: &Path,
) -> Result<(), String> {
    match subcommand {
        DomainModelingSubcommand::Check(args) => {
            let skill_dir = if let Some(p) = &args.path {
                sanitize_path(p, Some(repo_root))?
            } else {
                repo_root.join("skills").join("domain-modeling")
            };

            match check_domain_modeling_health(&skill_dir) {
                Ok(_) => {
                    println!("Domain Modeling skill health check passed cleanly.");
                    Ok(())
                }
                Err(err) => Err(err),
            }
        }
        DomainModelingSubcommand::ScaffoldEntity(args) => {
            let target_dir = sanitize_path(&args.path, Some(repo_root))?;
            let file_path = scaffold_entity_stub(&args.name, &target_dir)?;
            println!(
                "🎉 Scaffolded domain entity stub at: {}",
                file_path.display()
            );
            Ok(())
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn test_check_domain_modeling_health_strict_assertions() {
        let dir = tempdir().unwrap();
        let base = dir.path().canonicalize().unwrap();
        let references = base.join("references");
        fs::create_dir_all(&references).unwrap();

        fs::write(base.join("SKILL.md"), "---").unwrap();
        fs::write(base.join("README.md"), "# Title").unwrap();
        fs::write(references.join("overview.md"), "# Overview").unwrap();
        fs::write(references.join("ddd-patterns.md"), "# DDD").unwrap();
        fs::write(references.join("state-machines.md"), "# States").unwrap();

        let result = check_domain_modeling_health(&base);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), Vec::<String>::new());
    }

    #[test]
    fn test_scaffold_entity_stub_strict_assertions() {
        let dir = tempdir().unwrap();
        let base = dir.path().canonicalize().unwrap();
        let path = scaffold_entity_stub("Order", &base).unwrap();

        assert!(path.exists());
        assert_eq!(path.file_name().unwrap().to_str().unwrap(), "order.rs");
        let content = fs::read_to_string(path).unwrap();
        assert!(content.contains("pub struct Order"));
        assert!(content.contains("pub fn new"));
    }
}
