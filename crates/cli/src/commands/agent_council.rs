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

    /// Preview commands without actually spawning subprocesses
    #[arg(long)]
    pub dry_run: bool,
}

#[derive(Args, Debug, Clone)]
pub struct JobPathArgs {
    /// Path or ID of the job directory
    pub job_path: String,
}

#[derive(Args, Debug, Clone)]
pub struct WaitArgs {
    /// Path or ID of the job directory
    pub job_path: String,

    /// Maximum timeout in seconds to wait for member processes (default: 120)
    #[arg(long, default_value_t = 120)]
    pub timeout: u64,
}

#[derive(Args, Debug, Clone)]
pub struct ResultsArgs {
    /// Path or ID of the job directory
    pub job_path: String,

    /// Print out individual response output from each sub-agent member (default: true)
    #[arg(long, default_value_t = true)]
    pub verbose: bool,
}

#[derive(Args, Debug, Clone)]
pub struct DoctorArgs {
    /// Show verbose diagnostic information
    #[arg(long)]
    pub verbose: bool,
}

#[derive(Subcommand, Debug, Clone)]
pub enum AgentCouncilSubcommand {
    /// Start background execution of sub-agent queries
    Start(StartCouncilArgs),
    /// Wait for background sub-agent processes to complete
    Wait(WaitArgs),
    /// Collect and synthesize sub-agent responses
    Results(ResultsArgs),
    /// Safely remove job directory
    Clean(JobPathArgs),
    /// Diagnose council member CLI availability and environment health
    Doctor(DoctorArgs),
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
            command: "claude --dangerously-skip-permissions -p".to_string(),
            emoji: "🧠".to_string(),
        },
        CouncilMember {
            name: "antigravity".to_string(),
            command: "agy --dangerously-skip-permissions -p".to_string(),
            emoji: "💎".to_string(),
        },
        CouncilMember {
            name: "copilot".to_string(),
            command: "copilot -p".to_string(),
            emoji: "✈️".to_string(),
        },
    ]
}

pub fn load_council_timeout(repo_root: &Path) -> u64 {
    let skill_dir = repo_root.join("skills").join("agent-council");
    let cfg = load_skill_config("agent-council", Some(&skill_dir), Some(repo_root), None);

    if let Some(map) = cfg.as_mapping() {
        if let Some(council) = map.get("council").and_then(|c| c.as_mapping()) {
            if let Some(settings) = council.get("settings").and_then(|s| s.as_mapping()) {
                if let Some(timeout) = settings.get("timeout").and_then(|t| t.as_u64()) {
                    if timeout > 0 {
                        return timeout;
                    }
                }
            }
        }
    }

    180
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

/// Enriches current PATH with standard user package manager directories
/// to ensure child sub-shells and IDE terminals locate installed CLIs.
pub fn get_enriched_path() -> std::ffi::OsString {
    let current = std::env::var_os("PATH").unwrap_or_default();
    let mut paths: Vec<PathBuf> = std::env::split_paths(&current).collect();

    if let Some(home) = dirs::home_dir() {
        let candidates = [
            home.join(".cargo").join("bin"),
            home.join("AppData").join("Roaming").join("npm"),
            home.join("AppData").join("Local").join("Programs"),
            home.join("AppData").join("Local").join("agy").join("bin"),
            home.join(".npm-global").join("bin"),
            home.join("scoop").join("shims"),
            home.join(".local").join("bin"),
            PathBuf::from(r"C:\Program Files\nodejs"),
            PathBuf::from(r"C:\Program Files\GitHub CLI"),
            PathBuf::from("/usr/local/bin"),
            PathBuf::from("/opt/homebrew/bin"),
        ];

        for c in candidates {
            if c.exists() && !paths.contains(&c) {
                paths.push(c);
            }
        }
    }

    std::env::join_paths(paths).unwrap_or(current)
}

/// Searches the enriched PATH for a command binary (resolving extensions on Windows).
pub fn find_cli_binary(cmd: &str) -> Option<PathBuf> {
    let raw_bin = cmd.split_whitespace().next().unwrap_or(cmd);
    let enriched_path = get_enriched_path();

    // 0. Direct check if raw_bin is an existing file path
    let direct_path = PathBuf::from(raw_bin);
    if direct_path.is_file() {
        return Some(direct_path);
    }

    // On Windows, prioritize native .exe before scripts/batches
    let pathext = if cfg!(windows) {
        vec![".exe", ".cmd", ".bat", ".com", ".ps1"]
    } else {
        vec![""]
    };

    // 1. Direct search for raw_bin across enriched PATH
    for dir in std::env::split_paths(&enriched_path) {
        // Special check for nested package executables (e.g. global npm packages)
        if cfg!(windows) && raw_bin == "claude" {
            let nested_claude = dir
                .join("node_modules")
                .join("@anthropic-ai")
                .join("claude-code")
                .join("bin")
                .join("claude.exe");
            if nested_claude.is_file() {
                return Some(nested_claude);
            }
        }

        if cfg!(windows) {
            for ext in &pathext {
                let cand = dir.join(format!("{}{}", raw_bin, ext));
                if cand.is_file() {
                    return Some(cand);
                }
                let cand_upper = dir.join(format!("{}{}", raw_bin, ext.to_uppercase()));
                if cand_upper.is_file() {
                    return Some(cand_upper);
                }
            }
        }
        let direct = dir.join(raw_bin);
        if direct.is_file() {
            return Some(direct);
        }
    }

    // 2. Smart fallback aliases
    let fallback_aliases: Vec<&str> = match raw_bin {
        "agy" => vec!["antigravity"],
        "antigravity" => vec!["agy"],
        "copilot" => vec!["gh"],
        "claude" => vec!["claude-code"],
        _ => vec![],
    };

    for alias in fallback_aliases {
        for dir in std::env::split_paths(&enriched_path) {
            if cfg!(windows) {
                for ext in &pathext {
                    let cand = dir.join(format!("{}{}", alias, ext));
                    if cand.is_file() {
                        return Some(cand);
                    }
                    let cand_upper = dir.join(format!("{}{}", alias, ext.to_uppercase()));
                    if cand_upper.is_file() {
                        return Some(cand_upper);
                    }
                }
            }
            let direct = dir.join(alias);
            if direct.is_file() {
                return Some(direct);
            }
        }
    }

    None
}

pub fn check_cli_available(cmd: &str) -> bool {
    find_cli_binary(cmd).is_some()
}

pub fn is_pid_alive(pid: u32) -> bool {
    #[cfg(unix)]
    {
        std::process::Command::new("kill")
            .args(["-0", &pid.to_string()])
            .output()
            .map(|o| o.status.success())
            .unwrap_or(false)
    }
    #[cfg(windows)]
    {
        let output = std::process::Command::new("tasklist")
            .args(["/FI", &format!("PID eq {}", pid), "/NH"])
            .output();
        if let Ok(out) = output {
            let s = String::from_utf8_lossy(&out.stdout);
            !s.contains("No tasks are running") && s.contains(&pid.to_string())
        } else {
            false
        }
    }
}

pub fn create_job_directory(
    question: &str,
    repo_root: &Path,
    dry_run: bool,
) -> anyhow::Result<PathBuf> {
    let job_id = generate_job_id(question);
    let jobs_dir = repo_root.join("skills").join("agent-council").join(".jobs");
    let job_dir = jobs_dir.join(&job_id);

    // Clean stale files from previous runs with the same question hash
    if job_dir.exists() {
        for entry in fs::read_dir(&job_dir).into_iter().flatten().flatten() {
            let p = entry.path();
            if p.is_file() {
                let _ = fs::remove_file(&p);
            }
        }
    }

    fs::create_dir_all(&job_dir)
        .map_err(|e| anyhow::anyhow!("Failed to create job directory: {e}"))?;

    let prompt_file = job_dir.join("prompt.txt");
    fs::write(&prompt_file, question)
        .map_err(|e| anyhow::anyhow!("Failed to write prompt.txt: {e}"))?;

    let members = load_council_members(repo_root);
    let mut member_details = Vec::new();
    let mut missing_members = Vec::new();
    let enriched_path = get_enriched_path();
    let mut children: Vec<(String, std::process::Child)> = Vec::new();

    for m in &members {
        let bin_opt = find_cli_binary(&m.command);
        let available = bin_opt.is_some();
        if !available {
            missing_members.push(m.name.clone());
        }

        let bin_path_str = bin_opt.as_ref().map(|p| p.to_string_lossy().to_string());

        member_details.push(serde_json::json!({
            "name": m.name,
            "command": m.command,
            "emoji": m.emoji,
            "available": available,
            "binary_path": bin_path_str,
        }));

        if let Some(bin_path) = bin_opt {
            if !dry_run {
                let out_file_path = job_dir.join(format!("{}.response.txt", m.name));
                let err_file_path = job_dir.join(format!("{}.err", m.name));

                let out_file = fs::File::create(&out_file_path).ok();
                let err_file = fs::File::create(&err_file_path).ok();

                if let (Some(out_f), Some(err_f)) = (out_file, err_file) {
                    let parts = shlex::split(&m.command).unwrap_or_else(|| vec![m.command.clone()]);

                    let mut cmd = std::process::Command::new(&bin_path);
                    cmd.env("PATH", &enriched_path);
                    cmd.current_dir(repo_root);

                    // Add all flags/args after the binary name
                    if parts.len() > 1 {
                        for arg in &parts[1..] {
                            cmd.arg(arg);
                        }
                    }
                    cmd.arg(question);

                    cmd.stdin(std::process::Stdio::null());
                    cmd.stdout(std::process::Stdio::from(out_f));
                    cmd.stderr(std::process::Stdio::from(err_f));

                    match cmd.spawn() {
                        Ok(child) => {
                            let pid_file = job_dir.join(format!("{}.pid", m.name));
                            let _ = fs::write(&pid_file, child.id().to_string());
                            children.push((m.name.clone(), child));
                        }
                        Err(e) => {
                            let _ = fs::write(
                                &err_file_path,
                                format!("Failed to spawn child process: {e}\n"),
                            );
                        }
                    }
                }
            }
        }
    }

    let meta = serde_json::json!({
        "job_id": job_id,
        "question": question,
        "members": members,
        "member_details": member_details,
        "missing_cli": !missing_members.is_empty(),
        "missing_members": missing_members,
        "status": if dry_run { "dry_run" } else { "running" },
        "created_at": chrono::Utc::now().to_rfc3339(),
    });

    let meta_file = job_dir.join("meta.json");
    fs::write(
        &meta_file,
        serde_json::to_string_pretty(&meta).unwrap_or_default(),
    )
    .map_err(|e| anyhow::anyhow!("Failed to write meta.json: {e}"))?;

    // Wait for all spawned children to complete concurrently (with timeout)
    let timeout_secs = load_council_timeout(repo_root);
    let timeout = std::time::Duration::from_secs(timeout_secs);
    let start_time = std::time::Instant::now();
    let mut remaining_children = children;

    while !remaining_children.is_empty() {
        if start_time.elapsed() >= timeout {
            for (name, mut child) in remaining_children {
                eprintln!("⚠️ Timeout waiting for {name}, killing process");
                let _ = child.kill();
            }
            break;
        }

        remaining_children.retain_mut(|(name, child)| match child.try_wait() {
            Ok(Some(_status)) => false,
            Ok(None) => true,
            Err(e) => {
                eprintln!("⚠️ Error waiting for {name}: {e}");
                false
            }
        });

        if !remaining_children.is_empty() {
            std::thread::sleep(std::time::Duration::from_millis(500));
        }
    }

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
                let trimmed = text.trim();
                if !trimmed.is_empty() {
                    responses.push((agent_name, trimmed.to_string()));
                }
            }
        }
    }

    responses.sort_by(|a, b| a.0.cmp(&b.0));
    Ok(responses)
}

pub fn run_doctor_command(repo_root: &Path, verbose: bool) -> anyhow::Result<()> {
    println!("=== Agent Council Environment Doctor ===");
    let members = load_council_members(repo_root);
    let enriched_path = get_enriched_path();

    if verbose {
        println!("\nEnriched PATH Search Locations:");
        for p in std::env::split_paths(&enriched_path) {
            println!("  - {}", p.display());
        }
        println!();
    }

    println!("\nMember CLI Health & Availability Checks:");
    let mut available_count = 0;

    for m in &members {
        let bin_opt = find_cli_binary(&m.command);
        match bin_opt {
            Some(bin_path) => {
                available_count += 1;

                let mut cmd = std::process::Command::new(&bin_path);
                let version_output = cmd
                    .arg("--version")
                    .env("PATH", &enriched_path)
                    .output()
                    .ok()
                    .and_then(|o| {
                        if o.status.success() {
                            Some(String::from_utf8_lossy(&o.stdout).trim().to_string())
                        } else {
                            None
                        }
                    })
                    .unwrap_or_else(|| "detected".to_string());

                let first_line = version_output.lines().next().unwrap_or("detected");
                println!(
                    "✅ [READY]   {} {} (`{}`)\n     Binary: {}\n     Version: {}",
                    m.emoji,
                    m.name,
                    m.command,
                    bin_path.display(),
                    first_line
                );
            }
            None => {
                let suggestion = match m.name.as_str() {
                    "claude" => "npm install -g @anthropic-ai/claude-code",
                    "antigravity" => {
                        if cfg!(windows) {
                            "irm https://antigravity.google/cli/install.ps1 | iex"
                        } else {
                            "curl -fsSL https://antigravity.google/cli/install.sh | bash"
                        }
                    }
                    "copilot" => "npm install -g @github/copilot (or install GitHub CLI: gh extension install github/gh-copilot)",
                    "codex" => "npm install -g @openai/codex",
                    _ => "Install the CLI and ensure its directory is in PATH.",
                };
                println!(
                    "❌ [MISSING] {} {} (`{}`)\n     Status: Executable not found in PATH\n     💡 Fix: {}",
                    m.emoji,
                    m.name,
                    m.command,
                    suggestion
                );
            }
        }
        println!();
    }

    println!("=== Summary ===");
    println!(
        "Available: {}/{} configured members ready.",
        available_count,
        members.len()
    );

    if available_count == 0 {
        println!("⚠️ No member CLIs were detected. Please install at least one CLI before running council queries.");
    } else if available_count < members.len() {
        println!("ℹ️ Some members are missing, but council can run with available members.");
    } else {
        println!("✨ All council member CLIs are configured and ready for parallel execution!");
    }

    Ok(())
}

pub fn run_agent_council_command(
    subcommand: &AgentCouncilSubcommand,
    repo_root: &Path,
) -> anyhow::Result<()> {
    match subcommand {
        AgentCouncilSubcommand::Start(args) => {
            let job_dir = create_job_directory(&args.question, repo_root, args.dry_run)?;
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

            // Read PIDs and wait with timeout
            let timeout_secs = if args.timeout > 0 {
                args.timeout
            } else {
                load_council_timeout(repo_root)
            };
            let start_time = std::time::Instant::now();

            loop {
                let mut any_running = false;
                if let Ok(entries) = fs::read_dir(&target) {
                    for entry in entries.flatten() {
                        let p = entry.path();
                        if p.is_file() && p.extension().is_some_and(|ext| ext == "pid") {
                            if let Ok(pid_str) = fs::read_to_string(&p) {
                                if let Ok(pid) = pid_str.trim().parse::<u32>() {
                                    if is_pid_alive(pid) {
                                        any_running = true;
                                        break;
                                    }
                                }
                            }
                        }
                    }
                }

                if !any_running || start_time.elapsed().as_secs() >= timeout_secs {
                    break;
                }
                std::thread::sleep(std::time::Duration::from_millis(500));
            }

            println!("Job processes completed at: {}", target.display());
            Ok(())
        }
        AgentCouncilSubcommand::Results(args) => {
            let target = sanitize_path(&args.job_path, Some(repo_root))?;
            let prompt_path = target.join("prompt.txt");
            let prompt =
                fs::read_to_string(&prompt_path).unwrap_or_else(|_| "Unknown prompt".to_string());

            let configured_members =
                if let Ok(meta_text) = fs::read_to_string(target.join("meta.json")) {
                    if let Ok(meta_json) = serde_json::from_str::<serde_json::Value>(&meta_text) {
                        if let Some(m_array) = meta_json.get("members").and_then(|m| m.as_array()) {
                            let members: Vec<CouncilMember> =
                                serde_json::from_value(serde_json::Value::Array(m_array.clone()))
                                    .unwrap_or_else(|_| load_council_members(repo_root));
                            members
                        } else {
                            load_council_members(repo_root)
                        }
                    } else {
                        load_council_members(repo_root)
                    }
                } else {
                    load_council_members(repo_root)
                };

            let responses = read_subagent_responses(&target)?;
            let response_map: std::collections::HashMap<String, String> =
                responses.into_iter().collect();

            println!("--- Agent Council Report ---");
            println!("Prompt: {prompt}\n");

            println!("=== Council Member Status ===");
            let mut responded_count = 0;
            let mut failed_members = Vec::new();

            for member in &configured_members {
                let cli_available = check_cli_available(&member.command);
                if let Some(response_text) = response_map.get(&member.name) {
                    responded_count += 1;
                    println!(
                        "- {} {} (`{}`): Responded ({} chars)",
                        member.emoji,
                        member.name,
                        member.command,
                        response_text.len()
                    );
                } else if !cli_available {
                    failed_members.push((
                        member.name.clone(),
                        format!("CLI not available in PATH (`{}`)", member.command),
                    ));
                    println!(
                        "- {} {} (`{}`): ⚠️ Missing CLI (not found in PATH)",
                        member.emoji, member.name, member.command
                    );
                } else {
                    failed_members.push((
                        member.name.clone(),
                        "No response or empty output returned".to_string(),
                    ));
                    println!(
                        "- {} {} (`{}`): ⚠️ No response received",
                        member.emoji, member.name, member.command
                    );
                }
            }

            if args.verbose {
                println!("\n=== Member Responses ===");
                for member in &configured_members {
                    if let Some(response_text) = response_map.get(&member.name) {
                        println!("\n--- {} {} ---", member.emoji, member.name);
                        println!("{response_text}");
                    } else if !check_cli_available(&member.command) {
                        println!("\n--- {} {} ---", member.emoji, member.name);
                        println!("⚠️ [ERROR] Member CLI is not usable: executable for '{}' was not found in PATH.", member.command);
                    } else {
                        println!("\n--- {} {} ---", member.emoji, member.name);
                        let err_file = target.join(format!("{}.err", member.name));
                        let err_text = fs::read_to_string(&err_file).unwrap_or_default();
                        let trimmed_err = err_text.trim();
                        if !trimmed_err.is_empty() {
                            println!(
                                "⚠️ [WARNING] No response returned by member CLI ('{}'). Stderr output:\n{trimmed_err}",
                                member.command
                            );
                        } else {
                            println!(
                                "⚠️ [WARNING] No response was returned by member CLI ('{}').",
                                member.command
                            );
                        }
                    }
                }
            }

            println!("\n=== Summary ===");
            if responded_count == 0 {
                println!(
                    "❌ ERROR: None of the {} configured council members returned a response.",
                    configured_members.len()
                );
                println!("Please verify your CLI installations via 'agent-skills agent-council doctor' and check 'council.config.yaml'.");
            } else if !failed_members.is_empty() {
                let failed_names: Vec<String> =
                    failed_members.iter().map(|(n, _)| n.clone()).collect();
                println!(
                    "⚠️ WARNING: {} of {} configured members failed or were unavailable ({}).",
                    failed_members.len(),
                    configured_members.len(),
                    failed_names.join(", ")
                );
                println!(
                    "Synthesized responses from {} active member(s).",
                    responded_count
                );
            } else {
                println!(
                    "✅ All {} configured council members responded successfully.",
                    configured_members.len()
                );
            }

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
        AgentCouncilSubcommand::Doctor(args) => run_doctor_command(repo_root, args.verbose),
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
        assert_eq!(members[1].name, "antigravity");
        assert_eq!(members[2].name, "copilot");
    }

    #[test]
    fn test_agent_council_job_lifecycle_strict_assertions() {
        let dir = tempdir().unwrap();
        let base = dir.path().canonicalize().unwrap();
        let job_dir = create_job_directory("Should we use Rust?", &base, true).unwrap();
        assert!(job_dir.exists());
        assert!(job_dir.join("prompt.txt").exists());
        assert!(job_dir.join("meta.json").exists());
    }

    #[test]
    fn test_agent_council_results_verbose_by_default_strict_assertions() {
        let dir = tempdir().unwrap();
        let base = dir.path().canonicalize().unwrap();
        let job_dir = create_job_directory("Should we use Rust?", &base, true).unwrap();

        fs::write(job_dir.join("claude.response.txt"), "Claude says yes").unwrap();
        fs::write(
            job_dir.join("antigravity.response.txt"),
            "Antigravity says yes",
        )
        .unwrap();

        let responses = read_subagent_responses(&job_dir).unwrap();
        assert_eq!(responses.len(), 2);
        assert_eq!(responses[0].0, "antigravity");
        assert_eq!(responses[0].1, "Antigravity says yes");
        assert_eq!(responses[1].0, "claude");
        assert_eq!(responses[1].1, "Claude says yes");
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
        let question = "What is the capital of France?";
        let dir = tempdir().unwrap();
        let job = create_job_directory(question, dir.path(), true).unwrap();
        let id = job.file_name().unwrap().to_str().unwrap().to_string();

        let timestamp_pattern = regex::Regex::new(r"job_\d{8}_\d{6}").unwrap();
        assert!(
            !timestamp_pattern.is_match(&id),
            "job ID must be a deterministic content hash of the question, not a timestamp. Got: {id}"
        );
    }

    #[test]
    fn test_create_job_missing_cli() {
        let dir = tempdir().unwrap();
        let base = dir.path().canonicalize().unwrap();
        let job_dir = create_job_directory("Test missing CLI", &base, true).unwrap();

        let meta_content = fs::read_to_string(job_dir.join("meta.json")).unwrap();
        assert!(
            meta_content.contains("missing_cli"),
            "meta.json must record missing_cli state, got: {meta_content}"
        );
        assert!(
            meta_content.contains("member_details"),
            "meta.json must record member_details, got: {meta_content}"
        );
    }

    #[test]
    fn test_load_council_members_reads_config_yaml() {
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
            "load_council_members must read the real on-disk config, got: {members:?}"
        );
        assert_eq!(members[0].name, "real_agent");
        assert_eq!(members[0].command, "real-cli -p");
    }

    #[test]
    fn test_create_and_clean_job_full_lifecycle() {
        let dir = tempdir().unwrap();
        let base = dir.path().canonicalize().unwrap();
        let job_dir = create_job_directory("Should we use Rust?", &base, true).unwrap();
        assert!(job_dir.exists());

        let job_path = job_dir.to_string_lossy().to_string();
        let clean_args = JobPathArgs { job_path };
        run_agent_council_command(&AgentCouncilSubcommand::Clean(clean_args), &base).unwrap();

        assert!(
            !job_dir.exists(),
            "Clean subcommand must remove the job directory via safe_rmtree"
        );
    }

    #[test]
    fn test_run_agent_council_results_outputs_status_and_warnings() {
        let dir = tempdir().unwrap();
        let base = dir.path().canonicalize().unwrap();
        let job_dir = create_job_directory("Which database to choose?", &base, true).unwrap();

        fs::write(
            job_dir.join("claude.response.txt"),
            "PostgreSQL is recommended.",
        )
        .unwrap();

        let results_args = ResultsArgs {
            job_path: job_dir.to_string_lossy().to_string(),
            verbose: true,
        };

        let result =
            run_agent_council_command(&AgentCouncilSubcommand::Results(results_args), &base);
        assert!(result.is_ok());
    }

    #[test]
    fn test_doctor_command_runs_cleanly() {
        let dir = tempdir().unwrap();
        let base = dir.path().canonicalize().unwrap();
        let res = run_doctor_command(&base, false);
        assert!(res.is_ok());
    }

    #[test]
    fn test_doctor_command_verbose_runs_cleanly() {
        let dir = tempdir().unwrap();
        let base = dir.path().canonicalize().unwrap();
        let res = run_doctor_command(&base, true);
        assert!(res.is_ok());
    }

    #[test]
    fn test_get_enriched_path_returns_non_empty_paths() {
        let enriched = get_enriched_path();
        let paths: Vec<PathBuf> = std::env::split_paths(&enriched).collect();
        assert!(
            !paths.is_empty(),
            "get_enriched_path must return non-empty PATH list"
        );
    }

    #[test]
    fn test_find_cli_binary_non_existent_returns_none() {
        let res = find_cli_binary("completely_non_existent_binary_xyz_999");
        assert!(res.is_none());
    }

    #[test]
    fn test_agent_council_wait_non_existent_pid() {
        let dir = tempdir().unwrap();
        let base = dir.path().canonicalize().unwrap();
        let job_dir = create_job_directory("Should we use Rust?", &base, true).unwrap();

        // Write a mock pid that is already dead / non-existent (e.g. 999999)
        fs::write(job_dir.join("antigravity.pid"), "999999").unwrap();

        let wait_args = WaitArgs {
            job_path: job_dir.to_string_lossy().to_string(),
            timeout: 120,
        };
        let res = run_agent_council_command(&AgentCouncilSubcommand::Wait(wait_args), &base);
        assert!(
            res.is_ok(),
            "Wait must return Ok even if processes already finished"
        );
    }

    /// Regression test: stale files from a previous run with the same question
    /// hash must be cleaned before spawning new processes. Without cleanup,
    /// old 0-byte response files persist and mask missing output.
    #[test]
    fn test_job_directory_reuse_cleans_stale_files() {
        let dir = tempdir().unwrap();
        let base = dir.path().canonicalize().unwrap();

        // First run (dry_run) — creates directory with prompt.txt and meta.json
        let job_dir_1 = create_job_directory("Stale file test", &base, true).unwrap();

        // Simulate stale response files left from a prior execution
        fs::write(job_dir_1.join("claude.response.txt"), "").unwrap();
        fs::write(job_dir_1.join("copilot.response.txt"), "stale data").unwrap();
        fs::write(job_dir_1.join("leftover.pid"), "12345").unwrap();

        // Second run with same question — should clean all stale files
        let job_dir_2 = create_job_directory("Stale file test", &base, true).unwrap();

        // Same job_id means same directory
        assert_eq!(
            job_dir_1.file_name(),
            job_dir_2.file_name(),
            "Same question must produce same job_id"
        );

        // Stale files must be gone — only fresh prompt.txt and meta.json remain
        assert!(
            !job_dir_2.join("leftover.pid").exists(),
            "Stale .pid files must be cleaned on reuse"
        );

        // prompt.txt and meta.json are recreated fresh
        assert!(job_dir_2.join("prompt.txt").exists());
        assert!(job_dir_2.join("meta.json").exists());
    }

    /// Integration test: spawn a real process (echo on Unix, cmd /C echo on Windows)
    /// using a mock council config and verify the response file actually contains
    /// the child's stdout output. This covers the dry_run=false path that was
    /// previously untested and caused the 0-byte response bug.
    #[test]
    fn test_spawn_real_process_captures_output() {
        let dir = tempdir().unwrap();
        let base = dir.path().canonicalize().unwrap();
        let council_skill_dir = base.join("skills").join("agent-council");
        fs::create_dir_all(&council_skill_dir).unwrap();

        // Configure a mock member using a simple echo command
        let (echo_cmd, expected_fragment) = if cfg!(windows) {
            // On Windows, cmd.exe /C echo outputs "hello" + the appended question
            ("cmd /C echo hello", "hello")
        } else {
            ("echo hello", "hello")
        };

        let config_content = format!(
            "council:\n  members:\n    - name: mock_echo\n      command: \"{echo_cmd}\"\n      emoji: \"🔧\"\n"
        );
        fs::write(council_skill_dir.join("config.yaml"), config_content).unwrap();

        // dry_run=false — actually spawns the process and waits
        let job_dir = create_job_directory("test question", &base, false).unwrap();

        let response_file = job_dir.join("mock_echo.response.txt");
        assert!(
            response_file.exists(),
            "Response file must be created for the mock member"
        );

        let content = fs::read_to_string(&response_file).unwrap();
        assert!(
            content.contains(expected_fragment),
            "Response file must contain the echo output '{expected_fragment}', got: '{content}'"
        );
    }

    /// Verify that create_job_directory with dry_run=false does NOT leave
    /// dangling child processes — it waits for completion before returning.
    #[test]
    fn test_spawn_waits_for_children_to_complete() {
        let dir = tempdir().unwrap();
        let base = dir.path().canonicalize().unwrap();
        let council_skill_dir = base.join("skills").join("agent-council");
        fs::create_dir_all(&council_skill_dir).unwrap();

        let echo_cmd = if cfg!(windows) {
            "cmd /C echo done"
        } else {
            "echo done"
        };

        let config_content = format!(
            "council:\n  members:\n    - name: waiter\n      command: \"{echo_cmd}\"\n      emoji: \"⏱️\"\n"
        );
        fs::write(council_skill_dir.join("config.yaml"), config_content).unwrap();

        let job_dir = create_job_directory("wait test", &base, false).unwrap();

        // After create_job_directory returns, the PID should be dead (process completed)
        let pid_file = job_dir.join("waiter.pid");
        if pid_file.exists() {
            let pid: u32 = fs::read_to_string(&pid_file)
                .unwrap()
                .trim()
                .parse()
                .unwrap();
            assert!(
                !is_pid_alive(pid),
                "After create_job_directory returns, spawned process (PID {pid}) must have exited"
            );
        }

        // And the response file must have content
        let response = fs::read_to_string(job_dir.join("waiter.response.txt")).unwrap();
        assert!(
            !response.trim().is_empty(),
            "Response file must not be empty after waiting for child"
        );
    }
}
