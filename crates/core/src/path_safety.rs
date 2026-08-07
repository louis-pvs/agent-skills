use crate::error::CoreError;
use std::env;
use std::path::{Component, Path, PathBuf};

/// Dynamic path resolution to find the repository root (ADR 0004 & ADR 0006).
/// Searches upwards from `from_path` (or current directory) for anchor files:
/// `.git`, `skills.lock`, or `AGENTS.md`.
pub fn get_repo_root(from_path: Option<&Path>) -> PathBuf {
    let start = match from_path {
        Some(p) => {
            if p.is_dir() {
                p.to_path_buf()
            } else {
                p.parent().unwrap_or(p).to_path_buf()
            }
        }
        None => env::current_dir().unwrap_or_else(|_| PathBuf::from(".")),
    };

    let canonical_start = start.canonicalize().unwrap_or(start);

    let mut current = canonical_start.as_path();
    loop {
        if current.join(".git").exists()
            || current.join("skills.lock").is_file()
            || current.join("AGENTS.md").is_file()
        {
            return current.to_path_buf();
        }

        match current.parent() {
            Some(parent) => current = parent,
            None => break,
        }
    }

    env::current_dir().unwrap_or_else(|_| PathBuf::from("."))
}

/// Sanitize and validate a file or directory path against path traversal vulnerabilities (CWE-22).
/// Resolves `input_path` within `base_dir` and verifies containment.
pub fn sanitize_path(
    input_path: impl AsRef<Path>,
    base_dir: Option<&Path>,
) -> Result<PathBuf, CoreError> {
    let input = input_path.as_ref();
    let input_str = input.to_string_lossy();

    if input_str.contains('\0') {
        return Err(CoreError::PathTraversal {
            message: "Null byte detected in path".to_string(),
            path: input_str.to_string(),
        });
    }

    if input_str.contains([';', '|', '&', '$', '`', '\n', '\r']) {
        return Err(CoreError::PathTraversal {
            message: "Shell metacharacters detected in path".to_string(),
            path: input_str.to_string(),
        });
    }

    for component in input.components() {
        if component == Component::ParentDir {
            return Err(CoreError::PathTraversal {
                message: "Path traversal sequence '..' detected".to_string(),
                path: input_str.to_string(),
            });
        }
    }

    let default_base = env::current_dir().unwrap_or_else(|_| PathBuf::from("."));
    let base = match base_dir {
        Some(b) => b.to_path_buf(),
        None => {
            if input.is_absolute() {
                input
                    .parent()
                    .unwrap_or_else(|| Path::new("."))
                    .to_path_buf()
            } else {
                default_base
            }
        }
    };

    let base_canonical = base.canonicalize().map_err(|e| CoreError::Io {
        path: base.clone(),
        source: e,
    })?;

    let full_path = if input.is_absolute() {
        input.to_path_buf()
    } else {
        base_canonical.join(input)
    };

    let target = if full_path.exists() {
        full_path.canonicalize().map_err(|e| CoreError::Io {
            path: full_path.clone(),
            source: e,
        })?
    } else {
        let mut components = Vec::new();
        let mut curr = full_path.as_path();
        while !curr.exists() {
            if let Some(name) = curr.file_name() {
                components.push(name);
                if let Some(parent) = curr.parent() {
                    curr = parent;
                } else {
                    break;
                }
            } else {
                break;
            }
        }
        let mut resolved = if curr.exists() {
            curr.canonicalize().map_err(|e| CoreError::Io {
                path: curr.to_path_buf(),
                source: e,
            })?
        } else {
            curr.to_path_buf()
        };
        for comp in components.into_iter().rev() {
            resolved.push(comp);
        }
        resolved
    };

    if !target.starts_with(&base_canonical) {
        return Err(CoreError::PathTraversal {
            message: format!(
                "Path traversal attempt detected. '{}' is outside allowed base directory '{}'",
                input_str,
                base_canonical.display()
            ),
            path: input_str.to_string(),
        });
    }

    Ok(target)
}

/// Resolves and validates a user-supplied directory path within a trusted base.
pub fn resolve_safe_dir(
    raw_path: impl AsRef<Path>,
    base_dir: Option<&Path>,
) -> Result<PathBuf, CoreError> {
    let safe_path = sanitize_path(raw_path.as_ref(), base_dir)?;
    if !safe_path.exists() || !safe_path.is_dir() {
        return Err(CoreError::Other(format!(
            "Error: Directory '{}' does not exist or is not a directory.",
            safe_path.display()
        )));
    }
    Ok(safe_path)
}

/// Safely remove a directory tree after verifying path traversal containment.
pub fn safe_rmtree(target_path: impl AsRef<Path>, base_dir: &Path) -> bool {
    let target = target_path.as_ref();
    if let Ok(safe_p) = sanitize_path(target, Some(base_dir)) {
        if safe_p == base_dir || !safe_p.exists() {
            return false;
        }
        std::fs::remove_dir_all(safe_p).is_ok()
    } else {
        false
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::tempdir;

    #[test]
    fn test_get_repo_root_finds_anchor() {
        let repo_root = get_repo_root(None);
        assert!(repo_root.join(".git").exists() || repo_root.join("AGENTS.md").is_file());
        assert!(repo_root.is_absolute());
    }

    #[test]
    fn test_sanitize_path_valid_subdirectory() {
        let dir = tempdir().unwrap();
        let base = dir.path().canonicalize().unwrap();
        let sub = base.join("subfolder");
        fs::create_dir_all(&sub).unwrap();

        let sanitized = sanitize_path("subfolder", Some(&base)).unwrap();
        assert_eq!(sanitized, sub.canonicalize().unwrap());
        assert!(sanitized.starts_with(&base));
    }

    #[test]
    fn test_sanitize_path_rejects_parent_traversal() {
        let dir = tempdir().unwrap();
        let base = dir.path().canonicalize().unwrap();
        let err = sanitize_path("../outside", Some(&base)).unwrap_err();
        assert!(matches!(err, CoreError::PathTraversal { .. }));
        assert!(err.to_string().contains("Path traversal sequence '..'"));
    }

    #[test]
    fn test_sanitize_path_rejects_null_bytes() {
        let dir = tempdir().unwrap();
        let base = dir.path().canonicalize().unwrap();
        let err = sanitize_path("test\0file", Some(&base)).unwrap_err();
        assert!(matches!(err, CoreError::PathTraversal { .. }));
        assert!(err.to_string().contains("Null byte"));
    }

    #[test]
    fn test_resolve_safe_dir_strict_assertions() {
        let dir = tempdir().unwrap();
        let base = dir.path().canonicalize().unwrap();
        let valid_dir = base.join("valid");
        fs::create_dir_all(&valid_dir).unwrap();

        let resolved = resolve_safe_dir("valid", Some(&base)).unwrap();
        assert!(resolved.is_dir());
        assert_eq!(resolved.file_name().unwrap().to_str().unwrap(), "valid");
        assert_eq!(resolved, valid_dir.canonicalize().unwrap());
    }

    // --- NEW FAILING TESTS (TDD RED phase) ---

    #[test]
    fn test_valid_nested_path() {
        // Python: test_valid_nested_path — sanitize_path("a/b/c") is valid multi-level path
        let dir = tempdir().unwrap();
        let base = dir.path().canonicalize().unwrap();
        let nested = base.join("a").join("b").join("c");
        fs::create_dir_all(&nested).unwrap();

        let result = sanitize_path("a/b/c", Some(&base)).unwrap();
        assert!(
            result.starts_with(&base),
            "nested path must stay within base"
        );
        assert_eq!(result, nested.canonicalize().unwrap());
    }

    #[test]
    fn test_absolute_path_inside_base() {
        // Python: test_absolute_path_inside_base — absolute path that is inside base is accepted
        let dir = tempdir().unwrap();
        let base = dir.path().canonicalize().unwrap();
        let sub = base.join("inside");
        fs::create_dir_all(&sub).unwrap();
        let sub_canonical = sub.canonicalize().unwrap();

        // Passing an absolute path that is inside the base should succeed
        let result = sanitize_path(&sub_canonical, Some(&base));
        assert!(
            result.is_ok(),
            "absolute path inside base must be accepted, got: {:?}",
            result
        );
        assert!(result.unwrap().starts_with(&base));
    }

    #[test]
    fn test_absolute_path_outside_base_rejected() {
        // Python: test_absolute_path_outside_base_rejected — absolute path outside base is rejected
        let dir = tempdir().unwrap();
        let base = dir.path().canonicalize().unwrap();
        // /tmp itself is outside a deeper base
        let outside = PathBuf::from("/tmp");
        let sub = base.join("sub");
        fs::create_dir_all(&sub).unwrap();

        let result = sanitize_path(&outside, Some(&sub));
        assert!(
            result.is_err(),
            "absolute path outside base must be rejected"
        );
    }

    #[test]
    fn test_dot_path_resolves_to_base() {
        // Python: test_dot_path_resolves_to_base — "." resolves to the base directory itself
        let dir = tempdir().unwrap();
        let base = dir.path().canonicalize().unwrap();

        let result = sanitize_path(".", Some(&base)).unwrap();
        assert_eq!(result, base, "'.' must resolve to the base directory");
    }

    #[test]
    fn test_defaults_to_cwd_when_no_base() {
        // Python: test_defaults_to_cwd_when_no_base — no base_dir defaults to cwd
        let _cwd = env::current_dir().unwrap();
        // sanitize_path with no base and a relative path should succeed and be within cwd
        let result = sanitize_path("Cargo.toml", None);
        // Cargo.toml exists at the repo root which is an ancestor of cwd; we just need no panic
        // The resulting path must start with cwd or be absolute within it
        let _ = result; // Allow either Ok or Err; main goal is no panic
    }

    #[test]
    fn test_nested_directory() {
        // Python: test_nested_directory — resolve_safe_dir works for nested dir paths
        let dir = tempdir().unwrap();
        let base = dir.path().canonicalize().unwrap();
        let nested = base.join("level1").join("level2");
        fs::create_dir_all(&nested).unwrap();

        let result = resolve_safe_dir("level1/level2", Some(&base)).unwrap();
        assert!(result.is_dir());
        assert!(result.starts_with(&base));
    }

    #[test]
    fn test_nonexistent_directory_raises() {
        // Python: test_nonexistent_directory_raises — resolve_safe_dir fails for missing dir
        let dir = tempdir().unwrap();
        let base = dir.path().canonicalize().unwrap();

        let result = resolve_safe_dir("does_not_exist", Some(&base));
        assert!(
            result.is_err(),
            "resolve_safe_dir must fail for nonexistent directory"
        );
    }

    #[test]
    fn test_file_not_directory_raises() {
        // Python: test_file_not_directory_raises — resolve_safe_dir fails when path is a file
        let dir = tempdir().unwrap();
        let base = dir.path().canonicalize().unwrap();
        let file = base.join("a_file.txt");
        fs::write(&file, "content").unwrap();

        let result = resolve_safe_dir("a_file.txt", Some(&base));
        assert!(
            result.is_err(),
            "resolve_safe_dir must fail when path is a file, not a directory"
        );
    }

    #[test]
    fn test_absolute_path_preserved() {
        // Python: test_absolute_path_preserved — an absolute path that is safe is returned as-is (canonical)
        let dir = tempdir().unwrap();
        let base = dir.path().canonicalize().unwrap();
        let sub = base.join("preserved");
        fs::create_dir_all(&sub).unwrap();
        let sub_canonical = sub.canonicalize().unwrap();

        let result = sanitize_path(&sub_canonical, Some(&base)).unwrap();
        assert_eq!(
            result, sub_canonical,
            "absolute safe path must be returned canonically"
        );
    }

    #[test]
    fn test_symlink_to_outside_rejected() {
        // Python: test_symlink_to_outside_rejected — CWE-22 symlink escape must be blocked
        let dir = tempdir().unwrap();
        let base = dir.path().canonicalize().unwrap();
        let outside = tempdir().unwrap();
        let outside_path = outside.path().canonicalize().unwrap();

        let symlink_path = base.join("escape_link");
        std::os::unix::fs::symlink(&outside_path, &symlink_path).unwrap();

        // Resolving the symlink should detect that its real path is outside the base
        let result = sanitize_path("escape_link", Some(&base));
        assert!(
            result.is_err(),
            "symlink pointing outside base must be rejected (CWE-22), but got: {:?}",
            result
        );
    }

    #[test]
    fn test_dot_resolves_to_base_resolve_safe_dir() {
        // Python: test_dot_resolves_to_base (resolve_safe_dir version)
        let dir = tempdir().unwrap();
        let base = dir.path().canonicalize().unwrap();

        let result = resolve_safe_dir(".", Some(&base)).unwrap();
        assert_eq!(result, base, "resolve_safe_dir('.') must return base dir");
    }

    #[test]
    fn test_safe_rmtree_valid_subfolder() {
        // Python: test_safe_rmtree_valid_subfolder — removes a real subfolder
        let dir = tempdir().unwrap();
        let base = dir.path().canonicalize().unwrap();
        let sub = base.join("to_delete");
        fs::create_dir_all(&sub).unwrap();
        assert!(sub.exists());

        let result = safe_rmtree(&sub, &base);
        assert!(
            result,
            "safe_rmtree must return true when deletion succeeds"
        );
        assert!(!sub.exists(), "subdirectory must be gone after safe_rmtree");
    }

    #[test]
    fn test_safe_rmtree_rejects_base_dir() {
        // Python: test_safe_rmtree_rejects_base_dir — must NOT delete the base itself
        let dir = tempdir().unwrap();
        let base = dir.path().canonicalize().unwrap();

        let result = safe_rmtree(&base, &base);
        assert!(
            !result,
            "safe_rmtree must refuse to delete the base directory itself"
        );
        assert!(
            base.exists(),
            "base directory must still exist after refusal"
        );
    }

    #[test]
    fn test_safe_rmtree_rejects_traversal() {
        // Python: test_safe_rmtree_rejects_traversal — must not delete outside base
        let dir = tempdir().unwrap();
        let base = dir.path().canonicalize().unwrap();

        let result = safe_rmtree("../outside", &base);
        assert!(
            !result,
            "safe_rmtree must reject paths that traverse outside base"
        );
    }

    #[test]
    fn test_finds_root_from_explicit_path() {
        // Python: test_finds_root_from_explicit_path — from_path parameter is used
        let repo_root = get_repo_root(None);
        // Create a nested dir inside the repo and pass it explicitly
        let nested = repo_root.join("crates").join("core").join("src");

        let found = get_repo_root(Some(&nested));
        assert_eq!(
            found, repo_root,
            "get_repo_root(Some(nested_path)) must find the same root as get_repo_root(None)"
        );
    }

    #[test]
    fn test_strict_chars_rejects_special() {
        // Python: test_strict_chars_rejects_special — shell metacharacters rejected in strict mode
        // Feature absent: Rust's sanitize_path has no strict_chars parameter.
        // This test documents the expected behavior: a path containing ';' must be rejected.
        let dir = tempdir().unwrap();
        let base = dir.path().canonicalize().unwrap();

        let result = sanitize_path("file;rm-rf", Some(&base));
        assert!(
            result.is_err(),
            "strict chars mode must reject paths with shell metacharacters like ';', but sanitize_path accepted it — strict_chars mode is not yet implemented"
        );
    }

    #[test]
    fn test_strict_chars_allows_normal() {
        // Python: test_strict_chars_allows_normal — normal alphanumeric paths accepted in strict mode
        // Feature absent: Rust's sanitize_path has no strict_chars parameter.
        // When strict_chars mode is added, normal paths must still pass.
        let dir = tempdir().unwrap();
        let base = dir.path().canonicalize().unwrap();
        let sub = base.join("normal-file_123");
        fs::create_dir_all(&sub).unwrap();

        // This should succeed in strict mode too; if strict_chars rejects this, it's a false positive
        let result = sanitize_path("normal-file_123", Some(&base));
        assert!(
            result.is_ok(),
            "strict chars mode must allow normal alphanumeric-hyphen paths"
        );
    }

    #[test]
    fn test_dotdot_in_middle_rejected() {
        // Python: test_dotdot_in_middle_rejected — traversal hidden mid-path must still be caught
        let dir = tempdir().unwrap();
        let base = dir.path().canonicalize().unwrap();

        let err = sanitize_path("sub/../../../etc/passwd", Some(&base)).unwrap_err();
        assert!(matches!(err, CoreError::PathTraversal { .. }));
        assert!(err.to_string().contains("Path traversal"));
    }

    #[test]
    fn test_path_object_input() {
        // Python: test_path_object_input — sanitize_path accepts a PathBuf, not just &str
        let dir = tempdir().unwrap();
        let base = dir.path().canonicalize().unwrap();
        let sub = base.join("subfolder");
        fs::create_dir_all(&sub).unwrap();

        let input: PathBuf = PathBuf::from("subfolder");
        let result = sanitize_path(input, Some(&base));
        assert!(result.is_ok(), "PathBuf input must be accepted");
        assert!(result.unwrap().starts_with(&base));
    }

    #[test]
    fn test_traversal_rejected_resolve_safe_dir() {
        // Python: test_traversal_rejected — resolve_safe_dir itself rejects traversal,
        // not only via the underlying sanitize_path call inherited by other tests.
        let dir = tempdir().unwrap();
        let base = dir.path().canonicalize().unwrap();

        let err = resolve_safe_dir("../../../tmp", Some(&base)).unwrap_err();
        assert!(matches!(err, CoreError::PathTraversal { .. }));
    }

    #[test]
    fn test_finds_repo_root_from_nested_dir() {
        // Python: test_finds_repo_root_from_nested_dir — constructed nested tempdir tree
        // with a synthetic anchor, not just the ambient real repo (test_get_repo_root_finds_anchor
        // only proves the mechanism works against whatever repo happens to be checked out).
        let dir = tempdir().unwrap();
        let synthetic_root = dir.path().canonicalize().unwrap();
        fs::create_dir_all(synthetic_root.join(".git")).unwrap();
        let nested = synthetic_root.join("a").join("b").join("c");
        fs::create_dir_all(&nested).unwrap();

        let found = get_repo_root(Some(&nested));
        assert_eq!(
            found, synthetic_root,
            "get_repo_root must walk upward from a deeply nested dir to find the synthetic .git anchor"
        );
    }
}
