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

    def test_checklist_score_is_real_fraction_not_a_lookup_table(self):
        """Score must be covered/total, not a step function saturating at 3 matched skills."""
        inventory = [
            {"name": "s1", "description": "adr documentation", "origin": "workspace"},
            {"name": "s2", "description": "aggregate entity value-object ddd", "origin": "workspace"},
        ]
        heatmap = calculate_taxonomy_heatmap(inventory)
        arch = heatmap["Architecture & DDD"]
        self.assertEqual(arch["total_count"], 4)
        self.assertEqual(arch["covered_count"], 2)
        self.assertAlmostEqual(arch["score"], 50.0)
        self.assertEqual(arch["status"], "Partial")
        # A single additional skill covering the remaining 2 items should reach 100%,
        # not saturate early regardless of how many *skills* (vs sub-capabilities) matched.
        inventory.append({"name": "s3", "description": "solid cupid dry yagni kiss architecture-review", "origin": "workspace"})
        heatmap2 = calculate_taxonomy_heatmap(inventory)
        arch2 = heatmap2["Architecture & DDD"]
        self.assertEqual(arch2["covered_count"], 4)
        self.assertAlmostEqual(arch2["score"], 100.0)

    def test_matches_full_body_not_just_description(self):
        """Keyword evidence must come from the full SKILL.md body, not the one-line description."""
        inventory = [
            {
                "name": "verbose-skill",
                "description": "A skill that does things.",
                "body": "\n## Workflow\nRuns a SAST scan and checks for cve entries via dependency-scan.\n",
                "origin": "workspace",
            }
        ]
        heatmap = calculate_taxonomy_heatmap(inventory)
        sec = heatmap["Security & Compliance"]
        covered_caps = {entry["capability"] for entry in sec["checklist"] if entry["covered"]}
        self.assertIn("Static application security testing (SAST)", covered_caps)
        self.assertIn("Dependency / supply-chain vulnerability scanning", covered_caps)

    def test_generic_keyword_does_not_inflate_unrelated_category(self):
        """A skill whose description merely contains 'audit' must not count as Security & Compliance coverage."""
        inventory = [
            {
                "name": "architecture-auditor",
                "description": "architectural audit of design principles",
                "origin": "workspace",
            },
            {
                "name": "skill-creator",
                "description": "audit new Agent Skills for spec compliance-review-x",
                "origin": "workspace",
            },
        ]
        heatmap = calculate_taxonomy_heatmap(inventory)
        sec = heatmap["Security & Compliance"]
        # "audit" alone is not a keyword for any Security & Compliance sub-capability.
        covered_caps = {entry["capability"] for entry in sec["checklist"] if entry["covered"]}
        self.assertNotIn("Static application security testing (SAST)", covered_caps)
        self.assertLess(sec["score"], 100.0)

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
        # Dynamic/emergent domains have no fixed sub-capability checklist to divide by, so
        # they report presence/absence rather than a fabricated percentage.
        self.assertEqual(heatmap["Quantum Computing"]["type"], "dynamic")
        self.assertEqual(heatmap["Quantum Computing"]["status"], "Detected")
        self.assertIsNone(heatmap["Quantum Computing"]["score"])
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
        self.assertIn("Capability Gap Checklist", md)
        self.assertIn("sub-capabilities covered by a workspace skill", md)

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

    def test_filter_heatmap_by_relevance_drops_out_of_scope_categories_from_report(self):
        """Out-of-scope baseline categories (e.g. Frontend for python-backend) must not appear
        in the displayed report at all, not just be excluded from scaffold suggestions."""
        from main import DOMAIN_RELEVANT_TAXONOMY, filter_heatmap_by_relevance

        inventory = scan_skills_inventory(_repo_root / "skills", include_global=True)
        heatmap = calculate_taxonomy_heatmap(inventory)
        relevant = DOMAIN_RELEVANT_TAXONOMY.get("python-backend")
        filtered = filter_heatmap_by_relevance(heatmap, relevant_categories=relevant)
        self.assertNotIn("Frontend & UI/UX", filtered)
        self.assertIn("Backend & Data Pipelines", filtered)

    def test_filter_heatmap_by_relevance_keeps_dynamic_domains(self):
        """Dynamic/emergent domains aren't part of the baseline relevance map and must survive filtering."""
        from main import filter_heatmap_by_relevance

        heatmap = {
            "Frontend & UI/UX": {"status": "Zero-Zone"},
            "Quantum Computing": {"type": "dynamic", "status": "Not Detected"},
        }
        filtered = filter_heatmap_by_relevance(heatmap, relevant_categories={"Backend & Data Pipelines"})
        self.assertNotIn("Frontend & UI/UX", filtered)
        self.assertIn("Quantum Computing", filtered)


if __name__ == "__main__":
    unittest.main()
