#!/usr/bin/env python3
"""Unit tests for scaffold_skill.py script."""

import sys
import tempfile
import unittest
from pathlib import Path

# Add scripts directory to module search path
SCRIPTS_DIR = Path(__file__).parent.parent.resolve()
sys.path.insert(0, str(SCRIPTS_DIR))

from scaffold_skill import (  # noqa: E402
    parse_frontmatter,
    scaffold_skill,
    validate_skill,
    validate_skill_metadata,
)


class TestScaffoldSkill(unittest.TestCase):

    def test_validate_skill_metadata_valid(self) -> None:
        valid, errors = validate_skill_metadata("my-skill-name", "Triggers when user asks for my skill.")
        self.assertTrue(valid)
        self.assertEqual(len(errors), 0)

    def test_validate_skill_metadata_invalid_name(self) -> None:
        valid, errors = validate_skill_metadata("Invalid_Name!", "Valid description")
        self.assertFalse(valid)
        self.assertGreater(len(errors), 0)

    def test_validate_skill_metadata_too_long(self) -> None:
        long_name = "a" * 70
        valid, errors = validate_skill_metadata(long_name, "Valid description")
        self.assertFalse(valid)

    def test_parse_frontmatter(self) -> None:
        content = """---
name: test-skill
description: A test skill description.
disable-model-invocation: true
---

# Test Title
Body text goes here.
"""
        meta, body = parse_frontmatter(content)
        self.assertEqual(meta.get("name"), "test-skill")
        self.assertEqual(meta.get("description"), "A test skill description.")
        self.assertEqual(meta.get("disable-model-invocation"), "true")
        self.assertTrue("Body text goes here." in body)

    def test_scaffold_and_validate_simple_skill(self) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            target_dir = Path(temp_dir)
            created_path = scaffold_skill(
                name="test-simple-skill",
                description="Test simple skill description.",
                target_dir=target_dir,
                skill_type="simple",
            )

            self.assertTrue(created_path.exists())
            self.assertTrue((created_path / "SKILL.md").exists())

            is_valid, issues = validate_skill(created_path)
            self.assertTrue(is_valid, msg=f"Validation failed with issues: {issues}")

    def test_scaffold_and_validate_user_invoked_router_skill(self) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            target_dir = Path(temp_dir)
            created_path = scaffold_skill(
                name="test-router-skill",
                description="Test router skill summary.",
                target_dir=target_dir,
                skill_type="router",
                user_invoked=True,
            )

            self.assertTrue(created_path.exists())
            skill_md = created_path / "SKILL.md"
            self.assertTrue(skill_md.exists())

            content = skill_md.read_text(encoding="utf-8")
            meta, body = parse_frontmatter(content)
            self.assertEqual(meta.get("disable-model-invocation"), "true")
            self.assertIn("Router skill for dispatching", body)

            is_valid, issues = validate_skill(created_path)
            self.assertTrue(is_valid, msg=f"Validation failed with issues: {issues}")

    def test_scaffold_and_validate_complex_skill(self) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            target_dir = Path(temp_dir)
            created_path = scaffold_skill(
                name="test-complex-skill",
                description="Test complex skill description.",
                target_dir=target_dir,
                skill_type="complex",
            )

            self.assertTrue(created_path.exists())
            self.assertTrue((created_path / "SKILL.md").exists())
            self.assertTrue((created_path / "references" / "overview.md").exists())
            self.assertTrue((created_path / "scripts" / "main.py").exists())
            self.assertTrue((created_path / "scripts" / "tests" / "test_main.py").exists())

            is_valid, issues = validate_skill(created_path)
            self.assertTrue(is_valid, msg=f"Validation failed with issues: {issues}")

    def test_validate_skill_missing_completion_criteria(self) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            target_dir = Path(temp_dir)
            skill_dir = target_dir / "no-criteria-skill"
            skill_dir.mkdir()
            skill_md = skill_dir / "SKILL.md"
            skill_md.write_text(
                """---
name: no-criteria-skill
description: Skill without criteria.
---

# Title
Just some steps without verification.
""",
                encoding="utf-8",
            )
            is_valid, issues = validate_skill(skill_dir)
            self.assertFalse(is_valid)
            self.assertTrue(any("Missing checkable completion criteria" in i for i in issues))


if __name__ == "__main__":
    unittest.main()
