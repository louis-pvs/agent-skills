#!/usr/bin/env python3
"""Unit tests for canonical configuration safety utilities (scripts/_config_safety.py)."""

import tempfile
import unittest
from pathlib import Path

from scripts._config_safety import (
    load_skill_config,
    load_yaml_file,
    parse_simple_yaml,
)


class TestConfigSafety(unittest.TestCase):
    def test_parse_simple_yaml_basic(self) -> None:
        sample_yaml = """
targets:
  - name: "Gemini / Antigravity"
    path: "~/.gemini/config/skills"
  - name: "Claude Code"
    path: "~/.claude/skills"
"""
        parsed = parse_simple_yaml(sample_yaml)
        self.assertIn("targets", parsed)
        self.assertEqual(len(parsed["targets"]), 2)
        self.assertEqual(parsed["targets"][0]["name"], "Gemini / Antigravity")

    def test_parse_simple_yaml_scalar_list(self) -> None:
        sample_yaml = """
gap_analyzer:
  custom_global_paths:
    - "~/.gemini/config/skills"
    - "~/.claude/skills"
"""
        parsed = parse_simple_yaml(sample_yaml)
        self.assertIn("gap_analyzer", parsed)
        self.assertIn("custom_global_paths", parsed["gap_analyzer"])
        self.assertEqual(
            parsed["gap_analyzer"]["custom_global_paths"],
            ["~/.gemini/config/skills", "~/.claude/skills"],
        )

    def test_parse_simple_yaml_nested_sections(self) -> None:
        sample_yaml = """
council:
  chairman:
    role: "auto"
  settings:
    timeout: 120
    enabled: true
    ratio: 0.75
"""
        parsed = parse_simple_yaml(sample_yaml)
        self.assertIn("council", parsed)
        self.assertEqual(parsed["council"]["chairman"]["role"], "auto")
        self.assertEqual(parsed["council"]["settings"]["timeout"], 120)
        self.assertIs(parsed["council"]["settings"]["enabled"], True)
        self.assertEqual(parsed["council"]["settings"]["ratio"], 0.75)

    def test_load_yaml_file_nonexistent(self) -> None:
        result = load_yaml_file(Path("/nonexistent/file.yaml"))
        self.assertEqual(result, {})

    def test_load_skill_config_hierarchy(self) -> None:
        with tempfile.TemporaryDirectory() as tmpdir:
            root = Path(tmpdir)
            (root / "skills.config.yaml").write_text("targets:\n  - name: Global\n    path: ~/.global/skills")
            skill_dir = root / "skills" / "my-skill"
            skill_dir.mkdir(parents=True)
            (skill_dir / "config.yaml").write_text("my_skill:\n  setting_a: true")

            config_bundle = load_skill_config("my-skill", skill_dir=skill_dir, repo_root=root)
            self.assertIn("repo_config", config_bundle)
            self.assertIn("skill_config", config_bundle)
            self.assertTrue(config_bundle["skill_config"]["my_skill"]["setting_a"])

    def test_load_skill_config_legacy_fallback(self) -> None:
        with tempfile.TemporaryDirectory() as tmpdir:
            root = Path(tmpdir)
            skill_dir = root / "skills" / "legacy-skill"
            skill_dir.mkdir(parents=True)
            (skill_dir / "legacy-skill.config.yaml").write_text("legacy:\n  enabled: true")

            config_bundle = load_skill_config("legacy-skill", skill_dir=skill_dir, repo_root=root)
            self.assertTrue(config_bundle["skill_config"]["legacy"]["enabled"])


if __name__ == "__main__":
    unittest.main()
