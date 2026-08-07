use std::path::PathBuf;

/// Unified error type for the `agent_skills_core` crate.
///
/// All core library operations (path safety, config loading, depgraph)
/// return `Result<T, CoreError>` instead of `Result<T, String>`.
#[derive(Debug, thiserror::Error)]
pub enum CoreError {
    /// Path traversal or null-byte injection detected (CWE-22).
    #[error("Security Error: {message} (path: {path})")]
    PathTraversal { message: String, path: String },

    /// Filesystem I/O failure with path context.
    #[error("I/O error at '{path}': {source}")]
    Io {
        path: PathBuf,
        source: std::io::Error,
    },

    /// YAML parse/serialization failure.
    #[error("YAML error in '{file}': {message}")]
    YamlParse { file: String, message: String },

    /// Skill dependency graph validation failure (cycles, missing nodes).
    #[error("Depgraph error: {0}")]
    Depgraph(String),

    /// Generic operational error with context.
    #[error("{0}")]
    Other(String),
}

impl From<String> for CoreError {
    fn from(s: String) -> Self {
        CoreError::Other(s)
    }
}

impl From<&str> for CoreError {
    fn from(s: &str) -> Self {
        CoreError::Other(s.to_string())
    }
}
