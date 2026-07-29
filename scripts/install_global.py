#!/usr/bin/env python3
"""Global Skill Installer.

Symlinks all skills in this repository into global configuration directories
for Gemini/Antigravity, Claude Code, and GitHub Copilot.
"""

import argparse
import os
import sys
from pathlib import Path
from typing import Dict, List

REPO_DIR = Path(__file__).parent.parent.resolve()
SKILLS_DIR = REPO_DIR / "skills"
CONFIG_FILE = REPO_DIR / "skills.config.yaml"


def load_global_skill_targets(config_path: Path = CONFIG_FILE) -> Dict[str, Path]:
    """Loads global skill target paths from skills.config.yaml with stdlib defaults."""
    default_targets = {
        "Gemini / Antigravity": Path.home() / ".gemini" / "config" / "skills",
        "Claude Code": Path.home() / ".claude" / "skills",
        "GitHub Copilot": Path.home() / ".copilot" / "skills",
    }
    if not config_path.exists():
        return default_targets

    try:
        content = config_path.read_text(encoding="utf-8")
        targets = {}
        curr_name = None
        for line in content.splitlines():
            line_str = line.split("#")[0].strip()
            if not line_str:
                continue
            if line_str.startswith("- name:"):
                curr_name = line_str.split(":", 1)[1].strip().strip("\"'")
            elif line_str.startswith("path:") and curr_name:
                raw_path = line_str.split(":", 1)[1].strip().strip("\"'")
                expanded_path = Path(os.path.expanduser(raw_path)).resolve()
                targets[curr_name] = expanded_path
                curr_name = None
        if targets:
            return targets
    except Exception:
        pass

    return default_targets


GLOBAL_SKILL_TARGETS: Dict[str, Path] = load_global_skill_targets()


def find_repo_skills(skills_dir: Path) -> List[Path]:
    """Finds all valid skill directories in the repository."""
    if not skills_dir.exists():
        return []
    skills = []
    for item in sorted(skills_dir.iterdir()):
        if item.is_dir() and (item / "SKILL.md").exists():
            skills.append(item)
    return skills


def install_skill(skill_dir: Path, target_base_dir: Path, dry_run: bool = False) -> str:
    """Symlinks a single skill into target_base_dir."""
    target_link = target_base_dir / skill_dir.name

    if not dry_run:
        target_base_dir.mkdir(parents=True, exist_ok=True)

    if target_link.is_symlink() or target_link.exists():
        try:
            current_target = target_link.resolve()
            if current_target == skill_dir.resolve():
                return f"[EXISTS] {skill_dir.name} already linked in {target_base_dir}"
        except OSError:
            pass  # Broken symlink

        if not dry_run:
            if target_link.is_symlink() or target_link.is_file():
                target_link.unlink()
            elif target_link.is_dir():
                import shutil

                shutil.rmtree(target_link)

    if dry_run:
        return f"[DRY-RUN] Would link {skill_dir.name} -> {target_link}"

    try:
        target_link.symlink_to(skill_dir, target_is_directory=True)
        return f"[LINKED] {skill_dir.name} -> {target_link}"
    except Exception as err:
        return f"[ERROR] Failed to link {skill_dir.name} in {target_base_dir}: {err}"


def uninstall_skill(skill_dir: Path, target_base_dir: Path, dry_run: bool = False) -> str:
    """Removes symlink for a single skill from target_base_dir."""
    target_link = target_base_dir / skill_dir.name

    if not target_link.exists() and not target_link.is_symlink():
        return f"[SKIP] {skill_dir.name} not found in {target_base_dir}"

    if dry_run:
        return f"[DRY-RUN] Would remove {target_link}"

    try:
        if target_link.is_symlink() or target_link.is_file():
            target_link.unlink()
        elif target_link.is_dir():
            import shutil

            shutil.rmtree(target_link)
        return f"[REMOVED] {target_link}"
    except Exception as err:
        return f"[ERROR] Failed to remove {target_link}: {err}"


def main() -> int:
    parser = argparse.ArgumentParser(
        description="Globally install or uninstall repository skills for Gemini, Claude, and Copilot."
    )
    parser.add_argument(
        "--unlink",
        action="store_true",
        help="Remove global symlinks instead of creating them.",
    )
    parser.add_argument(
        "--dry-run",
        action="store_true",
        help="Show planned actions without modifying filesystem.",
    )
    args = parser.parse_args()

    skills = find_repo_skills(SKILLS_DIR)
    if not skills:
        print(f"No valid skills found in {SKILLS_DIR}")
        return 1

    if not args.unlink:
        print("Validating skill dependency graph...")
        try:
            from depgraph import generate_lockfile, verify_graph

            lockfile_path = REPO_DIR / "skills.lock"
            is_valid, errors, warnings = verify_graph(SKILLS_DIR, lockfile_path)

            if warnings:
                for w in warnings:
                    print(f"  ⚠️ {w}")

            if not is_valid:
                print("  Attempting to regenerate lockfile...")
                if generate_lockfile(SKILLS_DIR, lockfile_path):
                    is_valid, errors, warnings = verify_graph(SKILLS_DIR, lockfile_path)

            if not is_valid:
                print("❌ Skill dependency graph validation FAILED:")
                for err in errors:
                    print(f"  ❌ {err}")
                return 1

            print("✅ Skill dependency graph validated successfully.\n")
        except Exception as err:
            print(f"⚠️ Dependency validation skipped: {err}\n")

    action_str = "Uninstalling" if args.unlink else "Installing"
    print(f"{action_str} {len(skills)} skill(s) globally...\n")

    for agent_name, target_dir in GLOBAL_SKILL_TARGETS.items():
        print(f"=== {agent_name} ({target_dir}) ===")
        for skill in skills:
            if args.unlink:
                msg = uninstall_skill(skill, target_dir, dry_run=args.dry_run)
            else:
                msg = install_skill(skill, target_dir, dry_run=args.dry_run)
            print(f"  {msg}")
        print()

    print("Done!")
    return 0


if __name__ == "__main__":
    sys.exit(main())
