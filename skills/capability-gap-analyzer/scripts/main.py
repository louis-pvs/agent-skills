#!/usr/bin/env python3
"""Capability Gap Analyzer Unified Entrypoint CLI.

Supports explicit domain capability analysis, workspace auto-detection,
and JSON structured output for AI Agent Tier 2 semantic evaluation.
"""

import argparse
import json
import re
import sys
from pathlib import Path

_repo_root = Path(__file__).resolve().parents[3]
if str(_repo_root) not in sys.path:
    sys.path.insert(0, str(_repo_root))

_script_dir = Path(__file__).resolve().parent
if str(_script_dir) not in sys.path:
    sys.path.insert(0, str(_script_dir))

try:
    from scripts._path_safety import resolve_safe_dir
except ImportError:

    def resolve_safe_dir(raw_path: str, base_dir: Path | None = None) -> Path:
        return Path(raw_path).resolve()


try:
    from domain_detector import detect_workspace_domains
    from gap_analyzer import (
        TAXONOMY_DOMAINS,
        calculate_taxonomy_heatmap,
        generate_heatmap_markdown,
        scan_skills_inventory,
    )
except ImportError:
    from .domain_detector import detect_workspace_domains  # type: ignore
    from .gap_analyzer import (  # type: ignore
        TAXONOMY_DOMAINS,
        calculate_taxonomy_heatmap,
        generate_heatmap_markdown,
        scan_skills_inventory,
    )


SCAFFOLD_SUGGESTIONS = {
    "Architecture & DDD": {
        "name": "architecture-scaffold",
        "description": "Draft ADRs, aggregate boundaries, and system design reviews.",
    },
    "Analysis & Refactoring": {
        "name": "refactor-scaffold",
        "description": "Analyze call graphs and blast radius before refactors.",
    },
    "Performance & Benchmark": {
        "name": "perf-scaffold",
        "description": "Benchmark latency, memory, and throughput regressions.",
    },
    "Frontend & UI/UX": {
        "name": "frontend-ui-review",
        "description": "Review and scaffold UI/UX, accessibility, and component design work.",
    },
    "Backend & Data Pipelines": {
        "name": "backend-data-pipeline",
        "description": "Design and review APIs, SQL schemas, and data pipeline code.",
    },
    "DevOps & Infrastructure": {
        "name": "devops-infra-review",
        "description": "Review and scaffold CI/CD, containerization, and infrastructure-as-code.",
    },
    "Security & Compliance": {
        "name": "security-compliance-scan",
        "description": "Scan for secrets, insecure code patterns, and vulnerable dependencies.",
    },
}


# Maps auto-detected domains to the taxonomy categories that are actually relevant.
# Categories absent from this set are out-of-scope for the project and must NOT be
# flagged as gaps, preventing domain-misclassification false positives (e.g. suggesting
# "frontend-ui-review" for a Python-only skill framework that has zero frontend code).
DOMAIN_RELEVANT_TAXONOMY: dict = {
    "python-backend": {
        "Architecture & DDD",
        "Analysis & Refactoring",
        "Performance & Benchmark",
        "Backend & Data Pipelines",
        "Security & Compliance",
        "DevOps & Infrastructure",
    },
    "frontend-web": {
        "Architecture & DDD",
        "Analysis & Refactoring",
        "Frontend & UI/UX",
        "Performance & Benchmark",
        "Security & Compliance",
    },
    "devops-infra": {
        "Architecture & DDD",
        "DevOps & Infrastructure",
        "Security & Compliance",
    },
    "data-engineering": {
        "Architecture & DDD",
        "Analysis & Refactoring",
        "Backend & Data Pipelines",
        "Performance & Benchmark",
        "Security & Compliance",
    },
    "rust-systems": {
        "Architecture & DDD",
        "Analysis & Refactoring",
        "Performance & Benchmark",
        "Backend & Data Pipelines",
        "Security & Compliance",
        "DevOps & Infrastructure",
    },
    "go-microservices": {
        "Architecture & DDD",
        "Analysis & Refactoring",
        "Performance & Benchmark",
        "Backend & Data Pipelines",
        "Security & Compliance",
        "DevOps & Infrastructure",
    },
    "general-software": {
        "Architecture & DDD",
        "Analysis & Refactoring",
        "Performance & Benchmark",
        "Backend & Data Pipelines",
        "Security & Compliance",
        "DevOps & Infrastructure",
        "Frontend & UI/UX",
    },
}


def filter_heatmap_by_relevance(heatmap, relevant_categories=None):
    """Drops baseline taxonomy categories that are out-of-scope for the detected project domain.

    Only filters categories that are part of the fixed TAXONOMY_DOMAINS checklist (e.g. don't
    flag "Frontend & UI/UX" as a gap for a project with no frontend). Dynamic/emergent domains
    harvested from a skill's `domain:` frontmatter are always kept, since they aren't part of
    the baseline taxonomy relevance map to begin with.
    """
    if relevant_categories is None:
        return heatmap
    return {
        category: meta
        for category, meta in heatmap.items()
        if category not in TAXONOMY_DOMAINS or category in relevant_categories
    }


def build_scaffold_suggestions(heatmap, relevant_categories=None):
    """Builds skill-creator scaffold commands for uncovered taxonomy categories.

    A category needs a scaffold if it's Zero-Zone (checklist categories with 0% coverage) or
    Not Detected (dynamic/emergent domains with no matching skill at all).
    """
    suggestions = []
    for category, meta in heatmap.items():
        if meta["status"] not in {"Zero-Zone", "Not Detected"}:
            continue
        is_baseline = category in TAXONOMY_DOMAINS
        if relevant_categories is not None and is_baseline and category not in relevant_categories:
            continue
        suggestion = SCAFFOLD_SUGGESTIONS.get(category)
        if suggestion:
            name = suggestion["name"]
            desc = suggestion["description"]
        else:
            slug = re.sub(r"[^a-z0-9]+", "-", category.lower()).strip("-")
            name = f"{slug}-scaffold" if slug else "domain-scaffold"
            desc = f"Tier 2 LLM Prompt Suggestion: Scaffold initial capability set for {category} domain."

        command = (
            f'python3 skills/skill-creator/scripts/scaffold_skill.py --name {name} --domain "{category}" --description "{desc}"'
        )
        suggestions.append({"category": category, "command": command, "name": name, "description": desc})
    return suggestions


def parse_args():
    parser = argparse.ArgumentParser(description="Capability Gap Analyzer CLI")
    parser.add_argument("--domain", type=str, help="Explicit target domain (e.g. frontend-web, devops-infra)")
    parser.add_argument("--auto-detect", action="store_true", help="Auto-detect domain mix from local workspace files")
    parser.add_argument("--dir", type=str, default=".", help="Target workspace path (default: current directory)")
    parser.add_argument("--json", action="store_true", help="Output raw JSON metadata for Tier 2 LLM evaluation")
    parser.add_argument("--scaffold-missing", action="store_true", help="Offer draft skill scaffolding for zero-coverage gaps")
    return parser.parse_args()


def main() -> int:
    args = parse_args()
    target_dir = resolve_safe_dir(args.dir)
    skills_dir = _repo_root / "skills"

    target_domain = args.domain or "Unspecified"
    detected_domains = []

    if args.auto_detect or not args.domain:
        detected_domains = detect_workspace_domains(target_dir)
        top_domain = detected_domains[0]["domain"]
        if not args.domain:
            target_domain = f"Auto-Detected ({top_domain})"

    inventory = scan_skills_inventory(skills_dir)
    heatmap = calculate_taxonomy_heatmap(inventory)

    relevant_categories = None
    if detected_domains:
        top_detected = detected_domains[0]["domain"]
        relevant_categories = DOMAIN_RELEVANT_TAXONOMY.get(top_detected)

    # Drop out-of-scope baseline categories (e.g. Frontend for a project with no frontend)
    # from the report itself, not just from scaffold suggestions.
    heatmap = filter_heatmap_by_relevance(heatmap, relevant_categories=relevant_categories)

    if args.json:
        result = {
            "target_domain": target_domain,
            "detected_domains": detected_domains,
            "skills_inventory": inventory,
            "taxonomy_heatmap": heatmap,
        }
        if args.scaffold_missing:
            result["suggested_scaffolds"] = build_scaffold_suggestions(heatmap, relevant_categories=relevant_categories)
        print(json.dumps(result, indent=2, default=list))
    else:
        print(generate_heatmap_markdown(heatmap, target_domain=target_domain))
        if detected_domains:
            print("\n### 🔍 Workspace Auto-Detection Summary")
            for d in detected_domains:
                matches = f" (files: {', '.join(d.get('matches', []))})" if d.get("matches") else ""
                print(f"- **{d['domain']}**: {d['confidence'] * 100:.0f}% confidence{matches}")

        if args.scaffold_missing:
            suggestions = build_scaffold_suggestions(heatmap, relevant_categories=relevant_categories)
            if suggestions:
                print("\n### 🛠️ Suggested Scaffolds")
                for s in suggestions:
                    print(f"- **{s['category']}**: `{s['command']}`")
            else:
                print("\n### 🛠️ Suggested Scaffolds\nNo zero-coverage gaps found.")

    return 0


if __name__ == "__main__":
    sys.exit(main())
