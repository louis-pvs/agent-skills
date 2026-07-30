#!/usr/bin/env python3
"""Capability Gap Analyzer Unified Entrypoint CLI.

Supports explicit domain capability analysis, workspace auto-detection,
and JSON structured output for AI Agent Tier 2 semantic evaluation.
"""

import argparse
import json
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
        calculate_taxonomy_heatmap,
        generate_heatmap_markdown,
        scan_skills_inventory,
    )
except ImportError:
    from .domain_detector import detect_workspace_domains  # type: ignore
    from .gap_analyzer import (  # type: ignore
        calculate_taxonomy_heatmap,
        generate_heatmap_markdown,
        scan_skills_inventory,
    )


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

    if args.json:
        result = {
            "target_domain": target_domain,
            "detected_domains": detected_domains,
            "skills_inventory": inventory,
            "taxonomy_heatmap": heatmap,
        }
        print(json.dumps(result, indent=2, default=list))
    else:
        print(generate_heatmap_markdown(heatmap, target_domain=target_domain))
        if detected_domains:
            print("\n### 🔍 Workspace Auto-Detection Summary")
            for d in detected_domains:
                matches = f" (files: {', '.join(d.get('matches', []))})" if d.get("matches") else ""
                print(f"- **{d['domain']}**: {d['confidence'] * 100:.0f}% confidence{matches}")

    return 0


if __name__ == "__main__":
    sys.exit(main())
