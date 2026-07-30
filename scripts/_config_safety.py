#!/usr/bin/env python3
"""Common Configuration Loader Utilities (ADR 0005).

Canonical, stdlib-only implementation of skill configuration resolution and parsing.
Single source of truth for loading skills.config.yaml and skills/<skill_name>/config.yaml.

Uses Python Standard Library only — no third-party dependencies.
"""

import sys
from pathlib import Path
from typing import Any, Dict, Optional

_repo_root = Path(__file__).resolve().parents[1]
if str(_repo_root) not in sys.path:
    sys.path.insert(0, str(_repo_root))

try:
    from scripts._path_safety import get_repo_root, sanitize_path
except ImportError:

    def sanitize_path(p: Any, base_dir: Optional[Path] = None) -> Path:
        base = base_dir.resolve() if base_dir else Path.cwd().resolve()
        candidate = Path(p).resolve()
        candidate.relative_to(base)
        return candidate

    def get_repo_root(p: Any = None) -> Path:
        return _repo_root


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
    """Lightweight zero-dependency YAML parser for skill configuration files."""
    data: Dict[str, Any] = {}
    current_key: Optional[str] = None
    current_section_dict: Optional[Dict[str, Any]] = None
    current_list_item: Optional[Dict[str, Any]] = None

    lines = content.splitlines()
    for line in lines:
        line_clean = line.split("#")[0].rstrip()
        if not line_clean:
            continue

        indent = len(line_clean) - len(line_clean.lstrip(" "))
        stripped = line_clean.strip()

        if indent == 0 and stripped.endswith(":") and not stripped.startswith("-"):
            current_key = stripped[:-1].strip()
            data[current_key] = {}
            current_section_dict = data[current_key]
            current_list_item = None
            continue

        if indent == 2 and stripped.endswith(":") and not stripped.startswith("-") and current_section_dict is not None:
            sub_key = stripped[:-1].strip()
            current_section_dict[sub_key] = {}
            current_list_item = None
            continue

        if stripped.startswith("- "):
            rest = stripped[2:].strip()
            item_dict: Dict[str, Any] = {}
            if ":" in rest:
                k, v = rest.split(":", 1)
                item_dict[k.strip()] = _parse_val(v.strip())

            if current_key:
                val = data.get(current_key)
                if isinstance(val, dict) and not val:
                    data[current_key] = [item_dict]
                elif isinstance(val, list):
                    val.append(item_dict)
                elif isinstance(val, dict):
                    sub_keys = list(val.keys())
                    if sub_keys and isinstance(val[sub_keys[-1]], list):
                        val[sub_keys[-1]].append(item_dict)
                    elif sub_keys and isinstance(val[sub_keys[-1]], dict) and not val[sub_keys[-1]]:
                        val[sub_keys[-1]] = [item_dict]
            current_list_item = item_dict
            continue

        if current_list_item is not None and ":" in stripped and indent >= 4:
            k, v = stripped.split(":", 1)
            current_list_item[k.strip()] = _parse_val(v.strip())
            continue

        if current_section_dict is not None and ":" in stripped:
            k, v = stripped.split(":", 1)
            sub_keys = [sk for sk, sv in current_section_dict.items() if isinstance(sv, dict)]
            if sub_keys and indent >= 4:
                current_section_dict[sub_keys[-1]][k.strip()] = _parse_val(v.strip())
            else:
                current_section_dict[k.strip()] = _parse_val(v.strip())

    return data


def load_yaml_file(file_path: Path) -> Dict[str, Any]:
    """Safely loads and parses a YAML configuration file."""
    if not file_path.exists() or not file_path.is_file():
        return {}
    try:
        content = file_path.read_text(encoding="utf-8")
        return parse_simple_yaml(content)
    except Exception as err:
        sys.stderr.write(f"Warning: Failed to parse {file_path}: {err}\n")
        return {}


def load_repo_config(repo_root: Optional[Path] = None) -> Dict[str, Any]:
    """Loads repository-level skills.config.yaml (Tier 3)."""
    root = repo_root or get_repo_root(__file__)
    config_path = root / "skills.config.yaml"
    return load_yaml_file(config_path)


def load_skill_config(
    skill_name: str,
    skill_dir: Optional[Path] = None,
    repo_root: Optional[Path] = None,
) -> Dict[str, Any]:
    """Loads and merges skill configuration following ADR 0005 4-tier hierarchy.

    Evaluates:
    - Tier 2: skill_dir/config.yaml or legacy skill_dir/<skill_name>.config.yaml
    - Tier 3: repo_root/skills.config.yaml
    """
    root = repo_root or get_repo_root(__file__)
    s_dir = skill_dir or (root / "skills" / skill_name)

    primary_config = s_dir / "config.yaml"
    legacy_config = s_dir / f"{skill_name}.config.yaml"

    skill_cfg = {}
    if primary_config.exists():
        skill_cfg = load_yaml_file(primary_config)
    elif legacy_config.exists():
        skill_cfg = load_yaml_file(legacy_config)

    repo_cfg = load_repo_config(root)

    return {
        "repo_config": repo_cfg,
        "skill_config": skill_cfg,
    }
