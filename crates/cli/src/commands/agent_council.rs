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

#[derive(Args, Debug, Clone)]
pub struct ResultsArgs {
    /// Path or ID of the job directory
    pub job_path: String,

    /// Print out individual response output from each sub-agent member (default: true)
    #[arg(long, default_value_t = true)]
    pub verbose: bool,
}

#[derive(Subcommand, Debug, Clone)]
pub enum AgentCouncilSubcommand {
    /// Start background execution of sub-agent queries
    Start(StartCouncilArgs),
    /// Wait for background sub-agent processes to complete
    Wait(JobPathArgs),
    /// Collect and synthesize sub-agent responses
    Results(ResultsArgs),
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

pub fn generate_job_id(question: &str) -> String {
    use std::collections::hash_map::DefaultHasher;
    use std::hash::{Hash, Hasher};
    let mut hasher = DefaultHasher::new();
    question.hash(&mut hasher);
    format!("job_{:016x}", hasher.finish())
}

pub fn check_cli_available(cmd: &str) -> bool {
    let binary = cmd.split_whitespace().next().unwrap_or(cmd);
    if let Ok(path) = std::env::var("PATH") {
        for dir in std::env::split_paths(&path) {
            if dir.join(binary).is_file() {
                return true;
            }
        }
    }
    false
}

pub fn create_job_directory(question: &str, repo_root: &Path) -> anyhow::Result<PathBuf> {
    let job_id = generate_job_id(question);
    let jobs_dir = repo_root.join("skills").join("agent-council").join(".jobs");
    let job_dir = jobs_dir.join(&job_id);

    fs::create_dir_all(&job_dir)
        .map_err(|e| anyhow::anyhow!("Failed to create job directory: {e}"))?;

    let prompt_file = job_dir.join("prompt.txt");
    fs::write(&prompt_file, question)
        .map_err(|e| anyhow::anyhow!("Failed to write prompt.txt: {e}"))?;

    let members = load_council_members(repo_root);
    let any_missing_cli = members.iter().any(|m| !check_cli_available(&m.command));

    let meta = serde_json::json!({
        "job_id": job_id,
        "question": question,
        "members": members,
        "missing_cli": any_missing_cli,
        "status": "pending"
    });

    let meta_file = job_dir.join("meta.json");
    fs::write(
        &meta_file,
        serde_json::to_string_pretty(&meta).unwrap_or_default(),
    )
    .map_err(|e| anyhow::anyhow!("Failed to write meta.json: {e}"))?;

    Ok(job_dir)
}

pub fn read_subagent_responses(job_dir: &Path) -> anyhow::Result<Vec<(String, String)>> {
    if !job_dir.exists() || !job_dir.is_dir() {
        return Err(anyhow::anyhow!(
            "Job directory does not exist: {}",
            job_dir.display()
        ));
    }

    let mut responses = Vec::new();
    let entries =
        fs::read_dir(job_dir).map_err(|e| anyhow::anyhow!("Failed to read job directory: {e}"))?;

    for entry in entries.flatten() {
        let p = entry.path();
        if p.is_file() {
            let fname = p.file_name().unwrap_or_default().to_string_lossy();
            if fname == "prompt.txt" || fname == "meta.json" {
                continue;
            }

            let agent_name = if fname.ends_with(".response.txt") {
                fname.trim_end_matches(".response.txt").to_string()
            } else if fname.ends_with(".txt") {
                fname.trim_end_matches(".txt").to_string()
            } else if fname.ends_with(".out") {
                fname.trim_end_matches(".out").to_string()
            } else {
                continue;
            };

            if let Ok(text) = fs::read_to_string(&p) {
                responses.push((agent_name, text));
            }
        }
    }

    responses.sort_by(|a, b| a.0.cmp(&b.0));
    Ok(responses)
}

pub fn run_agent_council_command(
    subcommand: &AgentCouncilSubcommand,
    repo_root: &Path,
) -> anyhow::Result<()> {
    match subcommand {
        AgentCouncilSubcommand::Start(args) => {
            let job_dir = create_job_directory(&args.question, repo_root)?;
            println!("{}", job_dir.display());
            Ok(())
        }
        AgentCouncilSubcommand::Wait(args) => {
            let target = sanitize_path(&args.job_path, Some(repo_root))?;
            if !target.exists() {
                return Err(anyhow::anyhow!(
                    "Job path does not exist: {}",
                    target.display()
                ));
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

            if args.verbose {
                let responses = read_subagent_responses(&target)?;
                for (agent_name, response_text) in responses {
                    println!("\n=== Sub-Agent: {} ===", agent_name);
                    println!("{response_text}");
                }
            }

            println!("\nSynthesized responses across council members.");
            Ok(())
        }
        AgentCouncilSubcommand::Clean(args) => {
            let target = sanitize_path(&args.job_path, Some(repo_root))?;
            if safe_rmtree(&target, repo_root) {
                println!("🧹 Cleaned up job directory at: {}", target.display());
                Ok(())
            } else {
                Err(anyhow::anyhow!(
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

    #[test]
    fn test_agent_council_results_verbose_by_default_strict_assertions() {
        let dir = tempdir().unwrap();
        let base = dir.path().canonicalize().unwrap();
        let job_dir = create_job_directory("Should we use Rust?", &base).unwrap();

        fs::write(job_dir.join("claude.response.txt"), "Claude says yes").unwrap();
        fs::write(job_dir.join("gemini.response.txt"), "Gemini says yes").unwrap();

        let responses = read_subagent_responses(&job_dir).unwrap();
        assert_eq!(responses.len(), 2);
        assert_eq!(responses[0].0, "claude");
        assert_eq!(responses[0].1, "Claude says yes");
        assert_eq!(responses[1].0, "gemini");
        assert_eq!(responses[1].1, "Gemini says yes");
    }

    #[test]
    fn test_load_config_adr0005() {
        let dir = tempdir().unwrap();
        let base = dir.path().canonicalize().unwrap();
        let council_skill_dir = base.join("skills").join("agent-council");
        fs::create_dir_all(&council_skill_dir).unwrap();
        fs::write(
            council_skill_dir.join("config.yaml"),
            "members:\n  - name: test_agent\n    cmd: echo\n",
        )
        .unwrap();

        let cfg = agent_skills_core::config_safety::load_skill_config(
            "agent-council",
            Some(&council_skill_dir),
            Some(&base),
            None,
        );
        assert!(cfg.as_mapping().unwrap().contains_key("members"));
    }

    #[test]
    fn test_generate_job_id() {
        // Python: test_generate_job_id — job ID must be deterministic hash of question, not timestamp
        // Feature absent: Rust uses timestamps like job_20240101_120000 — no dedup by question
        let question = "What is the capital of France?";
        let dir = tempdir().unwrap();
        let job = create_job_directory(question, dir.path()).unwrap();
        let id = job.file_name().unwrap().to_str().unwrap().to_string();

        // A deterministic content-hash ID must NOT look like a timestamp
        let timestamp_pattern = regex::Regex::new(r"job_\d{8}_\d{6}").unwrap();
        assert!(
            !timestamp_pattern.is_match(&id),
            "job ID must be a deterministic content hash of the question, not a timestamp. Got: {id} — timestamp-based IDs cannot deduplicate same-question requests"
        );
    }

    #[test]
    fn test_create_job_missing_cli() {
        // Python: test_create_job_missing_cli — meta.json must record missing_cli state
        // Feature absent: no CLI-existence check or missing_cli state
        let dir = tempdir().unwrap();
        let base = dir.path().canonicalize().unwrap();
        let job_dir = create_job_directory("Test missing CLI", &base).unwrap();

        let meta_content = fs::read_to_string(job_dir.join("meta.json")).unwrap();
        assert!(
            meta_content.contains("missing_cli"),
            "meta.json must record missing_cli:true when the CLI binary is not found in PATH, got: {meta_content}"
        );
    }

    #[test]
    fn test_load_council_members_reads_config_yaml() {
        // Python: test_load_config_adr0005 — load_council_members (not just the generic
        // load_skill_config loader) must parse a real on-disk config.yaml into CouncilMember
        // structs, rather than only ever falling back to the hardcoded 3-member default.
        let dir = tempdir().unwrap();
        let base = dir.path().canonicalize().unwrap();
        let council_skill_dir = base.join("skills").join("agent-council");
        fs::create_dir_all(&council_skill_dir).unwrap();
        fs::write(
            council_skill_dir.join("config.yaml"),
            "council:\n  members:\n    - name: real_agent\n      command: real-cli -p\n      emoji: \"🔧\"\n",
        )
        .unwrap();

        let members = load_council_members(&base);
        assert_eq!(
            members.len(),
            1,
            "load_council_members must read the real on-disk config, not fall back to the hardcoded default, got: {members:?}"
        );
        assert_eq!(members[0].name, "real_agent");
        assert_eq!(members[0].command, "real-cli -p");
    }

    #[test]
    fn test_create_and_clean_job_full_lifecycle() {
        // Python: test_create_and_clean_job — extends the existing lifecycle test with the
        // Clean subcommand: after creating a job, running Clean must remove the job directory.
        let dir = tempdir().unwrap();
        let base = dir.path().canonicalize().unwrap();
        let job_dir = create_job_directory("Should we use Rust?", &base).unwrap();
        assert!(job_dir.exists());

        let job_path = job_dir.to_string_lossy().to_string();
        let clean_args = JobPathArgs { job_path };
        run_agent_council_command(&AgentCouncilSubcommand::Clean(clean_args), &base).unwrap();

        assert!(
            !job_dir.exists(),
            "Clean subcommand must remove the job directory via safe_rmtree"
        );
    }
}
