#!/usr/bin/env python3
"""Capability Gap Analyzer - Tier 1 Deterministic Manifest Inventory Scanner.

Parses all skills/*/SKILL.md manifests on the fly across workspace and global skill roots
to build a structured inventory JSON and taxonomy coverage matrix for Tier 2 LLM semantic evaluation.
"""

import argparse
import json
import os
import re
import sys
from pathlib import Path
from typing import Any, Dict, List, Optional

_repo_root = Path(__file__).resolve().parents[3]
if str(_repo_root) not in sys.path:
    sys.path.insert(0, str(_repo_root))

try:
    from scripts._config_safety import load_skill_config
    from scripts._path_safety import resolve_safe_dir
except ImportError:

    def resolve_safe_dir(raw_path: str, base_dir: Path | None = None) -> Path:
        return Path(raw_path).resolve()

    def load_skill_config(name: str, s_dir: Any = None, r_root: Any = None) -> Dict[str, Any]:
        return {}


TAXONOMY_DOMAINS: Dict[str, Dict[str, Any]] = {
    "Architecture & DDD": {
        "keywords": {"domain-modeling", "ubiquitous-language", "aggregate", "solid", "cupid", "adr", "architecture"},
        "skills": [],
    },
    "Analysis & Refactoring": {
        "keywords": {"ast", "call-graph", "blast-radius", "self-healing", "tdd", "rollback", "refactoring", "gatherer"},
        "skills": [],
    },
    "Performance & Benchmark": {
        "keywords": {"latency", "memory", "throughput", "benchmark", "profiling", "sla", "performance"},
        "skills": [],
    },
    "Frontend & UI/UX": {
        "keywords": {"frontend", "ui", "ux", "css", "tailwind", "react", "components", "dom", "accessibility", "web"},
        "skills": [],
    },
    "Backend & Data Pipelines": {
        "keywords": {"backend", "api", "sql", "dataform", "dbt", "bigquery", "spark", "pipeline", "data"},
        "skills": [],
    },
    "DevOps & Infrastructure": {
        "keywords": {"docker", "kubernetes", "terraform", "ci-cd", "helm", "cloud", "gcloud", "infra"},
        "skills": [],
    },
    "Security & Compliance": {
        "keywords": {"security", "saif", "compliance", "vulnerability", "auth", "audit", "prevention"},
        "skills": [],
    },
}


def load_global_skill_paths() -> List[Path]:
    """Loads global skill lookup paths dynamically following ADR 0005 configuration hierarchy."""
    home = Path.home()
    default_paths = [
        home / ".gemini" / "config" / "skills",
        home / ".claude" / "skills",
        home / ".copilot" / "skills",
    ]

    try:
        cfg = load_skill_config("capability-gap-analyzer", _repo_root / "skills" / "capability-gap-analyzer", _repo_root)
        repo_cfg = cfg.get("repo_config", {})
        skill_cfg = cfg.get("skill_config", {})

        custom_paths = skill_cfg.get("gap_analyzer", {}).get("custom_global_paths", [])
        if custom_paths and isinstance(custom_paths, list):
            resolved = []
            for p in custom_paths:
                if isinstance(p, str):
                    resolved.append(Path(os.path.expanduser(p)).resolve())
                elif isinstance(p, dict) and "path" in p:
                    resolved.append(Path(os.path.expanduser(str(p["path"]))).resolve())
            if resolved:
                return resolved

        targets = repo_cfg.get("targets", [])
        if targets and isinstance(targets, list):
            resolved_targets = []
            for t in targets:
                if isinstance(t, dict) and t.get("path"):
                    resolved_targets.append(Path(os.path.expanduser(str(t["path"]))).resolve())
                elif isinstance(t, str):
                    resolved_targets.append(Path(os.path.expanduser(t)).resolve())
            if resolved_targets:
                return resolved_targets
    except Exception as err:
        sys.stderr.write(f"Warning: Failed to resolve global skill paths: {err}\n")

    return default_paths


def parse_skill_frontmatter(skill_md_path: Path) -> Dict[str, Any]:
    """Extracts YAML frontmatter and title from a SKILL.md file."""
    if not skill_md_path.exists():
        return {}

    content = skill_md_path.read_text(encoding="utf-8")
    fm_match = re.search(r"^---\s*\n(.*?)\n---", content, re.DOTALL | re.MULTILINE)
    data: Dict[str, Any] = {"name": skill_md_path.parent.name, "description": "", "path": str(skill_md_path)}

    if fm_match:
        fm_text = fm_match.group(1)
        for line in fm_text.splitlines():
            if ":" in line:
                key, val = line.split(":", 1)
                key = key.strip()
                val = val.strip().strip("\"'")
                if key in {"name", "description", "version", "domain"}:
                    data[key] = val
                elif key == "tags":
                    if val.startswith("[") and val.endswith("]"):
                        tag_items = [t.strip().strip("\"'") for t in val[1:-1].split(",") if t.strip()]
                    else:
                        tag_items = [t.strip().strip("\"'") for t in val.split(",") if t.strip()]
                    data["tags"] = tag_items

    return data


def canonicalize_domain(domain_str: str) -> str:
    """Normalizes raw domain strings into clean, canonical taxonomy category headers."""
    if not domain_str:
        return "General"
    cleaned = domain_str.strip()
    for baseline_cat in TAXONOMY_DOMAINS:
        if cleaned.lower() == baseline_cat.lower():
            return baseline_cat
    words = cleaned.replace("-", " ").replace("_", " ").split()
    return " ".join(w.capitalize() for w in words)


def scan_skills_inventory(skills_dir: Optional[Path] = None, include_global: bool = True) -> List[Dict[str, Any]]:
    """Scans workspace and global customization skill roots for SKILL.md manifests."""
    if skills_dir is None:
        skills_dir = _repo_root / "skills"

    roots = [(skills_dir, "workspace")]
    if include_global:
        global_paths = load_global_skill_paths()
        for gpath in global_paths:
            if gpath.exists() and gpath.is_dir() and gpath != skills_dir:
                roots.append((gpath, "global"))

    inventory = []
    seen_canonical = set()
    seen_names = set()

    for root_path, origin in roots:
        if not root_path.exists() or not root_path.is_dir():
            continue

        for child in sorted(root_path.iterdir()):
            if child.is_dir():
                skill_md = child / "SKILL.md"
                if skill_md.exists():
                    try:
                        canonical = str(skill_md.resolve())
                    except OSError:
                        canonical = str(skill_md)

                    if canonical in seen_canonical:
                        continue
                    seen_canonical.add(canonical)

                    name = child.name
                    if name in seen_names:
                        continue
                    seen_names.add(name)

                    fm = parse_skill_frontmatter(skill_md)
                    fm["origin"] = origin
                    inventory.append(fm)

    return inventory


def calculate_taxonomy_heatmap(inventory: List[Dict[str, Any]]) -> Dict[str, Dict[str, Any]]:
    """Maps inventory skills against taxonomy domains and calculates coverage scores."""
    heatmap: Dict[str, Dict[str, Any]] = {
        cat: {"keywords": set(meta["keywords"]), "matched_skills": [], "score": 0.0, "status": "Zero-Zone"}
        for cat, meta in TAXONOMY_DOMAINS.items()
    }

    # Harvest explicit dynamic domains from scanned inventory
    for skill in inventory:
        raw_dom = skill.get("domain")
        if raw_dom:
            cat_name = canonicalize_domain(raw_dom)
            if cat_name not in heatmap:
                heatmap[cat_name] = {
                    "keywords": set(skill.get("tags", [])) | {cat_name.lower()},
                    "matched_skills": [],
                    "score": 0.0,
                    "status": "Zero-Zone",
                }

    for skill in inventory:
        name = skill.get("name", "")
        desc = skill.get("description", "")
        origin = skill.get("origin", "workspace")
        raw_dom = skill.get("domain")
        tags = skill.get("tags", [])
        text = f"{name} {desc} {' '.join(tags)}".lower()

        # If skill explicitly declared a domain, attach directly
        if raw_dom:
            cat_name = canonicalize_domain(raw_dom)
            already_matched_names = {s["name"] for s in heatmap[cat_name]["matched_skills"]}
            if name not in already_matched_names:
                heatmap[cat_name]["matched_skills"].append({"name": name, "origin": origin})

        # Match against keywords for all taxonomy categories
        for _cat, meta in heatmap.items():
            keywords = meta["keywords"]
            matches = []
            for kw in keywords:
                pattern = r"\b" + re.escape(kw.lower()) + r"\b"
                if re.search(pattern, text) or name == kw:
                    matches.append(kw)

            already_matched_names = {s["name"] for s in meta["matched_skills"]}
            if matches and name not in already_matched_names:
                meta["matched_skills"].append({"name": name, "origin": origin})

    for _cat, meta in heatmap.items():
        count = len(meta["matched_skills"])
        if count >= 2:
            meta["score"] = min(100.0, 50.0 + count * 25.0)
            meta["status"] = "Strong"
        elif count == 1:
            meta["score"] = 50.0
            meta["status"] = "Partial"
        else:
            meta["score"] = 0.0
            meta["status"] = "Zero-Zone"

    return heatmap


def generate_heatmap_markdown(heatmap: Dict[str, Dict[str, Any]], target_domain: str = "General") -> str:
    """Formats taxonomy heatmap into clean GFM Markdown."""
    lines = [
        f"# 📊 Capability Gap Taxonomy Heatmap: `{target_domain}`",
        "",
        "| Domain Category | Status | Coverage Score | Matched Skills | Zero-Coverage Gaps |",
        "| :--- | :--- | :--- | :--- | :--- |",
    ]

    for cat, meta in heatmap.items():
        status = meta["status"]
        badge = "🟢 Strong" if status == "Strong" else ("🟡 Partial" if status == "Partial" else "🔴 Zero-Zone")
        matched_items = meta["matched_skills"]
        if matched_items:
            skills_str = ", ".join(f"`{s['name']}` [{s['origin']}]" for s in matched_items)
        else:
            skills_str = "*None*"

        gap_str = (
            "High priority missing capability" if status == "Zero-Zone" else ("Minor gap" if status == "Partial" else "None")
        )
        lines.append(f"| **{cat}** | {badge} | `{meta['score']:.0f}%` | {skills_str} | {gap_str} |")

    lines.append("")
    lines.append("> [!NOTE]")
    lines.append(
        "> **Multi-Root Two-Tier Evaluation**: Tier 1 manifest inventory baseline scanned across workspace & global skill roots."
    )

    return "\n".join(lines)


def parse_args():
    parser = argparse.ArgumentParser(description="Capability Gap Analyzer Tier 1 Manifest Scanner")
    parser.add_argument("--json", action="store_true", help="Output raw JSON data")
    parser.add_argument("--no-global", action="store_true", help="Exclude global pre-builtin skills from scan")
    return parser.parse_args()


def main() -> int:
    args = parse_args()
    skills_dir = _repo_root / "skills"
    include_global = not getattr(args, "no_global", False)

    inventory = scan_skills_inventory(skills_dir, include_global=include_global)
    heatmap = calculate_taxonomy_heatmap(inventory)

    if getattr(args, "json", False):
        print(json.dumps({"inventory": inventory, "heatmap": heatmap}, indent=2, default=list))
    else:
        print(generate_heatmap_markdown(heatmap))

    return 0


if __name__ == "__main__":
    sys.exit(main())
