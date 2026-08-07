use crate::error::CoreError;
use crate::path_safety::sanitize_path;
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet, VecDeque};
use std::fs;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct SkillNode {
    pub name: String,
    pub version: String,
    pub requires: Vec<String>,
    pub enhances: Vec<String>,
    pub transitive_requires: Vec<String>,
    pub transitive_enhances: Vec<String>,
    pub path: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Lockfile {
    pub version: String,
    pub generated_by: String,
    pub topological_order: Vec<String>,
    pub skills: BTreeMap<String, SkillNode>,
}

#[derive(Debug, Clone)]
pub struct RawSkillMeta {
    pub name: String,
    pub version: String,
    pub description: String,
    pub requires: Vec<String>,
    pub enhances: Vec<String>,
    pub dir_path: PathBuf,
    pub rel_path: String,
}

pub fn parse_skill_frontmatter(skill_md_path: &Path) -> Result<RawSkillMeta, CoreError> {
    let content = fs::read_to_string(skill_md_path).map_err(|e| CoreError::Io {
        path: skill_md_path.to_path_buf(),
        source: e,
    })?;

    if !content.starts_with("---") {
        return Err(CoreError::YamlParse {
            file: skill_md_path.display().to_string(),
            message: "SKILL.md missing YAML frontmatter.".to_string(),
        });
    }

    let parts: Vec<&str> = content.split("---").collect();
    if parts.len() < 3 {
        return Err(CoreError::YamlParse {
            file: skill_md_path.display().to_string(),
            message: "YAML frontmatter not closed.".to_string(),
        });
    }

    let yaml_block = parts[1];
    let yaml: serde_yaml::Value =
        serde_yaml::from_str(yaml_block).map_err(|e| CoreError::YamlParse {
            file: skill_md_path.display().to_string(),
            message: e.to_string(),
        })?;

    let map = yaml.as_mapping().ok_or_else(|| CoreError::YamlParse {
        file: skill_md_path.display().to_string(),
        message: "YAML frontmatter is not a mapping.".to_string(),
    })?;

    let name = map
        .get(serde_yaml::Value::String("name".to_string()))
        .and_then(|v| v.as_str())
        .map(|s| s.trim().to_string())
        .unwrap_or_else(|| {
            skill_md_path
                .parent()
                .and_then(|p| p.file_name())
                .and_then(|n| n.to_str())
                .unwrap_or("unknown")
                .to_string()
        });

    let version = map
        .get(serde_yaml::Value::String("version".to_string()))
        .and_then(|v| v.as_str())
        .map(|s| s.trim().to_string())
        .unwrap_or_else(|| "1.0.0".to_string());

    let parse_string_list = |key: &str| -> Vec<String> {
        let mut list = Vec::new();
        if let Some(val) = map.get(serde_yaml::Value::String(key.to_string())) {
            if let Some(seq) = val.as_sequence() {
                for item in seq {
                    if let Some(s) = item.as_str() {
                        list.push(s.trim().to_string());
                    }
                }
            } else if let Some(s) = val.as_str() {
                for item in s.split(',') {
                    let trimmed = item.trim();
                    if !trimmed.is_empty() {
                        list.push(trimmed.to_string());
                    }
                }
            }
        }
        list
    };

    let description = map
        .get(serde_yaml::Value::String("description".to_string()))
        .and_then(|v| v.as_str())
        .map(|s| s.trim().to_string())
        .unwrap_or_default();

    let requires = parse_string_list("requires");
    let enhances = parse_string_list("enhances");
    let dir_path = skill_md_path
        .parent()
        .unwrap_or(skill_md_path)
        .to_path_buf();
    let rel_path = format!(
        "skills/{}",
        dir_path.file_name().unwrap_or_default().to_string_lossy()
    );

    Ok(RawSkillMeta {
        name,
        version,
        description,
        requires,
        enhances,
        dir_path,
        rel_path,
    })
}

pub fn scan_skills_directory(
    skills_dir: &Path,
) -> Result<BTreeMap<String, RawSkillMeta>, CoreError> {
    let sanitized_dir = sanitize_path(skills_dir, None)?;
    if !sanitized_dir.is_dir() {
        return Err(CoreError::Other(format!(
            "Skills path is not a directory: {}",
            sanitized_dir.display()
        )));
    }

    let mut map = BTreeMap::new();
    let entries = fs::read_dir(&sanitized_dir).map_err(|e| CoreError::Io {
        path: sanitized_dir.clone(),
        source: e,
    })?;

    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() {
            let skill_md = path.join("SKILL.md");
            if skill_md.is_file() {
                if let Ok(meta) = parse_skill_frontmatter(&skill_md) {
                    map.insert(meta.name.clone(), meta);
                }
            }
        }
    }

    Ok(map)
}

pub fn build_topological_order(
    raw_skills: &BTreeMap<String, RawSkillMeta>,
) -> Result<Vec<String>, CoreError> {
    let mut in_degree: BTreeMap<String, usize> = BTreeMap::new();
    let mut graph: BTreeMap<String, BTreeSet<String>> = BTreeMap::new();

    for name in raw_skills.keys() {
        in_degree.insert(name.clone(), 0);
        graph.insert(name.clone(), BTreeSet::new());
    }

    for (name, meta) in raw_skills {
        for req in &meta.requires {
            if raw_skills.contains_key(req) && graph.get_mut(req).unwrap().insert(name.clone()) {
                *in_degree.get_mut(name).unwrap() += 1;
            }
        }
    }

    let mut queue: VecDeque<String> = in_degree
        .iter()
        .filter(|(_, &deg)| deg == 0)
        .map(|(name, _)| name.clone())
        .collect();

    let mut topo = Vec::new();

    while let Some(node) = queue.pop_front() {
        topo.push(node.clone());
        if let Some(neighbors) = graph.get(&node) {
            for neighbor in neighbors {
                let deg = in_degree.get_mut(neighbor).unwrap();
                *deg -= 1;
                if *deg == 0 {
                    queue.push_back(neighbor.clone());
                }
            }
        }
    }

    if topo.len() != raw_skills.len() {
        return Err(CoreError::Depgraph(
            "Circular dependency detected in skill requirement graph.".to_string(),
        ));
    }

    Ok(topo)
}

pub fn compute_transitive_relations(
    raw_skills: &BTreeMap<String, RawSkillMeta>,
) -> BTreeMap<String, (Vec<String>, Vec<String>)> {
    let mut result = BTreeMap::new();

    for (name, meta) in raw_skills {
        let mut trans_req = BTreeSet::new();
        let mut req_queue: VecDeque<String> = meta.requires.iter().cloned().collect();
        while let Some(req) = req_queue.pop_front() {
            if trans_req.insert(req.clone()) {
                if let Some(parent_meta) = raw_skills.get(&req) {
                    for child in &parent_meta.requires {
                        req_queue.push_back(child.clone());
                    }
                }
            }
        }

        let mut trans_enh = BTreeSet::new();
        let mut enh_queue: VecDeque<String> = meta.enhances.iter().cloned().collect();
        while let Some(enh) = enh_queue.pop_front() {
            if trans_enh.insert(enh.clone()) {
                if let Some(parent_meta) = raw_skills.get(&enh) {
                    for child in &parent_meta.enhances {
                        enh_queue.push_back(child.clone());
                    }
                }
            }
        }

        let req_vec: Vec<String> = trans_req.into_iter().collect();
        let enh_vec: Vec<String> = trans_enh.into_iter().collect();
        result.insert(name.clone(), (req_vec, enh_vec));
    }

    result
}

pub fn generate_lockfile(skills_dir: &Path, lockfile_path: &Path) -> Result<Lockfile, CoreError> {
    let raw_skills = scan_skills_directory(skills_dir)?;
    let topo_order = build_topological_order(&raw_skills)?;
    let transitive_map = compute_transitive_relations(&raw_skills);

    let mut skill_nodes = BTreeMap::new();
    for (name, meta) in raw_skills {
        let (t_req, t_enh) = transitive_map.get(&name).cloned().unwrap_or_default();
        let node = SkillNode {
            name: meta.name,
            version: meta.version,
            requires: meta.requires,
            enhances: meta.enhances,
            transitive_requires: t_req,
            transitive_enhances: t_enh,
            path: meta.rel_path,
        };
        skill_nodes.insert(name, node);
    }

    let lockfile = Lockfile {
        version: "1.0.0".to_string(),
        generated_by: "agent-skills depgraph".to_string(),
        topological_order: topo_order,
        skills: skill_nodes,
    };

    let json_str = serde_json::to_string_pretty(&lockfile)
        .map_err(|e| CoreError::Other(format!("Failed to serialize lockfile: {e}")))?;

    fs::write(lockfile_path, json_str).map_err(|e| CoreError::Io {
        path: lockfile_path.to_path_buf(),
        source: e,
    })?;

    Ok(lockfile)
}

pub fn verify_graph(skills_dir: &Path, lockfile_path: &Path) -> (bool, Vec<String>, Vec<String>) {
    let mut errors = Vec::new();
    let mut warnings = Vec::new();

    let raw_skills = match scan_skills_directory(skills_dir) {
        Ok(s) => s,
        Err(e) => return (false, vec![e.to_string()], warnings),
    };

    if let Err(e) = build_topological_order(&raw_skills) {
        errors.push(e.to_string());
    }

    if !lockfile_path.exists() {
        errors.push(format!(
            "Lockfile missing at '{}'.",
            lockfile_path.display()
        ));
        return (false, errors, warnings);
    }

    let content = match fs::read_to_string(lockfile_path) {
        Ok(c) => c,
        Err(e) => {
            errors.push(format!("Failed to read lockfile: {e}"));
            return (false, errors, warnings);
        }
    };

    let lockfile: Lockfile = match serde_json::from_str(&content) {
        Ok(l) => l,
        Err(e) => {
            errors.push(format!("Invalid JSON lockfile: {e}"));
            return (false, errors, warnings);
        }
    };

    for (name, meta) in &raw_skills {
        match lockfile.skills.get(name) {
            None => errors.push(format!(
                "Skill '{name}' present in skills/ but missing in skills.lock."
            )),
            Some(node) => {
                if node.requires != meta.requires {
                    warnings.push(format!(
                        "Skill '{name}' requirements changed since lockfile generation."
                    ));
                }
            }
        }
    }

    let is_valid = errors.is_empty();
    (is_valid, errors, warnings)
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn test_depgraph_topological_sort_strict_assertions() {
        let dir = tempdir().unwrap();
        let skills_dir = dir.path().join("skills");
        fs::create_dir_all(&skills_dir).unwrap();

        // Skill A (no deps)
        let skill_a = skills_dir.join("skill-a");
        fs::create_dir_all(&skill_a).unwrap();
        fs::write(
            skill_a.join("SKILL.md"),
            "---\nname: skill-a\n---\n# Skill A\n## Completion Criteria\n- [ ] Done\n",
        )
        .unwrap();

        // Skill B (requires skill-a)
        let skill_b = skills_dir.join("skill-b");
        fs::create_dir_all(&skill_b).unwrap();
        fs::write(
            skill_b.join("SKILL.md"),
            "---\nname: skill-b\nrequires: [skill-a]\n---\n# Skill B\n## Completion Criteria\n- [ ] Done\n",
        )
        .unwrap();

        let lockfile_path = dir.path().join("skills.lock");
        let lockfile = generate_lockfile(&skills_dir, &lockfile_path).unwrap();

        assert_eq!(lockfile.topological_order.len(), 2);
        assert_eq!(lockfile.topological_order[0], "skill-a");
        assert_eq!(lockfile.topological_order[1], "skill-b");

        let b_node = lockfile
            .skills
            .get("skill-b")
            .expect("skill-b node must exist");
        assert_eq!(b_node.requires, vec!["skill-a".to_string()]);
        assert_eq!(b_node.transitive_requires, vec!["skill-a".to_string()]);

        let (is_valid, errors, warnings) = verify_graph(&skills_dir, &lockfile_path);
        assert!(is_valid);
        assert_eq!(errors, Vec::<String>::new());
        assert_eq!(warnings, Vec::<String>::new());
    }

    // --- NEW FAILING TEST (TDD RED phase) ---

    #[test]
    fn test_topological_sort_cycle() {
        // Python: test_topological_sort_cycle — build_topological_order must error on circular deps
        let mut raw = BTreeMap::new();

        // skill-a requires skill-b, skill-b requires skill-a → cycle
        raw.insert(
            "skill-a".to_string(),
            RawSkillMeta {
                name: "skill-a".to_string(),
                version: "1.0.0".to_string(),
                description: String::new(),
                requires: vec!["skill-b".to_string()],
                enhances: vec![],
                dir_path: std::path::PathBuf::from("/tmp/skill-a"),
                rel_path: "skills/skill-a".to_string(),
            },
        );
        raw.insert(
            "skill-b".to_string(),
            RawSkillMeta {
                name: "skill-b".to_string(),
                version: "1.0.0".to_string(),
                description: String::new(),
                requires: vec!["skill-a".to_string()],
                enhances: vec![],
                dir_path: std::path::PathBuf::from("/tmp/skill-b"),
                rel_path: "skills/skill-b".to_string(),
            },
        );

        let result = build_topological_order(&raw);
        assert!(
            result.is_err(),
            "build_topological_order must return Err on circular dependency, got Ok"
        );
        let err_msg = result.unwrap_err().to_string();
        assert!(
            err_msg.to_lowercase().contains("circular") || err_msg.to_lowercase().contains("cycle"),
            "error message must mention circular dependency, got: {err_msg}"
        );
    }

    #[test]
    fn test_parse_frontmatter_description_field() {
        // Python: test_parse_frontmatter — version/enhances/description extraction,
        // specifically the description field which previously had no home on RawSkillMeta.
        let dir = tempdir().unwrap();
        let skill_dir = dir.path().join("skill-x");
        fs::create_dir_all(&skill_dir).unwrap();
        fs::write(
            skill_dir.join("SKILL.md"),
            "---\nname: skill-x\nversion: 2.3.4\ndescription: Does the thing.\nenhances: [skill-y]\n---\n# Skill X\n",
        )
        .unwrap();

        let meta = parse_skill_frontmatter(&skill_dir.join("SKILL.md")).unwrap();
        assert_eq!(meta.version, "2.3.4");
        assert_eq!(meta.description, "Does the thing.");
        assert_eq!(meta.enhances, vec!["skill-y".to_string()]);
    }

    #[test]
    fn test_transitive_deps_multihop() {
        // Python: test_transitive_deps — multi-hop chain (C requires B requires A),
        // C's transitive_requires must include both A and B, not just the direct parent.
        let mut raw = BTreeMap::new();
        let meta = |name: &str, requires: Vec<&str>| RawSkillMeta {
            name: name.to_string(),
            version: "1.0.0".to_string(),
            description: String::new(),
            requires: requires.into_iter().map(String::from).collect(),
            enhances: vec![],
            dir_path: std::path::PathBuf::from(format!("/tmp/{name}")),
            rel_path: format!("skills/{name}"),
        };
        raw.insert("skill-a".to_string(), meta("skill-a", vec![]));
        raw.insert("skill-b".to_string(), meta("skill-b", vec!["skill-a"]));
        raw.insert("skill-c".to_string(), meta("skill-c", vec!["skill-b"]));

        let transitive = compute_transitive_relations(&raw);
        let (c_req, _) = transitive.get("skill-c").unwrap();
        assert!(
            c_req.contains(&"skill-a".to_string()),
            "skill-c's transitive_requires must include skill-a (2 hops away), got: {c_req:?}"
        );
        assert!(
            c_req.contains(&"skill-b".to_string()),
            "skill-c's transitive_requires must include skill-b (1 hop away), got: {c_req:?}"
        );
    }

    #[test]
    fn test_lockfile_verify_flags_drift_after_frontmatter_change() {
        // Python: test_lockfile_generation_and_verification — the "flags out-of-sync drift"
        // half: generate a lockfile, then mutate a skill's requires after the fact, and
        // confirm verify_graph surfaces a warning about the drift.
        let dir = tempdir().unwrap();
        let skills_dir = dir.path().join("skills");
        fs::create_dir_all(&skills_dir).unwrap();

        let skill_a = skills_dir.join("skill-a");
        fs::create_dir_all(&skill_a).unwrap();
        fs::write(
            skill_a.join("SKILL.md"),
            "---\nname: skill-a\n---\n# Skill A\n",
        )
        .unwrap();

        let skill_b = skills_dir.join("skill-b");
        fs::create_dir_all(&skill_b).unwrap();
        fs::write(
            skill_b.join("SKILL.md"),
            "---\nname: skill-b\n---\n# Skill B\n",
        )
        .unwrap();

        let lockfile_path = dir.path().join("skills.lock");
        generate_lockfile(&skills_dir, &lockfile_path).unwrap();

        // Drift: skill-b now requires skill-a, but the lockfile was generated before this change.
        fs::write(
            skill_b.join("SKILL.md"),
            "---\nname: skill-b\nrequires: [skill-a]\n---\n# Skill B\n",
        )
        .unwrap();

        let (_is_valid, _errors, warnings) = verify_graph(&skills_dir, &lockfile_path);
        assert!(
            !warnings.is_empty(),
            "verify_graph must warn when a skill's requires drifted since lockfile generation"
        );
    }
}
