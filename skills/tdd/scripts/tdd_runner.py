#!/usr/bin/env python3
"""TDD Test Runner & State Verifier.

Auto-detects project test framework and enforces TDD state validation
(verifying RED state failures and GREEN state passes).
"""

import argparse
import json
import shlex
import subprocess
import sys
from pathlib import Path
from typing import List, Optional, Tuple


def detect_test_runner(workspace_dir: Path) -> Tuple[Optional[str], Optional[List[str]]]:
    """Detect test runner command based on project files."""
    # Python detection
    if (
        (workspace_dir / "pyproject.toml").exists()
        or (workspace_dir / "pytest.ini").exists()
        or (workspace_dir / "setup.cfg").exists()
    ):
        return "pytest", ["pytest"]

    if list(workspace_dir.glob("**/test_*.py")) or list(workspace_dir.glob("**/*_test.py")):
        return "unittest", [sys.executable, "-m", "unittest", "discover", "-s", "."]

    # Node.js / JS / TS detection
    package_json = workspace_dir / "package.json"
    if package_json.exists():
        try:
            with open(package_json, encoding="utf-8") as f:
                data = json.load(f)
                scripts = data.get("scripts", {})
                if "test" in scripts:
                    return "npm test", ["npm", "test"]
        except Exception:
            pass
        return "jest", ["npx", "jest"]

    # Go detection
    if (workspace_dir / "go.mod").exists():
        return "go test", ["go", "test", "./..."]

    # Rust detection
    if (workspace_dir / "Cargo.toml").exists():
        return "cargo test", ["cargo", "test"]

    return None, None


def run_test_command(cmd: List[str], cwd: Path) -> Tuple[int, str, str]:
    """Execute test command and return exit code, stdout, stderr."""
    try:
        proc = subprocess.run(
            cmd,
            cwd=str(cwd),
            capture_output=True,
            text=True,
            timeout=120,
        )
        return proc.returncode, proc.stdout, proc.stderr
    except subprocess.TimeoutExpired:
        return 124, "", "Test execution timed out after 120 seconds"
    except Exception as e:
        return 1, "", str(e)


def main() -> int:
    parser = argparse.ArgumentParser(description="TDD Runner and State Verifier")
    parser.add_argument(
        "--path",
        type=str,
        default=".",
        help="Target directory to inspect and run tests in",
    )
    parser.add_argument(
        "--cmd",
        type=str,
        default=None,
        help="Custom test command override (e.g. 'pytest tests/test_foo.py')",
    )
    parser.add_argument(
        "--detect",
        action="store_true",
        help="Output detected test runner and exit",
    )
    parser.add_argument(
        "--verify-red",
        action="store_true",
        help="Assert that test execution FAILS (RED state)",
    )
    parser.add_argument(
        "--verify-green",
        action="store_true",
        help="Assert that test execution PASSES (GREEN state)",
    )
    parser.add_argument(
        "--json",
        action="store_true",
        help="Output result in JSON format",
    )

    args = parser.parse_args()
    target_path = Path(args.path).resolve()

    if not target_path.exists():
        print(f"Error: Target path '{target_path}' does not exist.", file=sys.stderr)
        return 1

    if args.cmd:
        runner_name = "custom"
        cmd = shlex.split(args.cmd)
    else:
        runner_name, cmd = detect_test_runner(target_path)

    if args.detect:
        result = {
            "path": str(target_path),
            "runner": runner_name,
            "cmd": cmd,
        }
        if args.json:
            print(json.dumps(result, indent=2))
        else:
            if runner_name:
                print(f"Detected runner: {runner_name} ({' '.join(cmd)})")
            else:
                print("No standard test runner detected.")
        return 0 if runner_name else 1

    if not cmd:
        print("Error: Could not auto-detect a test runner. Use --cmd to specify one manually.", file=sys.stderr)
        return 1

    returncode, stdout, stderr = run_test_command(cmd, target_path)

    if args.verify_red:
        # RED state verification: tests MUST fail
        if returncode != 0:
            msg = "SUCCESS (RED Verified): Tests failed as expected."
            status = "pass"
            exit_code = 0
        else:
            msg = "FAILURE (RED Violation): Tests PASSED, but expected them to FAIL."
            status = "fail"
            exit_code = 1
    elif args.verify_green:
        # GREEN state verification: tests MUST pass
        if returncode == 0:
            msg = "SUCCESS (GREEN Verified): Tests passed as expected."
            status = "pass"
            exit_code = 0
        else:
            msg = "FAILURE (GREEN Violation): Tests FAILED, but expected them to PASS."
            status = "fail"
            exit_code = 1
    else:
        # Simple run mode
        msg = f"Test execution completed with exit code {returncode}."
        status = "pass" if returncode == 0 else "fail"
        exit_code = returncode

    if args.json:
        out = {
            "status": status,
            "verification": "red" if args.verify_red else ("green" if args.verify_green else "none"),
            "message": msg,
            "returncode": returncode,
            "command": " ".join(cmd),
            "stdout": stdout,
            "stderr": stderr,
        }
        print(json.dumps(out, indent=2))
    else:
        print(msg)
        if exit_code != 0 and (stdout or stderr):
            print("\n--- Output ---")
            if stdout:
                print(stdout)
            if stderr:
                print(stderr, file=sys.stderr)

    return exit_code


if __name__ == "__main__":
    sys.exit(main())
