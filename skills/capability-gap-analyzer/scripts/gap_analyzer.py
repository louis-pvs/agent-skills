#!/usr/bin/env python3
"""Capability Gap Analyzer - Tier 1 Deterministic Manifest Inventory Scanner.

Parses all skills/*/SKILL.md manifests on the fly to build a structured inventory JSON
and taxonomy coverage matrix for Tier 2 LLM semantic evaluation.
"""

import argparse
import json
import re
import sys
from pathlib import Path
from typing import Any, Dict, List

_repo_root = Path(__file__).resolve().parents[3]
if str(_repo_root) not in sys.path:
    sys.path.insert(0, str(_repo_root))

try:
    from scripts._path_safety import resolve_safe_dir
except ImportError:

    def resolve_safe_dir(raw_path: str, base_dir: Path | None = None) -> Path:
        return Path(raw_path).resolve()


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
                if key in {"name", "description", "version"}:
                    data[key] = val

    return data


def scan_skills_inventory(skills_dir: Path) -> List[Dict[str, Any]]:
    """Scans all skills/*/SKILL.md files on the fly."""
    inventory = []
    if not skills_dir.exists() or not skills_dir.is_dir():
        return inventory

    for child in sorted(skills_dir.iterdir()):
        if child.is_dir():
            skill_md = child / "SKILL.md"
            if skill_md.exists():
                fm = parse_skill_frontmatter(skill_md)
                inventory.append(fm)

    return inventory


def calculate_taxonomy_heatmap(inventory: List[Dict[str, Any]]) -> Dict[str, Dict[str, Any]]:
    """Maps inventory skills against taxonomy domains and calculates coverage scores."""
    heatmap = {
        cat: {"keywords": meta["keywords"], "matched_skills": [], "score": 0.0, "status": "Zero-Zone"}
        for cat, meta in TAXONOMY_DOMAINS.items()
    }

    for skill in inventory:
        name = skill.get("name", "")
        desc = skill.get("description", "")
        text = f"{name} {desc}".lower()

        for _cat, meta in heatmap.items():
            keywords = meta["keywords"]
            matches = []
            for kw in keywords:
                pattern = r"\b" + re.escape(kw.lower()) + r"\b"
                if re.search(pattern, text) or name == kw:
                    matches.append(kw)
            if matches and name not in meta["matched_skills"]:
                meta["matched_skills"].append(name)

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
        skills_str = ", ".join(f"`{s}`" for s in meta["matched_skills"]) if meta["matched_skills"] else "*None*"
        gap_str = (
            "High priority missing capability" if status == "Zero-Zone" else ("Minor gap" if status == "Partial" else "None")
        )
        lines.append(f"| **{cat}** | {badge} | `{meta['score']:.0f}%` | {skills_str} | {gap_str} |")

    lines.append("")
    lines.append("> [!NOTE]")
    lines.append(
        "> **Two-Tier Evaluation**: Tier 1 manifest inventory baseline scanned on-the-fly. Tier 2 LLM semantic evaluation active."
    )

    return "\n".join(lines)


def parse_args():
    parser = argparse.ArgumentParser(description="Capability Gap Analyzer Tier 1 Manifest Scanner")
    parser.add_argument("--json", action="store_true", help="Output raw JSON data")
    return parser.parse_args()


def main() -> int:
    args = parse_args()
    skills_dir = _repo_root / "skills"
    inventory = scan_skills_inventory(skills_dir)
    heatmap = calculate_taxonomy_heatmap(inventory)

    if getattr(args, "json", False):
        print(json.dumps({"inventory": inventory, "heatmap": heatmap}, indent=2, default=list))
    else:
        print(generate_heatmap_markdown(heatmap))

    return 0


if __name__ == "__main__":
    sys.exit(main())
