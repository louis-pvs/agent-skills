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
    generate_heatmap_markdown,
    load_global_skill_paths,
    scan_skills_inventory,
)


class TestCapabilityGapAnalyzer(unittest.TestCase):
    def test_load_global_skill_paths(self):
        paths = load_global_skill_paths()
        self.assertIsInstance(paths, list)
        self.assertGreater(len(paths), 0)

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


if __name__ == "__main__":
    unittest.main()
