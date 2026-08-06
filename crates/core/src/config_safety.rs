use crate::path_safety::get_repo_root;
use std::fs;
use std::path::Path;

/// Loads repository-level `skills.config.yaml` (Tier 3).
pub fn load_repo_config(repo_root: Option<&Path>) -> serde_yaml::Value {
    let root = repo_root
        .map(|p| p.to_path_buf())
        .unwrap_or_else(|| get_repo_root(None));
    let config_path = root.join("skills.config.yaml");

    if config_path.is_file() {
        if let Ok(content) = fs::read_to_string(&config_path) {
            if let Ok(yaml) = serde_yaml::from_str(&content) {
                return yaml;
            }
        }
    }

    serde_yaml::Value::Mapping(serde_yaml::Mapping::new())
}

/// Merges secondary YAML mapping into primary YAML mapping.
/// Primary mapping values override secondary values.
fn merge_yaml_mappings(
    primary: serde_yaml::Value,
    secondary: serde_yaml::Value,
) -> serde_yaml::Value {
    match (primary, secondary) {
        (serde_yaml::Value::Mapping(mut p_map), serde_yaml::Value::Mapping(s_map)) => {
            for (k, v) in s_map {
                if !p_map.contains_key(&k) {
                    p_map.insert(k, v);
                } else {
                    let existing_v = p_map.remove(&k).unwrap();
                    let merged_v = merge_yaml_mappings(existing_v, v);
                    p_map.insert(k, merged_v);
                }
            }
            serde_yaml::Value::Mapping(p_map)
        }
        (p, _s) => p,
    }
}

/// Loads and merges skill configuration following ADR 0005 & ADR 0006 4-tier hierarchy.
/// Evaluates:
/// - Tier 2: `skill_dir/config.yaml` or legacy `skill_dir/<skill_name>.config.yaml`
/// - Tier 3: `repo_root/skills.config.yaml`
/// - Tier 4: Embedded defaults passed as fallback.
pub fn load_skill_config(
    skill_name: &str,
    skill_dir: Option<&Path>,
    repo_root: Option<&Path>,
    defaults: Option<serde_yaml::Value>,
) -> serde_yaml::Value {
    let root = repo_root
        .map(|p| p.to_path_buf())
        .unwrap_or_else(|| get_repo_root(None));

    let s_dir = match skill_dir {
        Some(d) => d.to_path_buf(),
        None => root.join("skills").join(skill_name),
    };

    let primary_config = s_dir.join("config.yaml");
    let legacy_config = s_dir.join(format!("{skill_name}.config.yaml"));

    let mut tier2_config = serde_yaml::Value::Mapping(serde_yaml::Mapping::new());

    if primary_config.is_file() {
        if let Ok(content) = fs::read_to_string(&primary_config) {
            if let Ok(parsed) = serde_yaml::from_str(&content) {
                tier2_config = parsed;
            }
        }
    } else if legacy_config.is_file() {
        if let Ok(content) = fs::read_to_string(&legacy_config) {
            if let Ok(parsed) = serde_yaml::from_str(&content) {
                tier2_config = parsed;
            }
        }
    }

    let tier3_config = load_repo_config(Some(&root));
    let tier4_config =
        defaults.unwrap_or_else(|| serde_yaml::Value::Mapping(serde_yaml::Mapping::new()));

    // Merge in priority order: Tier 2 > Tier 3 > Tier 4
    let merged_tier3_4 = merge_yaml_mappings(tier3_config, tier4_config);
    merge_yaml_mappings(tier2_config, merged_tier3_4)
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn test_4tier_config_hierarchy_strict_assertions() {
        let dir = tempdir().unwrap();
        let root = dir.path().canonicalize().unwrap();

        // Create repo config (Tier 3)
        let repo_config_path = root.join("skills.config.yaml");
        fs::write(
            &repo_config_path,
            "global_setting: repo_value\nshared_flag: from_repo\n",
        )
        .unwrap();

        // Create skill dir with config (Tier 2)
        let skill_dir = root.join("skills").join("test-skill");
        fs::create_dir_all(&skill_dir).unwrap();
        let skill_config_path = skill_dir.join("config.yaml");
        fs::write(
            &skill_config_path,
            "skill_setting: skill_value\nshared_flag: from_skill\n",
        )
        .unwrap();

        // Tier 4 defaults
        let mut defaults_map = serde_yaml::Mapping::new();
        defaults_map.insert(
            serde_yaml::Value::String("default_only".to_string()),
            serde_yaml::Value::String("from_default".to_string()),
        );
        defaults_map.insert(
            serde_yaml::Value::String("shared_flag".to_string()),
            serde_yaml::Value::String("from_default".to_string()),
        );

        let merged = load_skill_config(
            "test-skill",
            Some(&skill_dir),
            Some(&root),
            Some(serde_yaml::Value::Mapping(defaults_map)),
        );

        let map = merged.as_mapping().expect("Config must be a YAML mapping");

        // Strict assertions on key presence, values, and types
        let shared_val = map
            .get("shared_flag")
            .expect("shared_flag must exist")
            .as_str()
            .expect("shared_flag must be string");
        assert_eq!(shared_val, "from_skill"); // Tier 2 overrides Tier 3 & Tier 4

        let global_val = map
            .get("global_setting")
            .expect("global_setting must exist")
            .as_str()
            .expect("global_setting must be string");
        assert_eq!(global_val, "repo_value");

        let skill_val = map
            .get("skill_setting")
            .expect("skill_setting must exist")
            .as_str()
            .expect("skill_setting must be string");
        assert_eq!(skill_val, "skill_value");

        let default_val = map
            .get("default_only")
            .expect("default_only must exist")
            .as_str()
            .expect("default_only must be string");
        assert_eq!(default_val, "from_default");
    }
}
