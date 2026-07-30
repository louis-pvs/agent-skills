#!/usr/bin/env python3
"""Capability Gap Analyzer - Tier 1 Deterministic Manifest Inventory Scanner.

Parses all skills/*/SKILL.md manifests on the fly across workspace and global skill roots
to build a structured inventory JSON and a sub-capability checklist per taxonomy category.

Scoring model: each baseline taxonomy category decomposes into a fixed list of concrete,
named sub-capabilities. A sub-capability counts as "covered" only if a workspace skill's
name/description/tags/body text matches one of its (narrow, specific) keywords. The
category score is COVERED_SUB_CAPABILITIES / TOTAL_SUB_CAPABILITIES — a real fraction with
a real denominator, not a lookup table over raw skill-match counts. This intentionally
under-reports rather than over-reports: a skill matching on a generic word like "audit"
no longer inflates unrelated categories to "100%".
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


# Each baseline taxonomy category decomposes into concrete, falsifiable sub-capabilities.
# Keywords are deliberately narrow (not the category name itself) so a skill only counts
# toward a sub-capability if it actually documents doing that specific thing.
TAXONOMY_DOMAINS: Dict[str, List[Dict[str, Any]]] = {
    "Architecture & DDD": [
        {"capability": "ADR / architecture-decision documentation", "keywords": {"adr", "architecture-decision", "design-doc"}},
        {
            "capability": "DDD modeling (aggregates, entities, value objects)",
            "keywords": {"aggregate", "entity", "value-object", "ubiquitous-language", "domain-modeling", "ddd"},
        },
        {
            "capability": "Design-principle review (SOLID/CUPID/DRY/YAGNI/KISS)",
            "keywords": {"solid", "cupid", "dry", "yagni", "kiss"},
        },
        {
            "capability": "System/component architecture review",
            "keywords": {"system-design", "component-architecture", "architecture-audit", "architecture-review"},
        },
    ],
    "Analysis & Refactoring": [
        {"capability": "Static call-graph / blast-radius analysis", "keywords": {"call-graph", "blast-radius", "ast"}},
        {
            "capability": "Bounded automated refactor / self-healing repair",
            "keywords": {"self-healing", "self-annealer", "rollback", "convergence"},
        },
        {"capability": "Test-driven development workflow", "keywords": {"tdd", "red-green-refactor", "test-first"}},
        {
            "capability": "Codebase context gathering / exploration",
            "keywords": {"context-gatherer", "codebase-exploration", "context-gathering"},
        },
    ],
    "Performance & Benchmark": [
        {"capability": "Latency/throughput benchmarking", "keywords": {"latency", "throughput", "benchmark"}},
        {"capability": "Memory/resource profiling", "keywords": {"memory", "profiling", "footprint"}},
        {
            "capability": "Baseline differential / regression analysis",
            "keywords": {"baseline", "regression", "differential"},
        },
    ],
    "Frontend & UI/UX": [
        {"capability": "Component/UI design review", "keywords": {"component", "react", "vue", "ui-review"}},
        {"capability": "Accessibility audit", "keywords": {"accessibility", "a11y", "wcag"}},
        {"capability": "CSS/styling systems", "keywords": {"css", "tailwind", "styling"}},
        {
            "capability": "Client-side state/data flow",
            "keywords": {"state-management", "frontend-data-flow"},
        },
    ],
    "Backend & Data Pipelines": [
        {"capability": "REST/GraphQL API design", "keywords": {"api-design", "rest", "graphql", "endpoint"}},
        {"capability": "SQL schema / data modeling", "keywords": {"sql", "schema", "database-design"}},
        {
            "capability": "ETL/data pipeline orchestration",
            "keywords": {"etl", "pipeline", "dataform", "dbt", "airflow", "orchestration"},
        },
        {"capability": "Data warehouse/analytics engineering", "keywords": {"bigquery", "warehouse", "spark"}},
    ],
    "DevOps & Infrastructure": [
        {"capability": "Containerization (Docker)", "keywords": {"docker", "container", "dockerfile"}},
        {
            "capability": "CI/CD pipeline automation",
            "keywords": {"ci-cd", "continuous-integration", "github-actions", "pipeline-automation"},
        },
        {"capability": "Infrastructure-as-code (Terraform/Helm)", "keywords": {"terraform", "helm", "iac"}},
        {"capability": "Cloud resource provisioning", "keywords": {"gcloud", "kubernetes", "gke", "provisioning"}},
    ],
    "Security & Compliance": [
        {
            "capability": "Secrets / credential leak scanning",
            "keywords": {"secrets-scanning", "credential-leak", "data-loss-prevention"},
        },
        {"capability": "Static application security testing (SAST)", "keywords": {"sast", "static-security"}},
        {
            "capability": "Dependency / supply-chain vulnerability scanning",
            "keywords": {"dependency-scan", "cve", "supply-chain", "sbom", "vulnerability"},
        },
        {"capability": "Auth / access-control review", "keywords": {"authn", "authz", "access-control"}},
        {"capability": "Compliance / policy review", "keywords": {"compliance", "policy-review", "saif"}},
    ],
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
    """Extracts YAML frontmatter, title, and body text from a SKILL.md file.

    The body is included (not just the one-line `description:`) because SKILL.md bodies
    document the concrete workflows/capabilities a skill actually performs — matching
    against the full text gives real evidence instead of matching a marketing blurb.
    """
    if not skill_md_path.exists():
        return {}

    content = skill_md_path.read_text(encoding="utf-8")
    fm_match = re.search(r"^---\s*\n(.*?)\n---", content, re.DOTALL | re.MULTILINE)
    data: Dict[str, Any] = {"name": skill_md_path.parent.name, "description": "", "path": str(skill_md_path), "body": ""}

    if fm_match:
        fm_text = fm_match.group(1)
        data["body"] = content[fm_match.end() :]
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
    else:
        data["body"] = content

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


def _skill_corpus(skill: Dict[str, Any]) -> str:
    """Builds the searchable text corpus for a skill: name + description + tags + full body."""
    name = skill.get("name", "")
    desc = skill.get("description", "")
    tags = skill.get("tags", [])
    body = skill.get("body", "")
    return f"{name} {desc} {' '.join(tags)} {body}".lower()


def _keyword_hits(corpus: str, keywords: set) -> List[str]:
    hits = []
    for kw in keywords:
        pattern = r"\b" + re.escape(kw.lower()) + r"\b"
        if re.search(pattern, corpus):
            hits.append(kw)
    return hits


def calculate_taxonomy_heatmap(inventory: List[Dict[str, Any]]) -> Dict[str, Dict[str, Any]]:
    """Maps inventory skills against a fixed sub-capability checklist per taxonomy category.

    Baseline categories (from TAXONOMY_DOMAINS) get a real fraction score:
    covered sub-capabilities / total sub-capabilities, where "covered" requires a
    *workspace* skill's corpus to match that sub-capability's own narrow keyword set.

    Dynamic/emergent categories (harvested from a skill's `domain:` frontmatter) have no
    predefined checklist to divide by, so they are reported honestly as Detected /
    Not Detected rather than being assigned a fabricated percentage.
    """
    heatmap: Dict[str, Dict[str, Any]] = {}
    for cat, items in TAXONOMY_DOMAINS.items():
        heatmap[cat] = {
            "type": "checklist",
            "checklist": [
                {"capability": item["capability"], "keywords": sorted(item["keywords"]), "covered": False, "evidence": []}
                for item in items
            ],
            "matched_skills": [],
            "score": 0.0,
            "status": "Zero-Zone",
        }

    # Harvest explicit dynamic domains declared via `domain:` frontmatter that aren't baseline categories.
    for skill in inventory:
        raw_dom = skill.get("domain")
        if raw_dom:
            cat_name = canonicalize_domain(raw_dom)
            if cat_name not in heatmap:
                heatmap[cat_name] = {
                    "type": "dynamic",
                    "keywords": sorted(set(skill.get("tags", [])) | {cat_name.lower()}),
                    "matched_skills": [],
                    "score": None,
                    "status": "Not Detected",
                }

    for skill in inventory:
        name = skill.get("name", "")
        origin = skill.get("origin", "workspace")
        corpus = _skill_corpus(skill)
        raw_dom = skill.get("domain")

        # Explicit self-declared domain: attach directly regardless of keyword match.
        if raw_dom:
            cat_name = canonicalize_domain(raw_dom)
            meta = heatmap[cat_name]
            already = {s["name"] for s in meta["matched_skills"]}
            if name not in already:
                meta["matched_skills"].append({"name": name, "origin": origin})

        for cat_name, meta in heatmap.items():
            if meta["type"] == "checklist":
                for entry in meta["checklist"]:
                    hits = _keyword_hits(corpus, set(entry["keywords"]))
                    if hits:
                        entry["evidence"].append({"skill": name, "origin": origin, "keywords": hits})
                        already = {s["name"] for s in meta["matched_skills"]}
                        if name not in already:
                            meta["matched_skills"].append({"name": name, "origin": origin})
            else:
                hits = _keyword_hits(corpus, set(meta["keywords"]))
                if hits and name != cat_name:
                    already = {s["name"] for s in meta["matched_skills"]}
                    if name not in already:
                        meta["matched_skills"].append({"name": name, "origin": origin})

    for meta in heatmap.values():
        workspace_count = sum(1 for s in meta["matched_skills"] if s.get("origin") == "workspace")
        global_count = len(meta["matched_skills"]) - workspace_count
        meta["workspace_count"] = workspace_count
        meta["global_count"] = global_count

        if meta["type"] == "checklist":
            for entry in meta["checklist"]:
                entry["covered"] = any(ev["origin"] == "workspace" for ev in entry["evidence"])
            total = len(meta["checklist"])
            covered = sum(1 for entry in meta["checklist"] if entry["covered"])
            meta["score"] = (covered / total * 100.0) if total else 0.0
            meta["covered_count"] = covered
            meta["total_count"] = total
            if meta["score"] >= 75.0:
                meta["status"] = "Strong"
            elif meta["score"] >= 25.0:
                meta["status"] = "Partial"
            else:
                meta["status"] = "Zero-Zone"
        else:
            # Dynamic domains: no fixed denominator exists, so we report presence, not a percentage.
            meta["status"] = "Detected" if workspace_count >= 1 else "Not Detected"

    return heatmap


def generate_heatmap_markdown(heatmap: Dict[str, Dict[str, Any]], target_domain: str = "General") -> str:
    """Formats the taxonomy checklist into GFM Markdown: real fraction scores with per-item evidence."""
    lines = [f"# 📊 Capability Gap Checklist: `{target_domain}`", ""]

    badge_map = {
        "Strong": "🟢 Strong",
        "Partial": "🟡 Partial",
        "Zero-Zone": "🔴 Zero-Zone",
        "Detected": "🟢 Detected",
        "Not Detected": "🔴 Not Detected",
    }

    for cat, meta in heatmap.items():
        badge = badge_map.get(meta["status"], meta["status"])
        lines.append(f"## {cat} — {badge}")
        lines.append("")

        if meta["type"] == "checklist":
            covered = meta.get("covered_count", 0)
            total = meta.get("total_count", len(meta["checklist"]))
            lines.append(f"**{covered}/{total} sub-capabilities covered by a workspace skill** (`{meta['score']:.0f}%`)")
            lines.append("")
            lines.append("| Sub-Capability | Status | Evidence |")
            lines.append("| :--- | :--- | :--- |")
            for entry in meta["checklist"]:
                mark = "✅" if entry["covered"] else "❌"
                workspace_ev = [ev for ev in entry["evidence"] if ev["origin"] == "workspace"]
                if workspace_ev:
                    ev_str = ", ".join(f"`{ev['skill']}` (matched: {', '.join(ev['keywords'])})" for ev in workspace_ev)
                elif entry["evidence"]:
                    ev_str = f"*global-only, not workspace coverage: {', '.join(ev['skill'] for ev in entry['evidence'])}*"
                else:
                    ev_str = "*no skill covers this*"
                lines.append(f"| {entry['capability']} | {mark} | {ev_str} |")
        else:
            matched = meta["matched_skills"]
            if matched:
                skills_str = ", ".join(f"`{s['name']}` [{s['origin']}]" for s in matched)
            else:
                skills_str = "*None*"
            lines.append(f"Declared/matched skills: {skills_str}")
            lines.append("")
            lines.append(
                "> No fixed sub-capability checklist exists for this emergent domain, "
                "so no percentage is reported — presence/absence only."
            )

        lines.append("")

    lines.append("> [!NOTE]")
    lines.append(
        "> Scores are `covered sub-capabilities / total sub-capabilities` for a workspace skill, "
        "computed from keyword evidence in each skill's name/description/tags/body — a deterministic "
        "Tier 1 pass. They are not an LLM judgment of skill quality or depth."
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
