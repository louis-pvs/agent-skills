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
) -> Result<PathBuf, String> {
    let input = input_path.as_ref();
    let input_str = input.to_string_lossy();

    if input_str.contains('\0') {
        return Err(format!(
            "Security Error: Null byte detected in path '{input_str}'"
        ));
    }

    for component in input.components() {
        if component == Component::ParentDir {
            return Err(format!(
                "Security Error: Path traversal sequence '..' detected in '{input_str}'"
            ));
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

    let base_canonical = base.canonicalize().map_err(|e| {
        format!(
            "Failed to canonicalize base directory '{}': {e}",
            base.display()
        )
    })?;

    let full_path = if input.is_absolute() {
        input.to_path_buf()
    } else {
        base_canonical.join(input)
    };

    let target = if full_path.exists() {
        full_path
            .canonicalize()
            .map_err(|e| format!("Failed to canonicalize path '{}': {e}", full_path.display()))?
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
            curr.canonicalize().map_err(|e| {
                format!(
                    "Failed to canonicalize parent path '{}': {e}",
                    curr.display()
                )
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
        return Err(format!(
            "Security Error: Path traversal attempt detected. '{input_str}' is outside allowed base directory '{}'",
            base_canonical.display()
        ));
    }

    Ok(target)
}

/// Resolves and validates a user-supplied directory path within a trusted base.
pub fn resolve_safe_dir(
    raw_path: impl AsRef<Path>,
    base_dir: Option<&Path>,
) -> Result<PathBuf, String> {
    let safe_path = sanitize_path(raw_path.as_ref(), base_dir)?;
    if !safe_path.exists() || !safe_path.is_dir() {
        return Err(format!(
            "Error: Directory '{}' does not exist or is not a directory.",
            safe_path.display()
        ));
    }
    Ok(safe_path)
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
        assert!(err.contains("Path traversal sequence '..' detected"));
    }

    #[test]
    fn test_sanitize_path_rejects_null_bytes() {
        let dir = tempdir().unwrap();
        let base = dir.path().canonicalize().unwrap();
        let err = sanitize_path("test\0file", Some(&base)).unwrap_err();
        assert!(err.contains("Null byte detected"));
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
}
