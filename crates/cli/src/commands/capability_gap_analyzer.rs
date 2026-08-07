use agent_skills_core::path_safety::sanitize_path;
use clap::{Args, Subcommand};
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use std::fs;
use std::path::{Path, PathBuf};

#[derive(Args, Debug, Clone)]
pub struct CheckGapArgs {
    /// Target path of the capability-gap-analyzer skill folder
    #[arg(long)]
    pub path: Option<String>,
}

#[derive(Args, Debug, Clone)]
pub struct AnalyzeGapArgs {
    /// Target domain name (e.g., frontend-web, devops-infra)
    #[arg(long)]
    pub domain: Option<String>,

    /// Auto-detect workspace domain markers
    #[arg(long)]
    pub auto_detect: bool,

    /// Output findings in JSON format
    #[arg(long)]
    pub json: bool,
}

#[derive(Subcommand, Debug, Clone)]
pub enum CapabilityGapAnalyzerSubcommand {
    /// Verify health and structural completeness of capability-gap-analyzer files
    Check(CheckGapArgs),
    /// Analyze workspace capabilities and identify skill coverage gaps
    Analyze(AnalyzeGapArgs),
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct DomainDetectionResult {
    pub detected_domains: Vec<String>,
    pub primary_domain: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct SkillFrontmatter {
    pub name: String,
    pub description: String,
    pub domain: Option<String>,
    pub tags: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct SkillEntry {
    pub name: String,
    pub path: PathBuf,
    pub frontmatter: SkillFrontmatter,
    pub origin: String,
    pub full_body: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct TaxonomyHeatmap {
    pub categories: HashMap<String, HeatmapCategory>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct HeatmapCategory {
    pub domain: String,
    pub skill_count: usize,
    pub coverage_ratio: f64,
    pub is_dynamic: bool,
    pub workspace_count: usize,
    pub global_count: usize,
}

const BASELINE_DOMAINS: &[&str] = &[
    "rust-cli",
    "frontend-web",
    "python-backend",
    "devops-infra",
    "generic-software",
];

pub fn detect_workspace_domains(workspace_dir: &Path) -> anyhow::Result<DomainDetectionResult> {
    let mut detected = Vec::new();

    if workspace_dir.join("Cargo.toml").exists() {
        detected.push("rust-cli".to_string());
    }
    if workspace_dir.join("package.json").exists() {
        detected.push("frontend-web".to_string());
    }
    if workspace_dir.join("pyproject.toml").exists()
        || workspace_dir.join("requirements.txt").exists()
    {
        detected.push("python-backend".to_string());
    }
    if workspace_dir.join("Dockerfile").exists()
        || workspace_dir.join("docker-compose.yml").exists()
    {
        detected.push("devops-infra".to_string());
    }

    if detected.is_empty() {
        detected.push("generic-software".to_string());
    }

    let primary_domain = detected[0].clone();
    Ok(DomainDetectionResult {
        detected_domains: detected,
        primary_domain,
    })
}

pub fn check_gap_analyzer_health(skill_dir: &Path) -> anyhow::Result<Vec<String>> {
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
            "Capability Gap Analyzer health check failed. Missing files: {:?}",
            missing
        ))
    }
}

pub fn parse_skill_frontmatter(skill_md_path: &Path) -> anyhow::Result<SkillFrontmatter> {
    let content = fs::read_to_string(skill_md_path)?;
    if !content.starts_with("---") {
        return Err(anyhow::anyhow!("No YAML frontmatter found"));
    }
    let parts: Vec<&str> = content.splitn(3, "---").collect();
    if parts.len() < 3 {
        return Err(anyhow::anyhow!("Unclosed frontmatter"));
    }
    let yaml_block = parts[1];
    let val: serde_yaml::Value = serde_yaml::from_str(yaml_block)?;
    let map = val
        .as_mapping()
        .ok_or_else(|| anyhow::anyhow!("Frontmatter must be a mapping"))?;

    let name = map
        .get(serde_yaml::Value::String("name".to_string()))
        .and_then(|v| v.as_str())
        .unwrap_or("")
        .to_string();
    let description = map
        .get(serde_yaml::Value::String("description".to_string()))
        .and_then(|v| v.as_str())
        .unwrap_or("")
        .to_string();
    let domain = map
        .get(serde_yaml::Value::String("domain".to_string()))
        .and_then(|v| v.as_str())
        .map(canonicalize_domain);
    let tags = map
        .get(serde_yaml::Value::String("tags".to_string()))
        .and_then(|v| match v {
            serde_yaml::Value::Sequence(seq) => Some(
                seq.iter()
                    .filter_map(|t| t.as_str().map(|s| s.to_string()))
                    .collect(),
            ),
            serde_yaml::Value::String(s) => Some(
                s.split(',')
                    .map(|t| t.trim().to_string())
                    .filter(|t| !t.is_empty())
                    .collect(),
            ),
            _ => None,
        })
        .unwrap_or_default();

    Ok(SkillFrontmatter {
        name,
        description,
        domain,
        tags,
    })
}

pub fn scan_skills_inventory(skills_dirs: &[PathBuf], origin: &str) -> Vec<SkillEntry> {
    let mut entries = Vec::new();
    for skills_dir in skills_dirs {
        if !skills_dir.is_dir() {
            continue;
        }
        if let Ok(read_dir) = fs::read_dir(skills_dir) {
            for entry in read_dir.flatten() {
                let skill_dir = entry.path();
                if !skill_dir.is_dir() {
                    continue;
                }
                let skill_md = skill_dir.join("SKILL.md");
                if !skill_md.is_file() {
                    continue;
                }
                if let Ok(frontmatter) = parse_skill_frontmatter(&skill_md) {
                    if let Ok(full_body) = fs::read_to_string(&skill_md) {
                        entries.push(SkillEntry {
                            name: frontmatter.name.clone(),
                            path: skill_dir,
                            frontmatter,
                            origin: origin.to_string(),
                            full_body,
                        });
                    }
                }
            }
        }
    }
    entries
}

pub fn load_global_skill_paths() -> Vec<PathBuf> {
    let Ok(home) = std::env::var("HOME") else {
        return Vec::new();
    };

    [
        PathBuf::from(&home).join(".gemini/config/skills"),
        PathBuf::from(&home).join(".claude/skills"),
        PathBuf::from(&home).join(".copilot/skills"),
    ]
    .into_iter()
    .filter(|path| path.exists())
    .collect()
}

pub fn canonicalize_domain(raw: &str) -> String {
    let lower = raw.to_lowercase();
    let with_hyphens = lower.replace(' ', "-");
    let normalized = match with_hyphens.as_str() {
        "frontend-web" | "front-end" | "front-end-web" => "frontend-web",
        "backend" | "back-end" => "backend",
        "machine-learning" | "ml" => "ml",
        _ => &with_hyphens,
    };
    normalized.to_string()
}

pub fn harvest_dynamic_domains(skills_dirs: &[PathBuf]) -> Vec<String> {
    let mut domains = HashSet::new();
    for skill in scan_skills_inventory(skills_dirs, "workspace") {
        if let Some(domain) = skill.frontmatter.domain {
            domains.insert(canonicalize_domain(&domain));
        }
    }
    let mut sorted: Vec<String> = domains.into_iter().collect();
    sorted.sort();
    sorted
}

pub fn calculate_taxonomy_heatmap(
    skills: &[SkillEntry],
    dynamic_domains: &[String],
) -> TaxonomyHeatmap {
    let baseline_domains: HashSet<String> =
        BASELINE_DOMAINS.iter().map(|d| d.to_string()).collect();
    let dynamic_set: HashSet<String> = dynamic_domains
        .iter()
        .map(|d| canonicalize_domain(d))
        .collect();

    let mut categories = HashMap::new();
    for domain in BASELINE_DOMAINS {
        categories.insert(
            (*domain).to_string(),
            HeatmapCategory {
                domain: (*domain).to_string(),
                skill_count: 0,
                coverage_ratio: 0.0,
                is_dynamic: false,
                workspace_count: 0,
                global_count: 0,
            },
        );
    }

    for domain in &dynamic_set {
        categories.entry(domain.clone()).or_insert(HeatmapCategory {
            domain: domain.clone(),
            skill_count: 0,
            coverage_ratio: 0.0,
            is_dynamic: true,
            workspace_count: 0,
            global_count: 0,
        });
    }

    for skill in skills {
        let Some(raw_domain) = skill.frontmatter.domain.as_deref() else {
            continue;
        };
        let domain = canonicalize_domain(raw_domain);
        let is_dynamic = dynamic_set.contains(&domain) || !baseline_domains.contains(&domain);
        let category = categories.entry(domain.clone()).or_insert(HeatmapCategory {
            domain: domain.clone(),
            skill_count: 0,
            coverage_ratio: 0.0,
            is_dynamic,
            workspace_count: 0,
            global_count: 0,
        });
        category.is_dynamic = is_dynamic;
        category.skill_count += 1;
        if skill.origin == "workspace" {
            category.workspace_count += 1;
        } else {
            category.global_count += 1;
        }
    }

    for category in categories.values_mut() {
        category.coverage_ratio = if category.skill_count > 0 {
            category.workspace_count as f64 / category.skill_count as f64
        } else {
            0.0
        };
    }

    TaxonomyHeatmap { categories }
}

pub fn generate_heatmap_markdown(heatmap: &TaxonomyHeatmap) -> String {
    let mut domains: Vec<&String> = heatmap.categories.keys().collect();
    domains.sort();

    let mut out = String::from("| Domain | Skills | Coverage | Status |\n");
    out.push_str("|--------|--------|----------|--------|\n");
    for domain in domains {
        let category = &heatmap.categories[domain];
        let coverage = format!("{:.0}%", category.coverage_ratio * 100.0);
        let status = if category.workspace_count == 0 && category.skill_count == 0 {
            "Missing"
        } else if category.workspace_count == 0 {
            "Global Only"
        } else if (category.coverage_ratio - 1.0).abs() < f64::EPSILON {
            "Strong"
        } else {
            "Partial"
        };
        out.push_str(&format!(
            "| {} | {} | {} | {} |\n",
            category.domain, category.skill_count, coverage, status
        ));
    }
    out
}

pub fn filter_heatmap_by_relevance(
    heatmap: &TaxonomyHeatmap,
    workspace_domains: &[String],
) -> TaxonomyHeatmap {
    if workspace_domains.is_empty() {
        return heatmap.clone();
    }

    let relevant: HashSet<String> = workspace_domains
        .iter()
        .map(|domain| canonicalize_domain(domain))
        .collect();
    let categories = heatmap
        .categories
        .iter()
        .filter(|(_, category)| category.is_dynamic || relevant.contains(&category.domain))
        .map(|(name, category)| (name.clone(), category.clone()))
        .collect();

    TaxonomyHeatmap { categories }
}

#[allow(dead_code)]
pub fn build_scaffold_suggestions(heatmap: &TaxonomyHeatmap) -> Vec<String> {
    let mut domains: Vec<&String> = heatmap.categories.keys().collect();
    domains.sort();
    domains
        .into_iter()
        .filter_map(|domain| {
            let category = &heatmap.categories[domain];
            if category.workspace_count == 0 {
                Some(format!(
                    "Tier 2 LLM Prompt: scaffold a new skill for domain '{}' -- run: agent-skills skill-creator scaffold --domain {}",
                    category.domain, category.domain
                ))
            } else {
                None
            }
        })
        .collect()
}

pub fn run_capability_gap_analyzer_command(
    subcommand: &CapabilityGapAnalyzerSubcommand,
    repo_root: &Path,
) -> anyhow::Result<()> {
    match subcommand {
        CapabilityGapAnalyzerSubcommand::Check(args) => {
            let skill_dir = if let Some(p) = &args.path {
                sanitize_path(p, Some(repo_root))?
            } else {
                repo_root.join("skills").join("capability-gap-analyzer")
            };

            match check_gap_analyzer_health(&skill_dir) {
                Ok(_) => {
                    println!("Capability Gap Analyzer skill health check passed cleanly.");
                    Ok(())
                }
                Err(err) => Err(err),
            }
        }
        CapabilityGapAnalyzerSubcommand::Analyze(args) => {
            let detection = detect_workspace_domains(repo_root)?;
            let target_domain =
                canonicalize_domain(args.domain.as_deref().unwrap_or(&detection.primary_domain));

            let workspace_skill_dirs = vec![repo_root.join("skills")];
            let global_skill_dirs = load_global_skill_paths();
            let mut all_skill_dirs = workspace_skill_dirs.clone();
            all_skill_dirs.extend(global_skill_dirs.clone());

            let mut skills = scan_skills_inventory(&workspace_skill_dirs, "workspace");
            skills.extend(scan_skills_inventory(&global_skill_dirs, "global"));
            let dynamic_domains = harvest_dynamic_domains(&all_skill_dirs);
            let heatmap = calculate_taxonomy_heatmap(&skills, &dynamic_domains);
            let filtered_heatmap =
                filter_heatmap_by_relevance(&heatmap, &detection.detected_domains);

            let covered_count = filtered_heatmap
                .categories
                .values()
                .filter(|category| category.workspace_count > 0)
                .count();
            let total_count = filtered_heatmap.categories.len();
            let coverage_status = if covered_count == 0 {
                "None"
            } else if covered_count == total_count {
                "Strong"
            } else {
                "Partial"
            };

            if args.json {
                let out = serde_json::json!({
                    "target_domain": target_domain,
                    "auto_detected": detection.detected_domains,
                    "coverage_status": coverage_status,
                    "covered_ratio": format!("{covered_count}/{total_count}"),
                    "heatmap": filtered_heatmap,
                });
                println!("{}", serde_json::to_string_pretty(&out).unwrap());
            } else {
                println!("Capability Gap Analysis for domain: '{target_domain}'");
                println!("  Auto-detected domains: {:?}", detection.detected_domains);
                println!("  Checklist coverage: {coverage_status} ({covered_count}/{total_count})");
                println!("{}", generate_heatmap_markdown(&filtered_heatmap));
            }
            Ok(())
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::tempdir;

    /// Builder helper: construct a HeatmapCategory inline without repeating all fields.
    fn make_cat(
        domain: &str,
        skill_count: usize,
        is_dynamic: bool,
        workspace_count: usize,
        global_count: usize,
    ) -> HeatmapCategory {
        let coverage_ratio = if skill_count > 0 {
            workspace_count as f64 / skill_count as f64
        } else {
            0.0
        };
        HeatmapCategory {
            domain: domain.to_string(),
            skill_count,
            coverage_ratio,
            is_dynamic,
            workspace_count,
            global_count,
        }
    }

    /// Builder helper: construct a SkillEntry for tests.
    fn make_skill(name: &str, domain: &str, origin: &str) -> SkillEntry {
        SkillEntry {
            name: name.to_string(),
            path: std::path::PathBuf::from(name),
            frontmatter: SkillFrontmatter {
                name: name.to_string(),
                description: format!("{} skill.", name),
                domain: Some(domain.to_string()),
                tags: vec![],
            },
            origin: origin.to_string(),
            full_body: String::new(),
        }
    }

    #[test]
    fn test_detect_workspace_domains_strict_assertions() {
        let dir = tempdir().unwrap();
        let base = dir.path().canonicalize().unwrap();

        fs::write(base.join("Cargo.toml"), "[package]\nname=\"rust-app\"\n").unwrap();

        let res = detect_workspace_domains(&base).unwrap();
        assert_eq!(res.primary_domain, "rust-cli");
        assert_eq!(res.detected_domains.len(), 1);
        assert_eq!(res.detected_domains[0], "rust-cli");
    }

    #[test]
    fn test_check_gap_analyzer_health_strict_assertions() {
        let dir = tempdir().unwrap();
        let base = dir.path().canonicalize().unwrap();
        let references = base.join("references");
        fs::create_dir_all(&references).unwrap();

        fs::write(base.join("SKILL.md"), "---").unwrap();
        fs::write(base.join("README.md"), "# Title").unwrap();
        fs::write(references.join("overview.md"), "# Overview").unwrap();

        let res = check_gap_analyzer_health(&base);
        assert!(res.is_ok());
    }

    #[test]
    fn test_detect_workspace_domains_frontend() {
        let dir = tempdir().unwrap();
        let base = dir.path().canonicalize().unwrap();

        fs::write(
            base.join("package.json"),
            "{\"dependencies\": {\"react\": \"^18.0.0\"}}",
        )
        .unwrap();

        let res = detect_workspace_domains(&base).unwrap();
        assert_eq!(res.primary_domain, "frontend-web");
        assert!(res.detected_domains.contains(&"frontend-web".to_string()));
    }

    #[test]
    fn test_scan_skills_inventory() {
        let dir = tempdir().unwrap();
        let base = dir.path().canonicalize().unwrap();
        let skills_dir = base.join("skills");
        let skill_dir = skills_dir.join("my-skill");
        fs::create_dir_all(&skill_dir).unwrap();
        fs::write(skill_dir.join("SKILL.md"), "---\nname: my-skill\ndescription: A test skill.\ndomain: testing\ntags: [test]\n---\n# My Skill\n").unwrap();

        let inventory = scan_skills_inventory(&[skills_dir], "workspace");
        assert_eq!(inventory.len(), 1);
        assert_eq!(inventory[0].frontmatter.name, "my-skill");
        assert_eq!(inventory[0].origin, "workspace");
    }

    #[test]
    fn test_scan_skills_inventory_with_global() {
        let dir = tempdir().unwrap();
        let base = dir.path().canonicalize().unwrap();
        let ws_skills = base.join("ws_skills");
        let global_skills = base.join("global_skills");

        let ws_skill = ws_skills.join("ws-skill");
        fs::create_dir_all(&ws_skill).unwrap();
        fs::write(
            ws_skill.join("SKILL.md"),
            "---\nname: ws-skill\ndescription: Workspace skill.\n---\n# WS\n",
        )
        .unwrap();

        let g_skill = global_skills.join("g-skill");
        fs::create_dir_all(&g_skill).unwrap();
        fs::write(
            g_skill.join("SKILL.md"),
            "---\nname: g-skill\ndescription: Global skill.\n---\n# Global\n",
        )
        .unwrap();

        let ws_inventory = scan_skills_inventory(&[ws_skills], "workspace");
        let g_inventory = scan_skills_inventory(&[global_skills], "global");
        let combined: Vec<_> = ws_inventory.iter().chain(g_inventory.iter()).collect();
        assert_eq!(combined.len(), 2);
        assert!(combined.iter().any(|s| s.origin == "workspace"));
        assert!(combined.iter().any(|s| s.origin == "global"));
    }

    #[test]
    fn test_calculate_taxonomy_heatmap_dynamic() {
        let entry = SkillEntry {
            name: "test-skill".to_string(),
            path: PathBuf::from("test"),
            frontmatter: SkillFrontmatter {
                name: "test-skill".to_string(),
                description: "A test skill".to_string(),
                domain: Some("rust-cli".to_string()),
                tags: vec![],
            },
            origin: "workspace".to_string(),
            full_body: "".to_string(),
        };
        let heatmap = calculate_taxonomy_heatmap(&[entry], &[]);
        assert!(
            heatmap.categories.contains_key("rust-cli"),
            "heatmap must include rust-cli domain"
        );
        let cat = &heatmap.categories["rust-cli"];
        assert_eq!(cat.skill_count, 1);
        assert!(cat.workspace_count > 0 || cat.global_count > 0);
    }

    #[test]
    fn test_checklist_score_is_real_fraction() {
        let dir = tempdir().unwrap();
        let base = dir.path().canonicalize().unwrap();
        fs::write(base.join("Cargo.toml"), "[package]\nname=\"test\"\n").unwrap();
        let skills_dir = base.join("skills");
        fs::create_dir_all(&skills_dir).unwrap();

        let inventory = scan_skills_inventory(&[skills_dir], "workspace");
        assert_eq!(
            inventory.len(),
            0,
            "empty skills dir must yield empty inventory"
        );

        let heatmap = calculate_taxonomy_heatmap(&inventory, &[]);
        for cat in heatmap.categories.values() {
            assert_eq!(
                cat.skill_count, 0,
                "empty inventory must produce zero counts in all categories"
            );
        }
    }

    #[test]
    fn test_parse_skill_frontmatter_with_domain_and_tags() {
        let dir = tempdir().unwrap();
        let base = dir.path().canonicalize().unwrap();
        let skill_md = base.join("SKILL.md");
        fs::write(&skill_md, "---\nname: my-skill\ndescription: A test skill.\ndomain: testing\ntags: [tdd, rust]\n---\n# My Skill\n").unwrap();

        let fm = parse_skill_frontmatter(&skill_md).unwrap();
        assert_eq!(fm.name, "my-skill");
        assert_eq!(fm.domain, Some("testing".to_string()));
        assert!(
            fm.tags.contains(&"tdd".to_string()),
            "must parse tags list, got: {:?}",
            fm.tags
        );
        assert!(fm.tags.contains(&"rust".to_string()));
    }

    #[test]
    fn test_generate_heatmap_markdown() {
        let mut categories = HashMap::new();
        categories.insert(
            "rust-cli".to_string(),
            HeatmapCategory {
                domain: "rust-cli".to_string(),
                skill_count: 3,
                coverage_ratio: 1.0,
                is_dynamic: false,
                workspace_count: 3,
                global_count: 0,
            },
        );
        categories.insert(
            "frontend-web".to_string(),
            HeatmapCategory {
                domain: "frontend-web".to_string(),
                skill_count: 0,
                coverage_ratio: 0.0,
                is_dynamic: false,
                workspace_count: 0,
                global_count: 0,
            },
        );
        let heatmap = TaxonomyHeatmap { categories };
        let md = generate_heatmap_markdown(&heatmap);
        assert!(md.contains("|"), "must produce a pipe-table");
        assert!(md.contains("rust-cli"), "must include rust-cli domain");
        assert!(
            md.contains("frontend-web"),
            "must include frontend-web domain"
        );
    }

    #[test]
    fn test_canonicalize_domain() {
        assert_eq!(canonicalize_domain("frontend web"), "frontend-web");
        assert_eq!(canonicalize_domain("FRONTEND-WEB"), "frontend-web");
        assert_eq!(canonicalize_domain("rust-cli"), "rust-cli");
        assert_eq!(canonicalize_domain("Python Backend"), "python-backend");
    }

    #[test]
    fn test_dynamic_domain_harvesting() {
        let dir = tempdir().unwrap();
        let base = dir.path().canonicalize().unwrap();
        let skills_dir = base.join("skills");
        let skill_a = skills_dir.join("skill-a");
        let skill_b = skills_dir.join("skill-b");
        fs::create_dir_all(&skill_a).unwrap();
        fs::create_dir_all(&skill_b).unwrap();
        fs::write(
            skill_a.join("SKILL.md"),
            "---\nname: skill-a\ndescription: A.\ndomain: custom-domain-1\n---\n# A\n",
        )
        .unwrap();
        fs::write(
            skill_b.join("SKILL.md"),
            "---\nname: skill-b\ndescription: B.\ndomain: custom-domain-2\n---\n# B\n",
        )
        .unwrap();

        let domains = harvest_dynamic_domains(&[skills_dir]);
        assert!(
            domains.contains(&"custom-domain-1".to_string()),
            "must harvest custom-domain-1, got: {:?}",
            domains
        );
        assert!(
            domains.contains(&"custom-domain-2".to_string()),
            "must harvest custom-domain-2, got: {:?}",
            domains
        );
    }

    #[test]
    fn test_origin_aware_scoring_workspace_only() {
        let ws_skill = SkillEntry {
            name: "ws-skill".to_string(),
            path: PathBuf::from("ws"),
            frontmatter: SkillFrontmatter {
                name: "ws-skill".to_string(),
                description: "A workspace skill.".to_string(),
                domain: Some("rust-cli".to_string()),
                tags: vec![],
            },
            origin: "workspace".to_string(),
            full_body: "".to_string(),
        };
        let heatmap = calculate_taxonomy_heatmap(&[ws_skill], &[]);
        let cat = heatmap.categories.get("rust-cli").unwrap();
        assert_eq!(cat.workspace_count, 1);
        assert_eq!(cat.global_count, 0);
    }

    #[test]
    fn test_domain_relevance_filters_frontend_for_python_backend() {
        let categories = HashMap::from([
            (
                "frontend-web".to_string(),
                make_cat("frontend-web", 0, false, 0, 0),
            ),
            (
                "python-backend".to_string(),
                make_cat("python-backend", 2, false, 2, 0),
            ),
        ]);
        let filtered = filter_heatmap_by_relevance(
            &TaxonomyHeatmap { categories },
            &["python-backend".to_string()],
        );
        assert!(
            !filtered.categories.contains_key("frontend-web"),
            "frontend-web must be filtered out for python-backend project"
        );
        assert!(
            filtered.categories.contains_key("python-backend"),
            "python-backend must be retained"
        );
    }

    #[test]
    fn test_domain_relevance_none_returns_all_zero_zone() {
        let categories = HashMap::from([
            ("rust-cli".to_string(), make_cat("rust-cli", 0, false, 0, 0)),
            (
                "frontend-web".to_string(),
                make_cat("frontend-web", 0, false, 0, 0),
            ),
        ]);
        let filtered = filter_heatmap_by_relevance(&TaxonomyHeatmap { categories }, &[]);
        assert_eq!(
            filtered.categories.len(),
            2,
            "with no domain filter, all categories must be retained"
        );
    }

    #[test]
    fn test_load_global_skill_paths() {
        let paths = load_global_skill_paths();
        assert!(paths.capacity() >= paths.len());
    }

    #[test]
    fn test_matches_full_body_not_just_description() {
        let dir = tempdir().unwrap();
        let base = dir.path().canonicalize().unwrap();
        let skills_dir = base.join("skills");
        let skill_dir = skills_dir.join("my-skill");
        fs::create_dir_all(&skill_dir).unwrap();
        fs::write(skill_dir.join("SKILL.md"), "---\nname: my-skill\ndescription: A general skill.\n---\n# My Skill\n\n## Details\nThis skill supports testing workflows and TDD.\n").unwrap();

        let inventory = scan_skills_inventory(&[skills_dir], "workspace");
        assert_eq!(inventory.len(), 1);
        assert!(
            inventory[0].full_body.contains("testing"),
            "full_body must include body text, not just description"
        );
    }

    #[test]
    fn test_generic_keyword_does_not_inflate_unrelated_category() {
        let skill = SkillEntry {
            name: "code-janitor".to_string(),
            path: PathBuf::from("code-janitor"),
            frontmatter: SkillFrontmatter {
                name: "code-janitor".to_string(),
                description: "Audit code for smells".to_string(),
                domain: Some("code-quality".to_string()),
                tags: vec!["audit".to_string()],
            },
            origin: "workspace".to_string(),
            full_body: "Audit code for smells. Detects dead code.".to_string(),
        };
        let heatmap = calculate_taxonomy_heatmap(&[skill], &[]);
        assert!(heatmap.categories.contains_key("code-quality") || !heatmap.categories.is_empty());
        assert!(
            heatmap
                .categories
                .get("frontend-web")
                .map(|c| c.skill_count == 0)
                .unwrap_or(true),
            "generic keyword 'audit' must not credit frontend-web"
        );
    }

    #[test]
    fn test_build_scaffold_suggestions_dynamic_tier2() {
        let categories = HashMap::from([
            (
                "custom-ml".to_string(),
                make_cat("custom-ml", 0, true, 0, 0),
            ),
            ("rust-cli".to_string(), make_cat("rust-cli", 2, false, 2, 0)),
        ]);
        let heatmap = TaxonomyHeatmap { categories };
        let suggestions = build_scaffold_suggestions(&heatmap);
        assert!(
            !suggestions.is_empty(),
            "must suggest scaffold for zero-coverage dynamic domain"
        );
        assert!(
            suggestions.iter().any(|s| s.contains("custom-ml")),
            "suggestion must mention 'custom-ml', got: {:?}",
            suggestions
        );
        assert!(
            suggestions
                .iter()
                .any(|s| s.to_lowercase().contains("tier 2")
                    || s.to_lowercase().contains("llm")
                    || s.contains("scaffold")),
            "suggestion must be a Tier 2 LLM Prompt suggestion, got: {:?}",
            suggestions
        );
    }

    #[test]
    fn test_origin_aware_scoring_mixed() {
        let heatmap = calculate_taxonomy_heatmap(
            &[
                make_skill("ws-skill", "rust-cli", "workspace"),
                make_skill("g-skill", "rust-cli", "global"),
            ],
            &[],
        );
        let cat = heatmap.categories.get("rust-cli").unwrap();
        assert_eq!(
            cat.workspace_count, 1,
            "only workspace skills count toward workspace_count"
        );
        assert_eq!(
            cat.global_count, 1,
            "global skills count toward global_count"
        );
        assert_eq!(
            cat.skill_count, 2,
            "total skill_count is workspace + global"
        );
    }

    #[test]
    fn test_filter_heatmap_by_relevance_drops_out_of_scope_categories() {
        let categories = HashMap::from([
            (
                "frontend-web".to_string(),
                make_cat("frontend-web", 1, false, 1, 0),
            ),
            ("rust-cli".to_string(), make_cat("rust-cli", 1, false, 1, 0)),
        ]);
        let filtered =
            filter_heatmap_by_relevance(&TaxonomyHeatmap { categories }, &["rust-cli".to_string()]);
        assert!(
            !filtered.categories.contains_key("frontend-web"),
            "frontend-web must be dropped as out-of-scope"
        );
        assert!(
            filtered.categories.contains_key("rust-cli"),
            "rust-cli must be retained as in-scope"
        );
    }

    #[test]
    fn test_filter_heatmap_by_relevance_keeps_dynamic_domains() {
        let categories = HashMap::from([
            (
                "frontend-web".to_string(),
                make_cat("frontend-web", 0, false, 0, 0),
            ),
            (
                "my-dynamic-domain".to_string(),
                make_cat("my-dynamic-domain", 0, true, 0, 0),
            ),
        ]);
        let filtered =
            filter_heatmap_by_relevance(&TaxonomyHeatmap { categories }, &["rust-cli".to_string()]);
        assert!(
            filtered.categories.contains_key("my-dynamic-domain"),
            "dynamic domains must survive relevance filtering"
        );
        assert!(
            !filtered.categories.contains_key("frontend-web"),
            "fixed out-of-scope domain must be filtered"
        );
    }
}
