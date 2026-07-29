#!/usr/bin/env python3
"""Skill Dependency Graph Resolver & Verifier.

Parses YAML frontmatter across SKILL.md files, builds execution graphs using
Kahn's topological sort algorithm, detects circular dependencies, resolves
transitive dependencies, and generates/verifies skills.lock.
"""

import argparse
import json
import os
import re
import sys
from collections import deque
from pathlib import Path
from typing import Any, Dict, List, Optional, Set, Tuple, Union

REPO_DIR = Path(__file__).parent.parent.resolve()
SKILLS_DIR = REPO_DIR / "skills"
LOCKFILE_PATH = REPO_DIR / "skills.lock"
CONFIG_FILE = REPO_DIR / "skills.config.yaml"


def sanitize_path(input_path: Union[str, Path], base_dir: Optional[Path] = None) -> Path:
    """Sanitize and validate a file or directory path against path traversal attacks.

    Resolves symlinks and relative elements ('..'). Validates that the resolved path
    is located strictly within base_dir (or REPO_DIR if base_dir is None).
    """
    base = os.path.realpath(str(base_dir if base_dir is not None else REPO_DIR))
    target = os.path.realpath(str(input_path))

    if os.path.commonpath([base, target]) != base:
        raise ValueError(
            f"Security Error: Path traversal attempt detected. '{input_path}' is outside allowed base directory '{base}'"
        )

    resolved_path = Path(target)
    try:
        resolved_path.relative_to(Path(base))
    except ValueError as err:
        raise ValueError(
            f"Security Error: Path traversal attempt detected. '{input_path}' is outside allowed base directory '{base}'"
        ) from err

    return resolved_path


def load_global_skill_config(config_path: Path = CONFIG_FILE) -> List[Dict[str, str]]:
    """Single source of truth for parsing skills.config.yaml's 'targets' list.

    Returns raw (un-expanded) {"name": ..., "path": ...} dicts. Both
    load_global_skill_paths() here and install_global.load_global_skill_targets()
    derive their shaped outputs from this, instead of each hand-parsing the
    config file independently.
    """
    default: List[Dict[str, str]] = [
        {"name": "Gemini / Antigravity", "path": "~/.gemini/config/skills"},
        {"name": "Claude Code", "path": "~/.claude/skills"},
        {"name": "GitHub Copilot", "path": "~/.copilot/skills"},
    ]
    if not config_path.exists():
        return default

    try:
        content = config_path.read_text(encoding="utf-8")
        entries: List[Dict[str, str]] = []
        curr_name: Optional[str] = None
        for line in content.splitlines():
            line_str = line.split("#")[0].strip()
            if not line_str:
                continue
            if line_str.startswith("- name:"):
                curr_name = line_str.split(":", 1)[1].strip().strip("\"'")
            elif line_str.startswith("path:"):
                raw_path = line_str.split(":", 1)[1].strip().strip("\"'")
                entries.append({"name": curr_name or "", "path": raw_path})
                curr_name = None
        if entries:
            return entries
    except Exception:
        pass

    return default


def load_global_skill_paths() -> List[Path]:
    """Loads target global skill search paths from skills.config.yaml."""
    return [Path(os.path.expanduser(entry["path"])).resolve() for entry in load_global_skill_config()]


class SkillNode:
    """Represents a parsed skill node in the dependency graph."""

    def __init__(
        self,
        name: str,
        path: Path,
        requires: Optional[List[str]] = None,
        enhances: Optional[List[str]] = None,
        version: Optional[str] = None,
        description: Optional[str] = None,
    ):
        self.name = name
        self.path = path
        self.requires = requires or []
        self.enhances = enhances or []
        self.version = version or "1.0.0"
        self.description = description or ""

    def to_dict(self) -> Dict[str, Any]:
        return {
            "name": self.name,
            "version": self.version,
            "requires": self.requires,
            "enhances": self.enhances,
            "path": str(self.path.relative_to(REPO_DIR)) if self.path.is_relative_to(REPO_DIR) else str(self.path),
        }


def parse_yaml_frontmatter(content: str) -> Dict[str, Any]:
    """Simple standard-library YAML frontmatter parser for SKILL.md files.

    Handles basic key-value pairs, inline lists `[a, b]`, and block lists `- a`.
    """
    match = re.match(r"^---\s*\n(.*?)\n---", content, re.DOTALL)
    if not match:
        return {}

    yaml_block = match.group(1)
    data: Dict[str, Any] = {}
    current_key: Optional[str] = None

    for line in yaml_block.splitlines():
        raw_line = line
        line = line.strip()
        if not line or line.startswith("#"):
            continue

        # Check block list item (e.g. "- item")
        if raw_line.startswith("  - ") or raw_line.startswith("    - ") or raw_line.startswith("- "):
            item_val = line.lstrip("-").strip().strip('"').strip("'")
            if current_key and isinstance(data.get(current_key), list):
                data[current_key].append(item_val)
            continue

        if ":" in line:
            key, val = line.split(":", 1)
            key = key.strip()
            val = val.strip()
            current_key = key

            if not val:
                data[key] = []
            elif val.startswith("[") and val.endswith("]"):
                # Inline list: [item1, item2]
                items = [item.strip().strip('"').strip("'") for item in val[1:-1].split(",") if item.strip()]
                data[key] = items
            else:
                data[key] = val.strip('"').strip("'")

    return data


def parse_skill_file(skill_md_path: Path) -> Optional[SkillNode]:
    """Parse SKILL.md into a SkillNode."""
    if not skill_md_path.exists():
        return None

    try:
        content = skill_md_path.read_text(encoding="utf-8")
        meta = parse_yaml_frontmatter(content)
        name = meta.get("name") or skill_md_path.parent.name
        requires = meta.get("requires") or []
        enhances = meta.get("enhances") or []
        version = meta.get("version") or "1.0.0"
        description = meta.get("description") or ""

        if isinstance(requires, str):
            requires = [requires]
        if isinstance(enhances, str):
            enhances = [enhances]

        return SkillNode(
            name=name,
            path=skill_md_path.parent,
            requires=requires,
            enhances=enhances,
            version=version,
            description=description,
        )
    except Exception as err:
        print(f"Warning: Failed to parse {skill_md_path}: {err}", file=sys.stderr)
        return None


def discover_skills(skills_dir: Path) -> Dict[str, SkillNode]:
    """Discover and parse all skills in skills_dir."""
    nodes: Dict[str, SkillNode] = {}
    if not skills_dir.exists():
        return nodes

    for item in sorted(skills_dir.iterdir()):
        if item.is_dir():
            skill_md = item / "SKILL.md"
            if skill_md.exists():
                node = parse_skill_file(skill_md)
                if node:
                    nodes[node.name] = node

    return nodes


def resolve_topological_sort(nodes: Dict[str, SkillNode]) -> Tuple[List[str], Optional[List[str]]]:
    """Perform Kahn's algorithm for topological sorting and cycle detection.

    Returns (topological_order, cycle_nodes_if_any).
    """
    in_degree: Dict[str, int] = {name: 0 for name in nodes}
    adj_list: Dict[str, List[str]] = {name: [] for name in nodes}

    # Build dependency graph edges (u -> v means u must be loaded before v)
    # If node A requires B, then B -> A (B is prerequisite for A)
    for name, node in nodes.items():
        for req in node.requires:
            if req in nodes:
                adj_list[req].append(name)
                in_degree[name] += 1

    # Kahn's algorithm: queue nodes with 0 in-degree
    queue = deque([name for name, deg in in_degree.items() if deg == 0])
    topo_order: List[str] = []

    while queue:
        u = queue.popleft()
        topo_order.append(u)
        for v in adj_list[u]:
            in_degree[v] -= 1
            if in_degree[v] == 0:
                queue.append(v)

    if len(topo_order) == len(nodes):
        return topo_order, None

    # Cycle detected: extract remaining nodes in cycle
    remaining = [name for name, deg in in_degree.items() if deg > 0]
    return [], remaining


def compute_transitive_deps(nodes: Dict[str, SkillNode]) -> Dict[str, Dict[str, List[str]]]:
    """Compute transitive requirements and enhancements for each skill."""
    transitive: Dict[str, Dict[str, List[str]]] = {}

    def get_all_requires(name: str, visited: Set[str]) -> Set[str]:
        if name not in nodes or name in visited:
            return set()
        visited.add(name)
        result: Set[str] = set()
        for req in nodes[name].requires:
            result.add(req)
            result.update(get_all_requires(req, visited.copy()))
        return result

    def get_all_enhances(name: str, visited: Set[str]) -> Set[str]:
        if name not in nodes or name in visited:
            return set()
        visited.add(name)
        result: Set[str] = set()
        for enh in nodes[name].enhances:
            result.add(enh)
            result.update(get_all_enhances(enh, visited.copy()))
        return result

    for name in sorted(nodes.keys()):
        transitive[name] = {
            "transitive_requires": sorted(list(get_all_requires(name, set()))),
            "transitive_enhances": sorted(list(get_all_enhances(name, set()))),
        }

    return transitive


def build_lockfile_data(nodes: Dict[str, SkillNode]) -> Tuple[Dict[str, Any], List[str]]:
    """Build lockfile dictionary structure.

    Returns (lockfile_dict, errors).
    """
    errors: List[str] = []
    topo_order, cycle = resolve_topological_sort(nodes)

    if cycle:
        errors.append(f"Circular dependency detected among skills: {', '.join(cycle)}")

    # Check for missing hard dependencies
    for name, node in nodes.items():
        for req in node.requires:
            if req not in nodes:
                errors.append(f"Skill '{name}' requires missing skill '{req}'")

    transitive = compute_transitive_deps(nodes)

    lockfile_data = {
        "version": "1.0.0",
        "generated_by": "scripts/depgraph.py",
        "topological_order": topo_order if not cycle else [],
        "skills": {
            name: {
                "version": node.version,
                "requires": node.requires,
                "enhances": node.enhances,
                "transitive_requires": transitive[name]["transitive_requires"],
                "transitive_enhances": transitive[name]["transitive_enhances"],
                "path": f"skills/{name}",
            }
            for name, node in sorted(nodes.items())
        },
    }

    return lockfile_data, errors


def generate_lockfile(skills_dir: Path, lockfile_path: Path, base_dir: Optional[Path] = None, dry_run: bool = False) -> bool:
    """Generate and save skills.lock file."""
    effective_base = base_dir if base_dir is not None else REPO_DIR
    skills_dir = sanitize_path(skills_dir, base_dir=effective_base)
    lock_filename = os.path.basename(str(lockfile_path))
    safe_lockfile = sanitize_path(effective_base / lock_filename, base_dir=effective_base)

    nodes = discover_skills(skills_dir)
    lock_data, errors = build_lockfile_data(nodes)

    if errors:
        print("Lockfile generation failed due to errors:", file=sys.stderr)
        for err in errors:
            print(f"  ❌ {err}", file=sys.stderr)
        return False

    if dry_run:
        print(f"[DRY-RUN] Would write generated lockfile to {safe_lockfile}")
        return True

    with open(safe_lockfile, "w", encoding="utf-8") as f:
        json.dump(lock_data, f, indent=2)
        f.write("\n")

    print(f"Successfully generated lockfile at {safe_lockfile}")
    return True


def verify_graph(skills_dir: Path, lockfile_path: Path, base_dir: Optional[Path] = None) -> Tuple[bool, List[str], List[str]]:
    """Verify graph integrity and lockfile sync.

    Returns (is_valid, errors, warnings).
    """
    errors: List[str] = []
    warnings: List[str] = []

    effective_base = base_dir if base_dir is not None else REPO_DIR
    skills_dir = sanitize_path(skills_dir, base_dir=effective_base)
    lock_filename = os.path.basename(str(lockfile_path))
    safe_lockfile = sanitize_path(effective_base / lock_filename, base_dir=effective_base)

    nodes = discover_skills(skills_dir)
    if not nodes:
        errors.append(f"No skills found in {skills_dir}")
        return False, errors, warnings

    # Verify YAML frontmatter syntax across all SKILL.md files
    for item in sorted(skills_dir.iterdir()):
        if item.is_dir():
            skill_md = item / "SKILL.md"
            if skill_md.exists():
                content = skill_md.read_text(encoding="utf-8")
                match = re.match(r"^---\s*\n(.*?)\n---", content, re.DOTALL)
                if match:
                    yaml_block = match.group(1)
                    # NOTE: This yaml-or-fallback frontmatter syntax check is intentionally
                    # duplicated in skills/skill-creator/scripts/scaffold_skill.py's
                    # check_yaml_frontmatter_syntax() — that script is symlinked standalone
                    # into other agent tool directories and must stay self-contained, so it
                    # can't import this module. Keep both blocks in sync manually.
                    try:
                        import yaml

                        try:
                            yaml.safe_load(yaml_block)
                        except Exception as err:
                            errors.append(f"Skill '{item.name}' has invalid YAML frontmatter syntax: {err}")
                    except ImportError:
                        for line in yaml_block.splitlines():
                            line_str = line.strip()
                            if line_str and not line_str.startswith("#") and ":" in line_str:
                                key, val = line_str.split(":", 1)
                                val_str = val.strip()
                                if ":" in val_str and not (val_str.startswith("'") or val_str.startswith('"')):
                                    errors.append(
                                        f"Skill '{item.name}' has unquoted colon in YAML field "
                                        f"'{key.strip()}'. Enclose value in quotes."
                                    )

    lock_data, build_errors = build_lockfile_data(nodes)
    errors.extend(build_errors)

    # Check soft dependencies (enhances) against local and global skill paths defined in skills.config.yaml
    global_skill_paths = load_global_skill_paths()

    for name, node in nodes.items():
        for enh in node.enhances:
            if enh in nodes:
                continue
            # Secondary lookup in user global skill configuration directories
            is_globally_installed = any(
                (g_dir / enh / "SKILL.md").exists() or (g_dir / enh).exists() for g_dir in global_skill_paths
            )
            if not is_globally_installed:
                warnings.append(
                    f"Optional enhancer '{enh}' for skill '{name}' is not found locally or globally "
                    f"(skill will gracefully use fallback handlers)."
                )

    # Verify lockfile synchronicity
    if not safe_lockfile.exists():
        errors.append(f"Lockfile missing: {safe_lockfile}. Run 'python3 scripts/depgraph.py --generate-lock' to create it.")
    else:
        try:
            with open(safe_lockfile, encoding="utf-8") as f:
                existing_lock = json.load(f)

            if existing_lock.get("skills") != lock_data.get("skills"):
                msg = (
                    "Lockfile 'skills.lock' is out of sync with SKILL.md frontmatters. "
                    "Run 'python3 scripts/depgraph.py --generate-lock' to update it."
                )
                errors.append(msg)
            if existing_lock.get("topological_order") != lock_data.get("topological_order"):
                errors.append("Lockfile topological_order is out of sync with current dependency graph.")
        except Exception as err:
            errors.append(f"Failed to read existing lockfile {safe_lockfile}: {err}")

    is_valid = len(errors) == 0
    return is_valid, errors, warnings


def main() -> int:
    parser = argparse.ArgumentParser(description="Skill Dependency Graph Resolver and Verifier")
    parser.add_argument(
        "--skills-dir",
        type=str,
        default=str(SKILLS_DIR),
        help="Path to skills directory",
    )
    parser.add_argument(
        "--lockfile",
        type=str,
        default=str(LOCKFILE_PATH),
        help="Path to skills.lock file",
    )
    parser.add_argument(
        "--generate-lock",
        action="store_true",
        help="Generate or update skills.lock file",
    )
    parser.add_argument(
        "--dry-run",
        action="store_true",
        help="Show planned actions without modifying lockfile on disk",
    )
    parser.add_argument(
        "--verify",
        action="store_true",
        help="Verify graph integrity and lockfile synchronization",
    )
    parser.add_argument(
        "--json",
        action="store_true",
        help="Output results in JSON format",
    )

    args = parser.parse_args()
    skills_dir = sanitize_path(args.skills_dir, base_dir=REPO_DIR)
    lockfile_path = sanitize_path(args.lockfile, base_dir=REPO_DIR)

    if args.generate_lock:
        success = generate_lockfile(skills_dir, lockfile_path, base_dir=REPO_DIR, dry_run=args.dry_run)
        return 0 if success else 1

    if args.verify:
        is_valid, errors, warnings = verify_graph(skills_dir, lockfile_path, base_dir=REPO_DIR)
        if args.json:
            out = {
                "valid": is_valid,
                "errors": errors,
                "warnings": warnings,
            }
            print(json.dumps(out, indent=2))
        else:
            if warnings:
                print("=== Warnings ===")
                for w in warnings:
                    print(f"  ⚠️ {w}")
            if errors:
                print("=== Verification Errors ===")
                for e in errors:
                    print(f"  ❌ {e}")
            if is_valid:
                print("✅ Skill dependency graph and skills.lock are VALID and synchronized.")

        return 0 if is_valid else 1

    # Default mode: print graph overview
    nodes = discover_skills(skills_dir)
    topo_order, cycle = resolve_topological_sort(nodes)

    if args.json:
        lock_data, _ = build_lockfile_data(nodes)
        print(json.dumps(lock_data, indent=2))
    else:
        print(f"Discovered {len(nodes)} skills in {skills_dir}:")
        for name, node in sorted(nodes.items()):
            reqs = f"requires: {node.requires}" if node.requires else ""
            enhs = f"enhances: {node.enhances}" if node.enhances else ""
            deps = ", ".join(filter(None, [reqs, enhs]))
            deps_str = f" ({deps})" if deps else ""
            print(f"  - {name}{deps_str}")

        print("\nTopological Execution Order:")
        if cycle:
            print(f"  ❌ Circular dependency detected: {', '.join(cycle)}")
        else:
            print(f"  {' -> '.join(topo_order)}")

    return 0


if __name__ == "__main__":
    sys.exit(main())
