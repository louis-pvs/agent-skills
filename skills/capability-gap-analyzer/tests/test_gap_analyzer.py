#!/usr/bin/env python3
"""Unit tests for capability-gap-analyzer script suite."""

import sys
import tempfile
import unittest
from pathlib import Path

_repo_root = Path(__file__).resolve().parents[3]
if str(_repo_root) not in sys.path:
    sys.path.insert(0, str(_repo_root))

_script_dir = Path(__file__).resolve().parents[1] / "scripts"
if str(_script_dir) not in sys.path:
    sys.path.insert(0, str(_script_dir))

from domain_detector import detect_workspace_domains  # noqa: E402
from gap_analyzer import (  # noqa: E402
    calculate_taxonomy_heatmap,
    canonicalize_domain,
    generate_heatmap_markdown,
    load_global_skill_paths,
    parse_skill_frontmatter,
    scan_skills_inventory,
)
from main import build_scaffold_suggestions  # noqa: E402


class TestCapabilityGapAnalyzer(unittest.TestCase):
    def test_load_global_skill_paths(self):
        paths = load_global_skill_paths()
        self.assertIsInstance(paths, list)
        self.assertGreater(len(paths), 0)
        # Verify all elements in paths are Path objects, not empty dicts
        for p in paths:
            self.assertIsInstance(p, Path)

    def test_scan_skills_inventory(self):
        inventory = scan_skills_inventory(_repo_root / "skills", include_global=False)
        self.assertIsInstance(inventory, list)
        self.assertGreater(len(inventory), 0)
        names = [s.get("name") for s in inventory]
        self.assertIn("capability-gap-analyzer", names)
        for s in inventory:
            self.assertEqual(s.get("origin"), "workspace")

    def test_scan_skills_inventory_with_global(self):
        inventory = scan_skills_inventory(_repo_root / "skills", include_global=True)
        self.assertIsInstance(inventory, list)
        self.assertGreater(len(inventory), 0)
        origins = {s.get("origin") for s in inventory}
        self.assertTrue(origins.intersection({"workspace", "global"}))

    def test_calculate_taxonomy_heatmap(self):
        inventory = [
            {"name": "domain-modeling", "description": "DDD domain entities aggregate root solid", "origin": "workspace"},
            {
                "name": "architecture-auditor",
                "description": "architecture solid cupid adr design",
                "origin": "workspace",
            },
            {
                "name": "what-if-analysis",
                "description": "AST call-graph blast-radius self-healing",
                "origin": "workspace",
            },
        ]
        heatmap = calculate_taxonomy_heatmap(inventory)
        self.assertIn("Architecture & DDD", heatmap)
        self.assertEqual(heatmap["Architecture & DDD"]["status"], "Strong")
        self.assertEqual(heatmap["Frontend & UI/UX"]["status"], "Zero-Zone")

    def test_dynamic_domain_harvesting(self):
        inventory = [
            {
                "name": "quantum-sim",
                "description": "Quantum statevector simulation",
                "domain": "Quantum Computing",
                "tags": ["quantum", "qiskit"],
                "origin": "workspace",
            }
        ]
        heatmap = calculate_taxonomy_heatmap(inventory)
        self.assertIn("Quantum Computing", heatmap)
        self.assertEqual(heatmap["Quantum Computing"]["status"], "Partial")
        matched = [s["name"] for s in heatmap["Quantum Computing"]["matched_skills"]]
        self.assertIn("quantum-sim", matched)

    def test_canonicalize_domain(self):
        self.assertEqual(canonicalize_domain("frontend & ui/ux"), "Frontend & UI/UX")
        self.assertEqual(canonicalize_domain("quantum-computing"), "Quantum Computing")
        self.assertEqual(canonicalize_domain(""), "General")

    def test_parse_skill_frontmatter_with_domain_and_tags(self):
        with tempfile.TemporaryDirectory() as tmpdir:
            skill_dir = Path(tmpdir) / "my-test-skill"
            skill_dir.mkdir()
            skill_md = skill_dir / "SKILL.md"
            skill_md.write_text(
                "---\nname: my-test-skill\ndescription: Test description\n"
                "domain: Quantum Computing\ntags: [qiskit, sim]\n---\n\n# Body\n"
            )
            fm = parse_skill_frontmatter(skill_md)
            self.assertEqual(fm.get("domain"), "Quantum Computing")
            self.assertEqual(fm.get("tags"), ["qiskit", "sim"])

    def test_build_scaffold_suggestions_dynamic_tier2(self):
        heatmap = {
            "Quantum Computing": {
                "keywords": {"quantum"},
                "matched_skills": [],
                "score": 0.0,
                "status": "Zero-Zone",
            }
        }
        suggestions = build_scaffold_suggestions(heatmap)
        self.assertEqual(len(suggestions), 1)
        self.assertEqual(suggestions[0]["category"], "Quantum Computing")
        self.assertEqual(suggestions[0]["name"], "quantum-computing-scaffold")
        self.assertIn("Tier 2 LLM Prompt Suggestion", suggestions[0]["description"])
        self.assertIn('--domain "Quantum Computing"', suggestions[0]["command"])

    def test_detect_workspace_domains(self):
        with tempfile.TemporaryDirectory() as tmpdir:
            tmp_path = Path(tmpdir)
            (tmp_path / "package.json").write_text("{}")
            (tmp_path / "tsconfig.json").write_text("{}")

            domains = detect_workspace_domains(tmp_path)
            self.assertGreater(len(domains), 0)
            self.assertEqual(domains[0]["domain"], "frontend-web")
            self.assertEqual(domains[0]["confidence"], 1.0)

    def test_generate_heatmap_markdown(self):
        inventory = scan_skills_inventory(_repo_root / "skills", include_global=True)
        heatmap = calculate_taxonomy_heatmap(inventory)
        md = generate_heatmap_markdown(heatmap, target_domain="Frontend")
        self.assertIn("Capability Gap Taxonomy Heatmap", md)
        self.assertIn("Multi-Root Two-Tier Evaluation", md)

    def test_origin_aware_scoring_workspace_only(self):
        """Global skills must NOT inflate workspace coverage scores."""
        inventory = [
            {"name": "gcp-backend", "description": "bigquery pipeline data sql api", "origin": "global"},
            {"name": "gcp-infra", "description": "cloud gcloud kubernetes docker infra", "origin": "global"},
        ]
        heatmap = calculate_taxonomy_heatmap(inventory)
        # Even though global skills keyword-matched, workspace_count must be 0 → Zero-Zone
        backend = heatmap["Backend & Data Pipelines"]
        self.assertEqual(backend["workspace_count"], 0)
        self.assertGreaterEqual(backend["global_count"], 1)  # at least one global matched
        self.assertEqual(backend["status"], "Zero-Zone")
        self.assertEqual(backend["score"], 0.0)

    def test_origin_aware_scoring_mixed(self):
        """One workspace skill + global skills: workspace_count drives partial status."""
        inventory = [
            {"name": "my-api", "description": "backend api pipeline data", "origin": "workspace"},
            {"name": "gcp-backend", "description": "bigquery pipeline data", "origin": "global"},
        ]
        heatmap = calculate_taxonomy_heatmap(inventory)
        backend = heatmap["Backend & Data Pipelines"]
        self.assertEqual(backend["workspace_count"], 1)
        self.assertEqual(backend["status"], "Partial")

    def test_domain_relevance_filters_frontend_for_python_backend(self):
        """Frontend scaffold must NOT be suggested for a python-backend project."""
        from main import DOMAIN_RELEVANT_TAXONOMY, build_scaffold_suggestions

        heatmap = {
            cat: {
                "keywords": set(),
                "matched_skills": [],
                "score": 0.0,
                "status": "Zero-Zone",
                "workspace_count": 0,
                "global_count": 0,
            }
            for cat in ["Frontend & UI/UX", "Backend & Data Pipelines", "DevOps & Infrastructure"]
        }
        relevant = DOMAIN_RELEVANT_TAXONOMY.get("python-backend")
        suggestions = build_scaffold_suggestions(heatmap, relevant_categories=relevant)
        suggested_names = {s["category"] for s in suggestions}
        self.assertNotIn("Frontend & UI/UX", suggested_names)
        self.assertIn("Backend & Data Pipelines", suggested_names)

    def test_domain_relevance_none_returns_all_zero_zone(self):
        """When relevant_categories=None (no domain detected), all Zero-Zone emit scaffolds."""
        from main import build_scaffold_suggestions

        heatmap = {
            cat: {
                "keywords": set(),
                "matched_skills": [],
                "score": 0.0,
                "status": "Zero-Zone",
                "workspace_count": 0,
                "global_count": 0,
            }
            for cat in ["Frontend & UI/UX", "Backend & Data Pipelines"]
        }
        suggestions = build_scaffold_suggestions(heatmap, relevant_categories=None)
        self.assertEqual(len(suggestions), 2)


if __name__ == "__main__":
    unittest.main()
