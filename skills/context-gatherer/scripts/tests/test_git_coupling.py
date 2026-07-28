#!/usr/bin/env python3
"""Tests for git_coupling — git temporal coupling analysis."""

import json
import os
import subprocess
import sys
import tempfile
import textwrap
import unittest
from pathlib import Path
from unittest.mock import MagicMock, patch

# Add scripts directory to path for imports
sys.path.insert(0, str(Path(__file__).resolve().parent.parent))

from git_coupling import (
    find_coupled_files,
    format_results,
    get_git_root,
    parse_git_log,
)


class TestGetGitRoot(unittest.TestCase):
    """Tests for detecting the git repository root."""

    def test_returns_path_in_git_repo(self) -> None:
        """In a git repo, get_git_root should return a valid directory path."""
        result = get_git_root()
        # We're running inside a git repo (agent-skills), so this should work
        self.assertIsNotNone(result)
        self.assertTrue(Path(result).is_dir())

    def test_returns_none_outside_git_repo(self) -> None:
        """Outside a git repo, get_git_root should return None."""
        with tempfile.TemporaryDirectory() as tmpdir:
            result = get_git_root(cwd=tmpdir)
            self.assertIsNone(result)


class TestParseGitLog(unittest.TestCase):
    """Tests for parsing git log --name-only output."""

    def test_parses_single_commit(self) -> None:
        log_output = textwrap.dedent("""\
            abc1234

            file_a.py
            file_b.py
        """)
        commits = parse_git_log(log_output)
        self.assertEqual(len(commits), 1)
        self.assertEqual(commits[0], {"file_a.py", "file_b.py"})

    def test_parses_multiple_commits(self) -> None:
        log_output = textwrap.dedent("""\
            abc1234

            file_a.py
            file_b.py

            def56789

            file_b.py
            file_c.py

            aaa90120

            file_a.py
            file_c.py
        """)
        commits = parse_git_log(log_output)
        self.assertEqual(len(commits), 3)
        self.assertEqual(commits[0], {"file_a.py", "file_b.py"})
        self.assertEqual(commits[1], {"file_b.py", "file_c.py"})
        self.assertEqual(commits[2], {"file_a.py", "file_c.py"})

    def test_empty_log(self) -> None:
        commits = parse_git_log("")
        self.assertEqual(commits, [])

    def test_commit_with_single_file(self) -> None:
        log_output = textwrap.dedent("""\
            abc1234

            only_file.py
        """)
        commits = parse_git_log(log_output)
        self.assertEqual(len(commits), 1)
        self.assertEqual(commits[0], {"only_file.py"})

    def test_ignores_blank_filenames(self) -> None:
        log_output = "abc1234\n\nfile_a.py\n\n\n"
        commits = parse_git_log(log_output)
        self.assertEqual(len(commits), 1)
        self.assertNotIn("", commits[0])


class TestFindCoupledFiles(unittest.TestCase):
    """Tests for computing temporal coupling from commit data."""

    def test_basic_coupling(self) -> None:
        commits = [
            {"a.py", "b.py"},
            {"a.py", "b.py"},
            {"a.py", "c.py"},
        ]
        results = find_coupled_files("a.py", commits, min_commits=1)
        # b.py co-changed with a.py in 2 commits, c.py in 1
        self.assertEqual(results[0]["file"], "b.py")
        self.assertEqual(results[0]["co_commits"], 2)
        self.assertEqual(results[1]["file"], "c.py")
        self.assertEqual(results[1]["co_commits"], 1)

    def test_min_commits_filter(self) -> None:
        commits = [
            {"a.py", "b.py"},
            {"a.py", "b.py"},
            {"a.py", "c.py"},
        ]
        results = find_coupled_files("a.py", commits, min_commits=2)
        # Only b.py has 2+ co-commits
        self.assertEqual(len(results), 1)
        self.assertEqual(results[0]["file"], "b.py")

    def test_limit(self) -> None:
        commits = [
            {"a.py", "b.py", "c.py", "d.py"},
        ]
        results = find_coupled_files("a.py", commits, min_commits=1, limit=2)
        self.assertEqual(len(results), 2)

    def test_target_file_excluded(self) -> None:
        commits = [{"a.py", "b.py"}]
        results = find_coupled_files("a.py", commits, min_commits=1)
        files = [r["file"] for r in results]
        self.assertNotIn("a.py", files)

    def test_file_not_in_commits(self) -> None:
        commits = [{"b.py", "c.py"}]
        results = find_coupled_files("a.py", commits, min_commits=1)
        self.assertEqual(results, [])

    def test_coupling_ratio(self) -> None:
        commits = [
            {"a.py", "b.py"},
            {"a.py", "b.py"},
            {"a.py", "c.py"},
            {"b.py", "c.py"},  # a.py not present
        ]
        results = find_coupled_files("a.py", commits, min_commits=1)
        # a.py appears in 3 commits, b.py co-appears in 2 → ratio = 2/3
        b_result = next(r for r in results if r["file"] == "b.py")
        self.assertAlmostEqual(b_result["coupling_ratio"], 2 / 3, places=2)

    def test_empty_commits(self) -> None:
        results = find_coupled_files("a.py", [], min_commits=1)
        self.assertEqual(results, [])


class TestFormatResults(unittest.TestCase):
    """Tests for formatting output."""

    def test_json_format(self) -> None:
        results = [
            {"file": "b.py", "co_commits": 5, "coupling_ratio": 0.83},
        ]
        output = format_results("a.py", results, fmt="json")
        parsed = json.loads(output)
        self.assertEqual(parsed["target_file"], "a.py")
        self.assertEqual(len(parsed["coupled_files"]), 1)

    def test_text_format(self) -> None:
        results = [
            {"file": "b.py", "co_commits": 5, "coupling_ratio": 0.83},
        ]
        output = format_results("a.py", results, fmt="text")
        self.assertIn("b.py", output)
        self.assertIn("5", output)

    def test_empty_results(self) -> None:
        output = format_results("a.py", [], fmt="text")
        self.assertIn("No coupled files", output)


if __name__ == "__main__":
    unittest.main()
