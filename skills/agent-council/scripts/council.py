#!/usr/bin/env python3
"""Agent Council Python Orchestrator.

Queries multiple AI agent CLIs in parallel, tracks status, and synthesizes results.
Uses Python Standard Library only to ensure zero-dependency execution across platforms.
"""

import argparse
import hashlib
import json
import os
import shlex
import signal
import subprocess
import sys
import time
from pathlib import Path
from typing import Any, Dict

SCRIPT_DIR = Path(__file__).parent.resolve()
SKILL_DIR = SCRIPT_DIR.parent

# Dynamic sys.path Root Resolution
_repo_root = Path(__file__).resolve().parents[3]
if str(_repo_root) not in sys.path:
    sys.path.insert(0, str(_repo_root))

try:
    from scripts._path_safety import get_repo_root

    REPO_ROOT = get_repo_root(__file__)
except ImportError:
    REPO_ROOT = SKILL_DIR.parent.parent

SKILL_CONFIG_FILE = SKILL_DIR / "council.config.yaml"
REPO_CONFIG_FILE = REPO_ROOT / "council.config.yaml"


def _parse_val(val: str) -> Any:
    val = val.strip().strip("\"'")
    if val.lower() == "true":
        return True
    if val.lower() == "false":
        return False
    if val.isdigit():
        return int(val)
    try:
        return float(val)
    except ValueError:
        return val


def parse_simple_yaml(content: str) -> Dict[str, Any]:
    """Lightweight zero-dependency YAML parser for council.config.yaml."""
    data: Dict[str, Any] = {}
    current_section: Dict[str, Any] = {}
    current_key = None
    current_list_item: Dict[str, Any] = {}

    lines = content.splitlines()
    i = 0
    while i < len(lines):
        line = lines[i].split("#")[0].rstrip()
        if not line:
            i += 1
            continue

        indent = len(line) - len(line.lstrip(" "))
        stripped = line.strip()

        if stripped.endswith(":") and not stripped.startswith("-"):
            key = stripped[:-1].strip()
            if indent == 0:
                data[key] = {}
                current_section = data[key]
            elif indent == 2 and isinstance(current_section, dict):
                if key == "members":
                    current_section[key] = []
                    current_key = "members"
                else:
                    current_section[key] = {}
                    current_key = key
            i += 1
            continue

        if stripped.startswith("- ") and isinstance(current_section, dict):
            rest = stripped[2:].strip()
            if ":" in rest:
                k, v = rest.split(":", 1)
                item = {k.strip(): _parse_val(v.strip())}
                if current_key == "members":
                    current_section["members"].append(item)
                    current_list_item = item
            i += 1
            continue

        if indent >= 6 and current_list_item and ":" in stripped:
            k, v = stripped.split(":", 1)
            current_list_item[k.strip()] = _parse_val(v.strip())

        elif indent == 4 and current_key and current_key != "members" and ":" in stripped:
            k, v = stripped.split(":", 1)
            if isinstance(current_section.get(current_key), dict):
                current_section[current_key][k.strip()] = _parse_val(v.strip())

        i += 1

    return data


def load_config() -> Dict[str, Any]:
    """Loads and parses council.config.yaml with robust defaults."""
    default_config = {
        "council": {
            "chairman": {"role": "auto"},
            "members": [
                {"name": "claude", "command": "claude -p", "emoji": "🧠", "color": "CYAN"},
                {"name": "codex", "command": "codex exec", "emoji": "🤖", "color": "BLUE"},
                {"name": "gemini", "command": "agy -p", "emoji": "💎", "color": "GREEN"},
                {"name": "copilot", "command": "copilot -p", "emoji": "✈️", "color": "BLUE"},
            ],
            "settings": {"exclude_chairman_from_members": True, "timeout": 120},
        }
    }

    config_path = SKILL_CONFIG_FILE if SKILL_CONFIG_FILE.exists() else REPO_CONFIG_FILE
    if not config_path.exists():
        return default_config

    try:
        content = config_path.read_text(encoding="utf-8")
        parsed = parse_simple_yaml(content)
        if "council" in parsed and isinstance(parsed["council"], dict):
            c = parsed["council"]
            if "members" in c and isinstance(c["members"], list) and len(c["members"]) > 0:
                return parsed
    except Exception as err:
        sys.stderr.write(f"Warning: Failed to parse {config_path}: {err}. Using defaults.\n")

    return default_config


def generate_job_id(question: str) -> str:
    """Generates unique job folder name based on timestamp and prompt hash."""
    timestamp = time.strftime("%Y-%m-%d-%H%M")
    h = hashlib.sha256(question.encode("utf-8")).hexdigest()[:6]
    return f"council-{timestamp}-{h}"


def create_job(question: str, jobs_dir: Path) -> Path:
    """Creates a new council job directory and spawns member processes."""
    config = load_config()
    council_cfg = config.get("council", {})
    members = council_cfg.get("members", [])
    settings = council_cfg.get("settings", {})
    timeout = settings.get("timeout", 120)

    job_name = generate_job_id(question)
    job_dir = jobs_dir / job_name
    job_dir.mkdir(parents=True, exist_ok=True)

    job_info = {
        "job_id": job_name,
        "question": question,
        "created_at": time.time(),
        "timeout": timeout,
        "members": [m.get("name") for m in members if isinstance(m, dict)],
    }
    (job_dir / "job.json").write_text(json.dumps(job_info, indent=2), encoding="utf-8")

    status_data = {
        "overallState": "running",
        "members": {},
    }

    for member in members:
        if not isinstance(member, dict):
            continue
        m_name = member.get("name")
        cmd_str = member.get("command")
        if not m_name or not cmd_str:
            continue

        log_file = job_dir / f"{m_name}.log"
        err_file = job_dir / f"{m_name}.err"

        status_data["members"][m_name] = {
            "state": "running",
            "emoji": member.get("emoji", "🤖"),
            "color": member.get("color", "BLUE"),
            "start_time": time.time(),
        }

        cmd_args = shlex.split(cmd_str) + [question]
        with open(log_file, "w", encoding="utf-8") as out_f, open(err_file, "w", encoding="utf-8") as err_f:
            proc = subprocess.Popen(
                cmd_args,
                stdout=out_f,
                stderr=err_f,
                cwd=str(REPO_ROOT),
            )
            pid_file = job_dir / f"{m_name}.pid"
            pid_file.write_text(str(proc.pid), encoding="utf-8")

    (job_dir / "status.json").write_text(json.dumps(status_data, indent=2), encoding="utf-8")
    return job_dir


def update_job_status(job_dir: Path) -> Dict[str, Any]:
    """Checks running processes and updates status.json."""
    status_file = job_dir / "status.json"
    job_file = job_dir / "job.json"

    if not status_file.exists() or not job_file.exists():
        return {"overallState": "unknown", "members": {}}

    status = json.loads(status_file.read_text(encoding="utf-8"))
    job_info = json.loads(job_file.read_text(encoding="utf-8"))
    timeout = job_info.get("timeout", 120)

    now = time.time()
    all_done = True

    for m_name, m_info in status.get("members", {}).items():
        if m_info.get("state") != "running":
            continue

        pid_file = job_dir / f"{m_name}.pid"
        start_time = m_info.get("start_time", now)

        is_running = False
        if pid_file.exists():
            try:
                pid = int(pid_file.read_text().strip())
                os.kill(pid, 0)
                is_running = True
            except (OSError, ValueError):
                is_running = False

        if not is_running:
            m_info["state"] = "done"
            m_info["end_time"] = now
        elif (now - start_time) > timeout:
            if pid_file.exists():
                try:
                    pid = int(pid_file.read_text().strip())
                    os.kill(pid, signal.SIGTERM)
                except OSError:
                    pass
            m_info["state"] = "timed_out"
            m_info["end_time"] = now
        else:
            all_done = False

    status["overallState"] = "done" if all_done else "running"
    status_file.write_text(json.dumps(status, indent=2), encoding="utf-8")
    return status


def get_results(job_dir: Path, is_json: bool = False) -> str:
    """Collects outputs from log files for all council members."""
    status = update_job_status(job_dir)
    results = {}

    for m_name, m_info in status.get("members", {}).items():
        log_file = job_dir / f"{m_name}.log"
        err_file = job_dir / f"{m_name}.err"

        output = ""
        if log_file.exists():
            output = log_file.read_text(encoding="utf-8").strip()
        if not output and err_file.exists():
            output = err_file.read_text(encoding="utf-8").strip()

        results[m_name] = {
            "state": m_info.get("state"),
            "emoji": m_info.get("emoji", "🤖"),
            "output": output if output else f"No output returned (State: {m_info.get('state')})",
        }

    if is_json:
        return json.dumps(results, indent=2)

    lines = []
    for m_name, res in results.items():
        lines.append(f"=== {res['emoji']} {m_name} ({res['state']}) ===")
        lines.append(res["output"])
        lines.append("")

    return "\n".join(lines)


def stop_job(job_dir: Path) -> None:
    """Terminates all running member processes for a job."""
    for pid_file in job_dir.glob("*.pid"):
        try:
            pid = int(pid_file.read_text().strip())
            os.kill(pid, signal.SIGTERM)
        except OSError:
            pass


def clean_job(job_dir: Path) -> None:
    """Removes job directory."""
    stop_job(job_dir)
    resolved_dir = job_dir.resolve()
    if not resolved_dir.exists():
        return
    safe_name = resolved_dir.name
    if not safe_name or safe_name in (".", ".."):
        return
    import shutil

    shutil.rmtree(str(resolved_dir), ignore_errors=True)


def parse_args():
    if len(sys.argv) > 1 and sys.argv[1] not in {"start", "status", "wait", "results", "stop", "clean", "-h", "--help"}:
        return argparse.Namespace(subcommand=None, prompt=" ".join(sys.argv[1:]))

    parser = argparse.ArgumentParser(description="Agent Council Orchestrator")
    subparsers = parser.add_subparsers(dest="subcommand")

    # Start
    start_parser = subparsers.add_parser("start")
    start_parser.add_argument("question", type=str)
    start_parser.add_argument("--dry-run", action="store_true", help="Preview council member commands without spawning CLIs")

    # Status
    status_parser = subparsers.add_parser("status")
    status_parser.add_argument("job_dir", type=str)
    status_parser.add_argument("--json", action="store_true")
    status_parser.add_argument("--text", action="store_true")
    status_parser.add_argument("--checklist", action="store_true")

    # Wait
    wait_parser = subparsers.add_parser("wait")
    wait_parser.add_argument("job_dir", type=str)

    # Results
    results_parser = subparsers.add_parser("results")
    results_parser.add_argument("job_dir", type=str)
    results_parser.add_argument("--json", action="store_true")

    # Stop
    stop_parser = subparsers.add_parser("stop")
    stop_parser.add_argument("job_dir", type=str)

    # Clean
    clean_parser = subparsers.add_parser("clean")
    clean_parser.add_argument("job_dir", type=str)

    return parser.parse_args()


def main() -> int:
    args = parse_args()
    jobs_dir = SKILL_DIR / ".jobs"

    if args.subcommand == "start":
        if getattr(args, "dry_run", False):
            config = load_config()
            members = [m.get("name") for m in config.get("council", {}).get("members", []) if isinstance(m, dict)]
            print(f"[DRY-RUN] Would launch Council job for question: '{args.question}' with members: {members}")
            return 0
        job_dir = create_job(args.question, jobs_dir)
        print(str(job_dir.resolve()))
        return 0

    elif args.subcommand == "status":
        job_dir = (jobs_dir / os.path.basename(args.job_dir)).resolve()
        status = update_job_status(job_dir)
        if args.json:
            print(json.dumps(status, indent=2))
        else:
            total = len(status.get("members", {}))
            done = sum(1 for m in status.get("members", {}).values() if m.get("state") != "running")
            print(f"members {done}/{total} done; overall: {status.get('overallState')}")
        return 0

    elif args.subcommand == "wait":
        job_dir = (jobs_dir / os.path.basename(args.job_dir)).resolve()
        while True:
            status = update_job_status(job_dir)
            if status.get("overallState") == "done":
                break
            time.sleep(1)
        print(json.dumps(status, indent=2))
        return 0

    elif args.subcommand == "results":
        job_dir = (jobs_dir / os.path.basename(args.job_dir)).resolve()
        print(get_results(job_dir, is_json=args.json))
        return 0

    elif args.subcommand == "stop":
        job_dir = (jobs_dir / os.path.basename(args.job_dir)).resolve()
        stop_job(job_dir)
        return 0

    elif args.subcommand == "clean":
        job_dir = (jobs_dir / os.path.basename(args.job_dir)).resolve()
        clean_job(job_dir)
        return 0

    elif args.prompt:
        # One-shot mode
        job_dir = create_job(args.prompt, jobs_dir)
        while True:
            status = update_job_status(job_dir)
            if status.get("overallState") == "done":
                break
            time.sleep(1)
        print(get_results(job_dir))
        clean_job(job_dir)
        return 0

    else:
        print('Usage: council.py [start|status|wait|results|stop|clean|"prompt"]')
        return 1


if __name__ == "__main__":
    sys.exit(main())
