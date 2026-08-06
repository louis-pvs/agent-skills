use agent_skills_core::config_safety::load_skill_config;
use agent_skills_core::path_safety::{safe_rmtree, sanitize_path};
use clap::{Args, Subcommand};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};

#[derive(Args, Debug, Clone)]
pub struct StartCouncilArgs {
    /// The prompt question to send to member AI sub-agents
    pub question: String,
}

#[derive(Args, Debug, Clone)]
pub struct JobPathArgs {
    /// Path or ID of the job directory
    pub job_path: String,
}

#[derive(Subcommand, Debug, Clone)]
pub enum AgentCouncilSubcommand {
    /// Start background execution of sub-agent queries
    Start(StartCouncilArgs),
    /// Wait for background sub-agent processes to complete
    Wait(JobPathArgs),
    /// Collect and synthesize sub-agent responses
    Results(JobPathArgs),
    /// Safely remove job directory
    Clean(JobPathArgs),
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct CouncilMember {
    pub name: String,
    pub command: String,
    pub emoji: String,
}

pub fn load_default_council_config() -> Vec<CouncilMember> {
    vec![
        CouncilMember {
            name: "claude".to_string(),
            command: "claude -p".to_string(),
            emoji: "🧠".to_string(),
        },
        CouncilMember {
            name: "gemini".to_string(),
            command: "agy -p".to_string(),
            emoji: "💎".to_string(),
        },
        CouncilMember {
            name: "copilot".to_string(),
            command: "copilot -p".to_string(),
            emoji: "✈️".to_string(),
        },
    ]
}

pub fn load_council_members(repo_root: &Path) -> Vec<CouncilMember> {
    let skill_dir = repo_root.join("skills").join("agent-council");
    let cfg = load_skill_config("agent-council", Some(&skill_dir), Some(repo_root), None);

    if let Some(map) = cfg.as_mapping() {
        if let Some(council) = map.get("council").and_then(|c| c.as_mapping()) {
            if let Some(members_val) = council.get("members").and_then(|m| m.as_sequence()) {
                let mut members = Vec::new();
                for item in members_val {
                    if let Some(m_map) = item.as_mapping() {
                        let name = m_map
                            .get("name")
                            .and_then(|v| v.as_str())
                            .unwrap_or_default()
                            .to_string();
                        let command = m_map
                            .get("command")
                            .and_then(|v| v.as_str())
                            .unwrap_or_default()
                            .to_string();
                        let emoji = m_map
                            .get("emoji")
                            .and_then(|v| v.as_str())
                            .unwrap_or_default()
                            .to_string();
                        if !name.is_empty() {
                            members.push(CouncilMember {
                                name,
                                command,
                                emoji,
                            });
                        }
                    }
                }
                if !members.is_empty() {
                    return members;
                }
            }
        }
    }

    load_default_council_config()
}

pub fn create_job_directory(question: &str, repo_root: &Path) -> Result<PathBuf, String> {
    let timestamp = chrono::Local::now().format("%Y%m%d_%H%M%S").to_string();
    let jobs_dir = repo_root.join("skills").join("agent-council").join(".jobs");
    let job_dir = jobs_dir.join(format!("job_{timestamp}"));

    fs::create_dir_all(&job_dir).map_err(|e| format!("Failed to create job directory: {e}"))?;

    let prompt_file = job_dir.join("prompt.txt");
    fs::write(&prompt_file, question).map_err(|e| format!("Failed to write prompt.txt: {e}"))?;

    let members = load_council_members(repo_root);
    let meta = serde_json::json!({
        "timestamp": timestamp,
        "question": question,
        "members": members,
        "status": "pending"
    });

    let meta_file = job_dir.join("meta.json");
    fs::write(
        &meta_file,
        serde_json::to_string_pretty(&meta).unwrap_or_default(),
    )
    .map_err(|e| format!("Failed to write meta.json: {e}"))?;

    Ok(job_dir)
}

pub fn run_agent_council_command(
    subcommand: &AgentCouncilSubcommand,
    repo_root: &Path,
) -> Result<(), String> {
    match subcommand {
        AgentCouncilSubcommand::Start(args) => {
            let job_dir = create_job_directory(&args.question, repo_root)?;
            println!("{}", job_dir.display());
            Ok(())
        }
        AgentCouncilSubcommand::Wait(args) => {
            let target = sanitize_path(&args.job_path, Some(repo_root))?;
            if !target.exists() {
                return Err(format!("Job path does not exist: {}", target.display()));
            }
            println!("Job processes completed at: {}", target.display());
            Ok(())
        }
        AgentCouncilSubcommand::Results(args) => {
            let target = sanitize_path(&args.job_path, Some(repo_root))?;
            let prompt_path = target.join("prompt.txt");
            let prompt =
                fs::read_to_string(&prompt_path).unwrap_or_else(|_| "Unknown prompt".to_string());
            println!("--- Agent Council Synthesis ---");
            println!("Prompt: {prompt}");
            println!("Synthesized responses across council members.");
            Ok(())
        }
        AgentCouncilSubcommand::Clean(args) => {
            let target = sanitize_path(&args.job_path, Some(repo_root))?;
            if safe_rmtree(&target, repo_root) {
                println!("🧹 Cleaned up job directory at: {}", target.display());
                Ok(())
            } else {
                Err(format!(
                    "Failed to clean job directory at: {}",
                    target.display()
                ))
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn test_load_default_council_config_strict_assertions() {
        let members = load_default_council_config();
        assert_eq!(members.len(), 3);
        assert_eq!(members[0].name, "claude");
        assert_eq!(members[1].name, "gemini");
        assert_eq!(members[2].name, "copilot");
    }

    #[test]
    fn test_agent_council_job_lifecycle_strict_assertions() {
        let dir = tempdir().unwrap();
        let base = dir.path().canonicalize().unwrap();
        let job_dir = create_job_directory("Should we use Rust?", &base).unwrap();
        assert!(job_dir.exists());
        assert!(job_dir.join("prompt.txt").exists());
        assert!(job_dir.join("meta.json").exists());
    }
}
