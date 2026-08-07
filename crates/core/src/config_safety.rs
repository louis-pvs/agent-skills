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

    // --- NEW FAILING TESTS (TDD RED phase) ---

    #[test]
    fn test_parse_simple_yaml_basic() {
        // Python: test_parse_simple_yaml_basic — load_repo_config parses scalar values correctly
        let dir = tempdir().unwrap();
        let root = dir.path().canonicalize().unwrap();

        fs::write(
            root.join("skills.config.yaml"),
            "key: \"value/with/slashes\"\nquoted: 'single'\n",
        )
        .unwrap();

        let cfg = load_repo_config(Some(&root));
        let map = cfg.as_mapping().expect("must be a mapping");

        let key_val = map
            .get("key")
            .and_then(|v| v.as_str())
            .expect("key must be a string");
        assert_eq!(
            key_val, "value/with/slashes",
            "slash-containing value must be parsed correctly"
        );

        let quoted_val = map
            .get("quoted")
            .and_then(|v| v.as_str())
            .expect("quoted must be a string");
        assert_eq!(quoted_val, "single");
    }

    #[test]
    fn test_parse_simple_yaml_scalar_list() {
        // Python: test_parse_simple_yaml_scalar_list — load_repo_config handles list-of-strings value
        let dir = tempdir().unwrap();
        let root = dir.path().canonicalize().unwrap();

        fs::write(
            root.join("skills.config.yaml"),
            "items:\n  - alpha\n  - beta\n  - gamma\n",
        )
        .unwrap();

        let cfg = load_repo_config(Some(&root));
        let map = cfg.as_mapping().expect("must be a mapping");

        let items = map
            .get("items")
            .and_then(|v| v.as_sequence())
            .expect("items must be a sequence");
        assert_eq!(items.len(), 3, "must have 3 items");
        assert_eq!(items[0].as_str().unwrap(), "alpha");
        assert_eq!(items[1].as_str().unwrap(), "beta");
        assert_eq!(items[2].as_str().unwrap(), "gamma");
    }

    #[test]
    fn test_parse_simple_yaml_nested_sections() {
        // Python: test_parse_simple_yaml_nested_sections — multi-level nesting and type coercion
        let dir = tempdir().unwrap();
        let root = dir.path().canonicalize().unwrap();

        fs::write(
            root.join("skills.config.yaml"),
            "outer:\n  inner:\n    value: 42\n    flag: true\n",
        )
        .unwrap();

        let cfg = load_repo_config(Some(&root));
        let map = cfg.as_mapping().expect("must be a mapping");

        let outer = map
            .get("outer")
            .and_then(|v| v.as_mapping())
            .expect("outer must be a mapping");
        let inner = outer
            .get("inner")
            .and_then(|v| v.as_mapping())
            .expect("inner must be a mapping");

        let value = inner
            .get("value")
            .and_then(|v| v.as_i64())
            .expect("value must be integer 42");
        assert_eq!(value, 42);

        let flag = inner
            .get("flag")
            .and_then(|v| v.as_bool())
            .expect("flag must be boolean true");
        assert!(flag);
    }

    #[test]
    fn test_load_yaml_file_nonexistent() {
        // Python: test_load_yaml_file_nonexistent — load_repo_config with missing file returns empty mapping
        let dir = tempdir().unwrap();
        let root = dir.path().canonicalize().unwrap();
        // No skills.config.yaml written — must not panic

        let cfg = load_repo_config(Some(&root));
        let map = cfg
            .as_mapping()
            .expect("must return empty mapping for missing file");
        assert!(
            map.is_empty(),
            "missing config file must return empty mapping, not error or panic"
        );
    }

    #[test]
    fn test_load_skill_config_legacy_fallback() {
        // Python: test_load_skill_config_legacy_fallback — reads <skill>.config.yaml when config.yaml absent
        let dir = tempdir().unwrap();
        let root = dir.path().canonicalize().unwrap();

        let skill_dir = root.join("skills").join("my-skill");
        fs::create_dir_all(&skill_dir).unwrap();

        // Write legacy config (no config.yaml, only my-skill.config.yaml)
        let legacy_path = skill_dir.join("my-skill.config.yaml");
        fs::write(&legacy_path, "legacy_key: legacy_value\n").unwrap();

        let cfg = load_skill_config("my-skill", Some(&skill_dir), Some(&root), None);
        let map = cfg.as_mapping().expect("must be a mapping");

        let val = map
            .get("legacy_key")
            .and_then(|v| v.as_str())
            .expect("legacy_key must exist when loaded from legacy config path");
        assert_eq!(
            val, "legacy_value",
            "load_skill_config must fall back to <skill>.config.yaml when config.yaml is absent"
        );
    }
}
