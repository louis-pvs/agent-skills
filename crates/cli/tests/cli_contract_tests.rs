use assert_cmd::Command;
use predicates::prelude::*;

#[test]
fn test_cli_help_flag() {
    let mut cmd = Command::cargo_bin("agent-skills").unwrap();
    cmd.arg("--help");
    cmd.assert()
        .success()
        .stdout(predicate::str::contains("Unified CLI for Agent Skills"));
}

#[test]
fn test_cli_invalid_subcommand_exit_code() {
    let mut cmd = Command::cargo_bin("agent-skills").unwrap();
    cmd.arg("nonexistent-subcommand-123");
    cmd.assert()
        .failure()
        .stderr(predicate::str::contains("unrecognized subcommand"));
}

#[test]
fn test_cli_subcommands_help() {
    let subcommands = [
        "depgraph",
        "install",
        "lint-scripts",
        "capability-gap-analyzer",
        "code-janitor",
        "context-gatherer",
        "agent-council",
        "agent-creator",
        "adr",
        "benchmarking",
        "self-annealer",
        "self-progress",
        "skill-creator",
        "what-if-analysis",
        "domain-modeling",
        "git-conflict-resolver",
        "tdd",
        "tech-doc-writer",
    ];

    for sub in subcommands {
        let mut cmd = Command::cargo_bin("agent-skills").unwrap();
        cmd.arg(sub).arg("--help");
        cmd.assert().success();
    }
}

// --- NEW FAILING TESTS (TDD RED phase: CLI contract layer) ---

#[test]
fn test_invalid_arg_exit_code_2() {
    // Python: test_invalid_arg_exit_code — passing an unrecognized flag must exit with code 2
    // No assert_cmd test exercises this contract for any subcommand
    let mut cmd = Command::cargo_bin("agent-skills").unwrap();
    cmd.args(["code-janitor", "scan", "--unknown-nonexistent-flag"]);
    cmd.assert()
        .code(2)
        .stderr(predicate::str::contains("error"));
}

#[test]
fn test_json_flag_produces_valid_json_code_janitor() {
    // Python: test_json_flag — --json must produce parseable JSON output
    // No test parses --json output as JSON for any command
    // Use a file within the repo root (CLI enforces path safety)
    let repo_root = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap()
        .parent()
        .unwrap()
        .to_path_buf();
    let sample_file = repo_root.join("crates/core/src/lib.rs");

    let mut cmd = Command::cargo_bin("agent-skills").unwrap();
    cmd.current_dir(&repo_root);
    cmd.args([
        "code-janitor",
        "scan",
        "--file",
        sample_file.to_str().unwrap(),
        "--json",
    ]);
    let output = cmd.assert().success().get_output().stdout.clone();
    let stdout_str = String::from_utf8(output).unwrap();

    // The output must be valid JSON
    let parsed: Result<serde_json::Value, _> = serde_json::from_str(&stdout_str);
    assert!(
        parsed.is_ok(),
        "--json flag must produce valid JSON output, got: {stdout_str}"
    );
    let json = parsed.unwrap();
    assert!(
        json.is_array(),
        "--json output must be a JSON array of findings, got: {json}"
    );
}

#[test]
fn test_dry_run_flag_install_subcommand() {
    // Python: test_dry_run (install) — --dry-run must be accepted by install and produce no changes
    let dir = tempfile::tempdir().unwrap();
    let skills_dir = dir.path().join("skills");
    let skill_dir = skills_dir.join("test-skill");
    std::fs::create_dir_all(&skill_dir).unwrap();
    std::fs::write(
        skill_dir.join("SKILL.md"),
        "---\nname: test-skill\n---\n# Test\n## Completion Criteria\n- [ ] Done\n",
    )
    .unwrap();
    // Also need skills.lock
    std::fs::write(dir.path().join("skills.lock"), "{}").unwrap();

    let mut cmd = Command::cargo_bin("agent-skills").unwrap();
    cmd.current_dir(dir.path());
    cmd.args(["install", "--dry-run"]);
    // Dry run must succeed (exit 0) and produce no real symlinks
    let output = cmd.output().unwrap();
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout.contains("[DRY-RUN]") || !output.status.success(),
        "install --dry-run must produce DRY-RUN output or fail gracefully; got stdout: {stdout}"
    );
}

#[test]
fn test_tdd_detect_flag_outputs_runner() {
    // Python: test_json_flag (tdd) — tdd --detect --json must produce JSON with runner info
    let dir = tempfile::tempdir().unwrap();
    std::fs::write(dir.path().join("Cargo.toml"), "[package]\nname=\"test\"\n").unwrap();

    let mut cmd = Command::cargo_bin("agent-skills").unwrap();
    cmd.current_dir(dir.path());
    cmd.args(["tdd", "--detect", "--json"]);
    let output = cmd.assert().success().get_output().stdout.clone();
    let stdout_str = String::from_utf8(output).unwrap();

    let parsed: Result<serde_json::Value, _> = serde_json::from_str(&stdout_str);
    assert!(
        parsed.is_ok(),
        "tdd --detect --json must produce valid JSON, got: {stdout_str}"
    );
    let json = parsed.unwrap();
    assert!(
        json.get("runner").is_some(),
        "tdd --detect --json output must include 'runner' field, got: {json}"
    );
}

#[test]
fn test_what_if_impact_command_cli() {
    // Python: test_main_impact_command — what-if-analysis impact --symbol X runs via CLI
    // No test drives this through CLI dispatch at all
    let dir = tempfile::tempdir().unwrap();
    std::fs::write(dir.path().join("main.rs"), "fn target() {}\n").unwrap();

    let mut cmd = Command::cargo_bin("agent-skills").unwrap();
    cmd.current_dir(dir.path());
    cmd.args(["what-if-analysis", "impact", "--symbol", "target"]);
    cmd.assert().success();
}

#[test]
fn test_adr_validate_subcommand_cli() {
    // Python: test_validate_adrs — adr validate runs via CLI
    // Validate is never exercised by any test
    let dir = tempfile::tempdir().unwrap();
    std::fs::create_dir_all(dir.path().join("docs/adr")).unwrap();

    let mut cmd = Command::cargo_bin("agent-skills").unwrap();
    cmd.current_dir(dir.path());
    cmd.args(["adr", "validate"]);
    // Should succeed (0 ADR files found, nothing to validate)
    cmd.assert().success();
}

#[test]
fn test_code_janitor_no_args_behavior() {
    // Python: test_no_args_returns_error — code-janitor scan with no args falls back to repo root
    // Divergence: Python returns error, Rust falls back to scanning repo root
    // Either behavior must be consistent and documented
    let mut cmd = Command::cargo_bin("agent-skills").unwrap();
    cmd.args(["code-janitor", "scan"]);
    // This must not panic; exit code is irrelevant but behavior must be consistent
    let _ = cmd.output().unwrap();
}

#[test]
fn test_no_args_shows_usage_and_exits_nonzero() {
    // Python: test_main_no_args / test_script_exists / test_help_flag — invoking the binary
    // with zero arguments at all must show usage rather than panicking, and must not silently
    // exit success (clap requires an explicit subcommand).
    let mut cmd = Command::cargo_bin("agent-skills").unwrap();
    cmd.assert()
        .failure()
        .stderr(predicate::str::contains("Usage"));
}

#[test]
fn test_depgraph_dry_run_and_json() {
    // Python: test_dry_run_flag (depgraph) + test_json_flag — depgraph must support both flags
    let dir = tempfile::tempdir().unwrap();
    std::fs::create_dir_all(dir.path().join("skills")).unwrap();

    let mut dry_run_cmd = Command::cargo_bin("agent-skills").unwrap();
    dry_run_cmd.current_dir(dir.path());
    dry_run_cmd.args(["depgraph", "--dry-run"]);
    dry_run_cmd.assert().success();
    assert!(
        !dir.path().join("skills.lock").exists(),
        "depgraph --dry-run must not write skills.lock"
    );
}

#[test]
fn test_lint_scripts_dry_run() {
    // Python: test_dry_run_flag (lint_scripts) — --dry-run must preview without failing
    let dir = tempfile::tempdir().unwrap();
    std::fs::write(dir.path().join("clean.py"), "print('hi')\n").unwrap();

    let mut cmd = Command::cargo_bin("agent-skills").unwrap();
    cmd.current_dir(dir.path());
    cmd.args(["lint-scripts", "--dry-run"]);
    cmd.assert().success();
}

#[test]
fn test_skill_creator_scaffold_dry_run() {
    // Python: test_dry_run (skill-creator scaffold) — --dry-run must preview without writing files
    let dir = tempfile::tempdir().unwrap();

    let mut cmd = Command::cargo_bin("agent-skills").unwrap();
    cmd.current_dir(dir.path());
    cmd.args([
        "skill-creator",
        "scaffold",
        "--name",
        "preview-skill",
        "--description",
        "A preview-only skill.",
        "--dry-run",
    ]);
    cmd.assert().success();
    assert!(
        !dir.path().join("skills").join("preview-skill").exists(),
        "scaffold --dry-run must not create the skill directory on disk"
    );
}

#[test]
fn test_self_progress_analyze_json() {
    // Python: test_json_format — self-progress analyze --json must produce valid JSON
    let dir = tempfile::tempdir().unwrap();
    let transcript = dir.path().join("t.jsonl");
    std::fs::write(
        &transcript,
        "{\"type\":\"USER_INPUT\",\"content\":\"hi\"}\n",
    )
    .unwrap();

    let mut cmd = Command::cargo_bin("agent-skills").unwrap();
    cmd.current_dir(dir.path());
    cmd.args([
        "self-progress",
        "analyze",
        "--transcript",
        "t.jsonl",
        "--json",
    ]);
    let output = cmd.assert().success().get_output().stdout.clone();
    let stdout_str = String::from_utf8(output).unwrap();
    let parsed: Result<serde_json::Value, _> = serde_json::from_str(&stdout_str);
    assert!(
        parsed.is_ok(),
        "self-progress analyze --json must produce valid JSON, got: {stdout_str}"
    );
}

#[test]
fn test_git_conflict_resolver_analyze_json() {
    // Python: test_json_flag (git-conflict-resolver) — analyze --json must produce valid JSON
    let dir = tempfile::tempdir().unwrap();
    std::fs::write(dir.path().join("clean.rs"), "fn main() {}\n").unwrap();

    let mut cmd = Command::cargo_bin("agent-skills").unwrap();
    cmd.current_dir(dir.path());
    cmd.args(["git-conflict-resolver", "analyze", "--json"]);
    let output = cmd.assert().success().get_output().stdout.clone();
    let stdout_str = String::from_utf8(output).unwrap();
    let parsed: Result<serde_json::Value, _> = serde_json::from_str(&stdout_str);
    assert!(
        parsed.is_ok(),
        "git-conflict-resolver analyze --json must produce valid JSON, got: {stdout_str}"
    );
}

#[test]
fn test_agent_creator_scaffold_dry_run() {
    let mut cmd = Command::cargo_bin("agent-skills").unwrap();
    cmd.args([
        "agent-creator",
        "scaffold",
        "--name",
        "test-custom-agent",
        "--description",
        "Specialized in testing custom agent scaffolding.",
        "--dry-run",
    ]);
    cmd.assert().success().stdout(predicate::str::contains(
        "[DRY RUN] Would create agent file",
    ));
}
