#!/usr/bin/env python3
"""Capability Gap Analyzer - Workspace Domain Auto-Detector.

Scans workspace files to auto-detect active project framework markers and domain mix
when a user request does not specify an explicit target domain.
"""

import argparse
import json
import sys
from pathlib import Path
from typing import Dict, List, Set

_repo_root = Path(__file__).resolve().parents[3]
if str(_repo_root) not in sys.path:
    sys.path.insert(0, str(_repo_root))

try:
    from scripts._path_safety import resolve_safe_dir
except ImportError:

    def resolve_safe_dir(raw_path: str, base_dir: Path | None = None) -> Path:
        return Path(raw_path).resolve()


DOMAIN_MARKERS: Dict[str, Set[str]] = {
    "frontend-web": {
        "package.json",
        "tsconfig.json",
        "vite.config.ts",
        "vite.config.js",
        "next.config.js",
        "next.config.mjs",
        "tailwind.config.js",
        "tailwind.config.ts",
    },
    "python-backend": {
        "pyproject.toml",
        "setup.py",
        "requirements.txt",
        "Pipfile",
        "poetry.lock",
        "uv.lock",
    },
    "devops-infra": {
        "Dockerfile",
        "docker-compose.yml",
        "docker-compose.yaml",
        "Terraform",
        "main.tf",
        "Helm",
    },
    "data-engineering": {
        "dbt_project.yml",
        "schema.sql",
        "dataform.json",
        "workflow_settings.yaml",
    },
    "rust-systems": {
        "Cargo.toml",
        "Cargo.lock",
    },
    "go-microservices": {
        "go.mod",
        "go.sum",
    },
}


def detect_workspace_domains(target_dir: Path) -> List[Dict[str, float]]:
    """Inspects target directory for domain markers and returns matched domain profiles with confidence scores."""
    detected = []
    if not target_dir.exists() or not target_dir.is_dir():
        return [{"domain": "general-software", "confidence": 0.5}]

    found_files = {p.name for p in target_dir.iterdir()}

    for domain, markers in DOMAIN_MARKERS.items():
        matched = found_files.intersection(markers)
        if matched:
            confidence = min(1.0, len(matched) / 2.0)
            detected.append({"domain": domain, "confidence": confidence, "matches": list(matched)})

    if not detected:
        detected.append({"domain": "general-software", "confidence": 0.5, "matches": []})

    detected.sort(key=lambda x: x["confidence"], reverse=True)
    return detected


def parse_args():
    parser = argparse.ArgumentParser(description="Workspace Domain Auto-Detector")
    parser.add_argument("target_path", nargs="?", default=".", help="Workspace target directory")
    parser.add_argument("--json", action="store_true", help="Output JSON results")
    return parser.parse_args()


def main() -> int:
    args = parse_args()
    target_dir = resolve_safe_dir(args.target_path)
    domains = detect_workspace_domains(target_dir)

    if getattr(args, "json", False):
        print(json.dumps(domains, indent=2))
    else:
        print("Detected Workspace Domains:")
        for d in domains:
            matches_str = f" (matched: {', '.join(d.get('matches', []))})" if d.get("matches") else ""
            print(f" - {d['domain']} (confidence: {d['confidence'] * 100:.0f}%){matches_str}")
    return 0


if __name__ == "__main__":
    sys.exit(main())
