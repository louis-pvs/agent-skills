#!/usr/bin/env python3
"""Git Conflict Analyzer CLI Utility.

Inspects Git repository state during rebase or merge operations, parses 3-way conflict
markers across unmerged files, extracts commit history context for opposing branches,
and verifies clean removal of conflict markers post-resolution.

Adheres to ADR 0001 (Stdlib First) and ADR 0005 (Config Safety).
"""

import argparse
import json
import re
import subprocess
import sys
from pathlib import Path
from typing import Any, Dict, List, Optional, Tuple

_repo_root = Path(__file__).resolve().parents[3]
if str(_repo_root) not in sys.path:
    sys.path.insert(0, str(_repo_root))

try:
    from scripts._config_safety import load_skill_config
except ImportError:

    def load_skill_config(skill_name: str, cli_args: Optional[Dict[str, Any]] = None) -> Dict[str, Any]:
        return cli_args or {}


try:
    from scripts._path_safety import sanitize_path
except ImportError:

    def sanitize_path(p: Any, base_dir: Optional[Path] = None) -> Path:
        base = base_dir.resolve() if base_dir else Path.cwd().resolve()
        candidate = Path(p).resolve()
        try:
            candidate.relative_to(base)
        except ValueError:
            pass
        return candidate


CONFLICT_START_REGEX = re.compile(r"^<{7}\s*(.*)$")
CONFLICT_BASE_REGEX = re.compile(r"^\|{7}\s*(.*)$")
CONFLICT_SEP_REGEX = re.compile(r"^={7}\s*$")
CONFLICT_END_REGEX = re.compile(r"^>{7}\s*(.*)$")


def run_git_command(args: List[str], cwd: Path) -> Tuple[int, str, str]:
    """Executes a git CLI command safely within the given working directory."""
    try:
        res = subprocess.run(
            ["git"] + args,
            cwd=str(cwd),
            capture_output=True,
            text=True,
            check=False,
        )
        return res.returncode, res.stdout, res.stderr
    except FileNotFoundError:
        return 127, "", "git executable not found in PATH"
    except Exception as err:
        return 1, "", str(err)


def detect_git_state(repo_dir: Path) -> Dict[str, Any]:
    """Detects active Git operational state (merge, rebase, cherry-pick, revert, or none)."""
    git_dir = repo_dir / ".git"

    # Handle git worktrees or submodules where .git is a file
    if git_dir.is_file():
        try:
            content = git_dir.read_text(encoding="utf-8").strip()
            if content.startswith("gitdir:"):
                git_dir = Path(content.split("gitdir:", 1)[1].strip())
                if not git_dir.is_absolute():
                    git_dir = (repo_dir / git_dir).resolve()
        except Exception:
            pass

    state_info: Dict[str, Any] = {
        "is_in_conflict": False,
        "operation": "none",
        "ours_branch": "HEAD",
        "theirs_branch": "UNKNOWN",
        "merge_head": None,
        "rebase_head": None,
    }

    if (git_dir / "MERGE_HEAD").exists():
        state_info["is_in_conflict"] = True
        state_info["operation"] = "merge"
        try:
            state_info["merge_head"] = (git_dir / "MERGE_HEAD").read_text(encoding="utf-8").strip()
        except Exception:
            pass
        _, branch_out, _ = run_git_command(["rev-parse", "--abbrev-ref", "HEAD"], repo_dir)
        state_info["ours_branch"] = branch_out.strip() if branch_out else "HEAD"
        state_info["theirs_branch"] = "MERGE_HEAD"

    elif (git_dir / "rebase-merge").exists() or (git_dir / "rebase-apply").exists():
        state_info["is_in_conflict"] = True
        state_info["operation"] = "rebase"
        rebase_dir = git_dir / "rebase-merge" if (git_dir / "rebase-merge").exists() else git_dir / "rebase-apply"
        head_name = rebase_dir / "head-name"
        onto_file = rebase_dir / "onto"
        if head_name.exists():
            try:
                state_info["ours_branch"] = head_name.read_text(encoding="utf-8").strip().replace("refs/heads/", "")
            except Exception:
                pass
        if onto_file.exists():
            try:
                state_info["theirs_branch"] = onto_file.read_text(encoding="utf-8").strip()
            except Exception:
                pass

    elif (git_dir / "CHERRY_PICK_HEAD").exists():
        state_info["is_in_conflict"] = True
        state_info["operation"] = "cherry-pick"
    elif (git_dir / "REVERT_HEAD").exists():
        state_info["is_in_conflict"] = True
        state_info["operation"] = "revert"

    # Also check git status porcelain for unmerged files
    code, stdout, _ = run_git_command(["status", "--porcelain"], repo_dir)
    unmerged_statuses = {"UU", "AA", "UD", "DU", "UA", "AU", "DD"}
    unmerged_files = []
    if code == 0:
        for line in stdout.splitlines():
            if len(line) >= 4:
                status_code = line[:2].strip()
                file_path_str = line[3:].strip()
                if status_code in unmerged_statuses or line[:2] in ("UU", "AA", "UD", "DU", "UA", "AU", "DD"):
                    unmerged_files.append(file_path_str)

    if unmerged_files:
        state_info["is_in_conflict"] = True
    state_info["unmerged_files_count"] = len(unmerged_files)
    state_info["unmerged_files"] = unmerged_files

    return state_info


def parse_conflict_markers(content: str) -> List[Dict[str, Any]]:
    """Parses conflict blocks from file content string into structured dictionaries."""
    lines = content.splitlines()
    conflicts: List[Dict[str, Any]] = []

    in_conflict = False
    in_base = False
    in_theirs = False

    current_ours: List[str] = []
    current_base: List[str] = []
    current_theirs: List[str] = []
    ours_label = ""
    base_label = ""
    theirs_label = ""
    start_line = 0

    for idx, line in enumerate(lines, 1):
        m_start = CONFLICT_START_REGEX.match(line)
        m_base = CONFLICT_BASE_REGEX.match(line)
        m_sep = CONFLICT_SEP_REGEX.match(line)
        m_end = CONFLICT_END_REGEX.match(line)

        if m_start:
            in_conflict = True
            in_base = False
            in_theirs = False
            ours_label = m_start.group(1).strip()
            current_ours = []
            current_base = []
            current_theirs = []
            start_line = idx
        elif m_base and in_conflict:
            in_base = True
            base_label = m_base.group(1).strip()
        elif m_sep and in_conflict:
            in_base = False
            in_theirs = True
        elif m_end and in_conflict:
            theirs_label = m_end.group(1).strip()
            conflicts.append(
                {
                    "start_line": start_line,
                    "end_line": idx,
                    "ours_label": ours_label or "HEAD",
                    "base_label": base_label or "BASE",
                    "theirs_label": theirs_label or "INCOMING",
                    "ours_content": "\n".join(current_ours),
                    "base_content": "\n".join(current_base) if base_label or current_base else None,
                    "theirs_content": "\n".join(current_theirs),
                }
            )
            in_conflict = False
            in_base = False
            in_theirs = False
        elif in_conflict:
            if in_theirs:
                current_theirs.append(line)
            elif in_base:
                current_base.append(line)
            else:
                current_ours.append(line)

    return conflicts


def get_commit_history(repo_dir: Path, max_commits: int = 5) -> Dict[str, List[Dict[str, str]]]:
    """Retrieves recent commit metadata for HEAD and MERGE_HEAD / incoming branch."""
    history: Dict[str, List[Dict[str, str]]] = {"ours": [], "theirs": []}

    code, stdout, _ = run_git_command(
        ["log", f"-n{max_commits}", "--pretty=format:%H|%an|%s"],
        repo_dir,
    )
    if code == 0 and stdout:
        for line in stdout.splitlines():
            parts = line.split("|", 2)
            if len(parts) == 3:
                history["ours"].append({"hash": parts[0], "author": parts[1], "subject": parts[2]})

    git_dir = repo_dir / ".git"
    if (git_dir / "MERGE_HEAD").exists():
        code, stdout, _ = run_git_command(
            ["log", f"-n{max_commits}", "--pretty=format:%H|%an|%s", "MERGE_HEAD"],
            repo_dir,
        )
        if code == 0 and stdout:
            for line in stdout.splitlines():
                parts = line.split("|", 2)
                if len(parts) == 3:
                    history["theirs"].append({"hash": parts[0], "author": parts[1], "subject": parts[2]})

    return history


def verify_zero_markers_in_content(content: str) -> Tuple[bool, List[str]]:
    """Checks if any unhandled conflict markers remain in the file content."""
    lines = content.splitlines()
    violations = []
    for idx, line in enumerate(lines, 1):
        if CONFLICT_START_REGEX.match(line):
            violations.append(f"Line {idx}: Unresolved start marker '<<<<<<<'")
        elif CONFLICT_BASE_REGEX.match(line):
            violations.append(f"Line {idx}: Unresolved base marker '|||||||'")
        elif CONFLICT_SEP_REGEX.match(line):
            violations.append(f"Line {idx}: Unresolved separator marker '======='")
        elif CONFLICT_END_REGEX.match(line):
            violations.append(f"Line {idx}: Unresolved end marker '>>>>>>>'")

    return len(violations) == 0, violations


def analyze_repository_conflicts(repo_dir: Path, max_commits: int = 5) -> Dict[str, Any]:
    """Performs full diagnostic analysis of current git conflicts in the repository."""
    state = detect_git_state(repo_dir)
    history = get_commit_history(repo_dir, max_commits=max_commits)

    file_analyses: List[Dict[str, Any]] = []

    for rel_path_str in state.get("unmerged_files", []):
        file_path = repo_dir / rel_path_str
        analysis: Dict[str, Any] = {
            "relative_path": rel_path_str,
            "exists": file_path.exists(),
            "conflict_blocks_count": 0,
            "conflict_blocks": [],
            "error": None,
        }

        if file_path.exists():
            try:
                content = file_path.read_text(encoding="utf-8", errors="replace")
                blocks = parse_conflict_markers(content)
                analysis["conflict_blocks_count"] = len(blocks)
                analysis["conflict_blocks"] = blocks
            except Exception as err:
                analysis["error"] = f"Failed to read file: {err}"
        else:
            analysis["error"] = "File listed as unmerged but does not exist on disk (deleted/modified conflict)."

        file_analyses.append(analysis)

    return {
        "git_state": state,
        "commit_history": history,
        "analyzed_files": file_analyses,
    }


def parse_args(args: Optional[List[str]] = None) -> argparse.Namespace:
    parser = argparse.ArgumentParser(description="Git Conflict Analyzer CLI Utility")
    parser.add_argument(
        "--repo-dir",
        type=str,
        default=".",
        help="Path to repository root (defaults to current directory)",
    )
    parser.add_argument("--check", action="store_true", help="Check if repository is currently in a conflict state")
    parser.add_argument("--analyze", action="store_true", help="Perform deep analysis of all conflicting files")
    parser.add_argument(
        "--verify",
        type=str,
        nargs="*",
        help="Verify specified file(s) or entire repo for zero remaining conflict markers",
    )
    parser.add_argument("--json", action="store_true", help="Format output as JSON")
    parser.add_argument("--max-commits", type=int, default=5, help="Number of recent commit log items to fetch")
    return parser.parse_args(args)


def main() -> int:
    args = parse_args()
    repo_dir = sanitize_path(Path(args.repo_dir))

    cfg = load_skill_config("git-conflict-resolver", vars(args))
    max_commits = cfg.get("max_commits", args.max_commits)

    if args.check:
        state = detect_git_state(repo_dir)
        if args.json:
            print(json.dumps(state, indent=2))
        else:
            if state["is_in_conflict"]:
                print(
                    f"⚠️  Git Repository is IN CONFLICT ({state['operation'].upper()} active). "
                    f"Unmerged files: {state['unmerged_files_count']}"
                )
            else:
                print("✅ Git Repository is clean. No active merge or rebase conflicts detected.")
        return 0 if not state["is_in_conflict"] else 2

    if args.verify is not None:
        targets = args.verify
        all_clean = True
        results: Dict[str, Any] = {"clean": True, "files": {}}

        if not targets or "." in targets or "all" in targets:
            state = detect_git_state(repo_dir)
            files_to_check = [repo_dir / f for f in state.get("unmerged_files", [])]
            if not files_to_check:
                # Scan modified tracked files if no porcelain unmerged list
                code, stdout, _ = run_git_command(["diff", "--name-only"], repo_dir)
                if code == 0:
                    files_to_check = [repo_dir / f.strip() for f in stdout.splitlines() if f.strip()]
        else:
            files_to_check = [repo_dir / f for f in targets]

        for file_path in files_to_check:
            rel_str = str(file_path.relative_to(repo_dir)) if file_path.is_relative_to(repo_dir) else str(file_path)
            if not file_path.exists():
                results["files"][rel_str] = {"clean": True, "status": "file_not_found"}
                continue
            content = file_path.read_text(encoding="utf-8", errors="replace")
            is_clean, violations = verify_zero_markers_in_content(content)
            results["files"][rel_str] = {"clean": is_clean, "violations": violations}
            if not is_clean:
                all_clean = False
                results["clean"] = False

        if args.json:
            print(json.dumps(results, indent=2))
        else:
            if all_clean:
                print("✅ All checked files are free of conflict markers!")
            else:
                print("❌ Conflict markers found in checked files:")
                for fname, info in results["files"].items():
                    if not info.get("clean"):
                        print(f"  - {fname}:")
                        for v in info.get("violations", []):
                            print(f"      {v}")
        return 0 if all_clean else 1

    # Default action: analyze
    results = analyze_repository_conflicts(repo_dir, max_commits=max_commits)
    if args.json:
        print(json.dumps(results, indent=2))
    else:
        state = results["git_state"]
        print("=== Git Conflict Analysis Report ===")
        print(f"Conflict State: {'Active' if state['is_in_conflict'] else 'None'}")
        print(f"Operation: {state['operation']}")
        print(f"Ours Branch: {state['ours_branch']} | Theirs Branch: {state['theirs_branch']}")
        print(f"Unmerged Files: {state['unmerged_files_count']}\n")

        for f_info in results["analyzed_files"]:
            print(f"File: {f_info['relative_path']}")
            if f_info.get("error"):
                print(f"  Error: {f_info['error']}")
            else:
                print(f"  Conflict Blocks: {f_info['conflict_blocks_count']}")
                for i, block in enumerate(f_info["conflict_blocks"], 1):
                    print(f"  Block #{i} (Lines {block['start_line']}-{block['end_line']}):")
                    print(f"    OURS ({block['ours_label']}): {len(block['ours_content'].splitlines())} lines")
                    if block["base_content"] is not None:
                        print(f"    BASE ({block['base_label']}): {len(block['base_content'].splitlines())} lines")
                    print(f"    THEIRS ({block['theirs_label']}): {len(block['theirs_content'].splitlines())} lines")
            print()

    return 0


if __name__ == "__main__":
    sys.exit(main())
