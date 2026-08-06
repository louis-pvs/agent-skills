use agent_skills_core::path_safety::sanitize_path;
use clap::Args;
use serde_json::json;
use std::fs;
use std::path::Path;
use std::process::Command;

#[derive(Args, Debug, Clone)]
pub struct TddArgs {
    /// Target directory to inspect and run tests inside (defaults to cwd)
    #[arg(long, default_value = ".")]
    pub path: String,

    /// Custom test command override (e.g. 'pytest tests/test_foo.py')
    #[arg(long)]
    pub cmd: Option<String>,

    /// Output detected test runner and exit
    #[arg(long)]
    pub detect: bool,

    /// Assert that test execution FAILS (RED state)
    #[arg(long)]
    pub verify_red: bool,

    /// Assert that test execution PASSES (GREEN state)
    #[arg(long)]
    pub verify_green: bool,

    /// Output result in JSON format
    #[arg(long)]
    pub json: bool,
}

pub fn detect_test_runner(workspace_dir: &Path) -> Option<(String, Vec<String>)> {
    // Python detection
    if workspace_dir.join("pyproject.toml").exists()
        || workspace_dir.join("pytest.ini").exists()
        || workspace_dir.join("setup.cfg").exists()
    {
        return Some(("pytest".to_string(), vec!["pytest".to_string()]));
    }

    let has_py_tests = |dir: &Path| -> bool {
        let mut found = false;
        let walk = |d: &Path| -> bool {
            if let Ok(entries) = fs::read_dir(d) {
                for entry in entries.flatten() {
                    let p = entry.path();
                    if p.is_file() {
                        let fname = p.file_name().unwrap_or_default().to_string_lossy();
                        if (fname.starts_with("test_") || fname.ends_with("_test.py"))
                            && fname.ends_with(".py")
                        {
                            return true;
                        }
                    }
                }
            }
            false
        };
        if walk(dir) || (dir.join("tests").is_dir() && walk(&dir.join("tests"))) {
            found = true;
        }
        found
    };

    if has_py_tests(workspace_dir) {
        return Some((
            "unittest".to_string(),
            vec![
                "python3".to_string(),
                "-m".to_string(),
                "unittest".to_string(),
                "discover".to_string(),
                "-s".to_string(),
                ".".to_string(),
            ],
        ));
    }

    // Node.js / JS / TS detection
    let package_json = workspace_dir.join("package.json");
    if package_json.exists() {
        if let Ok(content) = fs::read_to_string(&package_json) {
            if let Ok(val) = serde_json::from_str::<serde_json::Value>(&content) {
                if val.get("scripts").and_then(|s| s.get("test")).is_some() {
                    return Some((
                        "npm test".to_string(),
                        vec!["npm".to_string(), "test".to_string()],
                    ));
                }
            }
        }
        return Some((
            "jest".to_string(),
            vec!["npx".to_string(), "jest".to_string()],
        ));
    }

    // Go detection
    if workspace_dir.join("go.mod").exists() {
        return Some((
            "go test".to_string(),
            vec!["go".to_string(), "test".to_string(), "./...".to_string()],
        ));
    }

    // Rust detection
    if workspace_dir.join("Cargo.toml").exists() {
        return Some((
            "cargo test".to_string(),
            vec!["cargo".to_string(), "test".to_string()],
        ));
    }

    None
}

pub fn run_test_command(cmd: &[String], cwd: &Path) -> (i32, String, String) {
    if cmd.is_empty() {
        return (1, String::new(), "Empty command provided.".to_string());
    }

    let program = &cmd[0];
    let args = &cmd[1..];

    match Command::new(program).args(args).current_dir(cwd).output() {
        Ok(output) => {
            let code = output.status.code().unwrap_or(1);
            let stdout = String::from_utf8_lossy(&output.stdout).to_string();
            let stderr = String::from_utf8_lossy(&output.stderr).to_string();
            (code, stdout, stderr)
        }
        Err(e) => (
            1,
            String::new(),
            format!("Failed to execute test command: {e}"),
        ),
    }
}

pub fn run_tdd_command(args: &TddArgs, repo_root: &Path) -> Result<(), String> {
    let target_path = sanitize_path(&args.path, Some(repo_root))?;

    if !target_path.exists() || !target_path.is_dir() {
        return Err(format!(
            "Error: Target path '{}' is not a valid directory.",
            target_path.display()
        ));
    }

    let (runner_name, cmd) = if let Some(custom_cmd_str) = &args.cmd {
        let parts: Vec<String> = custom_cmd_str
            .split_whitespace()
            .map(|s| s.to_string())
            .collect();
        if parts.is_empty() {
            return Err("Error: Custom command string is empty.".to_string());
        }
        ("custom".to_string(), parts)
    } else {
        match detect_test_runner(&target_path) {
            Some((r_name, r_cmd)) => (r_name, r_cmd),
            None => {
                if args.detect {
                    if args.json {
                        println!("{}", json!({ "runner": null, "cmd": null }));
                    } else {
                        println!("No standard test runner detected.");
                    }
                    return Ok(());
                }
                return Err("Error: Could not auto-detect a test runner. Use --cmd to specify one manually.".to_string());
            }
        }
    };

    if args.detect {
        if args.json {
            println!(
                "{}",
                json!({
                    "path": target_path.display().to_string(),
                    "runner": runner_name,
                    "cmd": cmd,
                })
            );
        } else {
            println!("Detected runner: {} ({})", runner_name, cmd.join(" "));
        }
        return Ok(());
    }

    let (returncode, stdout, stderr) = run_test_command(&cmd, &target_path);

    let (msg, status, exit_success) = if args.verify_red {
        if returncode != 0 {
            (
                "SUCCESS (RED Verified): Tests failed as expected.".to_string(),
                "pass".to_string(),
                true,
            )
        } else {
            (
                "FAILURE (RED Violation): Tests PASSED, but expected them to FAIL.".to_string(),
                "fail".to_string(),
                false,
            )
        }
    } else if args.verify_green {
        if returncode == 0 {
            (
                "SUCCESS (GREEN Verified): Tests passed as expected.".to_string(),
                "pass".to_string(),
                true,
            )
        } else {
            (
                "FAILURE (GREEN Violation): Tests FAILED, but expected them to PASS.".to_string(),
                "fail".to_string(),
                false,
            )
        }
    } else {
        (
            format!("Test execution completed with exit code {returncode}."),
            if returncode == 0 { "pass" } else { "fail" }.to_string(),
            returncode == 0,
        )
    };

    if args.json {
        let out = json!({
            "status": status,
            "verification": if args.verify_red { "red" } else if args.verify_green { "green" } else { "none" },
            "message": msg,
            "returncode": returncode,
            "command": cmd.join(" "),
            "stdout": stdout,
            "stderr": stderr,
        });
        println!("{}", serde_json::to_string_pretty(&out).unwrap());
    } else {
        println!("{msg}");
        if !exit_success && (!stdout.is_empty() || !stderr.is_empty()) {
            println!("\n--- Output ---");
            if !stdout.is_empty() {
                println!("{stdout}");
            }
            if !stderr.is_empty() {
                eprintln!("{stderr}");
            }
        }
    }

    if exit_success {
        Ok(())
    } else {
        Err(msg)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn test_detect_test_runner_cargo_strict_assertions() {
        let dir = tempdir().unwrap();
        let base = dir.path().canonicalize().unwrap();
        fs::write(base.join("Cargo.toml"), "[package]\nname=\"test\"\n").unwrap();

        let (runner, cmd) = detect_test_runner(&base).expect("Cargo runner must be detected");
        assert_eq!(runner, "cargo test");
        assert_eq!(cmd, vec!["cargo".to_string(), "test".to_string()]);
    }

    #[test]
    fn test_detect_test_runner_pytest_strict_assertions() {
        let dir = tempdir().unwrap();
        let base = dir.path().canonicalize().unwrap();
        fs::write(base.join("pyproject.toml"), "[tool.pytest]\n").unwrap();

        let (runner, cmd) = detect_test_runner(&base).expect("Pytest runner must be detected");
        assert_eq!(runner, "pytest");
        assert_eq!(cmd, vec!["pytest".to_string()]);
    }
}
