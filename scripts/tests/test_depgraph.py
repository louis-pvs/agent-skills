#!/usr/bin/env python3
"""Unit tests for Skill Dependency Graph Resolver (scripts/depgraph.py)."""

import tempfile
import unittest
from pathlib import Path

from scripts.depgraph import (
    SkillNode,
    compute_transitive_deps,
    generate_lockfile,
    parse_yaml_frontmatter,
    resolve_topological_sort,
    verify_graph,
)


class TestDepGraph(unittest.TestCase):
    """Test suite for depgraph functions and algorithms."""

    def test_parse_frontmatter(self) -> None:
        content = """---
name: test-skill
description: A sample skill for testing.
requires:
  - dep-a
  - dep-b
enhances: [opt-c, opt-d]
version: 1.2.0
---

# Body
Some markdown content.
"""
        parsed = parse_yaml_frontmatter(content)
        self.assertEqual(parsed.get("name"), "test-skill")
        self.assertEqual(parsed.get("requires"), ["dep-a", "dep-b"])
        self.assertEqual(parsed.get("enhances"), ["opt-c", "opt-d"])
        self.assertEqual(parsed.get("version"), "1.2.0")

    def test_topological_sort_dag(self) -> None:
        nodes = {
            "c": SkillNode("c", Path("skills/c"), requires=["b"]),
            "b": SkillNode("b", Path("skills/b"), requires=["a"]),
            "a": SkillNode("a", Path("skills/a"), requires=[]),
        }
        order, cycle = resolve_topological_sort(nodes)
        self.assertIsNone(cycle)
        self.assertEqual(order, ["a", "b", "c"])

    def test_topological_sort_cycle(self) -> None:
        nodes = {
            "node1": SkillNode("node1", Path("skills/node1"), requires=["node2"]),
            "node2": SkillNode("node2", Path("skills/node2"), requires=["node1"]),
        }
        order, cycle = resolve_topological_sort(nodes)
        self.assertEqual(order, [])
        self.assertIsNotNone(cycle)
        self.assertIn("node1", cycle)
        self.assertIn("node2", cycle)

    def test_transitive_deps(self) -> None:
        nodes = {
            "a": SkillNode("a", Path("skills/a")),
            "b": SkillNode("b", Path("skills/b"), requires=["a"]),
            "c": SkillNode("c", Path("skills/c"), requires=["b"], enhances=["a"]),
        }
        transitive = compute_transitive_deps(nodes)
        self.assertEqual(transitive["c"]["transitive_requires"], ["a", "b"])
        self.assertEqual(transitive["c"]["transitive_enhances"], ["a"])

    def test_lockfile_generation_and_verification(self) -> None:
        with tempfile.TemporaryDirectory() as tmpdir:
            tmp_path = Path(tmpdir)
            skills_dir = tmp_path / "skills"
            lockfile_path = tmp_path / "skills.lock"

            # Create mock skill directories
            skill_a = skills_dir / "skill-a"
            skill_b = skills_dir / "skill-b"
            skill_a.mkdir(parents=True)
            skill_b.mkdir(parents=True)

            (skill_a / "SKILL.md").write_text(
                "---\nname: skill-a\ndescription: Base skill.\nversion: 1.0.0\n---\n",
                encoding="utf-8",
            )
            (skill_b / "SKILL.md").write_text(
                "---\nname: skill-b\ndescription: Dependent skill.\nrequires:\n  - skill-a\nversion: 1.0.0\n---\n",
                encoding="utf-8",
            )

            # Test lockfile generation
            success = generate_lockfile(skills_dir, lockfile_path)
            self.assertTrue(success)
            self.assertTrue(lockfile_path.exists())

            # Test graph verification
            is_valid, errors, warnings = verify_graph(skills_dir, lockfile_path)
            self.assertTrue(is_valid)
            self.assertEqual(len(errors), 0)

            # Modify frontmatter to cause lockfile drift
            (skill_b / "SKILL.md").write_text(
                "---\nname: skill-b\ndescription: Modified skill.\nrequires:\n  - skill-a\nversion: 2.0.0\n---\n",
                encoding="utf-8",
            )

            is_valid_drift, errors_drift, _ = verify_graph(skills_dir, lockfile_path)
            self.assertFalse(is_valid_drift)
            self.assertTrue(any("out of sync" in err for err in errors_drift))


if __name__ == "__main__":
    unittest.main()
